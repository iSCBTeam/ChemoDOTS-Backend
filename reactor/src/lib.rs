use std::collections::HashMap;
use std::fs::{File, self};
use std::io::{Write, self};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam::channel::unbounded as mpmc;
use field_count::FieldCount;
use itertools::{Itertools, Either};
use rayon::{prelude::*, ThreadPool};
use unicode_segmentation::UnicodeSegmentation;
use zip_next as zip;
use zip::ZipWriter;

use rdkit_rust::*;
use rdkit_rust::contrib::chem::mol2writer::Mol2WriteImpl;
use rdkit_rust::general::props::prelude::*;
use rdkit_rust::graphmol::chemreactions::reaction::*;
use rdkit_rust::graphmol::depictor::DepictorMutImpl;
use rdkit_rust::graphmol::descriptors::prelude::*;
use rdkit_rust::graphmol::distgeomhelpers::embedder::EmbedderImpl;
use rdkit_rust::graphmol::{moldraw2d::*, rwmol};
use rdkit_rust::graphmol::moldraw2d::moldraw2dsvg::*;
use rdkit_rust::graphmol::molops::prelude::*;
use rdkit_rust::graphmol::romol::*;
use rdkit_rust::graphmol::rwmol::*;
use rdkit_rust::graphmol::substruct::substructmatch::*;
use rdkit_rust::prelude::*;

use chemodots_db as db;
use chemodots_common as common;

use db::model::{CompoundProvider, ExperimentPostprocFilter, ExperimentProductDescFilter, NewExperiment, NewExperimentFrag, NewExperimentFragReactant, NewExperimentPostprocFilter, NewExperimentProductOrigin, NewExperimentSelectedProvider, Reaction};
use db::model::{Experiment, NewExperimentProduct};

pub mod plot;

pub fn experiment_create(db_pool: &db::DBPool, exp_name: &str, frag_smiles: &str, frag_mol: Option<&str>, idx_atoms: &[i32], id_reactions: &[i64], provider_names: &[&str]) -> Experiment {
	let mut conn = db_pool.get().unwrap();

	let ent_experiment = db::model::create_experiment(&mut conn, &NewExperiment {
		name: exp_name,
		status: "",
		ts_start: chrono::Utc::now().naive_utc(),
		ts_end: None,
	}).unwrap();

	{
		let mut conn1 = db_pool.get().unwrap();

		db::model::get_compound_providers_by_name(&mut conn, provider_names)
			.unwrap()
			.map(|item| item.unwrap())
			.for_each(|ent_compound_provider| {
				db::model::create_experiment_selected_provider(&mut conn1, &NewExperimentSelectedProvider {
					id_experiment: ent_experiment.id,
					id_compound_provider: ent_compound_provider.id,
				}).unwrap();
			});
	}

	// TODO
	let ent_moiety = db::model::get_moiety(&mut conn, 1).unwrap();

	let frag = new_local!(RWMol);
	let frag = if let Some(frag_mol) = frag_mol {
		frag
			.init(ParseMolBlockParams {
				mol_block: frag_mol,
				sanitize: Default::default(),
				remove_hs: Default::default(),
				strict_parsing: Default::default(),
			})
			.unwrap()
	} else {
		frag
			.init(ParseSmilesParams {
				text: frag_smiles,
				debug_parse: Default::default(),
				sanitize: Default::default(),
				replacements: (),
			})
			.unwrap()
	};

	let frag_pickle = frag.to_pickle(Some(common::DEFAULT_MOL_PICKLE_OPTIONS)).unwrap();
	let frag_smiles = frag.to_smiles().unwrap();

	let ent_experiment_frag = db::model::create_experiment_frag(&mut conn, &NewExperimentFrag {
		id_experiment: ent_experiment.id,
		id_moiety: ent_moiety.id,
		idx: 0,
		rdpickle: &frag_pickle,
		smiles: &frag_smiles,
		moiety_atoms: idx_atoms,
	}).unwrap();

	id_reactions
		.into_iter()
		.for_each(|id_reaction| {
			let ent_reaction = db::model::get_reaction(&mut conn, *id_reaction as i64).unwrap();
			let reaction = new_local!(ChemicalReaction);

			let reaction = reaction
				.init(ChemicalReactionFromPickleParams {
					pickle: &ent_reaction.rdpickle,
				})
				.unwrap();

			let reactants = reaction.get_reactants();
			let reactant_count = reactants.size();

			(0..reactant_count)
				.into_iter()
				.for_each(|reactant_idx| {
					let reactant = reactants.get(reactant_idx).unwrap();

					let matches = new_local!(MatchVectTypeVec);
					let matches = matches
						.init(&MatchVectTypeVecInitParamsFromSubstructMatch::new(&frag, &reactant));
					if matches.is_err() {
						return;
					}

					let matches = matches.unwrap();
					let entry_count = matches.len();

					let mut found = false;
					let mut real_idx_atoms = Vec::new();

					for idx_entry in 0..entry_count {
						let pair_count = matches.entry_len(idx_entry);

						for idx_pair in 0..pair_count {
							let (_, idx_frag_atom) = matches.entry_get_atom_pair(idx_entry, idx_pair).unwrap();

							found = idx_atoms.iter().contains(&idx_frag_atom);
							if found {
								break
							}
						}

						if found {
							real_idx_atoms.resize_with(pair_count, Default::default);

							for idx_pair in 0..pair_count {
								let (_, idx_frag_atom) = matches.entry_get_atom_pair(idx_entry, idx_pair).unwrap();
								real_idx_atoms[idx_pair] = idx_frag_atom;
							}

							break
						}
					}

					if found {
						db::model::create_experiment_frag_reactant(&mut conn, &NewExperimentFragReactant {
							id_experiment_frag: ent_experiment_frag.id,
							id_reaction: ent_reaction.id,
							reactant_idx: reactant_idx.try_into().unwrap(),
							moiety_atoms: &real_idx_atoms,
						}).unwrap();
					}
				});
		});

	ent_experiment
}

#[derive(Debug, Default)]
pub struct ReactionCounterAtomic {
	pub reacted_building_blocks: AtomicUsize,
	pub raw_products: AtomicUsize,
	pub dup_products: AtomicUsize,
	pub undesired_products: AtomicUsize,
	pub final_products: AtomicUsize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReactionCounter {
	pub reacted_building_blocks: usize,
	pub raw_products: usize,
	pub dup_products: usize,
	pub undesired_products: usize,
	pub final_products: usize,
}

impl From<ReactionCounterAtomic> for ReactionCounter {
	fn from(value: ReactionCounterAtomic) -> Self {
		Self {
			reacted_building_blocks: value.reacted_building_blocks.load(Ordering::Relaxed),
			raw_products: value.raw_products.load(Ordering::Relaxed),
			dup_products: value.dup_products.load(Ordering::Relaxed),
			undesired_products: value.undesired_products.load(Ordering::Relaxed),
			final_products: value.final_products.load(Ordering::Relaxed),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct ReactionResult {
	pub id: i64,
	pub name: String,
	pub counter: ReactionCounter,
}

#[derive(Clone, Debug, Default)]
pub struct ExperimentGenProductsResult {
	pub reactions: Vec<ReactionResult>,
}

pub fn experiment_gen_products(thread_pool: &ThreadPool, db_pool: &db::DBPool, ent_experiment: &mut Experiment) -> ExperimentGenProductsResult {
	let mut conn = db_pool.get().unwrap();

	let mut result = ExperimentGenProductsResult::default();

	let ent_frag_reactants = db::model::MergedExperimentFragReactant::get_with_experiment_and_idx(&mut conn, &ent_experiment, 0)
		.unwrap()
		.filter_map(|e| e.ok());

	let mut reaction_infos = HashMap::<i64, (Reaction, ReactionCounterAtomic)>::default();

	for ent_frag_reactant in ent_frag_reactants {
		let ent_reaction = db::model::get_reaction(&mut conn, ent_frag_reactant.id_reaction)
			.unwrap();

		eprintln!("Computing reaction {}...", ent_reaction.name);

		let reaction_info = reaction_infos.entry(ent_reaction.id).or_insert((ent_reaction.clone(), ReactionCounterAtomic::default()));

		let reaction_counter = &reaction_info.1;

		let counter_reacted_building_blocks = &reaction_counter.reacted_building_blocks;
		let counter_raw_products = &reaction_counter.raw_products;
		let counter_dup_products = &reaction_counter.dup_products;
		let _counter_undesired_products = &reaction_counter.undesired_products;
		let counter_final_products = &reaction_counter.final_products;

		let frag_mol = new_local!(ROMol);
		let mut frag_mol = frag_mol.init(ROMolFromPickleParams {
				pickle: &ent_frag_reactant.rdpickle
			})
			.unwrap();

		let atom_count = frag_mol.get_num_atoms();
		for idx_atom in 0..atom_count {
			// Do not protect reacting atoms
			if ent_frag_reactant.moiety_atoms.contains(&Some(idx_atom as i32)) {
				continue
			}

			let mut atom = frag_mol.get_atom_mut(idx_atom).unwrap();
			atom.set_prop_i32("_protected", 1);
		}

		eprintln!("  Generating products...");

		thread_pool.in_place_scope(|scope| {
			let (tx, rx) = mpmc();

			let ent_experiment = &ent_experiment;
			let frag_mol = &frag_mol;

			scope.spawn(move |_scope| {
				let mut conn = db_pool.get().unwrap();

				let reaction = new_local!(ChemicalReaction);
				let mut reaction = reaction
					.init(ChemicalReactionFromPickleParams {
						pickle: &ent_reaction.rdpickle,
					})
					.unwrap();

				reaction.init_reactant_matchers();

				thread_pool.in_place_scope(|scope| {
					let ent_bb_reactants = db::model::MergedBuildingBlockReactant::get_with_experiment_and_reaction(&mut conn, &ent_experiment, &ent_reaction)
						.unwrap()
						.filter_map(|x| x.ok());

					for ent_bb_reactant in ent_bb_reactants {
						let tx = &tx;
						let ent_frag_reactant = &ent_frag_reactant;
						let reaction = &reaction;

						scope.spawn(move |_scope| {
							let frag_mol_ = new_local!(ROMol);
							let frag_mol = frag_mol_
								.init(ROMolInitParamsROMol {
									romol: frag_mol
								})
								.unwrap();

							let bb_mol = new_local!(ROMol);
							let bb_mol = bb_mol
								.init(ROMolFromPickleParams {
									pickle: &ent_bb_reactant.rdpickle
								})
								.unwrap();

							let reactants = new_local!(ROMolSptrVec);
							let mut reactants = reactants
								.init(())
								.unwrap();

							reactants.set(ent_frag_reactant.reactant_idx.try_into().unwrap(), frag_mol);
							reactants.set(ent_bb_reactant.reactant_idx.try_into().unwrap(), bb_mol);

							reaction
								.run_reactants(&reactants)
								.and_then(|products_vec| {
									let mut found = None;

									// Strict mode: We iterate through the result sets to enforce that only one distinct product (identified by its smiles) was generated by the reaction

									let products_vec_count = products_vec.size();
									for i in 0..products_vec_count {
										let products = products_vec.get(i).unwrap();
										let products_count = products.size();

										// Enforce that the reactions must always generate one product per result set
										if products_count != 1 {
											return None;
										}

										let product = products.get(0).unwrap();
										let mut product = RWMol::new(RWMolInitParamsROMol {
												romol: product.get_ref(),
											})
											.unwrap();

										if product.sanitize().is_err() {
											continue;
										}

										let smiles = product.to_smiles().unwrap();

										if let Some((found_smiles, _)) = &found {
											// Strict mode: Reject all the building blocks leading to several distinct products
											if *found_smiles != smiles {
												return None;
											}
										} else {
											found = Some((smiles, product));
										}
									}

									if let Some((found_smiles, found_product)) = found {
										counter_reacted_building_blocks.fetch_add(1, Ordering::Relaxed);

										tx.send((found_smiles, (found_product, ent_frag_reactant.id, ent_bb_reactant.id, ent_bb_reactant.name, ent_bb_reactant.fullname))).unwrap();
										return Some(());
									}

									None
								});
						});
					}
				});
			});

			let mut grouped = rx
				.into_iter()
				.into_group_map();

			eprintln!("   completed.");
			eprintln!("  Inserting products...");

			grouped
				.par_drain()
				.map(|elem| -> Result<_, &str> {
					let (smiles, v) = elem;
					let dup_count = v.len();

					// Use the first duplicate (TODO: trace the origin of all the duplicates).
					let (mut product, id_frag_reactant, id_bb_reactant, name, fullname) = v
						.into_iter()
						.next()
						.ok_or("No products were generated")?;

					product.compute_2d_coords();

					let pickle = product.to_pickle(Some(common::DEFAULT_MOL_PICKLE_OPTIONS))
						.map_err(|_| "Failed to generate pickle")?;

					let fsp3 = product.calc_fraction_csp3() as f32;
					let hba = product.calc_num_hba().try_into()
						.map_err(|_| "HBA descriptor out of bounds")?;
					let hbd = product.calc_num_hbd().try_into()
						.map_err(|_| "HBD descriptor out of bounds")?;

					let clogp = product.calc_clogp() as f32;
					let mw = product.calc_exact_mw() as f32;
					let tpsa = product.calc_tpsa() as f32;

					counter_raw_products.fetch_add(dup_count, Ordering::Relaxed);
					counter_dup_products.fetch_add(dup_count - 1, Ordering::Relaxed);
					counter_final_products.fetch_add(1, Ordering::Relaxed);

					Ok((id_frag_reactant, id_bb_reactant, name, fullname, smiles, pickle, dup_count, fsp3, hba, hbd, clogp, mw, tpsa))
				})
				.filter_map(|e| e.ok())
				.collect::<Vec<_>>()
				.par_chunks(65535 / NewExperimentProduct::field_count().max(NewExperimentProductOrigin::field_count()))
				.map(|e| -> Result<_, &'static str> {
					let mut conn = db_pool.get().unwrap();

					let prods: Vec<_> = e
						.iter()
						.map(|(id_frag_reactant, _, name, fullname, smiles, pickle, dup_count, fsp3, hba, hbd, clogp, mw, tpsa)| NewExperimentProduct {
							id_experiment_frag_reactant: *id_frag_reactant,
							name: &name,
							fullname: &fullname,
							rdpickle: &pickle,
							smiles: &smiles,
							dup_count: *dup_count as i32,
							desc_fsp3: *fsp3,
							desc_hba: *hba,
							desc_hbd: *hbd,
							desc_clogp: *clogp,
							desc_mw: *mw,
							desc_tpsa: *tpsa,
						})
						.collect();

					let ent_experiment_products = db::model::create_experiment_products(&mut conn, &prods)
						.map_err(|_| "Failed to insert experiment products")?;

					let prod_origs: Vec<_> = ent_experiment_products
						.into_iter()
						.zip(e)
						.map(|(ent_experiment_product, e)| NewExperimentProductOrigin {
							id_building_block_reactant: e.1,
							id_experiment_product: ent_experiment_product.id,
						})
						.collect();

					db::model::create_experiment_product_origins(&mut conn, &prod_origs)
						.map_err(|_| "Failed to insert experiment product origins")?;

					Ok(())
				})
				.filter_map(|e| e.err())
				.for_each(|err| eprintln!("Error: {err}"));
		});

		eprintln!("   completed.");

		eprintln!(" completed.");
	}

	result.reactions = reaction_infos
		.into_iter()
		.map(|(_, (ent_reaction, counter))| ReactionResult {
			id: ent_reaction.id,
			name: ent_reaction.name,
			counter: counter.into()
		})
		.collect();

	gen_files(thread_pool, db_pool, ent_experiment, "raw", "overall", true);

	*ent_experiment = db::model::update_experiment(&mut conn, ent_experiment.id, &NewExperiment {
		name: &ent_experiment.name,
		status: &ent_experiment.status,
		ts_start: ent_experiment.ts_start,
		ts_end: Some(chrono::Utc::now().naive_utc()),
	}).unwrap();

	result
}

pub fn gen_files(thread_pool: &ThreadPool, db_pool: &db::DBPool, ent_experiment: &Experiment, prefix: &str, filename_prefix: &str, gen_img: bool) {
	gen_files_filtered(thread_pool, db_pool, ent_experiment, prefix, filename_prefix, gen_img, None);
}

pub fn gen_files_filtered(thread_pool: &ThreadPool, db_pool: &db::DBPool, ent_experiment: &Experiment, prefix: &str, filename_prefix: &str, gen_img: bool, ent_experiment_postproc_filter: Option<&ExperimentPostprocFilter>) {
	let exp_uuid_str = ent_experiment.uuid.to_string();

	let path_prefix = Path::new(prefix);
	fs::create_dir(path_prefix)
		.or_else(|e| match e.kind() {
			io::ErrorKind::AlreadyExists => Ok(()),
			kind => Err(kind),
	   })
	   .expect("Failed to create the output directory");

	let zip_opts = common::default_zip_opts();

	let file_out_zip = File::create(Path::join(path_prefix, format!("{filename_prefix}.zip")))
		.map_err(|_| format!("Failed to create output archive for experiment {exp_uuid_str}")).unwrap();

	let mut file_out_zip = ZipWriter::new(file_out_zip);

	eprintln!("Fetching building blocks...");

	thread_pool.in_place_scope(|scope| {
		let (tx, rx) = mpmc();

		let mut conn = db_pool.get().unwrap();
		scope.spawn(move |_| {
			let it = if let Some(ent) = ent_experiment_postproc_filter {
				Either::Left(db::model::ExportableBuildingBlock::get_with_experiment_postproc_filter(&mut conn, &ent).unwrap())
			} else {
				Either::Right(db::model::ExportableBuildingBlock::get_with_experiment(&mut conn, &ent_experiment).unwrap())
			};

			it
				.filter_map(|e| e.ok())
				.for_each(|e| {
					tx.send(e).unwrap();
				});
		});

		let mut ent_building_blocks: Vec<_> = rx
			.into_iter()
			.par_bridge()
			.map(|ent_building_block| {
				let bb_mol = new_local!(ROMol);
				let mut bb_mol = bb_mol
					.init(ROMolFromPickleParams {
						pickle: &ent_building_block.rdpickle
					})
					.unwrap();

				bb_mol.set_prop_str("_Name", &ent_building_block.name);

				let mw = bb_mol.calc_exact_mw();
				let sdf = bb_mol
					.to_sd()
					.unwrap();

				(mw, ent_building_block.name, ent_building_block.smiles, sdf)
			})
			.collect();

		eprintln!(" completed.");

		eprintln!("Sorting building blocks...");

		ent_building_blocks.par_sort_unstable_by(|(mw0, _, _, _), (mw1, _, _, _)|
			f64::total_cmp(mw0, mw1));

		eprintln!(" completed.");

		eprintln!("Writing building blocks...");

		file_out_zip.start_file(format!("{filename_prefix}_bbs.smi"), zip_opts.clone()).unwrap();

		writeln!(&mut file_out_zip, "Smiles\tName").unwrap();
		ent_building_blocks
			.iter()
			.for_each(|(_, name, smiles, _)| {
				writeln!(&mut file_out_zip, "{smiles}\t{name}")
					.expect(&format!("Failed to write building block to SMILES file for experiment {exp_uuid_str}"));
			});

		file_out_zip.start_file(format!("{filename_prefix}_bbs.sdf"), zip_opts.clone()).unwrap();

		ent_building_blocks
			.into_iter()
			.for_each(|(_, _, _, sdf)| {
				file_out_zip.write_all(sdf.as_bytes())
					.expect(&format!("Failed to write building block to SDF file for experiment {exp_uuid_str}"));
			});

		eprintln!(" completed.");

		eprintln!("Fetching products...");

		let (tx, rx) = mpmc();

		let mut conn = db_pool.get().unwrap();
		scope.spawn(move |_| {
			let it = if let Some(ent) = ent_experiment_postproc_filter {
				Either::Left(db::model::ExportableExperimentProduct::get_with_experiment_postproc_filter(&mut conn, &ent).unwrap())
			} else {
				Either::Right(db::model::ExportableExperimentProduct::get_with_experiment(&mut conn, &ent_experiment).unwrap())
			};

			it
				.filter_map(|e| e.ok())
				.for_each(|e| {
					tx.send(e).unwrap();
				});
		});

		let mut ent_products: Vec<_> = rx
			.into_iter()
			.par_bridge()
			.map(|ent_product| {
				let mut prod_mol = ROMol::new(ROMolFromPickleParams {
						pickle: &ent_product.rdpickle
					})
					.unwrap();

				prod_mol.set_prop_str("_Name", &ent_product.fullname);

				let mw = prod_mol.calc_exact_mw();
				let sdf = prod_mol.to_sd().unwrap();

				let mut name = ent_product.name;

				if name.graphemes(true).count() > 50 {
					name = name.clone();
					name.truncate(name.grapheme_indices(true).nth(47).unwrap().0);
					name.push_str("...");
				}

				(mw, prod_mol, name, ent_product.fullname, ent_product.smiles, sdf, ent_product.id_reaction)
			})
		.collect();

		eprintln!(" completed.");

		eprintln!("Sorting products...");

		ent_products.par_sort_unstable_by(|(mw0, _, _, _, _, _, _), (mw1, _, _, _, _, _, _)|
			f64::total_cmp(mw0, mw1));

		eprintln!(" completed.");

		eprintln!("Drawing products subset images...");

		if gen_img {
			let mut conn = db_pool.get().unwrap();

			let ent_reactions = db::model::get_reactions_with_experiment(&mut conn, &ent_experiment)
				.unwrap()
				.for_each(|ent_reaction| {
					let ent_reaction = ent_reaction.unwrap();
					let num_products = 100;

					let (mols, legends): (Vec<_>, Vec<_>) = ent_products
						.iter()
						.filter_map(|(_, prod_mol, name, _, _, _, id_reaction)|
							(*id_reaction == ent_reaction.id)
								.then_some((prod_mol, name.as_str())))
						.take(num_products)
						.unzip();

					let canvas_small = new_local!(MolDraw2DSVG);
					let mut canvas_small = canvas_small
						.init(&MolDraw2DSVGInitParams {
							width: 256,
							height: 256,
							panel_width: Some(128),
							panel_height: Some(128),
						})
						.unwrap();

					let canvas = new_local!(MolDraw2DSVG);
					let mut canvas = canvas
						.init(&MolDraw2DSVGInitParams {
							width: 2560,
							height: 2560,
							panel_width: Some(256),
							panel_height: Some(256),
						})
						.unwrap();

					let mol_cnt = mols.len();
					if mol_cnt >= 1 {
						let cnt_small = mol_cnt.min(4);

						canvas_small.draw_molecules(&mols[0..cnt_small], Some(&legends[0..cnt_small]));
						canvas.draw_molecules(&mols, Some(&legends));
					}

					canvas_small.finish_drawing();
					canvas.finish_drawing();

					let img_subsetsmall_text = canvas_small
						.get_drawing_text()
						.unwrap();
					let img_subset_text = canvas
						.get_drawing_text()
						.unwrap();

					fs::write(Path::join(path_prefix, format!("reaction{}-subset4.svg", ent_reaction.id)), img_subsetsmall_text.as_bytes())
						.expect(&format!("Failed to write products subset(4) SVG file for experiment {exp_uuid_str} and reaction {}", ent_reaction.id));
		
					fs::write(Path::join(path_prefix, format!("reaction{}-subset100.svg", ent_reaction.id)), img_subset_text.as_bytes())
						.expect(&format!("Failed to write products subset(100) SVG file for experiment {exp_uuid_str} and reaction {}", ent_reaction.id));
				});

			eprintln!(" completed.");
		} else {
			eprintln!(" skipped.");
		}

		eprintln!("Writing products...");

		file_out_zip.start_file(format!("{filename_prefix}_products.smi"), zip_opts.clone()).unwrap();

		writeln!(&mut file_out_zip, "Smiles\tName").unwrap();
		ent_products
			.iter()
			.for_each(|(_, _, _, fullname, smiles, sdf, _)| {
				writeln!(&mut file_out_zip, "{smiles}\t{fullname}")
					.expect(&format!("Failed to write product to SMILES file for experiment {exp_uuid_str}"));
			});

		file_out_zip.start_file(format!("{filename_prefix}_products.sdf"), zip_opts.clone()).unwrap();

		ent_products
			.iter()
			.for_each(|(_, _, _, fullname, smiles, sdf, _)| {
				file_out_zip.write_all(sdf.as_bytes())
					.expect(&format!("Failed to write product to SDF file for experiment {exp_uuid_str}"));
			});

		file_out_zip
			.finish()
			.unwrap();

		eprintln!(" completed.");
	});
}

pub fn gen_files_filtered_3d(thread_pool: &ThreadPool, db_pool: &db::DBPool, ent_experiment: &Experiment, prefix: &str, filename_prefix: &str, ent_experiment_postproc_filter: &ExperimentPostprocFilter) {
	let mut conn = db_pool.get().unwrap();

	let exp_uuid_str = ent_experiment.uuid.to_string();

	let path_prefix = Path::new(prefix);
	fs::create_dir(path_prefix)
		.or_else(|e| match e.kind() {
			io::ErrorKind::AlreadyExists => Ok(()),
			kind => Err(kind),
	   })
	   .expect("Failed to create the output directory");

	let zip_opts = common::default_zip_opts();

	let file_out_zip = File::create(Path::join(path_prefix, format!("{filename_prefix}_3d.zip")))
		.map_err(|_| format!("Failed to create output archive for experiment {exp_uuid_str}")).unwrap();

	let mut file_out_zip = ZipWriter::new(file_out_zip);

	thread_pool.in_place_scope(|scope| {
		let (tx, rx) = mpmc();

		scope.spawn(move |_| {
			db::model::ExportableExperimentProduct::get_with_experiment_postproc_filter(&mut conn, &ent_experiment_postproc_filter)
				.unwrap()
				.filter_map(|e| e.ok())
				.for_each(|e| {
					tx.send(e).unwrap();
				});
		});

		let mut ent_products: Vec<_> = rx
			.into_iter()
			.par_bridge()
			.map(|ent_product| -> Result<_, String> {
				let prod_mol = new_local!(ROMol);
				let mut prod_mol = prod_mol
					.init(ROMolFromPickleParams {
						pickle: &ent_product.rdpickle
					})
					.unwrap();

				prod_mol.set_prop_str("_Name", &ent_product.fullname);
				prod_mol.embed_molecule()
					.map_err(|_| format!("Failed to embed product '{}'", ent_product.fullname))?;

				let mw = prod_mol.calc_exact_mw();
				let sdf = prod_mol.to_sd().unwrap();
				let mol2 = prod_mol.to_mol2().unwrap();
	
				Ok((mw, sdf, mol2))
			})
			.filter_map(|e| {
				if let Err(err) = &e {
					eprintln!("{err}");
				}
				e.ok()
			})
			.collect();
	
		ent_products.par_sort_unstable_by(|(mw0, _, _), (mw1, _, _)|
			f64::total_cmp(mw0, mw1));

		file_out_zip.start_file(format!("{filename_prefix}_products_3d.sdf"), zip_opts.clone()).unwrap();
		ent_products
			.iter()
			.for_each(|(_, sdf, _)| {
				file_out_zip.write_all(sdf.as_bytes())
					.expect(&format!("Failed to write product to SDF file for experiment {exp_uuid_str}"));
			});

		file_out_zip.start_file(format!("{filename_prefix}_products_3d.mol2"), zip_opts.clone()).unwrap();
		ent_products
			.into_iter()
			.for_each(|(_, _, mol2)| {
				file_out_zip.write_all(mol2.as_bytes())
					.expect(&format!("Failed to write product to MOL2 file for experiment {exp_uuid_str}"));
			});

		file_out_zip.finish().unwrap();
	});
}

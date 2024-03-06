use chemodots_db as db;
use itertools::Itertools;
use rdkit_rust::graphmol::descriptors::crippen::CrippenImpl;
use rdkit_rust::graphmol::descriptors::lipinski::LipinskiImpl;
use rdkit_rust::graphmol::descriptors::mol::MolDescriptorsImpl;
use rdkit_rust::graphmol::descriptors::molsurf::MolSurfImpl;
use rdkit_rust::graphmol::moldraw2d::*;
use rdkit_rust::graphmol::moldraw2d::moldraw2dsvg::*;
use serde_json::{json, Value};

use rdkit_rust::graphmol::substruct::substructmatch::{MatchVectTypeVec, MatchVectTypeVecInitParamsFromSubstructMatch, MatchVectTypeVecImplRef};
use rdkit_rust::prelude::*;
use rdkit_rust::graphmol::chemreactions::reaction::*;
use rdkit_rust::graphmol::rwmol::*;
use rdkit_rust::*;

pub fn mol_info(frag_mol: &str) {
	let frag = new_local!(RWMol);
	let frag = frag.init(ParseMolBlockParams {
			mol_block: frag_mol,
			sanitize: Default::default(),
			remove_hs: Default::default(),
			strict_parsing: Default::default(),
		})
		.unwrap();

	let drawer = new_local!(MolDraw2DSVG);
	let mut drawer = drawer.init(&MolDraw2DSVGInitParams {
		width: 256,
		height: 256,
		panel_width: Some(256),
		panel_height: Some(256),
	}).unwrap();

	drawer.draw_molecule(&frag, None);
	drawer.finish_drawing();
	let frag_svg = drawer.get_drawing_text();

	let mw = frag.calc_exact_mw();
	let hba = frag.calc_num_hba();
	let hbd = frag.calc_num_hbd();
	let rot = frag.calc_num_rotatable_bonds();
	let clogp = frag.calc_clogp();
	let tpsa = frag.calc_tpsa();
	let chiral = frag.calc_num_atom_stereo_centers();

	let res_json = json!({
		"frag_svg": frag_svg,
		"descs": {
			"mw": mw,
			"hba": hba,
			"hbd": hbd,
			"rot": rot,
			"clogp": clogp,
			"tpsa": tpsa,
			"chiral": chiral,
		},
	});

	println!("{}", res_json.to_string());
}

pub fn compatible_reactions_probe(db_pool: &db::DBPool, frag_mol: &str, idx_atoms: &[i32]) {
	let mut conn = db_pool.get().unwrap();

	let frag = new_local!(RWMol);
	let frag = frag.init(ParseMolBlockParams {
			mol_block: frag_mol,
			sanitize: Default::default(),
			remove_hs: Default::default(),
			strict_parsing: Default::default(),
		})
		.unwrap();

	let json_reactions = db::model::Reaction::get_all(&mut conn).unwrap()
		.filter_map(|e| e.ok())
		.filter(|ent_reaction| -> bool {
			let reaction = new_local!(ChemicalReaction);

			let reaction = reaction.init(ChemicalReactionFromPickleParams {
					pickle: &ent_reaction.rdpickle,
				})
				.unwrap();

			let reactants = reaction.get_reactants();
			let reactant_count = reactants.size();

			for reactant_idx in 0..reactant_count {
				let reactant = reactants.get(reactant_idx).unwrap();

				let matches = new_local!(MatchVectTypeVec);
				let matches = matches
					.init(&MatchVectTypeVecInitParamsFromSubstructMatch::new(&frag, &reactant));
				if matches.is_err() {
					continue;
				}
				let matches = matches.unwrap();
				let entry_count = matches.len();

				for idx_entry in 0..entry_count {
					let pair_count = matches.entry_len(idx_entry);

					for idx_pair in 0..pair_count {
						let (_, idx_frag_atom) = matches.entry_get_atom_pair(idx_entry, idx_pair).unwrap();

						let found = idx_atoms.iter().contains(&idx_frag_atom);
						if found {
							return true;
						}
					}
				}
			}

			return false;
		})
		.map(|ent_reaction| {
			json!({
				"id": ent_reaction.id,
				"name": ent_reaction.name,
			})
		})
		.collect_vec();

	let res_json = Value::Array(json_reactions);

	println!("{}", res_json.to_string());
}

use std::io::{self, Write};
use std::ops::RangeInclusive;

use chrono::Utc;
use common::slugify;
use crossbeam::channel::unbounded as mpsc;
use db::model::{NewBuildingBlock, NewCompound, NewCompoundProvider, NewBuildingBlockReactant, NewMoiety, NewMoietyGroup, NewReaction, NewBuildingBlockOrigin};
use field_count::FieldCount;
use itertools::Itertools;
use rayon::{prelude::*, ThreadPool};
use rdkit_rust::*;
use rdkit_rust::graphmol::atom::*;
use rdkit_rust::graphmol::chemreactions::reaction::*;
use rdkit_rust::graphmol::descriptors::prelude::*;
use rdkit_rust::graphmol::molops::prelude::*;
use rdkit_rust::graphmol::molstandardize::prelude::*;
use rdkit_rust::graphmol::romol::*;
use rdkit_rust::graphmol::rwmol::*;
use rdkit_rust::graphmol::substruct::substructmatch::*;
use rdkit_rust::prelude::*;

use chemodots_db as db;
use chemodots_common as common;

#[derive(Debug)]
pub enum StandardizeError {
	ChooseLargestFragment,
	ValidateAllowedAtoms,
	ValidateHAC,
	RemoveHs,
	Uncharge,
	Reionize,
	AssignStereo,
	ValidateNumUnspecStereo,
	ValidateNumRotatableBonds,
	ValidateNumRings,
	ValidateRingSize,
}

impl std::fmt::Display for StandardizeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}

pub struct ChemodotsStandardizer<'s> {
	largest_fragment_chooser: InitializedHeap<'s, LargestFragmentChooser>,
	allowed_atoms: Vec<InitializedHeap<'s, Atom>>,
	range_hac: RangeInclusive<u32>,
	remove_hs_params: RemoveHsParameters,
	uncharger: InitializedHeap<'s, Uncharger>,
	reionizer: InitializedHeap<'s, Reionizer>,
	range_num_unspec_stereo: RangeInclusive<u32>,
	range_num_rot_bonds: RangeInclusive<u32>,
	range_num_rings: RangeInclusive<u32>,
	range_ring_size: RangeInclusive<u32>,
}

impl ChemodotsStandardizer<'_> {
	pub fn new() -> Self {
		let atom_symbols = [
			"H",
			"C",
			"N",
			"O",
			"P",
			"S",

			// Halogens
			"F",
			"Cl",
			"Br",
			"I",

			// Others
			"B",
			"Sn",
			// "Si",
		];
		let atoms = atom_symbols
			.iter()
			.map(|atom_symbol| Atom::new(&AtomInitParamsSymbol { symbol: atom_symbol }))
			.filter_map(|atom| atom.ok())
			.collect_vec();

		let cleanup_params = new_local!(CleanupParameters);
		let cleanup_params = cleanup_params.init(&CleanupParametersInitParams {
				prefer_organic: Some(true),
				do_canonical: Some(true),
				max_tautomers: Some(1000),
				max_transforms: Some(1000),
				tautomer_remove_sp3_stereo: Some(false),
				tautomer_remove_bond_stereo: Some(true),
				tautomer_remove_isotopic_hs: Some(true),
				tautomer_reassign_stereo: Some(true),
				largest_fragment_chooser_use_atom_count: Some(true),
				largest_fragment_chooser_count_heavy_atoms_only: Some(false),
				..Default::default()
			})
			.unwrap();

		Self {
			largest_fragment_chooser: LargestFragmentChooser::new(&LargestFragmentChooserInitParams {
					cleanup_params: &cleanup_params,
				})
				.unwrap(),
			allowed_atoms: atoms,
			range_hac: 5..=24,
			remove_hs_params: RemoveHsParameters {
				remove_degree_zero: true,
				remove_higher_degrees: false,
				remove_only_h_neighbors: true,
				remove_isotopes: true,
				remove_and_track_isotopes: true,
				remove_dummy_neighbors: true,
				remove_defining_bond_stereo: true,
				remove_with_wedged_bond: true,
				remove_with_query: false,
				remove_mapped: true,
				remove_in_s_groups: true,
				show_warnings: false,
				remove_non_implicit: true,
				update_explicit_count: false,
				remove_hydrides: true,
				remove_non_tetrahedral_neighbors: true,
				..Default::default()
			},
			uncharger: Uncharger::new(()).unwrap(),
			reionizer: Reionizer::new(()).unwrap(),
			range_num_unspec_stereo: 0..=1,
			range_num_rot_bonds: 0..=16,
			range_num_rings: 0..=3,
			range_ring_size: 0..=7,
		}
	}

	pub fn standardize<'s>(&self, mol: InitializedLocal<'s, RWMol>) -> Result<InitializedLocal<'s, RWMol>, StandardizeError>
	{
		let romol = self.largest_fragment_chooser.choose(&mol)
			.map_err(|_| StandardizeError::ChooseLargestFragment)?;
		let mut mol = mol
			.uninit()
			.init(RWMolInitParamsROMol {
				romol: &romol,
			})
			.unwrap();

		let atom_count = mol.get_num_atoms();
		for idx in 0..atom_count {
			let atom = mol.get_atom(idx).unwrap();

			let mut matched = false;

			for query_atom in &self.allowed_atoms {
				// Each explicit (non-default) property of query is matched against atom
				matched = query_atom.match_atom(&atom)
					// We also enforce that atom is not an explicit isotope
					&& atom.get_isotope() == 0;

				if matched {
					break;
				}
			}

			if !matched {
				Err(StandardizeError::ValidateAllowedAtoms)?
			}
		}

		let hac = mol.get_num_heavy_atoms();
		if !self.range_hac.contains(&hac) {
			Err(StandardizeError::ValidateHAC)?;
		}

		mol.remove_hs(&self.remove_hs_params, Some(true))
			.map_err(|_| StandardizeError::RemoveHs)?;

		self.uncharger.uncharge(&mut mol)
			.map_err(|_| StandardizeError::Uncharge)?;

		self.reionizer.reionize(&mut mol)
			.map_err(|_| StandardizeError::Reionize)?;

		mol.assign_stereochemistry()
			.map_err(|_| StandardizeError::AssignStereo)?;

		let num_unspec_stereo = mol.calc_num_unspecified_atom_stereo_centers();
		if !self.range_num_unspec_stereo.contains(&num_unspec_stereo) {
			Err(StandardizeError::ValidateNumUnspecStereo)?;
		}

		let num_rot_bonds = mol.calc_num_rotatable_bonds();
		if !self.range_num_rot_bonds.contains(&num_rot_bonds) {
			Err(StandardizeError::ValidateNumRotatableBonds)?;
		}

		let num_rings = mol.calc_num_rings();
		if !self.range_num_rings.contains(&num_rings) {
			Err(StandardizeError::ValidateNumRings)?;
		}

		// TODO: Ring size

		Ok(mol)
	}
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum Provider {
	Enamine,
	Molport,
}

fn boostrap(db_pool: &db::DBPool) -> Result<(), ()> {
	let mut conn = db_pool.get().unwrap();

	// Moieties

	println!("  Bootstrapping moieties...");

	let ent_moiety_group = db::model::create_moiety_group(&mut conn, &NewMoietyGroup {
		name: "Primary arylamine",
	}).unwrap();

	db::model::create_moiety(&mut conn, &NewMoiety {
		name: "Primary arylamine",
		id_moiety_group: ent_moiety_group.id,
		priority: 0,
		rdpickle: &Vec::<u8>::new(),
		smarts: "",
	}).unwrap();

	println!("   completed.");

	// Reactions

	println!("  Bootstrapping reactions...");

	let react_defs = [
		("Hartenfeller 1: Pictet-Spengler", "[cH1:1]1:[c:2](-[CH2:7]-[CH2:8]-[NH2:9]):[c:3]:[c:4]:[c:5]:[c:6]:1.[#6:11]-[CH1;R0:10]=[OD1]>>[c:1]12:[c:2](-[CH2:7]-[CH2:8]-[NH1:9]-[C:10]-2(-[#6:11])):[c:3]:[c:4]:[c:5]:[c:6]:1"),
		("Hartenfeller 2: Benzimidazole derivatives carboxylic-acid/ester", "[c;r6:1](-[NH1;$(N-[#6]):2]):[c;r6:3](-[NH2:4]).[#6:6]-[C;R0:5](=[OD1])-[#8;H1,$(O-[CH3])]>>[c:3]2:[c:1]:[n:2]:[c:5](-[#6:6]):[n:4]@2"),
		("Hartenfeller 3: Benzimidazole derivatives aldehyde", "[c;r6:1](-[NH1;$(N-[#6]):2]):[c;r6:3](-[NH2:4]).[#6:6]-[CH1;R0:5](=[OD1])>>[c:3]2:[c:1]:[n:2]:[c:5](-[#6:6]):[n:4]@2"),
		("Hartenfeller 4: Benzothiazole", "[c;r6:1](-[SH1:2]):[c;r6:3](-[NH2:4]).[#6:6]-[CH1;R0:5](=[OD1])>>[c:3]2:[c:1]:[s:2]:[c:5](-[#6:6]):[n:4]@2"),
		("Hartenfeller 5: Benzoxazole arom-aldehyde", "[c:1](-[OH1;$(Oc1ccccc1):2]):[c;r6:3](-[NH2:4]).[c:6]-[CH1;R0:5](=[OD1])>>[c:3]2:[c:1]:[o:2]:[c:5](-[c:6]):[n:4]@2"),
		("Hartenfeller 6: Benzoxazole carboxylic-acid", "[c;r6:1](-[OH1:2]):[c;r6:3](-[NH2:4]).[#6:6]-[C;R0:5](=[OD1])-[OH1]>>[c:3]2:[c:1]:[o:2]:[c:5](-[#6:6]):[n:4]@2"),
		("Hartenfeller 7: Thiazole", "[#6:6]-[C;R0:1](=[OD1])-[CH1;R0:5](-[#6:7])-[*;#17,#35,#53].[NH2:2]-[C:3]=[SD1:4]>>[c:1]2(-[#6:6]):[n:2]:[c:3]:[s:4][c:5]([#6:7]):2"),
		("Hartenfeller 8: Niementowski quinazoline", "[c:1](-[C;$(C-c1ccccc1):2](=[OD1:3])-[OH1]):[c:4](-[NH2:5]).[N;!H0;!$(N-N);!$(N-C=N);!$(N(-C=O)-C=O):6]-[C;H1,$(C-[#6]):7]=[OD1]>>[c:4]2:[c:1]-[C:2](=[O:3])-[N:6]-[C:7]=[N:5]-2"),
		("Placeholder 9", ">>"),
		("Hartenfeller 10: Tetrazole connect regioisomere 1", "[CH0;$(C-[#6]):1]#[NH0:2].[C;A;!$(C=O):3]-[*;#17,#35,#53]>>[C:1]1=[N:2]-N(-[C:3])-N=N-1"),
		("Hartenfeller 11: Tetrazole connect regioisomere 2", "[CH0;$(C-[#6]):1]#[NH0:2].[C;A;!$(C=O):3]-[*;#17,#35,#53]>>[C:1]1=[N:2]-N=N-N-1(-[C:3])"),
		("Hartenfeller 12: Huisgen Cu-catalyzed 1,4-subst", "[CH0;$(C-[#6]):1]#[CH1:2].[C;H1,H2;A;!$(C=O):3]-[*;#17,#35,#53,OH1]>>[C:1]1=[C:2]-N(-[C:3])-N=N-1"),
		("Hartenfeller 13: Huisgen Ru-catalyzed 1,5 subst", "[CH0;$(C-[#6]):1]#[CH1:2].[C;H1,H2;A;!$(C=O):3]-[*;#17,#35,#53,OH1]>>[C:1]1=[C:2]-N=NN(-[C:3])-1"),
		("Hartenfeller 14: Huisgen disubst-alkyne", "[CH0;$(C-[#6]):1]#[CH0;$(C-[#6]):2].[C;H1,H2;A;!$(C=O):3]-[*;#17,#35,#53,OH1]>>[C:1]1=[C:2]-N=NN(-[C:3])-1"),
		("Hartenfeller 15: 1,2,4-triazole acetohydrazide", "[CH0;$(C-[#6]):1]#[NH0:2].[NH2:3]-[NH1:4]-[CH0;$(C-[#6]);R0:5]=[OD1]>>[N:2]1-[C:1]=[N:3]-[N:4]-[C:5]=1"),
		("Hartenfeller 16: 1,2,4-triazole carboxylic-acid/ester", "[CH0;$(C-[#6]):1]#[NH0:2].[CH0;$(C-[#6]);R0:5](=[OD1])-[#8;H1,$(O-[CH3]),$(O-[CH2]-[CH3])]>>[N:2]1-[C:1]=N-N-[C:5]=1"),
		("Placeholder 17", ">>"),
		("Hartenfeller 18: Spiro-chromanone", "[c:1](-[C;$(C-c1ccccc1):2](=[OD1:3])-[CH3:4]):[c:5](-[OH1:6]).[C;$(C1-[CH2]-[CH2]-[N,C]-[CH2]-[CH2]-1):7](=[OD1])>>[O:6]1-[c:5]:[c:1]-[C:2](=[OD1:3])-[C:4]-[C:7]-1"),
		("Hartenfeller 19: Pyrazole", "[#6;!$([#6](-C=O)-C=O):4]-[CH0:1](=[OD1])-[C;H1&!$(C-[*;!#6])&!$(C-C(=O)O),H2:2]-[CH0;R0:3](=[OD1])-[#6;!$([#6](-C=O)-C=O):5].[NH2:6]-[N;!H0;$(N-[#6]),H2:7]>>[C:1]1(-[#6:4])-[C:2]=[C:3](-[#6:5])-[N:7]-[N:6]=1"),
		("Hartenfeller 20: Phthalazinone", "[c;r6:1](-[C;$(C=O):6]-[OH1]):[c;r6:2]-[C;H1,$(C-C):3]=[OD1].[NH2:4]-[NH1;$(N-[#6]);!$(NC=[O,S,N]):5]>>[c:1]1:[c:2]-[C:3]=[N:4]-[N:5]-[C:6]-1"),
		("Hartenfeller 21: Paal-Knorr pyrrole", "[#6:5]-[C;R0:1](=[OD1])-[C;H1,H2:2]-[C;H1,H2:3]-[C:4](=[OD1])-[#6:6].[NH2;$(N-[C,N]);!$(NC=[O,S,N]);!$(N([#6])[#6]);!$(N~N~N):7]>>[C:1]1(-[#6:5])=[C:2]-[C:3]=[C:4](-[#6:6])-[N:7]-1"),
		("Hartenfeller 22: Triaryl-imidazole", "[C;$(C-c1ccccc1):1](=[OD1])-[C;D3;$(C-c1ccccc1):2]~[O;D1,H1].[CH1;$(C-c):3]=[OD1]>>[C:1]1-N=[C:3]-[NH1]-[C:2]=1"),
		("Hartenfeller 23: Fischer indole", "[NH1;$(N-c1ccccc1):1](-[NH2])-[c:5]:[cH1:4].[C;$(C([#6])[#6]):2](=[OD1])-[CH2;$(C([#6])[#6]);!$(C(C=O)C=O):3]>>[C:5]1-[N:1]-[C:2]=[C:3]-[C:4]:1"),
		("Hartenfeller 24: Friedlaender chinoline", "[NH2;$(N-c1ccccc1):1]-[c:2]:[c:3]-[CH1:4]=[OD1].[C;$(C([#6])[#6]):6](=[OD1])-[CH2;$(C([#6])[#6]);!$(C(C=O)C=O):5]>>[N:1]1-[c:2]:[c:3]-[C:4]=[C:5]-[C:6]:1"),
		("Hartenfeller 25: Benzofuran", "[*;Br,I;$(*c1ccccc1)]-[c:1]:[c:2]-[OH1:3].[CH1:5]#[C;$(C-[#6]):4]>>[c:1]1:[c:2]-[O:3]-[C:4]=[C:5]-1"),
		("Hartenfeller 26: Benzothiophene", "[*;Br,I;$(*c1ccccc1)]-[c:1]:[c:2]-[SD2:3]-[CH3].[CH1:5]#[C;$(C-[#6]):4]>>[c:1]1:[c:2]-[S:3]-[C:4]=[C:5]-1"),
		("Hartenfeller 27: Indole", "[*;Br,I;$(*c1ccccc1)]-[c:1]:[c:2]-[NH2:3].[CH1:5]#[C;$(C-[#6]):4]>>[c:1]1:[c:2]-[N:3]-[C:4]=[C:5]-1"),
		("Hartenfeller 28: Oxadiazole", "[#6:6][C:5]#[#7;D1:4].[#6:1][C:2](=[OD1:3])[OH1]>>[#6:6][c:5]1[n:4][o:3][c:2]([#6:1])n1"),
		("Hartenfeller 29: Williamson ether", "[#6;$([#6]~[#6]);!$([#6]=O):2][#8;H1:3].[Cl,Br,I][#6;H2;$([#6]~[#6]):4]>>[CH2:4][O:3][#6:2]"),
		("Hartenfeller 30: Reductive amination", "[#6:4]-[C;H1,$([CH0](-[#6])[#6]):1]=[OD1].[N;H2,$([NH1;D2](C)C);!$(N-[#6]=[*]):3]-[C:5]>>[#6:4][C:1]-[N:3]-[C:5]"),
		("Hartenfeller 31: Suzuki", "[#6;H0;D3;$([#6](~[#6])~[#6]):1]B(O)O.[#6;H0;D3;$([#6](~[#6])~[#6]):2][Cl,Br,I]>>[#6:2][#6:1]"),
		("Hartenfeller 32: Piperidine indole", "[c;H1:3]1:[c:4]:[c:5]:[c;H1:6]:[c:7]2:[nH:8]:[c:9]:[c;H1:1]:[c:2]:1:2.O=[C:10]1[#6;H2:11][#6;H2:12][N:13][#6;H2:14][#6;H2:15]1>>[#6;H2:12]3[#6;H1:11]=[C:10]([c:1]1:[c:9]:[n:8]:[c:7]2:[c:6]:[c:5]:[c:4]:[c:3]:[c:2]:1:2)[#6;H2:15][#6;H2:14][N:13]3"),
		("Hartenfeller 33: Negishi", "[#6;$([#6]~[#6]);!$([#6]~[S,N,O,P]):1][Cl,Br,I].[Cl,Br,I][#6;$([#6]~[#6]);!$([#6]~[S,N,O,P]):2]>>[#6:2][#6:1]"),
		("Hartenfeller 34: Mitsunobu imide", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[NH1;$(N(C=O)C=O):2]>>[C:1][N:2]"),
		("Hartenfeller 35: Mitsunobu phenole", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[OH1;$(Oc1ccccc1):2]>>[C:1][O:2]"),
		("Hartenfeller 36: Mitsunobu sulfonamide", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[NH1;$(N([#6])S(=O)=O):2]>>[C:1][N:2]"),
		("Placeholder 37", ">>"),
		("Placeholder 38", ">>"),
		("Placeholder 39", ">>"),
		("Placeholder 40", ">>"),
		("Hartenfeller 41: Heck terminal vinyl", "[#6;c,$(C(=O)O),$(C#N):3][#6;H1:2]=[#6;H2:1].[#6;$([#6]=[#6]),$(c:c):4][Cl,Br,I]>>[#6:4]/[#6:1]=[#6:2]/[#6:3]"),
		("Hartenfeller 42: Heck non-terminal vinyl", "[#6;c,$(C(=O)O),$(C#N):3][#6:2]([#6:5])=[#6;H1;$([#6][#6]):1].[#6;$([#6]=[#6]),$(c:c):4][Cl,Br,I]>>[#6:4][#6;H0:1]=[#6:2]([#6:5])[#6:3]"),
		("Hartenfeller 43: Stille", "[#6;$(C=C-[#6]),$(c:c):1][Br,I].[Cl,Br,I][c:2]>>[c:2][#6:1]"),
		("Hartenfeller 44: Grignard carbonyl", "[#6:1][C:2]#[#7;D1].[Cl,Br,I][#6;$([#6]~[#6]);!$([#6]([Cl,Br,I])[Cl,Br,I]);!$([#6]=O):3]>>[#6:1][C:2](=O)[#6:3]"),
		("Hartenfeller 45: Grignard alcohol", "[#6:1][C;H1,$([C]([#6])[#6]):2]=[OD1:3].[Cl,Br,I][#6;$([#6]~[#6]);!$([#6]([Cl,Br,I])[Cl,Br,I]);!$([#6]=O):4]>>[C:1][#6:2]([OH1:3])[#6:4]"),
		("Hartenfeller 46: Sonogashira", "[#6;$(C=C-[#6]),$(c:c):1][Br,I].[CH1;$(C#CC):2]>>[#6:1][C:2]"),
		("Hartenfeller 47: Schotten-Baumann amide", "[C;$(C=O):1][OH1].[N;$(N[#6]);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[O,N]);!$(N[C,S]=[S,O,N]):2]>>[C:1][N+0:2]"),
		//("Hartenfeller 48: Sulfonamide", "[S;$(S(=O)(=O)[C,N]):1][Cl].[N;$(NC);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[c,O]);!$(N[C,S]=[S,O,N]):2]>>[S:1][N+0:2]"),
		("Hartenfeller 48: Sulfonamide", "[S;$(S(=O)(=O)[#6,N]):1][Cl,OH,O-].[N;$(N[#6]);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(NO);!$(N[C,S]=[S,O,N]):2]>>[S:1][N+0:2]"),
		("Hartenfeller 49: N-arylation heterocycles", "[c:1]B(O)O.[nH1;+0;r5;!$(n[#6]=[O,S,N]);!$(n~n~n);!$(n~n~c~n);!$(n~c~n~n):2]>>[c:1][n:2]"),
		("Hartenfeller 50: Wittig", "[#6:3]-[C;H1,$([CH0](-[#6])[#6]);!$(CC=O):1]=[OD1].[Cl,Br,I][C;H2;$(C-[#6]);!$(CC[I,Br]);!$(CCO[CH3]):2]>>[C:3][C:1]=[C:2]"),
		("Hartenfeller 51: Buchwald-Hartwig", "[Cl,Br,I][c;$(c1:[c,n]:[c,n]:[c,n]:[c,n]:[c,n]:1):1].[N;$(NC)&!$(N=*)&!$([N-])&!$(N#*)&!$([ND3])&!$([ND4])&!$(N[c,O])&!$(N[C,S]=[S,O,N]),H2&$(Nc1:[c,n]:[c,n]:[c,n]:[c,n]:[c,n]:1):2]>>[c:1][N:2]"),
		("Hartenfeller 52: Imidazole", "[C;$(C([#6])[#6;!$([#6]Br)]):4](=[OD1])[CH;$(C([#6])[#6]):5]Br.[#7;H2:3][C;$(C(=N)(N)[c,#7]):2]=[#7;H1;D1:1]>>[C:4]1=[CH0:5][NH:3][C:2]=[N:1]1"),
		("Hartenfeller 53: Decarboxylative coupling", "[c;$(c1[c;$(c[C,S,N](=[OD1])[*;R0;!OH1])]cccc1):1][C;$(C(=O)[O;H1])].[c;$(c1aaccc1):2][Cl,Br,I]>>[c:1][c:2]"),
		("Hartenfeller 54: Heteroaromatic nuc sub", "[c;!$(c1ccccc1);$(c1[n,c]c[n,c]c[n,c]1):1][Cl,F].[N;$(NC);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[c,O]);!$(N[C,S]=[S,O,N]):2]>>[c:1][N:2]"),
		("Hartenfeller 55: Nucl sub aromatic ortho nitro", "[c;$(c1c(N(~O)~O)cccc1):1][Cl,F].[N;$(NC);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[c,O]);!$(N[C,S]=[S,O,N]):2]>>[c:1][N:2]"),
		("Hartenfeller 56: Nucl sub aromatic para nitro", "[c;$(c1ccc(N(~O)~O)cc1):1][Cl,F].[N;$(NC);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[c,O]);!$(N[C,S]=[S,O,N]):2]>>[c:1][N:2]"),
		("Hartenfeller 57: Urea", "[N;$(N-[#6]):3]=[C;$(C=O):1].[N;$(N[#6]);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[O,N]);!$(N[C,S]=[S,O,N]):2]>>[N:3]-[C:1]-[N+0:2]"),
		("Hartenfeller 58: Thiourea", "[N;$(N-[#6]):3]=[C;$(C=S):1].[N;$(N[#6]);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[O,N]);!$(N[C,S]=[S,O,N]):2]>>[N:3]-[C:1]-[N+0:2]"),
		("Placeholder 59", ">>"),
		("Placeholder 60", ">>"),
		("iSCB 61: Benzimidazole aldehyde (fuzzy)", "[c;r6:1](-[NH2:2]):[c;r6:3](-[NH2:4]).[#6:6]-[CH1;R0:5](=[OD1])>>[c:3]2:[c:1]:[nH:2]:[c:5](-[#6:6]):[n:4]@2"),
		("iSCB 62: Benzimidazole carboxylic-acid-ester (fuzzy)", "[c;r6:1](-[NH2:2]):[c;r6:3](-[NH2:4]).[#6:6]-[C;R0:5](=[OD1])-[#8;H1,$(O-[CH3])]>>[c:3]2:[c:1]:[nH:2]:[c:5](-[#6:6]):[n:4]@2"),
		("iSCB 63: Williamson-like alcohol", "[CH2;$([#6]~[#6]):1][Br,I].[#8;H1:2][#6;$([#6]~[#6]);!$([#6]=O):3]>>[C:1][O:2][#6:3]"),
		("iSCB 64: Williamson-like thiol", "[CH2;$([#6]~[#6]):1][Br,I].[#16;H1:2][#6;$([#6]~[#6]);!$([#6]=O):3]>>[C:1][S:2][#6:3]"),
		("iSCB 65: Williamson-like amine", "[CH2;$([#6]~[#6]):1][Br,I].[#7;H2:2][#6;$([#6]~[#6]);!$([#6]=O):3]>>[C:1][N:2][#6:3]"),
		("iSCB 66: Stille organo-stannane", "[c:1][Sn](C)(C)(C).[Cl,Br,I][c:2]>>[c:1][c:2]"),
		("iSCB 67: Sulfonyl ester", "[S;$(S(=O)(=O)[#6]):1][Cl,Br].[OH;$(O[#6]);!$(OC=O):2]>>[S:1][O:2]"),
		("iSCB 68: Amide acyl-chloride", "[C;$(C=O):1][Cl,Br].[N;$(N[#6]);!$(N=*);!$([N-]);!$(N#*);!$([ND3]);!$([ND4]);!$(N[O,N]);!$(N[C,S]=[S,O,N]):2]>>[C:1][N+0:2]"),
		("iSCB 69: Heck terminal vinyl (fuzzy)", "[c:3][#6;H1:2]=[#6;H2:1].[#6;$([#6]=[#6]),$(c:c):4][Cl,Br,I]>>[#6:4]/[#6:1]=[#6:2]/[#6:3]"),
		("iSCB 70: Ester", "[C;$(C=O):1][OH1].[OH1;$(O[#6]);!$(OC=O):2]>>[C:1][O:2]"),

		// Disabled reactions

		// Single reactant
		// ("Hartenfeller 9: Tetrazole terminal", "[CH0;$(C-[#6]):1]#[NH0:2]>>[C:1]1=[N:2]-N-N=N-1"),
		// ("Hartenfeller 17: 3-nitrile-pyridine", "[#6;!$([#6](-C=O)-C=O):4]-[CH0:1](=[OD1])-[C;H1&!$(C-[*;!#6])&!$(C-C(=O)O),H2:2]-[CH0;R0:3](=[OD1])-[#6;!$([#6](-C=O)-C=O):5]>>[c:1]1(-[#6:4]):[c:2]:[c:3](-[#6:5]):n:c(-O):c(-C#N):1"),

		// Too many products
		// ("Hartenfeller 37: Mitsunobu tetrazole 1", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[#7H1:2]1~[#7:3]~[#7:4]~[#7:5]~[#6:6]~1>>[C:1][#7:2]1:[#7:3]:[#7:4]:[#7:5]:[#6:6]:1"),
		// ("Hartenfeller 38: Mitsunobu tetrazole 2", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[#7H1:2]1~[#7:3]~[#7:4]~[#7:5]~[#6:6]~1>>[#7H0:2]1:[#7:3]:[#7H0:4]([C:1]):[#7:5]:[#6:6]:1"),
		// ("Hartenfeller 39: Mitsunobu tetrazole 3", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[#7:2]1~[#7:3]~[#7H1:4]~[#7:5]~[#6:6]~1>>[C:1][#7H0:2]1:[#7:3]:[#7H0:4]:[#7:5]:[#6:6]:1"),
		// ("Hartenfeller 40: Mitsunobu tetrazole 4", "[C;H1&$(C([#6])[#6]),H2&$(C[#6]):1][OH1].[#7:2]1~[#7:3]~[#7H1:4]~[#7:5]~[#6:6]~1>>[#7:2]1:[#7:3]:[#7:4]([C:1]):[#7:5]:[#6:6]:1"),
	];

	react_defs
		.into_iter()
		.for_each(|(name, smarts)| {
			let react = new_local!(ChemicalReaction);
			let react = react
				.init(ParseSmartsParamsChemicalReaction {
					text: smarts,
				})
				.unwrap();
			let pickle = react.to_pickle().unwrap();
			let slug = &slugify(name
				.split_once(':')
				.map(|(prefix, _)| prefix)
				.unwrap_or(name));

			db::model::create_reaction(&mut conn, &NewReaction {
				name,
				slug,
				smarts,
				rdpickle: &pickle,
				multistep: false,
				reference: None,
			}).unwrap();
		});

	// Providers

	println!("  Bootstrapping providers...");

	let ent_provider_enamine = db::model::get_compound_provider_by_name(&mut conn, "Enamine")
		.or_else(|_| db::model::create_compound_provider(&mut conn, &NewCompoundProvider {
			name: "Enamine",
			ts_upd: Some(Utc::now().naive_utc())
		})).unwrap();

	let ent_provider_molport = db::model::get_compound_provider_by_name(&mut conn, "MolPort")
		.or_else(|_| db::model::create_compound_provider(&mut conn, &NewCompoundProvider {
			name: "MolPort",
			ts_upd: Some(Utc::now().naive_utc())
		})).unwrap();

	println!("   completed.");

	Ok(())
}

fn remove_prev_compounds(db_pool: &db::DBPool) -> Result<(), ()> {
	let mut conn = db_pool.get().unwrap();

	db::model::delete_all_building_block_origins(&mut conn).unwrap();
	db::model::delete_all_compounds(&mut conn).unwrap();

	Ok(())
}

fn import_new_compounds(db_pool: &db::DBPool, thread_pool: &ThreadPool) -> Result<(), ()> {
	let mut conn = db_pool.get().unwrap();
	let cpu_cnt = num_cpus::get();

	let standardizer = ChemodotsStandardizer::new();

	let ent_provider_enamine = db::model::get_compound_provider_by_name(&mut conn, "Enamine").unwrap();
	let ent_provider_molport = db::model::get_compound_provider_by_name(&mut conn, "MolPort").unwrap();

	thread_pool.in_place_scope(|scope| {
		let (tx, rx) = mpsc();

		// Fetch and standardize the compounds

		scope.spawn(move |_scope| {
			println!("  Fetching and standardizing the compounds...");

			let suppl_molport = new_local!(MultithreadedSDMolSupplier);
			let mut suppl_molport = suppl_molport.init(MultithreadedSDMolSupplierInitParamsFilenameEx {
					filename: "in-molport.sdf",
					sanitize: Some(false),
					remove_hs: Some(false),
					strict_parsing: Some(true),
					num_writer_threads: Some(cpu_cnt),
					size_input_queue: Some(128 * cpu_cnt),
					size_output_queue: Some(128 * cpu_cnt),
				})
				.unwrap();

			let suppl_enamine = new_local!(MultithreadedSDMolSupplier);
			let mut suppl_enamine = suppl_enamine.init(MultithreadedSDMolSupplierInitParamsFilenameEx {
					filename: "in-enamine.sdf",
					sanitize: Some(false),
					remove_hs: Some(false),
					strict_parsing: Some(true),
					num_writer_threads: Some(cpu_cnt),
					size_input_queue: Some(128 * cpu_cnt),
					size_output_queue: Some(128 * cpu_cnt),
				})
				.unwrap();

			suppl_molport
				.par_bridge()
				.map(|e| (Provider::Molport, e))
				.chain(suppl_enamine
					.par_bridge()
					.map(|e| (Provider::Enamine, e)))
				.map(|(provider, mol)| -> Result<_, String> {
					let rwmol = new_local!(RWMol);
					let mut mol = rwmol
						.init(RWMolInitParamsROMol {
							romol: &mol,
						})
						.unwrap();

					let mut refid = mol
						.get_prop_str("PUBCHEM_EXT_DATASOURCE_REGID")
						.filter(|refid| !refid.is_empty())
						.or_else(|| mol
							.get_prop_str("ID")
							.filter(|refid| !refid.is_empty()))
						.or_else(|| mol
							.get_prop_str("Id")
							.filter(|refid| !refid.is_empty()))
						.or_else(|| mol
							.get_prop_str("id")
							.filter(|refid| !refid.is_empty()))
						.ok_or(format!("Compound {provider:?}<unknown>: Failed to get compound name"))?;

					match provider {
						Provider::Molport => {
							if let Some(stripped) = refid.strip_prefix("MolPort-") {
								refid = stripped.to_owned();
							}
						},
						_ => {},
					}

					mol.set_prop_str("_Name", &refid);

					let mol = standardizer
						.standardize(mol)
						.map_err(|err| match err {
							StandardizeError::ChooseLargestFragment => "Failed to choose the largest fragment",
							StandardizeError::ValidateAllowedAtoms => "Forbidden atoms found",
							StandardizeError::ValidateHAC => "Heavy atoms count out of bounds",
							StandardizeError::RemoveHs => "Failed to remove hydrogens",
							StandardizeError::Uncharge => "Failed to uncharge",
							StandardizeError::Reionize => "Failed to reionize",
							StandardizeError::AssignStereo => "Failed to assign stereochemistry",
							StandardizeError::ValidateNumUnspecStereo => "Unspecified stereo centers count out of bounds",
							StandardizeError::ValidateNumRotatableBonds => "Rotatable bonds count out of bounds",
							StandardizeError::ValidateNumRings => "Rings (SSSR) count out of bounds",
							StandardizeError::ValidateRingSize => "At least one ring size out of bounds",
						})
						.map_err(|msg| format!("Compound {provider:?}.{refid}: {msg}"))?;

					let mol = ROMol::new(ROMolInitParamsROMol {
							romol: &mol,
						})
						.unwrap();

					let smiles = mol
						.to_smiles()
						.map_err(|_| format!("Compound {provider:?}.{refid}: Failed to generate SMILES"))?;

					Ok(((provider, refid), (smiles, mol)))
				})
				.filter_map(|e| {
					if let Err(err) = &e {
						eprintln!("{err}");
					}
					e.ok()
				})
				.for_each_with(tx, |s, elem| {
					s.send(elem).unwrap();
				});

			println!("   completed.");
		});

		// Handle duplicate compound identifiers

		let mut grouped_refid = rx
			.into_iter()
			.into_group_map();

		let (tx, rx) = mpsc();

		scope.spawn(move |_scope| {
			println!("  Removing compounds with duplicate identifiers...");

			grouped_refid
				.par_drain()
				.filter_map(|((provider, refid), v)| {
					let unique_count = match v.len() {
						len if len <= 1 => len,
						_ => v
							.iter()
							.unique_by(|(smiles, _)| smiles)
							.count(),
					};

					let res = match unique_count {
						0 => Err(format!("Compound {provider:?}.{refid}: Missing smiles.")),
						1 => {
							let (smiles, mol) = v.into_iter().next().unwrap();
							Ok((smiles, (provider, refid, mol)))
						},
						cnt => Err(format!("Compound {provider:?}.{refid}: {cnt} conflicting entries.")),
					};

					if let Err(err) = &res {
						eprintln!("{err}");
					}
					res.ok()
				})
				.for_each_with(tx, |s, elem| {
					s.send(elem).unwrap();
				});

				println!("   completed.");
		});

		// Group duplicate building blocks

		let mut grouped_smiles = rx
			.into_iter()
			.into_group_map();

		println!("  Grouping duplicate compounds...");

		let infos = grouped_smiles
			.par_drain()
			.map(|e: (String, Vec<(Provider, String, InitializedHeap<ROMol>)>)| -> Result<_, &str> {
				let (smiles, v) = e;
				let (_, _, mol0) = &v[0];

				let rdpickle = mol0
					.to_pickle(Some(common::DEFAULT_MOL_PICKLE_OPTIONS))
					.map_err(|_| "Failed to generate pickle")?;

				let compound_refs = v
					.into_iter()
					.map(|(provider, refid, _)| (provider, refid))
					.collect_vec();

				Ok((smiles, rdpickle, compound_refs))
			})
			.filter_map(|e| e.ok())
			.collect::<Vec<_>>();

		println!("   completed.");

		println!("  Inserting building blocks...");

		infos
			.par_chunks(infos.len().div_ceil(cpu_cnt)
				.min(65535 / NewBuildingBlock::field_count()))
			.try_for_each(|e: &[(String, Vec<u8>, Vec<(Provider, String)>)]| -> Result<_, String> {
				let mut conn = db_pool.get().unwrap();

				let ent_building_blocks = e
					.into_iter()
					.map(|(smiles, rdpickle, _)| NewBuildingBlock {
						smiles,
						rdpickle,
					})
					.collect_vec();

				let ent_building_blocks = db::model::get_or_create_building_blocks(&mut conn, &ent_building_blocks)
					.map_err(|err| format!("Failed to insert building blocks: {err:?}"))?;

				let chunks = e
					.into_iter()
					.enumerate()
					.flat_map(|(idx_bbs, e)| {
						let (_, _, compound_refs) = e;

						let ent_provider_enamine = &ent_provider_enamine;
						let ent_provider_molport = &ent_provider_molport;

						compound_refs
							.into_iter()
							.map(move |(provider, refid)| {
								let id_provider = match provider {
									Provider::Enamine => ent_provider_enamine.id,
									Provider::Molport => ent_provider_molport.id,
								};

								(
									idx_bbs,
									NewCompound {
										id_compound_provider: id_provider,
										refid,
										sdf: None,
										smiles: None,
										available: true,
									},
								)
							})
					})
					.chunks(65535 / NewCompound::field_count());

				chunks
					.into_iter()
					.try_for_each(|chunk| -> Result<(), String> {
						let mut conn = db_pool.get().unwrap();

						let (idx_bbs, ent_compounds): (Vec<_>, Vec<_>) = chunk.unzip();

						let ent_compounds = db::model::create_compounds_return(&mut conn, &ent_compounds)
							.map_err(|err| format!("Failed to insert compounds: {err:?}"))?;

						let ent_building_block_origins = idx_bbs
							.into_iter()
							.zip(ent_compounds)
							.map(|(idx_bb, ent_compound)| NewBuildingBlockOrigin {
								id_building_block: ent_building_blocks[idx_bb].id,
								id_compound: ent_compound.id,
							})
							.collect_vec();

						let chunks = ent_building_block_origins
							.chunks(65535 / NewBuildingBlockOrigin::field_count());

						chunks
							.into_iter()
							.try_for_each(|chunk| -> Result<(), String> {
								db::model::create_building_block_origins(&mut conn, &chunk)
									.map_err(|err| format!("Failed to insert building block origins: {err:?}"))?;

								Ok(())
							})?;

						Ok(())
					})?;

					Ok(())
			})
			// Failure to insert is a hard error
			.unwrap();

		println!("   completed.");
	});

	Ok(())
}

fn compute_building_block_reactants(db_pool: &db::DBPool, thread_pool: &ThreadPool) -> Result<(), ()> {
	let mut conn = db_pool.get().unwrap();
	let cpu_cnt = num_cpus::get();

	thread_pool.in_place_scope(|scope| {
		let (tx, rx) = mpsc();

		// No need to parallelize, it is a very short list
		let reactions = db::model::get_reactions(&mut conn)
			.unwrap()
			.filter_map(|e| e.ok())
			.map(|ent_reaction| {
				let reaction = ChemicalReaction::new(ChemicalReactionFromPickleParams {
						pickle: &ent_reaction.rdpickle
					})
					.unwrap();

				(ent_reaction.id, reaction)
			})
			.collect_vec();

		scope.spawn(move |_scope| {
			db::model::get_building_blocks(&mut conn)
				.unwrap()
				.filter_map(|x| x.ok())
				.collect_vec()
				.into_par_iter()
				.for_each(|ent_building_block| {
					let building_block = new_local!(ROMol);
					let building_block = building_block.init(ROMolFromPickleParams {
							pickle: &ent_building_block.rdpickle,
						})
						.unwrap();

					let reactions = &reactions;
					for (id_reaction, reaction) in reactions {
						let reactants = reaction.get_reactants();
						let reactant_count = reactants.size();

						// If a building block matches several reactant templates for the same reaction, then
						// it will polymerize which is undesirable, so it must be discarded for this reaction.

						let reactant_idx = (0..reactant_count)
							.filter_map(|idx| {
								let reactant = reactants.get(idx).unwrap();

								let matches = new_local!(MatchVectTypeVec);
								let matches = matches
									.init(&MatchVectTypeVecInitParamsFromSubstructMatch::new(&building_block, &reactant));

								matches
									.ok()
									.map(|_| idx)
							})
							.exactly_one();

						if let Ok(reactant_idx) = reactant_idx {
							let bbr = NewBuildingBlockReactant {
								id_reaction: *id_reaction,
								id_building_block: ent_building_block.id,
								reactant_idx: reactant_idx.try_into().unwrap(),
							};

							tx.send(bbr).unwrap();
						}
					}
				});
		});

		let bbrs = rx
			.into_iter()
			.par_bridge()
			.collect::<Vec<_>>();

		bbrs
			.par_chunks(bbrs.len()
				.div_ceil(cpu_cnt)
				.min(65535 / NewBuildingBlockReactant::field_count()))
			.for_each(|chunk| {
				let mut conn = db_pool.get().unwrap();
				db::model::create_building_block_reactants(&mut conn, chunk).unwrap();
			});
	});

	Ok(())
}

fn main() {
	println!("Update started.");

	let (db_pool, db_thread_pool) = db::pool_with_envfile();
	let thread_pool = rayon::ThreadPoolBuilder::new()
		.num_threads(0)
		.build()
		.unwrap();

	println!("Bootstrapping...");

	boostrap(&db_pool).unwrap();

	println!("   completed.");

	println!("Removing previous compounds...");

	remove_prev_compounds(&db_pool).unwrap();

	println!(" completed.");

	println!("Importing new compounds...");

	import_new_compounds(&db_pool, &thread_pool).unwrap();

	println!(" completed.");

	println!("Computing building block reactants...");

	compute_building_block_reactants(&db_pool, &thread_pool).unwrap();

	println!(" completed.");

	println!("Optimizing database...");

	db::vacuum_full_analyze(&db_pool).unwrap();

	println!(" completed.");

	drop(db_pool);

	while db_thread_pool.strong_count() != 0 {
		std::thread::sleep(std::time::Duration::from_millis(1));
	}

	println!("Update completed successfully.");
}

use itertools::Itertools;
use zip_next as zip;

use rdkit_rust::PickleMolOptions;

pub static DEFAULT_MOL_PICKLE_OPTIONS: PickleMolOptions = PickleMolOptions {
	mol_props: true,
	atom_props: true,
	bond_props: true,
	private_props: false,
	computed_props: false,
	coords_as_double: true,
	no_conformers: false,
};

pub fn slugify(s: &str) -> String {
   let mut s = s
		.replace(|c: char| !c.is_ascii_alphanumeric(), " ")
		.split_whitespace()
		.join("_");
	s.make_ascii_lowercase();
	s
}

pub fn default_zip_opts() -> zip::write::FileOptions {
	zip::write::FileOptions::default()
		.compression_method(zip::CompressionMethod::Deflated)
		.compression_level(Some(9))
		.large_file(true)
}

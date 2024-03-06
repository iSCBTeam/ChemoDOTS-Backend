use std::io::{self, Read};

use chemodots_db as db;
use chemodots_toolkit as toolkit;
use itertools::Itertools;
use serde_json::Value;

fn main() -> Result<(), &'static str> {
	let (db_pool, db_thread_pool) = db::pool_with_envfile();

	let mut contents = String::new();
	io::stdin().read_to_string(&mut contents)
		.expect("Should have been able to read stdin");

	let v: Value = serde_json::from_str(&contents).unwrap();
	let mode = v["mode"].as_str().unwrap();

	let result = match mode {
		"mol_info" => {
			let frag_mol = v["mol"].as_str().unwrap();

			toolkit::mol_info(frag_mol);
			Ok(())
		},
		"probe_reactions" => {
			let frag_mol = v["mol"].as_str().unwrap();
			let atoms = v["atoms"].as_array().unwrap().into_iter().map(|v| v.as_u64().unwrap() as i32).collect_vec();

			toolkit::compatible_reactions_probe(&db_pool, frag_mol, &atoms);
			Ok(())
		},
		_ => Err("Invalid 'mode'")
	};

	drop(db_pool);

	while db_thread_pool.strong_count() != 0 {
		std::thread::sleep(std::time::Duration::from_millis(1));
	}

	result
}

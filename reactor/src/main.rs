use std::{io::{self, Read, Write}, fs::File};

use chemodots_db as db;
use chemodots_reactor as reactor;
use itertools::Itertools;
use reactor::ReactionCounter;
use serde_json::{json, Value};

fn format_duration_hh_mm_ss(d: &chrono::Duration) -> String {
	let tot_secs = d.num_seconds();
	let tot_mins = tot_secs / 60;
	let tot_hours = tot_mins / 60;

	let secs = tot_secs % 60;
	let mins = tot_mins % 60;
	let hours = tot_hours;

	format!("{:0>2}:{:0>2}:{:0>2}", hours, mins, secs)
}

fn main() {
	let mut contents = String::new();
	io::stdin()
		.read_to_string(&mut contents)
		.expect("Should have been able to read stdin");

	let thread_pool = rayon::ThreadPoolBuilder::new()
		.num_threads(0)
		.build()
		.unwrap();
	let (db_pool, db_thread_pool) = db::pool_with_envfile();

	let v: Value = serde_json::from_str(&contents).unwrap();
	let name = v["name"].as_str().unwrap();
	let smiles = v["smiles"].as_str().unwrap();
	let mol = v["mol"].as_str().unwrap();
	let atoms = v["atoms"].as_array().unwrap().into_iter().map(|v| v.as_u64().unwrap() as i32).collect_vec();
	let rules = v["rules"].as_array().unwrap().into_iter().map(|v| v.as_u64().unwrap() as i64).collect_vec();
	let bb_dbs = v["bb_dbs"].as_array().unwrap().into_iter().map(|v| v.as_str().unwrap()).collect_vec();

	let mut ent_experiment = reactor::experiment_create(&db_pool, name, smiles, Some(mol), &atoms, &rules, &bb_dbs);

	let exp_uuid_str = ent_experiment.uuid.to_string();

	std::fs::create_dir(&exp_uuid_str).unwrap();
	std::env::set_current_dir(&exp_uuid_str).unwrap();

	let result = reactor::experiment_gen_products(&thread_pool, &db_pool, &mut ent_experiment);
	reactor::plot::gen_plots(&db_pool, &ent_experiment);

	let total_bb_count = db::model::count_building_blocks_with_experiment_providers(&mut db_pool.get().unwrap(), &ent_experiment).unwrap();

	let overall: ReactionCounter = result.reactions
		.iter()
		.map(|r| r.counter)
		.reduce(|x, y| ReactionCounter {
			reacted_building_blocks: x.reacted_building_blocks + y.reacted_building_blocks, // TODO: Fetch from db
			raw_products: x.raw_products + y.raw_products,
			dup_products: x.dup_products + y.dup_products,
			undesired_products: x.undesired_products + y.undesired_products,
			final_products: x.final_products + y.final_products,
		})
		.unwrap_or(ReactionCounter::default());

	let mut reactions_json = Vec::new();

	reactions_json.push(json!({
		"id": None::<i64>,
		"name": "Overall",
		"total_bb_cnt": total_bb_count,
		"reacted_bb_cnt": overall.reacted_building_blocks,
		"generated_prod_cnt": overall.raw_products,
		"duplicate_prod_cnt": overall.dup_products,
		"undesired_prod_cnt": overall.undesired_products,
		"final_prod_cnt": overall.final_products,
	}));

	reactions_json.append(
		&mut result.reactions
			.iter()
			.sorted_by_key(|r| r.id)
			.map(|r| json!({
				"id": r.id,
				"name": r.name,
				"total_bb_cnt": total_bb_count,
				"reacted_bb_cnt": r.counter.reacted_building_blocks,
				"generated_prod_cnt": r.counter.raw_products,
				"duplicate_prod_cnt": r.counter.dup_products,
				"undesired_prod_cnt": r.counter.undesired_products,
				"final_prod_cnt": r.counter.final_products,
			}))
			.collect_vec());

	let duration = ent_experiment.ts_end.unwrap() - ent_experiment.ts_start;
	let res_json = json!({
		"duration": format_duration_hh_mm_ss(&duration),
		"reactions": reactions_json,
	});

	let mut file = File::create("info.json")
		.map_err(|_| format!("Failed to create json file for experiment {exp_uuid_str}")).unwrap();

	file.write_all(res_json.to_string().as_bytes())
		.map_err(|_| format!("Failed to write json data for experiment {exp_uuid_str}")).unwrap();

	let res_json = json!({
		"uuid": exp_uuid_str,
	});
	println!("{}", res_json.to_string());

	drop(db_pool);

	while db_thread_pool.strong_count() != 0 {
		std::thread::sleep(std::time::Duration::from_millis(1));
	}
}

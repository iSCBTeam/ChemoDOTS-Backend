use std::io;
use std::io::Read;
use std::ops::Bound;

use db::custom::RealrangeType;
use chrono;
use db::model::NewExperimentPostprocFilter;
use serde_json::json;
use serde::Deserialize;
use uuid::Uuid;

use chemodots_db as db;
use chemodots_reactor as reactor;

#[derive(Deserialize)]
struct FilterQuery {
	pub uuid: Uuid,
	pub filters: db::model::ExperimentProductDescFilter,
}

fn read_filter_query() -> FilterQuery {
	let mut contents = Vec::new();
	io::stdin().read_to_end(&mut contents)
		.expect("Failed to read stdin");

	serde_json::from_slice(&contents)
		.expect("Failed to deserialize the query")
}

#[derive(Deserialize)]
struct Generate3DQuery {
	pub uuid: Uuid,
}

fn read_generate_3d_query() -> Generate3DQuery {
	let mut contents = Vec::new();
	io::stdin().read_to_end(&mut contents)
		.expect("Failed to read stdin");

	serde_json::from_slice(&contents)
		.expect("Failed to deserialize the query")
}

pub fn filter(db_pool: &db::DBPool) {
	let mut conn = db_pool.get().unwrap();

	let query = read_filter_query();

	let ent_exp = db::model::get_experiment_with_uuid(&mut conn, query.uuid).unwrap();

	let total_cnt = db::model::ExperimentProduct::count_with_experiment(&mut conn, &ent_exp).unwrap();
	let selected_cnt = db::model::ExperimentProduct::count_with_experiment_and_descs(&mut conn, &ent_exp, &query.filters).unwrap();

	let res_json = json!({
		"total": total_cnt,
		"selected": selected_cnt,
	});
	println!("{}", res_json.to_string());
}

fn f32_filter_to_range((min, max): (f32, f32)) -> RealrangeType {
	RealrangeType::new(std::ops::Bound::Included(min), std::ops::Bound::Included(max))
}

fn i32_filter_to_range((min, max): (i32, i32)) -> (Bound<i32>, Bound<i32>) {
	(std::ops::Bound::Included(min), std::ops::Bound::Included(max))
}

pub fn generate2d(db_pool: &db::DBPool) {
	let thread_pool = rayon::ThreadPoolBuilder::new()
		.num_threads(0)
		.build()
		.unwrap();
	let mut conn = db_pool.get().unwrap();

	let query = read_filter_query();

	let ent_exp = db::model::get_experiment_with_uuid(&mut conn, query.uuid).unwrap();

	std::env::set_current_dir(ent_exp.uuid.to_string()).unwrap();

	let ent_experiment_postproc_filter = db::model::create_experiment_postproc_filter(&mut conn, &NewExperimentPostprocFilter {
		desc_clogp: f32_filter_to_range(query.filters.clogp.unwrap_or((f32::NEG_INFINITY, f32::INFINITY))),
		desc_fsp3: f32_filter_to_range(query.filters.fsp3.unwrap_or((f32::NEG_INFINITY, f32::INFINITY))),
		desc_hba: i32_filter_to_range(query.filters.hba.unwrap_or((i32::MIN, i32::MAX))),
		desc_hbd: i32_filter_to_range(query.filters.hbd.unwrap_or((i32::MIN, i32::MAX))),
		desc_mw: f32_filter_to_range(query.filters.mw.unwrap_or((f32::NEG_INFINITY, f32::INFINITY))),
		desc_tpsa: f32_filter_to_range(query.filters.tpsa.unwrap_or((f32::NEG_INFINITY, f32::INFINITY))),
		id_experiment: ent_exp.id,
		ts: chrono::Utc::now().naive_utc(),
	}).unwrap();

	reactor::gen_files_filtered(&thread_pool, db_pool, &ent_exp, "filtered", "overall_filtered", false, Some(&ent_experiment_postproc_filter));

	println!("{{}}");
}

pub fn generate3d(db_pool: &db::DBPool) {
	let thread_pool = rayon::ThreadPoolBuilder::new()
		.num_threads(0)
		.build()
		.unwrap();
	let mut conn = db_pool.get().unwrap();

	let query = read_generate_3d_query();

	let ent_exp = db::model::get_experiment_with_uuid(&mut conn, query.uuid).unwrap();

	std::env::set_current_dir(ent_exp.uuid.to_string()).unwrap();

	// TODO: Handle result
	let ent_experiment_postproc_filter = db::model::get_last_experiment_postproc_filter_with_experiment(&mut conn, &ent_exp)
		.unwrap();

	reactor::gen_files_filtered_3d(&thread_pool, db_pool, &ent_exp, "filtered_3d", "overall_filtered", &ent_experiment_postproc_filter);

	println!("{{}}");
}

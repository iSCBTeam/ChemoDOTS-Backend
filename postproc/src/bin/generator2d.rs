use chemodots_db as db;

fn main() {
	let (db_pool, db_thread_pool) = db::pool_with_envfile();

	eprintln!("Generate 2D files...");

	chemodots_postproc::generate2d(&db_pool);

	drop(db_pool);

	while db_thread_pool.strong_count() != 0 {
		std::thread::sleep(std::time::Duration::from_millis(1));
	}

	eprintln!(" completed.");
}

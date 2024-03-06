use chemodots_db as db;

fn main() {
	let (db_pool, db_thread_pool) = db::pool_with_envfile();

	db::run_migrations(&db_pool)
		.expect("Failed to apply pending migrations");

	drop(db_pool);
	
	while db_thread_pool.strong_count() != 0 {
		std::thread::sleep(std::time::Duration::from_millis(1));
	}

	println!("Pending migrations applied successfully.");
}

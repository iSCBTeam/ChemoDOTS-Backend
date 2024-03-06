use diesel::pg::PgConnection;
use diesel::{prelude::*, sql_query};
use diesel::r2d2::{Pool, ConnectionManager};
use diesel_migrations::{ embed_migrations, EmbeddedMigrations, MigrationHarness };
use dotenvy::dotenv;
use num_cpus;
use scheduled_thread_pool::{ScheduledThreadPool, OnPoolDropBehavior};

use std::env;
use std::error::Error;
use std::sync::{Arc, Weak};

pub mod custom;
pub mod expression;
pub mod model;
pub mod schema;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn database_url_from_env() -> Result<String, env::VarError> {
	dotenv().ok();
	dotenvy::var("DATABASE_URL").or_else(|_| {
		env::var("DATABASE_URL")
	})
}

pub fn connect_with_envfile() -> Result<PgConnection, ConnectionError> {
	let database_url = database_url_from_env().unwrap();
	PgConnection::establish(&database_url)
}

pub fn pool_with_envfile() -> (DBPool, Weak<ScheduledThreadPool>) {
	let cpu_cnt = num_cpus::get().try_into().unwrap_or(u32::MAX);
	let url = database_url_from_env().unwrap();
	let manager = ConnectionManager::<PgConnection>::new(url);

	let thread_pool = Arc::new(ScheduledThreadPool::builder()
		.num_threads(3)
		.thread_name_pattern("r2d2-worker-{}")
		.on_drop_behavior(OnPoolDropBehavior::CompletePendingScheduled)
		.build());

	let weak_thread_pool = Arc::downgrade(&thread_pool);

	let pool = Pool::builder()
		.test_on_check_out(true)
		.min_idle(Some(1))
		.max_size(cpu_cnt * 2)
		.thread_pool(thread_pool)
		.build(manager)
		.expect("Could not build connection pool");

	(pool, weak_thread_pool)
}

pub fn run_migrations(db_pool: &DBPool) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	let mut conn = db_pool.get()?;
	conn.run_pending_migrations(MIGRATIONS)?;

	Ok(())
}

pub fn vacuum_full_analyze(db_pool: &DBPool) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	let mut conn = db_pool.get()?;

	sql_query("VACUUM FULL ANALYZE")
		.execute(&mut conn)?;

	Ok(())
}

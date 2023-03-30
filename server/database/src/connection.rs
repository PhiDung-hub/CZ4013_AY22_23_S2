use deadpool_sqlite::{Config, Manager, Pool, Runtime};
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const DB_PATH: &str = "flight_informations.db";

lazy_static! {
    static ref CONNECTION: Arc<Mutex<Connection>> =
        Arc::new(Mutex::new(Connection::open(DB_PATH).expect("Failed to establish a database connection")));
}

pub fn get_global_db_connection() -> Arc<Mutex<Connection>> {
    CONNECTION.clone()
}

lazy_static! {
    static ref DB_POOL: Pool = {
        let config = Config::new(DB_PATH);
        let manager = Manager::from_config(&config, Runtime::Tokio1);
        let pool = Pool::builder(manager)
            .max_size(8)
            .build()
            .expect("Pool build error, did you install tokio?");
        pool
    };
}

pub fn get_connection_pool() -> &'static Pool {
    &DB_POOL
}


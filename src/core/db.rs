use std::time::Duration;

use sqlx::{pool::PoolOptions, Pool, Postgres};

use crate::settings::Config;

pub async fn init_pool(config: &Config) -> Pool<Postgres> {
    PoolOptions::new()
        .min_connections(5)
        .max_connections(100)
        .idle_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database")
}

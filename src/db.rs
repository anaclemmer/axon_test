///database connection pool

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::env;

pub async fn get_pool() -> Result<SqlitePool, sqlx::Error> {
    let db_url = env::var("DATABASE_URL").map_err(|_| AppError::Config("DATABASE_URL must be set".to_string()))?;

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(AppError::from)
}
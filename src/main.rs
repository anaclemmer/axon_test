///server setup, entry point
//import modules
mod db;
mod errors;
mod models;
mod routes;
#[cfg(test)]
mod tests;

use crate::db::get_pool;
use crate::errors::AppError;
use crate::routes::create_router;
use axum::serve;
use tokio::net::TcpListener;
use tracing::info;

//bind main to tokio's runtime to make it asynchronous
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    dotenvy::dotenv().ok(); //loading .env
    tracing_subscriber::fmt::init(); //start logging

    let pool = get_pool().await?;
    sqlx::migrate!().run(&pool).await?;

    let router = create_router(pool);

    info!("Running on http://localhost:3000");

    //set to listen on all interfaces on port 3000
    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(|e| AppError::ServerError(e.to_string()))?;
    serve(listener, router)
        .await
        .map_err(|e| AppError::ServerError(e.to_string()))
}

///server setup, entry point
//import modules
mod db;
mod models;
mod routes;
mod errors;

use dotenvy::dotenv;
use std::env;
use tracing::info;
use tracing_subscriber;
use errors::AppError;
use hyper::Server;

use crate::db::get_pool;
use crate::routes::create_router;

//import axum router
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

//bind main to tokio's runtime to make it asynchronous
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    dotenvy::dotenv().ok(); //loading .env
    tracing_subscriber::fmt::init();//start logging

    let pool = get_pool().await?;
    sqlx::migrate!().run(&pool).await?;

    let router = create_router(pool); 

    info!("Running on http://localhost:3000");
    
    //set to listen on all interfaces on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .map_err(|e| AppError::ServerError(e.to_string()))
}
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use sqlx::Error as SqlxError;
use sqlx::migrate::MigrateError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error")]
    Sqlx(#[from] SqlxError),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Task not found")]
    NotFound,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Migration error: {0}")]
    Migration(#[from] MigrateError),

    #[error("Parsing priority error: {0}")]
    ParsePriority(#[from] ParsePriorityError),

    #[error("Parsing status error: {0}")]
    ParseStatus(#[from] ParseStatusError),    
}

//convert error message into http response with status code 
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ParsePriority(_) => StatusCode::BAD_REQUEST, // parsing errors usually are client errors
        };

        (status, self.to_string()).into_response()
    }
}
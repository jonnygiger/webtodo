use thiserror::Error;
use rocket::serde::json::Json;
use crate::ApiError;
use crate::ErrorDetail;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Password hashing error: {0}")]
    HashingError(#[from] bcrypt::BcryptError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Unauthorized access: {0}")]
    Unauthorized(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        let detail = err.to_string();
        match err {
            ServiceError::NotFound(_) => ApiError::NotFound(Json(ErrorDetail { error: detail })),
            ServiceError::Unauthorized(_) => ApiError::Unauthorized(Json(ErrorDetail { error: detail })),
            ServiceError::Conflict(_) => ApiError::Conflict(Json(ErrorDetail { error: detail })),
            ServiceError::InvalidInput(_) => ApiError::BadRequest(Json(ErrorDetail { error: detail })),
            ServiceError::DatabaseError(_) | ServiceError::HashingError(_) | ServiceError::InternalError(_) => {
                ApiError::InternalError(Json(ErrorDetail { error: detail }))
            }
        }
    }
}

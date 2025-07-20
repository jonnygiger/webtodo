use rocket::serde::json::Json;
use crate::ApiError;
use crate::ErrorDetail;

pub enum ServiceError {
    NotFound(String),
    Unauthorized(String),
    Conflict(String),
    InternalError(String),
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(detail) => ApiError::NotFound(Json(ErrorDetail { error: detail })),
            ServiceError::Unauthorized(detail) => ApiError::Unauthorized(Json(ErrorDetail { error: detail })),
            ServiceError::Conflict(detail) => ApiError::Conflict(Json(ErrorDetail { error: detail })),
            ServiceError::InternalError(detail) => ApiError::InternalError(Json(ErrorDetail { error: detail })),
        }
    }
}

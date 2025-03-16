use axum::response::IntoResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AdminError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AdminError::NotFound(_) => StatusCode::NOT_FOUND,
            AdminError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AdminError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AdminError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl From<serde_json::Error> for AdminError {
    fn from(err: serde_json::Error) -> Self {
        AdminError::ValidationError(err.to_string())
    }
} 
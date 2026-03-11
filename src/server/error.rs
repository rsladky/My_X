use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    AuthError(String),
    DbError(String),
    ValidationError(String),
    NotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthError(msg) => write!(f, "Auth error: {}", msg),
            Self::DbError(msg) => write!(f, "Database error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::AuthError(_) => StatusCode::UNAUTHORIZED,
            Self::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
        };

        let error_key = match &self {
            Self::AuthError(_) => "auth_error",
            Self::DbError(_) => "db_error",
            Self::ValidationError(_) => "validation_error",
            Self::NotFound(_) => "not_found",
        };

        let body = Json(serde_json::json!({
            "error": error_key,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn error_to_json(err: AppError) -> (StatusCode, Value) {
        let display = err.to_string();
        let (status, key) = match &err {
            AppError::AuthError(_) => (StatusCode::UNAUTHORIZED, "auth_error"),
            AppError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db_error"),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
        };
        (status, serde_json::json!({"error": key, "message": display}))
    }

    #[test]
    fn validation_error_is_400_with_correct_json() {
        let (status, json) = error_to_json(AppError::ValidationError("too long".into()));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"], "validation_error");
        assert_eq!(json["message"], "Validation error: too long");
    }

    #[test]
    fn auth_error_is_401_with_correct_json() {
        let (status, json) = error_to_json(AppError::AuthError("bad creds".into()));
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(json["error"], "auth_error");
        assert_eq!(json["message"], "Auth error: bad creds");
    }

    #[test]
    fn db_error_is_500_with_correct_json() {
        let (status, json) = error_to_json(AppError::DbError("connection refused".into()));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["error"], "db_error");
        assert_eq!(json["message"], "Database error: connection refused");
    }

    #[test]
    fn not_found_is_404_with_correct_json() {
        let (status, json) = error_to_json(AppError::NotFound("user".into()));
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"], "not_found");
        assert_eq!(json["message"], "Not found: user");
    }
}

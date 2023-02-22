use thiserror::Error;
use axum::extract::rejection::QueryRejection;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use validator::ValidationErrors;
use sea_orm::error::DbErr;

#[derive(Debug, Error)]
pub enum ProfileServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    SeaOrmDbError(#[from] DbErr),
    #[error("realm '{0}' is not configured")]
    RealmNotConfigured(String),
    #[error("realm '{0}' digest '{1}' incorrect")]
    RealmDigestIncorrect(String, String),
}

impl IntoResponse for ProfileServerError {
    fn into_response(self) -> Response {
        match self {
            ProfileServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ProfileServerError::AxumQueryRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ProfileServerError::SeaOrmDbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ProfileServerError::RealmNotConfigured(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ProfileServerError::RealmDigestIncorrect(_, _) => (StatusCode::UNAUTHORIZED, self.to_string()),
        }
        .into_response()
    }
}
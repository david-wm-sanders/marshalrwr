use thiserror::Error;
use axum::extract::rejection::QueryRejection;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use validator::ValidationErrors;
use sea_orm::error::DbErr;
use quick_xml::Error as QXmlError;

#[derive(Debug, Error)]
pub enum ProfileServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    SeaOrmDbError(#[from] DbErr),
    #[error(transparent)]
    QuickXmlError(#[from] QXmlError),
    #[error("realm '{0}' is not configured")]
    RealmNotConfigured(String),
    #[error("realm '{0}' digest '{1}' incorrect")]
    RealmDigestIncorrect(String, String),
    #[error("player '{1}' [hash:{0}] sid {2} != {3}")]
    PlayerSidMismatch(i64, String, i64, i64),
    #[error("player '{1}' [hash:{0}, sid:{2}] rid '{3}' incorrect")]
    PlayerRidIncorrect(i64, String, i64, String),
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
            ProfileServerError::QuickXmlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ProfileServerError::RealmNotConfigured(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ProfileServerError::RealmDigestIncorrect(_, _) => (StatusCode::UNAUTHORIZED, self.to_string()),
            ProfileServerError::PlayerSidMismatch(_, _, _, _) => (StatusCode::UNAUTHORIZED, self.to_string()),
            ProfileServerError::PlayerRidIncorrect(_, _, _, _) => (StatusCode::UNAUTHORIZED, self.to_string()),
        }
        .into_response()
    }
}
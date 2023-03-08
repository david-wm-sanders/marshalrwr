use axum::extract::rejection::QueryRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::error::DbErr;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    SeaOrmDbError(#[from] DbErr),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ServerError::AxumQueryRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::SeaOrmDbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
        .into_response()
    }
}

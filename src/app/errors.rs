use thiserror::Error;
use axum::extract::rejection::QueryRejection;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    SurrealDbError(#[from] surrealdb::Error)
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ServerError::AxumQueryRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::SurrealDbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
        .into_response()
    }
}
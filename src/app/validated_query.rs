use async_trait::async_trait;
use axum::extract::{rejection::QueryRejection, FromRequestParts, Query};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use validator::Validate;

use super::errors::ServerError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<T>::from_request_parts(parts, state).await?;
        params.validate()?;
        Ok(ValidatedQuery(params))
    }
}

use std::ops::{Deref, DerefMut};

use axum::http::{StatusCode, header, Request};
use axum::response::{Response, IntoResponse};
use axum::extract::{State, FromRequest};
use axum::body::{Bytes, HttpBody, Body};
use axum::{async_trait, BoxError};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use validator::Validate;

use super::errors::ProfileServerError;
use super::validation::ValidatedQuery;
use super::super::state::AppState;

use super::util::{check_realm_is_configured,};

use super::params::SetProfileParams;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedXmlBody<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedXmlBody<T>
where
    T: DeserializeOwned /*+ Validate*/,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = ProfileServerError;
    
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let body = Bytes::from_request(req, state).await?;
        let data = quick_xml::de::from_reader(&*body)?;
        // do validation...
        // let data: T = body.into();
        Ok(Self(data))
    }
}

// impl<T> Deref for ValidatedXmlBody<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<T> DerefMut for ValidatedXmlBody<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

#[derive(Debug, Deserialize)]
pub struct SetProfileData {
    
}

#[debug_handler]
pub async fn rwr1_set_profile_handler(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<SetProfileParams>,
    ValidatedXmlBody(data): ValidatedXmlBody<SetProfileData>)
    -> Result<Response, ProfileServerError> {
    let headers  = [(header::CONTENT_TYPE, "text/xml")];
    
    tracing::debug!("{data:#?}");
    
    Ok((StatusCode::OK).into_response())
}
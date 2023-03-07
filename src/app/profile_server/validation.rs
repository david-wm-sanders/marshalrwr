use lazy_static::lazy_static;
use validator::ValidationError;
use regex::Regex;
// use async_trait::async_trait;
use axum::extract::{Query, FromRequest, FromRequestParts, rejection::QueryRejection};
use axum::http::request::{Request, Parts};
use axum::body::{Bytes, HttpBody};
use axum::{async_trait, BoxError};
use serde::de::DeserializeOwned;
use validator::Validate;
use percent_encoding::percent_decode_str;

use super::errors::ProfileServerError;
use super::params::GetProfileParams;
use super::super::hasher::rwr1_hash_username;

lazy_static! {
    pub static ref RE_HEX_STR: Regex = Regex::new(r"^([0-9A-Fa-f]{2})+$").unwrap();
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>
{
    type Rejection = ProfileServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<T>::from_request_parts(parts, state).await?;
        params.validate()?;
        Ok(ValidatedQuery(params))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedXmlBody<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedXmlBody<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = ProfileServerError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let body = Bytes::from_request(req, state).await?;
        // the xml body bytes are percent/url-encoded, decode first
        let body_vec = body.into_iter().collect::<Vec<u8>>();
        let xml_str = std::str::from_utf8(&body_vec)?;
        // tracing::debug!("{xml_str:#?}");
        let decoded_xml_str = percent_decode_str(xml_str).decode_utf8()?;
        tracing::debug!("{decoded_xml_str}");
        let data: T = quick_xml::de::from_str(decoded_xml_str.as_ref())?;
        // validate the xml data
        data.validate()?;
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

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains("  ") {
        return Err(ValidationError::new("username contains multiple consecutive spaces"));
    }
    if username.starts_with(' ') {
        return Err(ValidationError::new("username starts with a space"));
    }
    if username.ends_with(' ') {
        return Err(ValidationError::new("username ends with a space"));
    }
    // todo: check for weird characters that aren't control but correspond to weird things in default rwr latin font
    Ok(())
}

pub fn validate_get_profile_params(params: &GetProfileParams) -> Result<(), ValidationError> {
    // calculate int hash from string username and confirm they match
    if params.hash != rwr1_hash_username(params.username.as_str()) {
        return Err(ValidationError::new("hash not for given username"));
    }
    Ok(())
}
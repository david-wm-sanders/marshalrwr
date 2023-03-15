use lazy_static::lazy_static;
use regex::Regex;
use validator::ValidationError;
// use async_trait::async_trait;
use axum::body::{Bytes, HttpBody};
use axum::extract::{rejection::QueryRejection, FromRequest, FromRequestParts, Query};
use axum::http::request::{Parts, Request};
use axum::{async_trait, BoxError};
use percent_encoding::percent_decode_str;
use serde::de::DeserializeOwned;
use validator::Validate;

use super::super::hasher::rwr1_hash_username;
use super::errors::ProfileServerError;
use super::params::GetProfileParams;
use super::util::USERNAME_BLOCKED_CHARS;

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
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
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
        // tracing::debug!("{decoded_xml_str}");
        let data: T = quick_xml::de::from_str(decoded_xml_str.as_ref())?;
        // validate the xml data
        data.validate()?;
        Ok(Self(data))
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains("  ") {
        return Err(ValidationError::new(
            "username contains multiple consecutive spaces",
        ));
    }
    if username.starts_with(' ') {
        return Err(ValidationError::new("username starts with a space"));
    }
    if username.ends_with(' ') {
        return Err(ValidationError::new("username ends with a space"));
    }
    if !username.chars().all(|c| {
        c.is_ascii_punctuation()
            || c.is_ascii_digit()
            || (c.is_ascii_alphabetic() && c.is_ascii_uppercase())
    }) {
        return Err(ValidationError::new(
            "username must be uppercase|punctuation|digit characters only",
        ));
    }
    if username.contains(USERNAME_BLOCKED_CHARS) {
        return Err(ValidationError::new(
            "username contains forbidden character",
        ));
    }
    Ok(())
}

pub fn validate_get_profile_params(params: &GetProfileParams) -> Result<(), ValidationError> {
    // calculate int hash from string username and confirm they match
    if params.hash != rwr1_hash_username(params.username.as_str()) {
        return Err(ValidationError::new("hash not for given username"));
    }
    Ok(())
}

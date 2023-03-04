use std::ops::{Deref, DerefMut};

use axum::http::{StatusCode, header, Request};
use axum::response::{Response, IntoResponse};
use axum::extract::{State, FromRequest};
use axum::body::{Bytes, HttpBody};
use axum::{async_trait, BoxError};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use validator::Validate;
use percent_encoding::percent_decode_str;

use super::errors::ProfileServerError;
use super::validation::ValidatedQuery;
use super::super::state::AppState;

use super::util::{check_realm_is_configured, get_realm};

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
        // the xml body bytes are percent/url-encoded, decode first
        let body_vec = body.into_iter().collect::<Vec<u8>>();
        // todo: get rid of these unwrap here, probably goto ? and add extra variants to ProfileServerError
        let xml_str = std::str::from_utf8(&body_vec).unwrap();
        // tracing::debug!("{xml_str:#?}");
        let decoded_xml_str = percent_decode_str(xml_str).decode_utf8().unwrap();
        tracing::debug!("{decoded_xml_str:#?}");
        let data = quick_xml::de::from_str(decoded_xml_str.as_ref())?;
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
pub struct PlayerXml {
    #[serde(rename = "@hash")]
    hash: i64,
    #[serde(rename = "@rid")]
    rid: String
}

#[derive(Debug, Deserialize)]
pub struct SetProfileDataXml {
    player: Vec<PlayerXml>
}

#[debug_handler]
pub async fn rwr1_set_profile_handler(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<SetProfileParams>,
    ValidatedXmlBody(data): ValidatedXmlBody<SetProfileDataXml>)
    -> Result<Response, ProfileServerError> {
    let headers  = [(header::CONTENT_TYPE, "text/xml")];
    
    // check that the realm has been configured, see fn comments for more detail
    check_realm_is_configured(&state, &params.realm)?;

    // get the realm, making it if it doesn't exist yet
    tracing::info!("locating realm '{}'", &params.realm);
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;

    tracing::debug!("{data:#?}");

    Ok((StatusCode::OK).into_response())
}
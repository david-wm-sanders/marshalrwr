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
        // tracing::debug!("{decoded_xml_str:#?}");
        let data = quick_xml::de::from_str(decoded_xml_str.as_ref())?;
        // do validation...
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
pub struct SetProfileDataXml {
    #[serde(rename = "player")]
    players: Vec<PlayerXml>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerXml {
    #[serde(rename = "@hash")]
    hash: i64,
    #[serde(rename = "@rid")]
    rid: String,
    person: PersonXml,
    profile: ProfileXml,
}

#[derive(Debug, Deserialize)]
pub struct PersonXml {
    #[serde(rename = "@max_authority_reached")]
    max_authority_reached: f32,
    #[serde(rename = "@authority")]
    authority: f32,
    #[serde(rename = "@job_points")]
    job_points: f32,
    #[serde(rename = "@faction")]
    faction: i32,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@soldier_group_id")]
    soldier_group_id: i32,
    #[serde(rename = "@soldier_group_name")]
    soldier_group_name: String,
    #[serde(rename = "@squad_size_setting")]
    squad_size_setting: i32,
    // todo: add loadout, stash, and backpack
    #[serde(rename = "item")]
    equipped_items: Vec<EquippedItemXml>,
}

#[derive(Debug, Deserialize)]
pub struct EquippedItemXml {
    #[serde(rename = "@slot")]
    slot: i32,
    #[serde(rename = "@index")]
    index: i32,
    #[serde(rename = "@amount")]
    amount: i32,
    #[serde(rename = "@key")]
    key: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfileXml {
    #[serde(rename = "@game_version")]
    game_version: i32,
    #[serde(rename = "@username")]
    username: String,
    #[serde(rename = "@sid")]
    sid: i64,
    #[serde(rename = "@rid")]
    rid: String,
    #[serde(rename = "@squad_tag")]
    squad_tag: String,
    // #[serde(rename = "@")]
    stats: StatsXml,
}

#[derive(Debug, Deserialize)]
pub struct StatsXml {
    #[serde(rename = "@kills")]
    kills: i32,
    #[serde(rename = "@deaths")]
    deaths: i32,
    #[serde(rename = "@time_played")]
    time_played: f32,
    #[serde(rename = "@player_kills")]
    player_kills: i32,
    #[serde(rename = "@teamkills")]
    teamkills: i32,
    #[serde(rename = "@longest_kill_streak")]
    longest_kill_streak: i32,
    #[serde(rename = "@targets_destroyed")]
    targets_destroyed: i32,
    #[serde(rename = "@vehicles_destroyed")]
    vehicles_destroyed: i32,
    #[serde(rename = "@soldiers_healed")]
    soldiers_healed: i32,
    #[serde(rename = "@distance_moved")]
    distance_moved: f32,
    #[serde(rename = "@shots_fired")]
    shots_fired: i32,
    #[serde(rename = "@throwables_thrown")]
    throwables_thrown: i32,
    #[serde(rename = "@rank_progression")]
    rank_progression: f32,
    // todo: add monitors
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
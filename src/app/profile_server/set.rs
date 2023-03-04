use axum::http::{StatusCode, header};
use axum::response::{Response, IntoResponse};
use axum::extract::State;
use axum_macros::debug_handler;

use super::errors::ProfileServerError;
use super::validation::{ValidatedQuery, ValidatedXmlBody};
use super::xml::SetProfileDataXml;
use super::super::state::AppState;

use super::util::{check_realm_is_configured, get_realm};

use super::params::SetProfileParams;

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
    for player in data.players.iter() {
        tracing::debug!("processing set xml for player '{}'", player.hash);
        // todo: get the player and check that the rid matches expected value from cache/db
        // todo: construct account active model from player xml
        // todo: add this to a vec?
    }
    // todo: insert many active model accounts with on_conflict to update

    Ok((StatusCode::OK).into_response())
}
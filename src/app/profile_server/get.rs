use axum::http::{StatusCode, header};
use axum::response::{Response, IntoResponse};
use axum::{extract::State, response::Html};
use axum_macros::debug_handler;

use super::errors::ProfileServerError;
use super::super::{state::AppState, validated_query::ValidatedQuery};
use super::util::{check_realm_is_configured, get_realm, get_player, enlist_player,
                  make_init_profile_xml};

use super::params::GetProfileParams;

#[debug_handler]
pub async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Result<Response, ProfileServerError> {
    let headers  = [(header::CONTENT_TYPE, "text/xml")];

    // check that the realm has been configured, see fn comments for more detail
    check_realm_is_configured(&state, &params.realm)?;

    // get the realm, making it if it doesn't exist yet
    tracing::info!("locating realm '{}'", &params.realm);
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;

    // find the player, if any
    tracing::info!("finding enlistment papers for player '{}'", &params.username);
    let opt_player = get_player(&state, &params).await?;
    match opt_player {
        None => {
            tracing::info!("player '{}' doesn't have any papers, enlisting them (pending checks)", &params.username);
            // enlist player and get back player model
            let player = enlist_player(&state, &params).await?;
            // make an initialisation profile for the player
            let init_profile_xml = make_init_profile_xml(&player.username, &player.rid)?;
            tracing::debug!("sending init profile: '{init_profile_xml}' to game server");
            // return xml response
            return Ok((StatusCode::OK, headers, init_profile_xml).into_response());
        },
        Some(player) => {
            tracing::debug!("Some player");
            // we have a player, try to get_account
        }
    }

    let s = format!("{params:#?} {state:#?}");
    // Ok(Html(s))
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/plain")], s).into_response())
}
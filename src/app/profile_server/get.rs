use std::net::SocketAddr;

use axum::http::{StatusCode, header};
use axum::response::{Response, IntoResponse};
use axum::extract::{State, ConnectInfo};
use axum_macros::debug_handler;

use super::errors::ProfileServerError;
use super::validation::ValidatedQuery;
use super::super::state::AppState;
use super::util::{check_ip_allowlist, check_realm_is_configured,
                  get_realm, get_player, get_account,
                  enlist_player, make_init_profile_xml,
                  make_account_xml};
use super::util::HEADERS;

use super::params::GetProfileParams;

#[debug_handler]
pub async fn rwr1_get_profile_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<GetProfileParams>
) -> Result<Response, ProfileServerError> {
    // check that the client addr is an allowed ip
    check_ip_allowlist(&state, addr.ip())?;

    // check that the realm has been configured, see fn comments for more detail
    check_realm_is_configured(&state, &params.realm)?;

    // get the realm, making it if it doesn't exist yet
    tracing::info!("locating realm '{}'", &params.realm);
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;

    // find the player, if any
    tracing::info!("finding enlistment papers for player '{}'", &params.username);
    let opt_player = get_player(&state, params.hash, &params.username,
                                                    params.sid, &params.rid).await?;
    match opt_player {
        None => {
            tracing::info!("player '{}' doesn't have any papers, enlisting them (pending checks)", &params.username);
            // enlist player and get back player model
            let player = enlist_player(&state, &params).await?;
            // make an initialisation profile for the player
            let init_profile_xml = make_init_profile_xml(&player.username, &player.rid)?;
            tracing::info!("sending init profile for '{}' in '{}' to game server", &player.username, &realm.name);
            // return xml response
            Ok((StatusCode::OK, HEADERS, init_profile_xml).into_response())
        },
        Some(player) => {
            tracing::info!("found papers for player '{}'", &player.username);
            // we have a player, try to retrieve an account for this player
            let opt_account = get_account(&state, &realm, &player).await?;
            match opt_account {
                None => {
                    // this is the edge-case, a game server can make multiple get_profile requests for a player
                    // before making the first set_profile that inserts/updates a player's account
                    tracing::info!("player '{}' isn't deployed in realm '{}' yet...", player.username, realm.name);
                    // resend another init profile here :D
                    let init_profile_xml = make_init_profile_xml(&player.username, &player.rid)?;
                    tracing::info!("sending init profile for '{}' in '{}' to game server", player.username, realm.name);
                    // return xml response
                    Ok((StatusCode::OK, HEADERS, init_profile_xml).into_response())
                },
                Some(account) => {
                    // found account for player in realm
                    tracing::info!("found account for player '{}' in realm '{}'", player.username, realm.name);
                    let account_xml = make_account_xml(&player, &account)?; 
                    tracing::debug!("{account_xml}");
                    tracing::info!("sending person-profile for '{}' in '{}' to game server", player.username, realm.name);
                    Ok((StatusCode::OK, HEADERS, account_xml).into_response())
                }
            }
        }
    }
}
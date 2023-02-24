use axum::{extract::State, response::Html};
use axum_macros::debug_handler;

use sea_orm::{DatabaseConnection, ActiveValue, ActiveModelTrait};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, error::DbErr};

// use crate::app::errors::ServerError;
use super::errors::ProfileServerError;
use super::super::{state::AppState, validated_query::ValidatedQuery};
use super::util::{digest_ok, get_realm, get_player_from_db, get_account_from_db};

use super::params::GetProfileParams;

#[debug_handler]
pub async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Result<Html<String>, ProfileServerError> {
    // check that this realm is in state.config, this acts as a guard whilst the realm digest algo remains a mystery
    // as we cannot derive the digest from knowing the realm secret and pw, the server expects the realms to be named (e.g. ["INCURSION"]) in the config instead
    // when the first request for a realm is received, it will be created in the db with the digest supplied in the first request
    // this should be fine when the IP allowlist for the profile server endpoints is implemented
    if !state.config.realms.iter().any(|realm_name| realm_name == &params.realm) {
        tracing::error!("realm '{}' not configured", &params.realm);
        return Err(ProfileServerError::RealmNotConfigured(String::from(&params.realm)));
    }
    
    // get the realm
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;
    // verify the realm digest
    if !digest_ok(&params.realm_digest, &realm.digest) {
        tracing::error!("digest provided for realm '{}' incorrect", &params.realm);
        return Err(ProfileServerError::RealmDigestIncorrect(String::from(&params.realm), String::from(&params.realm_digest)));
    }

    // todo: find player in db, if not exist make and send init profile xml
    // if let Some(p) = get_player_from_db(&state.db, params.hash).await? {
    //     tracing::debug!("found player '{}' in db, verifying rid :eyes:", p.username);
    // } else {
    //     tracing::info!("player '{}' not found in db, enlisting them...", &params.username);
    // }

    tracing::info!("checking db for player '{}' in '{}' realm", &params.username, &params.realm);
    // get an optional player and optional account, then match on Some|None to flow to logic for:
    // (None<Player>, None<Account>) - create player and then init profile for player in realm
    // (Some<Player>, None<Account>) - player was created (by get) previously but no set, resend init
    // (Some<Player>, Some<Account>) - the player has some account for this realm, send it to them
    let opt_player = get_player_from_db(&state.db, params.hash).await?;
    let opt_account = get_account_from_db(&state.db, realm.id, params.hash).await?;
    match (opt_player, opt_account) {
        (None, None) => {
            tracing::info!("player '{}' not found in db, enlisting them (pending checks)", &params.username);
            // todo: run complex validation on username here :D
            // todo: create player and then init profile for player in realm
        },
        (Some(player), None) => {
            // todo: player was created (by get) previously but no set, resend init
        },
        (Some(player), Some(account)) => {
            // todo: the player has some account for this realm, send it to them
        }
        (None, Some(_)) => unreachable!("no player but some account wtf")
    }

    let s = format!("{params:#?} {state:#?}");
    Ok(Html(s))
}


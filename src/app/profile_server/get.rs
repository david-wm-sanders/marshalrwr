use axum::{extract::State, response::Html};
use axum_macros::debug_handler;

use sea_orm::{DatabaseConnection, ActiveValue};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, error::DbErr};
use serde::Deserialize;
use validator::Validate;
use subtle::ConstantTimeEq;

// use crate::app::errors::ServerError;
use super::errors::ProfileServerError;

use super::super::{state::AppState, validated_query::ValidatedQuery};
use super::validation::{validate_get_profile_params, validate_username, RE_HEX_STR};
use entity::{Realm, RealmModel, RealmActiveModel, RealmColumn};
use entity::{Player, PlayerModel, PlayerActiveModel, PlayerColumn};

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function="validate_get_profile_params"))]
pub struct GetProfileParams {
    #[validate(range(min=1, max="u32::MAX"))]
    pub hash: u64,
    #[validate(length(min=1, max=32))]
    #[validate(non_control_character)]
    #[validate(custom(function="validate_username"))]
    pub username: String,
    #[validate(length(equal=64))]
    #[validate(regex(path="RE_HEX_STR", code="rid not hexadecimal"))]
    pub rid: String,
    #[validate(range(min=1, max="u32::MAX"))]
    pub sid: u64,
    #[validate(length(min=1, max=32))]
    pub realm: String,
    #[validate(length(equal=64))]
    #[validate(regex(path="RE_HEX_STR", code="realm digest not hexadecimal"))]
    pub realm_digest: String
}

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
    
    // try to get the realm from the db
    // todo: re-evaluate - yuck, get from db every time?!, put a realm cache into state?
    // todo: double re-evaluate - need to use realm model outside of if-let oof
    if let Some(r) = get_realm_from_db(&state.db, &params.realm).await? {
        tracing::debug!("found realm '{}' in db, verifying digest :eyes:", r.name);
        if !digest_ok(&r.digest, &params.realm_digest) {
            tracing::error!("realm digest verification failed!");
            return Err(ProfileServerError::RealmDigestIncorrect(String::from(&params.realm), String::from(&params.realm_digest)));
        }
    } else {
        tracing::info!("realm '{}' not found in db, creating it...", &params.realm);
        // create new realm active model
        let new_realm = RealmActiveModel {
            name: ActiveValue::Set(params.realm.to_owned()),
            digest: ActiveValue::Set(params.realm_digest.to_owned()),
            ..Default::default()
        };
        // insert this new realm into the db
        let insertion_result = Realm::insert(new_realm).exec(&state.db).await?;
        tracing::info!("realm '{}' inserted into db with id:{}", &params.realm, insertion_result.last_insert_id)
    }
    
    // todo: find player in db, if not exist make and send init profile xml
    // if let Some(p) = get_player_from_db(&state.db, params.hash).await? {
    //     tracing::debug!("found player '{}' in db, verifying rid :eyes:", p.username);
    // } else {
    //     tracing::info!("player '{}' not found in db, enlisting them...", &params.username);
    // }

    // get an optional player and optional account, then match on Some|None to flow to logic for:
    // (None<Player>, None<Account>) - create player and then init profile for player in realm
    // (Some<Player>, None<Account>) - player was created (by get) previously but no set, resend init
    // (Some<Player>, Some<Account>) - the player has some account for this realm, send it to them
    let opt_player = get_player_from_db(&state.db, params.hash).await?;

    let s = format!("{params:#?} {state:#?}");
    Ok(Html(s))
}

pub fn digest_ok(given_digest: &str, valid_digest: &str) -> bool {
    // check the realm digest in constant time mit subtle crate
    // todo: validate that this actually works in constant time XD
    let given_digest_bytes = given_digest.as_bytes();
    let valid_digest_bytes = valid_digest.as_bytes();
    given_digest_bytes.ct_eq(valid_digest_bytes).into()
}

pub async fn get_realm_from_db(db_conn: &DatabaseConnection, realm_name: &str) -> Result<Option<RealmModel>, DbErr> {
    Ok(Realm::find().filter(RealmColumn::Name.eq(realm_name)).one(db_conn).await?)
}

pub async fn get_player_from_db(db_conn: &DatabaseConnection, player_hash: u64) -> Result<Option<PlayerModel>, DbErr> {
    Ok(Player::find().filter(PlayerColumn::Hash.eq(player_hash)).one(db_conn).await?)
}

pub async fn get_player_from_db_by_name(db_conn: &DatabaseConnection, username: &str) -> Result<Option<()>, DbErr> {
    Ok(None)
}

pub async fn get_realm(state: State<AppState>, realm_name: &str) -> Option<RealmModel> {
    None
    // todo: try get from realm cache in state and return Some
    // todo: if not found, try from db, and if found in db add to realm cache then return Some
}
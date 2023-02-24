use std::sync::Arc;

use sea_orm::{DatabaseConnection, ActiveValue, ActiveModelTrait};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, error::DbErr};
use subtle::ConstantTimeEq;

use super::errors::ProfileServerError;
use super::params::GetProfileParams;
use super::super::{state::AppState, validated_query::ValidatedQuery};
use entity::{Realm, RealmModel, RealmActiveModel, RealmColumn};
use entity::{Player, PlayerModel, PlayerActiveModel, PlayerColumn};
use entity::{Account, AccountModel, AccountActiveModel, AccountColumn};

pub fn check_realm_is_configured(state: &AppState, realm: &str) -> Result<(), ProfileServerError> {
    // check that this realm is in state.config, this acts as a guard whilst the realm digest algo remains a mystery
    // as we cannot derive the digest from knowing the realm secret and pw, the server expects the realms to be named (e.g. ["INCURSION"]) in the config instead
    // when the first request for a realm is received, it will be created in the db with the digest supplied in the first request
    // this should be fine when the IP allowlist for the profile server endpoints is implemented
    if !state.config.realms.iter().any(|realm_name| realm_name == realm) {
        tracing::error!("realm '{}' not configured", realm);
        return Err(ProfileServerError::RealmNotConfigured(String::from(realm)));
    }
    Ok(())
}

pub fn digest_ok(given_digest: &str, valid_digest: &str) -> bool {
    // check the realm digest in constant time mit subtle crate
    // todo: validate that this actually works in constant time XD
    let given_digest_bytes = given_digest.as_bytes();
    let valid_digest_bytes = valid_digest.as_bytes();
    given_digest_bytes.ct_eq(valid_digest_bytes).into()
}

pub fn verify_digest(realm_name: &str, realm_digest: &str, valid_digest: &str) -> Result<(), ProfileServerError> {
    if !digest_ok(realm_digest, valid_digest) {         
        tracing::error!("digest provided for realm '{}' incorrect", realm_name);
        return Err(ProfileServerError::RealmDigestIncorrect(
            String::from(realm_name),
            String::from(realm_digest)));
    }
    Ok(())
}

pub async fn get_realm_from_db(db_conn: &DatabaseConnection, realm_name: &str) -> Result<Option<RealmModel>, DbErr> {
    Ok(Realm::find().filter(RealmColumn::Name.eq(realm_name)).one(db_conn).await?)
}

pub async fn get_player_from_db(db_conn: &DatabaseConnection, player_hash: i64) -> Result<Option<PlayerModel>, DbErr> {
    // get player by i64 hash id
    Ok(Player::find_by_id(player_hash).one(db_conn).await?)
}

pub async fn get_account_from_db(db_conn: &DatabaseConnection, realm_id: i32, player_hash: i64) -> Result<Option<AccountModel>, DbErr> {
    // get the account by (realm_id, player_hash)
    Ok(Account::find_by_id((realm_id, player_hash)).one(db_conn).await?)
}

pub async fn get_realm(state: &AppState, realm_name: &str, realm_digest: &str) -> Result<Arc<RealmModel>, ProfileServerError> {
    tracing::debug!("searching for realm '{realm_name}' in realm cache");
    match state.cache.realms.get(realm_name) {
        Some(realm) => {
            tracing::debug!("located realm '{realm_name}' [{}] in realm cache", realm.id);
            // verify the realm digest
            verify_digest(realm_name, realm_digest, &realm.digest)?;
            return Ok(realm);
        },
        None => {
            tracing::debug!("realm '{realm_name}' not found in cache, querying db");
            match get_realm_from_db(&state.db, realm_name).await? {
                Some(realm) => {
                    tracing::debug!("located realm '{realm_name}' [{}] in db, caching it", realm.id);
                    // insert the model into the realm cache
                    let arc_model = Arc::new(realm.clone());
                    state.cache.realms.insert(String::from(realm_name), arc_model.clone()).await;
                    // verify the realm digest
                    verify_digest(realm_name, realm_digest, &realm.digest)?;
                    return Ok(arc_model);
                },
                None => {
                    tracing::debug!("realm '{}' not found in db, creating it...", realm_name);
                    // create new realm active model
                    let new_realm = RealmActiveModel {
                        name: ActiveValue::Set(realm_name.to_owned()),
                        digest: ActiveValue::Set(realm_digest.to_owned()),
                        ..Default::default()
                    };
                    // insert this new realm into the db and return model
                    let realm = new_realm.insert(&state.db).await?;
                    tracing::debug!("created realm '{}' [{}] in db", realm_name, realm.id);
                    // insert the model into the realm cache
                    let arc_model = Arc::new(realm);
                    state.cache.realms.insert(String::from(realm_name), arc_model.clone()).await;
                    return Ok(arc_model);
                }
            }
        }
    }
}

// pub async fn get_player_from_db_by_name(db_conn: &DatabaseConnection, username: &str) -> Result<Option<()>, DbErr> {
//     todo!();
//     Ok(None)
// }

pub async fn get_player(state: &AppState, params: &GetProfileParams) -> Result<Option<PlayerModel>, ProfileServerError> {
    // tracing::debug!("searching for player '{}' in player cache", params.username);
    // let mut opt_player: Option<PlayerModel> = None;
    // {
    //     // we enclose cache_reader operations inside a scope here to ensure that the compiler
    //     // understands it won't persist across any await (and thus require Send, which it isn't)
    //     let cache_reader = state.player_cache.read().unwrap();
    //     if let Some(cached_model) = cache_reader.get(&params.hash) {
    //         opt_player = Some(cached_model.clone())
    //     }
    // }
    // // if some player with this hash can be found in the player cache, return it
    // if let Some(player) = opt_player {
    //     tracing::debug!("located player '{}' in player cache", &params.username);
    //     return Ok(Some(player.clone()));
    // } else {
    //     tracing::debug!("player '{}' not found in cache, querying db", &params.username);
    // }
    
    // else {
    //     // if some realm with this name can be found in the db, add it to the cache and return it
    //     tracing::debug!("realm '{realm_name}' not found in cache, querying db for realm");
    //     if let Some(realm) = get_realm_from_db(&state.db, realm_name).await? {
    //         tracing::debug!("located realm '{realm_name}' [{}] in db, caching it", realm.id);
    //         // insert the realm into the realm cache
    //         let mut cache_writer = state.realm_cache.write().unwrap();
    //         // todo: perhaps should double-check here that realm wasn't added by other thread/task before this write lock acquired?
    //         cache_writer.insert(String::from(realm_name), realm.clone());
    //         drop(cache_writer);
    //         return Ok(realm);
    //     } else {
    //         tracing::debug!("realm '{}' not found in db, creating it...", realm_name);
    //         // create new realm active model
    //         let new_realm = RealmActiveModel {
    //             name: ActiveValue::Set(realm_name.to_owned()),
    //             digest: ActiveValue::Set(realm_digest.to_owned()),
    //             ..Default::default()
    //         };
    //         // insert this new realm into the db and return model
    //         let realm = new_realm.insert(&state.db).await?;
    //         tracing::debug!("created realm '{}' in db", realm_name);
    //         // insert the realm into the realm cache
    //         let mut cache_writer = state.realm_cache.write().unwrap();
    //         // todo: perhaps should double-check here that realm wasn't added by other thread/task before this write lock acquired?
    //         cache_writer.insert(String::from(realm_name), realm.clone());
    //         drop(cache_writer);
    //         return Ok(realm);
    //     }
    // }
    Ok(None)
}
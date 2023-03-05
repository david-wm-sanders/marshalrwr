use std::sync::Arc;
use std::io::Cursor;

use sea_orm::{DatabaseConnection, ActiveValue, ActiveModelTrait};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, error::DbErr};
use serde::Serialize;
use subtle::ConstantTimeEq;
use quick_xml::{events::{Event, BytesStart, BytesEnd}, writer::Writer, escape::{escape, unescape}};
use quick_xml::se::Serializer as QuickXmlSerializer;

use super::errors::ProfileServerError;
use super::params::GetProfileParams;
use super::super::state::AppState;
use super::xml::GetProfileDataXml;
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

pub fn verify_realm_digest(realm_name: &str, realm_digest: &str, valid_digest: &str) -> Result<(), ProfileServerError> {
    if !digest_ok(realm_digest, valid_digest) {         
        tracing::error!("digest provided for realm '{}' incorrect", realm_name);
        return Err(ProfileServerError::RealmDigestIncorrect(
            String::from(realm_name),
            String::from(realm_digest)));
    }
    Ok(())
}

pub fn verify_player_sid_and_rid(hash: i64, username: &str,
                                 sid: i64, expected_sid: i64,
                                 rid: &str, valid_rid: &str) -> Result<(), ProfileServerError> {
    if sid != expected_sid {
        return Err(ProfileServerError::PlayerSidMismatch(hash, String::from(username), sid, expected_sid));
    }

    if !digest_ok(rid, valid_rid) {
        tracing::error!("rid provided for player '{}' incorrect", username);
        return Err(ProfileServerError::PlayerRidIncorrect(hash, String::from(username), sid, String::from(rid)));
    }
    Ok(())
}

pub async fn get_realm_from_db(db_conn: &DatabaseConnection, realm_name: &str) -> Result<Option<RealmModel>, DbErr> {
    let realm = Realm::find().filter(RealmColumn::Name.eq(realm_name)).one(db_conn).await?;
    Ok(realm)
}

pub async fn get_realm(state: &AppState, realm_name: &str, realm_digest: &str) -> Result<Arc<RealmModel>, ProfileServerError> {
    tracing::debug!("searching for realm '{realm_name}' in realm cache");
    match state.cache.realms.get(realm_name) {
        Some(realm) => {
            tracing::debug!("located realm '{realm_name}' [{}] in realm cache", realm.id);
            // verify the realm digest
            verify_realm_digest(realm_name, realm_digest, &realm.digest)?;
            Ok(realm)
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
                    verify_realm_digest(realm_name, realm_digest, &realm.digest)?;
                    Ok(arc_model)
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
                    Ok(arc_model)
                }
            }
        }
    }
}

// pub async fn get_player_from_db_by_name(db_conn: &DatabaseConnection, username: &str) -> Result<Option<()>, DbErr> {
//     todo!();
//     Ok(None)
// }

pub async fn get_player_from_db(db_conn: &DatabaseConnection, player_hash: i64) -> Result<Option<PlayerModel>, DbErr> {
    // get player by i64 hash id
    let player = Player::find_by_id(player_hash).one(db_conn).await?;
    Ok(player)
}

pub async fn get_player(state: &AppState,
                        player_hash: i64, username: &str,
                        sid: i64, rid: &str
                        ) -> Result<Option<Arc<PlayerModel>>, ProfileServerError> {
    tracing::debug!("searching for player '{}' [{}] in player cache", username, player_hash);
    match state.cache.players.get(&player_hash) {
        Some(player) => {
            tracing::debug!("found player '{}' [{}] in player cache", username, player_hash);
            // verify the player sid and rid (digest)
            verify_player_sid_and_rid(player_hash, username,
                                      sid, player.sid,
                                      rid, &player.rid)?;
            Ok(Some(player))
        },
        None => {
            tracing::debug!("player '{}' [{}] not found in cache, querying db", username, player_hash);
            match get_player_from_db(&state.db, player_hash).await? {
                Some(player) => {
                    tracing::debug!("found player '{}' [{}] in db, caching it", username, player.hash);
                    // insert the model into the player cache
                    let arc_model = Arc::new(player.clone());
                    state.cache.players.insert(player_hash, arc_model.clone()).await;
                    // verify the player sid and rid (digest)
                    verify_player_sid_and_rid(player_hash, username,
                                              sid, player.sid,
                                              rid, &player.rid)?;
                    Ok(Some(arc_model))
                },
                None => {
                    tracing::debug!("player '{}' [{}] not found in db", username, player_hash);
                    Ok(None)
                }
            }
        }
    }
}

pub async fn get_account_from_db(db_conn: &DatabaseConnection, realm_id: i32, player_hash: i64) -> Result<Option<AccountModel>, DbErr> {
    // get the account by (realm_id, player_hash)
    let account = Account::find_by_id((realm_id, player_hash)).one(db_conn).await?;
    Ok(account)
}

pub async fn get_account(state: &AppState, realm: &Arc<RealmModel>, player: &Arc<PlayerModel>) -> Result<Option<Arc<AccountModel>>, ProfileServerError> {
    tracing::debug!("searching for account ('{}','{}') in account cache", realm.name, player.username);
    match state.cache.accounts.get(&(realm.id, player.hash)) {
        Some(account) => {
            tracing::debug!("found account ('{}','{}') in account cache", realm.name, player.username);
            Ok(Some(account))
        },
        None => {
            tracing::debug!("account ('{}','{}') not found in account cache, querying db", realm.name, player.username);
            match get_account_from_db(&state.db, realm.id, player.hash).await? {
                Some(account) => {
                    tracing::debug!("found account ('{}','{}') in db, caching it", realm.name, player.username);
                    // insert the model into the account cache
                    let arc_model = Arc::new(account.clone());
                    state.cache.accounts.insert((realm.id, player.hash), arc_model.clone()).await;
                    Ok(Some(arc_model))
                },
                None => {
                    tracing::debug!("account ('{}','{}') not found in db", realm.name, player.username);
                    Ok(None)
                }
            }
        }
    }
}

pub async fn enlist_player(state: &AppState, params: &GetProfileParams) -> Result<Arc<PlayerModel>, ProfileServerError> {
    // todo: do any stateful validation of params now - e.g. check username against blocklist
    tracing::debug!("creating papers for player '{}'", &params.username);
    let new_player = PlayerActiveModel {
        hash: ActiveValue::Set(params.hash),
        username: ActiveValue::Set(params.username.to_owned()),
        sid: ActiveValue::Set(params.sid),
        rid: ActiveValue::Set(params.rid.to_owned())
    };
    // insert new player into db
    let player = new_player.insert(&state.db).await?;
    tracing::debug!("inserted papers for player '{}' into db", &params.username);
    let arc_player = Arc::new(player);
    state.cache.players.insert(params.hash, arc_player.clone()).await;
    tracing::debug!("inserted papers for player '{}' into player cache", &params.username);
    Ok(arc_player)
}

pub fn make_init_profile_xml(username: &str, rid: &str) -> Result<String, ProfileServerError> {
    let mut init_xml_writer = Writer::new(Cursor::new(Vec::new()));
    let mut data_element_start = BytesStart::new("data");
    let data_element_end = BytesEnd::new("data");
    data_element_start.push_attribute(("ok", "1"));
    init_xml_writer.write_event(Event::Start(data_element_start))?;
    let mut profile_element = BytesStart::new("profile");
    // the username could contain characters like <, >, etc so must be escaped
    profile_element.push_attribute(("username", escape(username).as_ref()));
    profile_element.push_attribute(("rid", rid));
    init_xml_writer.write_event(Event::Empty(profile_element))?;
    init_xml_writer.write_event(Event::End(data_element_end))?;
    let mut result = String::from_utf8(init_xml_writer.into_inner().into_inner()).unwrap();
    // append a newline to the end of the XML otherwise the rwr game server XML parser won't be happy :D
    result.push_str("\n");
    Ok(result)
}

pub fn make_account_xml(player: &Arc<PlayerModel>, account: &Arc<AccountModel>) -> String {
    // todo: make return Result and improve error handling
    let data = GetProfileDataXml::new(player, account);
    let serializer = QuickXmlSerializer::with_root(String::new(), Some("data")).unwrap();
    let mut xml = data.serialize(serializer).unwrap();
    xml.push_str("\n");
    xml
}
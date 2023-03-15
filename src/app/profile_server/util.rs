use std::io::Cursor;
use std::net::IpAddr;
use std::sync::Arc;

use axum::http::header::{self, HeaderName};
use quick_xml::se::Serializer as QuickXmlSerializer;
use quick_xml::{
    escape::escape,
    events::{BytesEnd, BytesStart, Event},
    writer::Writer,
};
use sea_orm::{error::DbErr, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use serde::Serialize;
use subtle::ConstantTimeEq;

use super::super::state::AppState;
use super::errors::ProfileServerError;
use super::json::{ItemStore, Loadout, KillCombos, CriteriaMonitor, CriteriaMonitors};
use super::params::GetProfileParams;
use super::xml::{GetProfileDataXml, PlayerXml};
use entity::{Account, AccountActiveModel, AccountColumn, AccountModel};
use entity::{Player, PlayerActiveModel, PlayerModel};
use entity::{Realm, RealmActiveModel, RealmColumn, RealmModel};

pub const HEADERS: [(HeaderName, &str); 1] = [(header::CONTENT_TYPE, "text/xml")];
pub const USERNAME_BLOCKED_CHARS: [char; 5] = ['"', '\'', ',', ';', '`'];
pub const ACCOUNT_COLUMNS: [AccountColumn; 31] = [
    AccountColumn::RealmId,
    AccountColumn::Hash,
    AccountColumn::GameVersion,
    AccountColumn::SquadTag,
    AccountColumn::MaxAuthorityReached,
    AccountColumn::Authority,
    AccountColumn::JobPoints,
    AccountColumn::Faction,
    AccountColumn::Name,
    AccountColumn::SoldierGroupId,
    AccountColumn::SoldierGroupName,
    AccountColumn::SquadSizeSetting,
    AccountColumn::Loadout,
    AccountColumn::Backpack,
    AccountColumn::Stash,
    AccountColumn::Kills,
    AccountColumn::Deaths,
    AccountColumn::TimePlayed,
    AccountColumn::PlayerKills,
    AccountColumn::Teamkills,
    AccountColumn::LongestKillStreak,
    AccountColumn::TargetsDestroyed,
    AccountColumn::VehiclesDestroyed,
    AccountColumn::SoldiersHealed,
    AccountColumn::DistanceMoved,
    AccountColumn::ShotsFired,
    AccountColumn::ThrowablesThrown,
    AccountColumn::RankProgression,
    AccountColumn::LongestDeathStreak,
    AccountColumn::KillCombos,
    AccountColumn::CriteriaMonitors,
];

pub fn check_ip_allowlist(state: &AppState, ip: IpAddr) -> Result<(), ProfileServerError> {
    if !state.config.ps_allowed_ips.contains(&ip) {
        return Err(ProfileServerError::ClientAddressNotAllowed(ip));
    }
    Ok(())
}

pub fn check_realm_is_configured(state: &AppState, realm: &str) -> Result<(), ProfileServerError> {
    // check that this realm is in state.config, this acts as a guard whilst the realm digest algo remains a mystery
    // as we cannot derive the digest from knowing the realm secret and pw, the server expects the realms to be named (e.g. ["INCURSION"]) in the config instead
    // when the first request for a realm is received, it will be created in the db with the digest supplied in the first request
    // this should be fine when the IP allowlist for the profile server endpoints is implemented
    if !state.config.ps_realms.contains(realm) {
        return Err(ProfileServerError::RealmNotConfigured(String::from(realm)));
    }
    Ok(())
}

pub fn check_sid(state: &AppState, sid: i64) -> Result<(), ProfileServerError> {
    if !state.config.ps_allowed_sids.is_empty() {
        if !state.config.ps_allowed_sids.contains(&sid) {
            return Err(ProfileServerError::SidNotAllowed(sid));
        }
    }
    if state.config.ps_blocked_sids.contains(&sid) {
        return Err(ProfileServerError::SidBlocked(sid));
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

pub fn verify_realm_digest(
    realm_name: &str,
    realm_digest: &str,
    valid_digest: &str,
) -> Result<(), ProfileServerError> {
    if !digest_ok(realm_digest, valid_digest) {
        return Err(ProfileServerError::RealmDigestIncorrect(
            String::from(realm_name),
            String::from(realm_digest),
        ));
    }
    Ok(())
}

pub fn verify_player_sid_and_rid(
    hash: i64,
    username: &str,
    sid: i64,
    expected_sid: i64,
    rid: &str,
    valid_rid: &str,
) -> Result<(), ProfileServerError> {
    if sid != expected_sid {
        return Err(ProfileServerError::PlayerSidMismatch(
            hash,
            String::from(username),
            sid,
            expected_sid,
        ));
    }

    if !digest_ok(rid, valid_rid) {
        return Err(ProfileServerError::PlayerRidIncorrect(
            hash,
            String::from(username),
            sid,
            String::from(rid),
        ));
    }
    Ok(())
}

pub async fn get_realm_from_db(
    db_conn: &DatabaseConnection,
    realm_name: &str,
) -> Result<Option<RealmModel>, DbErr> {
    let realm = Realm::find()
        .filter(RealmColumn::Name.eq(realm_name))
        .one(db_conn)
        .await?;
    Ok(realm)
}

pub async fn get_realm(
    state: &AppState,
    realm_name: &str,
    realm_digest: &str,
) -> Result<Arc<RealmModel>, ProfileServerError> {
    // search for realm in cache
    match state.cache.realms.get(realm_name) {
        Some(realm) => {
            tracing::debug!("located realm '{realm_name}' [{}] in cache", realm.id);
            // verify the realm digest
            verify_realm_digest(realm_name, realm_digest, &realm.digest)?;
            Ok(realm)
        }
        None => {
            // realm not found in cache, query db
            match get_realm_from_db(&state.db, realm_name).await? {
                Some(realm) => {
                    tracing::debug!(
                        "located realm '{realm_name}' [{}] in db, caching it",
                        realm.id
                    );
                    // insert the model into the realm cache
                    let arc_model = Arc::new(realm.clone());
                    state
                        .cache
                        .realms
                        .insert(String::from(realm_name), arc_model.clone())
                        .await;
                    // verify the realm digest
                    verify_realm_digest(realm_name, realm_digest, &realm.digest)?;
                    Ok(arc_model)
                }
                None => {
                    tracing::debug!("realm '{}' not found in db, creating it...", realm_name);
                    // create new realm active model
                    let new_realm = RealmActiveModel {
                        name: ActiveValue::Set(realm_name.to_owned()),
                        digest: ActiveValue::Set(realm_digest.to_owned()),
                        ..Default::default()
                    };
                    // insert this new realm into the db
                    let realm = new_realm.insert(&state.db).await?;
                    tracing::debug!("created realm '{}' [{}] in db", realm_name, realm.id);
                    // insert the model into the realm cache
                    let arc_model = Arc::new(realm);
                    state
                        .cache
                        .realms
                        .insert(String::from(realm_name), arc_model.clone())
                        .await;
                    Ok(arc_model)
                }
            }
        }
    }
}

pub async fn get_player_from_db(
    db_conn: &DatabaseConnection,
    player_hash: i64,
) -> Result<Option<PlayerModel>, DbErr> {
    // get player by i64 hash id
    let player = Player::find_by_id(player_hash).one(db_conn).await?;
    Ok(player)
}

pub async fn get_player(
    state: &AppState,
    player_hash: i64,
    username: &str,
    sid: i64,
    rid: &str,
) -> Result<Option<Arc<PlayerModel>>, ProfileServerError> {
    // search for player in cache
    match state.cache.players.get(&player_hash) {
        Some(player) => {
            tracing::debug!(
                "found player '{}' [{}] in cache",
                username,
                player_hash
            );
            // verify the player sid and rid (digest)
            verify_player_sid_and_rid(player_hash, username, sid, player.sid, rid, &player.rid)?;
            Ok(Some(player))
        }
        None => {
            tracing::debug!(
                "player '{}' [{}] not found in cache, querying db",
                username,
                player_hash
            );
            match get_player_from_db(&state.db, player_hash).await? {
                Some(player) => {
                    tracing::debug!(
                        "found player '{}' [{}] in db, caching it",
                        username,
                        player.hash
                    );
                    // insert the model into the player cache
                    let arc_model = Arc::new(player.clone());
                    state
                        .cache
                        .players
                        .insert(player_hash, arc_model.clone())
                        .await;
                    // verify the player sid and rid (digest)
                    verify_player_sid_and_rid(
                        player_hash,
                        username,
                        sid,
                        player.sid,
                        rid,
                        &player.rid,
                    )?;
                    Ok(Some(arc_model))
                }
                None => {
                    tracing::debug!("player '{}' [{}] not found in db", username, player_hash);
                    Ok(None)
                }
            }
        }
    }
}

pub async fn get_account_from_db(
    db_conn: &DatabaseConnection,
    realm_id: i32,
    player_hash: i64,
) -> Result<Option<AccountModel>, DbErr> {
    // get the account by (realm_id, player_hash)
    let account = Account::find_by_id((realm_id, player_hash))
        .one(db_conn)
        .await?;
    Ok(account)
}

pub async fn get_account(
    state: &AppState,
    realm: &Arc<RealmModel>,
    player: &Arc<PlayerModel>,
) -> Result<Option<Arc<AccountModel>>, ProfileServerError> {
    // search for account in cache
    match state.cache.accounts.get(&(realm.id, player.hash)) {
        Some(account) => {
            tracing::debug!(
                "found account ('{}','{}') in cache",
                realm.name,
                player.username
            );
            Ok(Some(account))
        }
        None => {
            // account not found in account cache, query db
            match get_account_from_db(&state.db, realm.id, player.hash).await? {
                Some(account) => {
                    tracing::debug!(
                        "found account ('{}','{}') in db, caching it",
                        realm.name,
                        player.username
                    );
                    // insert the model into the account cache
                    let arc_model = Arc::new(account.clone());
                    state
                        .cache
                        .accounts
                        .insert((realm.id, player.hash), arc_model.clone())
                        .await;
                    Ok(Some(arc_model))
                }
                None => {
                    tracing::debug!(
                        "account ('{}','{}') not found in db",
                        realm.name,
                        player.username
                    );
                    Ok(None)
                }
            }
        }
    }
}

pub async fn enlist_player(
    state: &AppState,
    params: &GetProfileParams,
) -> Result<Arc<PlayerModel>, ProfileServerError> {
    // todo: do any stateful validation of params now - e.g. check username against blocklist
    tracing::debug!("creating papers for player '{}'...", &params.username);
    let new_player = PlayerActiveModel {
        hash: ActiveValue::Set(params.hash),
        username: ActiveValue::Set(params.username.to_owned()),
        sid: ActiveValue::Set(params.sid),
        rid: ActiveValue::Set(params.rid.to_owned()),
    };
    // insert new player into db
    let player = new_player.insert(&state.db).await?;
    tracing::debug!("inserted papers for player '{}' into db", &params.username);
    let arc_player = Arc::new(player);
    state
        .cache
        .players
        .insert(params.hash, arc_player.clone())
        .await;
    tracing::debug!(
        "inserted papers for player '{}' into cache",
        &params.username
    );
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
    let mut result = String::from_utf8(init_xml_writer.into_inner().into_inner())?;
    // append a newline to the end of the XML otherwise the rwr game server XML parser won't be happy :D
    result.push('\n');
    Ok(result)
}

pub fn make_account_model(
    realm_id: i32,
    player_xml: &PlayerXml,
) -> Result<AccountActiveModel, ProfileServerError> {
    // process loadout, backpack and stash
    let loadout = Loadout::new(&player_xml.person.equipped_items);
    let loadout_json = serde_json::to_string(&loadout)?;
    let backpack_store = ItemStore::new(&player_xml.person.backpack.items);
    let backpack_json = serde_json::to_string(&backpack_store)?;
    let stash_store = ItemStore::new(&player_xml.person.stash.items);
    let stash_json = serde_json::to_string(&stash_store)?;
    // process monitors
    let mut longest_death_steak = 0;
    let mut kill_combo_json = String::new();
    let mut monitors: Vec<CriteriaMonitor> = Vec::new();
    for monitor_xml in &player_xml.profile.stats.monitors {
        if monitor_xml.name == Some(String::from("kill combo")) {
            // process the kill combo monitor
            let kill_combo = KillCombos::new(&monitor_xml.entries);
            kill_combo_json = serde_json::to_string(&kill_combo)?;
        } else if monitor_xml.name == Some(String::from("death streak")) {
            // process the death streak monitor
            longest_death_steak = monitor_xml.longest_death_streak.unwrap_or(0);
        } else if monitor_xml.name == None {
            // some monitor xml are empty xd, skip
            continue;
        } else {
            // every other monitor
            let critera_monitor = CriteriaMonitor::new(monitor_xml);
            monitors.push(critera_monitor);
        }
    }
    let criteria_monitors = CriteriaMonitors { monitors };
    let criteria_monitors_json = serde_json::to_string(&criteria_monitors)?;

    // make an active account model
    let account_model = AccountActiveModel {
        realm_id: ActiveValue::Set(realm_id),
        hash: ActiveValue::Set(player_xml.hash),
        game_version: ActiveValue::Set(player_xml.profile.game_version),
        squad_tag: ActiveValue::Set(player_xml.profile.squad_tag.to_owned()),
        max_authority_reached: ActiveValue::Set(player_xml.person.max_authority_reached as f64),
        authority: ActiveValue::Set(player_xml.person.authority as f64),
        job_points: ActiveValue::Set(player_xml.person.job_points as f64),
        faction: ActiveValue::Set(player_xml.person.faction),
        name: ActiveValue::Set(player_xml.person.name.to_owned()),
        soldier_group_id: ActiveValue::Set(player_xml.person.soldier_group_id),
        soldier_group_name: ActiveValue::Set(player_xml.person.soldier_group_name.to_owned()),
        squad_size_setting: ActiveValue::Set(player_xml.person.squad_size_setting),
        loadout: ActiveValue::Set(loadout_json),
        backpack: ActiveValue::Set(backpack_json),
        stash: ActiveValue::Set(stash_json),
        kills: ActiveValue::Set(player_xml.profile.stats.kills),
        deaths: ActiveValue::Set(player_xml.profile.stats.deaths),
        time_played: ActiveValue::Set(player_xml.profile.stats.time_played as i32),
        player_kills: ActiveValue::Set(player_xml.profile.stats.player_kills),
        teamkills: ActiveValue::Set(player_xml.profile.stats.teamkills),
        longest_kill_streak: ActiveValue::Set(player_xml.profile.stats.longest_kill_streak),
        targets_destroyed: ActiveValue::Set(player_xml.profile.stats.targets_destroyed),
        vehicles_destroyed: ActiveValue::Set(player_xml.profile.stats.vehicles_destroyed),
        soldiers_healed: ActiveValue::Set(player_xml.profile.stats.soldiers_healed),
        distance_moved: ActiveValue::Set(player_xml.profile.stats.distance_moved as f64),
        shots_fired: ActiveValue::Set(player_xml.profile.stats.shots_fired),
        throwables_thrown: ActiveValue::Set(player_xml.profile.stats.throwables_thrown),
        rank_progression: ActiveValue::Set(player_xml.profile.stats.rank_progression as f64),
        longest_death_streak: ActiveValue::Set(longest_death_steak),
        kill_combos: ActiveValue::Set(kill_combo_json),
        criteria_monitors: ActiveValue::Set(criteria_monitors_json),
    };
    Ok(account_model)
}

pub fn make_account_xml(
    player: &Arc<PlayerModel>,
    account: &Arc<AccountModel>,
) -> Result<String, ProfileServerError> {
    let data = GetProfileDataXml::new(player, account)?;
    let serializer = QuickXmlSerializer::with_root(String::new(), Some("data"))?;
    let mut xml = data.serialize(serializer)?;
    xml.push('\n');
    Ok(xml)
}

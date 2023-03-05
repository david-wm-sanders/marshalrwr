use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_macros::debug_handler;
use sea_orm::ActiveValue;

use super::super::state::AppState;
use super::errors::ProfileServerError;
use super::validation::{ValidatedQuery, ValidatedXmlBody};
use super::xml::SetProfileDataXml;

use super::util::{check_realm_is_configured, get_realm, get_player};
use super::params::SetProfileParams;

use entity::AccountActiveModel;

#[debug_handler]
pub async fn rwr1_set_profile_handler(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<SetProfileParams>,
    ValidatedXmlBody(data): ValidatedXmlBody<SetProfileDataXml>,
) -> Result<Response, ProfileServerError> {
    let headers = [(header::CONTENT_TYPE, "text/xml")];

    // check that the realm has been configured, see fn comments for more detail
    check_realm_is_configured(&state, &params.realm)?;

    // get the realm, making it if it doesn't exist yet
    tracing::info!("locating realm '{}'", &params.realm);
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;

    tracing::debug!("{data:#?}");
    let mut accounts_to_update: Vec<AccountActiveModel> = Vec::new();
    for player_xml in data.players.iter() {
        tracing::debug!("processing set xml for player '{}'", player_xml.hash);
        // get the player from cache/db, remembering that get_player does all the account sid/rid verification
        // by itself if it encounters an existing player in the cache or db
        let opt_player = get_player(&state, player_xml.hash, &player_xml.profile.username,
                                                        player_xml.profile.sid, &player_xml.rid).await?;
        match opt_player {
            None => {
                // a set request was made for a player not in db (for which no get was made first)
                // atm, this will invalidate the entire set xml data by erroring thus:
                return Err(ProfileServerError::PlayerNotFound(player_xml.hash, player_xml.profile.username.to_owned(), player_xml.profile.sid));
                // todo: improve?!
            },
            Some(player) => {
                tracing::debug!("creating account model for player '{}' from xml...", &player.username);
                // todo: construct account active model from player xml
                let account = AccountActiveModel {
                    realm_id: ActiveValue::Set(realm.id),
                    hash: ActiveValue::Set(player_xml.hash),
                    // todo: grr annoying, have to make Some(v) here because schema doesn't use .not_null() for these...
                    game_version: ActiveValue::Set(Some(player_xml.profile.game_version)),
                    squad_tag: ActiveValue::Set(Some(player_xml.profile.squad_tag.to_owned())),
                    // todo: double annoy - docs says schema .float() should make f32 in entity, entity gen issue?
                    max_authority_reached: ActiveValue::Set(Some(player_xml.person.max_authority_reached as f64)),
                    authority: ActiveValue::Set(Some(player_xml.person.authority as f64)),
                    job_points: ActiveValue::Set(Some(player_xml.person.job_points as f64)),
                    faction: ActiveValue::Set(Some(player_xml.person.faction)),
                    name: ActiveValue::Set(Some(player_xml.person.name.to_owned())),
                    soldier_group_id: ActiveValue::Set(Some(player_xml.person.soldier_group_id)),
                    soldier_group_name: ActiveValue::Set(Some(player_xml.person.soldier_group_name.to_owned())),
                    squad_size_setting: ActiveValue::Set(Some(player_xml.person.squad_size_setting)),
                    kills: ActiveValue::Set(Some(player_xml.profile.stats.kills)),
                    deaths: ActiveValue::Set(Some(player_xml.profile.stats.deaths)),
                    time_played: ActiveValue::Set(Some(player_xml.profile.stats.time_played as i32)),
                    player_kills: ActiveValue::Set(Some(player_xml.profile.stats.player_kills)),
                    teamkills: ActiveValue::Set(Some(player_xml.profile.stats.teamkills)),
                    longest_kill_streak: ActiveValue::Set(Some(player_xml.profile.stats.longest_kill_streak)),
                    targets_destroyed: ActiveValue::Set(Some(player_xml.profile.stats.targets_destroyed)),
                    vehicles_destroyed: ActiveValue::Set(Some(player_xml.profile.stats.vehicles_destroyed)),
                    soldiers_healed: ActiveValue::Set(Some(player_xml.profile.stats.soldiers_healed)),
                    distance_moved: ActiveValue::Set(Some(player_xml.profile.stats.distance_moved as f64)),
                    shots_fired: ActiveValue::Set(Some(player_xml.profile.stats.shots_fired)),
                    throwables_thrown: ActiveValue::Set(Some(player_xml.profile.stats.throwables_thrown)),
                    rank_progression: ActiveValue::Set(Some(player_xml.profile.stats.rank_progression as f64))
                };
                accounts_to_update.push(account);
            }
        }
    }
    // invalidate these accounts in the cache
    for account in accounts_to_update.iter() {
        let hash = account.hash.clone().unwrap();
        state.cache.accounts.invalidate(&(realm.id, hash)).await;
    }
    // todo: insert many active model accounts with on_conflict to update
    Ok((StatusCode::OK).into_response())
}

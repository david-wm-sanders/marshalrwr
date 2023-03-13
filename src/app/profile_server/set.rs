use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_macros::debug_handler;
use migration::OnConflict;
use sea_orm::EntityTrait;

use super::super::state::AppState;
use super::errors::ProfileServerError;
use super::validation::{ValidatedQuery, ValidatedXmlBody};
use super::xml::SetProfileDataXml;

use super::params::SetProfileParams;
use super::util::{
    check_ip_allowlist, check_realm_is_configured, get_player, get_realm, make_account_model,
};
use super::util::{ACCOUNT_COLUMNS, HEADERS};

use entity::{Account, AccountActiveModel, AccountColumn, AccountModel};

#[debug_handler]
pub async fn rwr1_set_profile_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<SetProfileParams>,
    ValidatedXmlBody(data): ValidatedXmlBody<SetProfileDataXml>,
) -> Result<Response, ProfileServerError> {
    // check that the client addr is an allowed ip
    check_ip_allowlist(&state, addr.ip())?;

    // check that the realm has been configured, see fn comments for more detail
    check_realm_is_configured(&state, &params.realm)?;

    // get the realm, making it if it doesn't exist yet
    tracing::info!("locating realm '{}'", &params.realm);
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;

    tracing::debug!("{data:#?}");
    let mut accounts_to_update: Vec<AccountActiveModel> = Vec::new();
    for player_xml in data.players.iter() {
        tracing::info!("processing set xml for player '{}'", player_xml.hash);
        // get the player from cache/db, remembering that get_player does all the account sid/rid verification
        // by itself if it encounters an existing player in the cache or db
        let opt_player = get_player(
            &state,
            player_xml.hash,
            &player_xml.profile.username,
            player_xml.profile.sid,
            &player_xml.rid,
        )
        .await?;
        match opt_player {
            None => {
                // a set request was made for a player not in db (for which no get was made first)
                // this will invalidate the entire set xml data by erroring thus:
                return Err(ProfileServerError::PlayerNotFound(
                    player_xml.hash,
                    player_xml.profile.username.to_owned(),
                    player_xml.profile.sid,
                ));
            }
            Some(player) => {
                tracing::info!(
                    "creating account model for player '{}' from xml...",
                    &player.username
                );
                // construct account active model from player xml
                let account = make_account_model(realm.id, player_xml)?;
                // add account to vec of accounts to update in bulk insert many
                accounts_to_update.push(account);
            }
        }
    }

    // update accounts models in cache
    tracing::debug!("inserting/updating accounts in cache...");
    for account in accounts_to_update.iter() {
        // this unwrap should be "safe" as the account was just made^
        let account_model: AccountModel = account.clone().try_into().unwrap();
        let hash = account_model.hash;
        let arc_model = Arc::new(account_model);
        state
            .cache
            .accounts
            .insert((realm.id, hash), arc_model)
            .await;
    }
    // insert many active model accounts with on_conflict to update
    tracing::info!("inserting account model(s) into db...");
    let res = Account::insert_many(accounts_to_update)
        .on_conflict(
            OnConflict::columns([AccountColumn::RealmId, AccountColumn::Hash])
                // update ALL columns
                .update_columns(ACCOUNT_COLUMNS)
                .to_owned(),
        )
        .exec(&state.db)
        .await?;

    tracing::info!(
        "inserted accounts into db, last insert = ({},{})",
        res.last_insert_id.0,
        res.last_insert_id.1
    );

    // respond to the game server
    Ok((StatusCode::OK, HEADERS, "<data ok=\"1\" />").into_response())
}

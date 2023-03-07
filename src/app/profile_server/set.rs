use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_macros::debug_handler;
use migration::OnConflict;
use sea_orm::{ActiveValue, EntityTrait};

use super::super::state::AppState;
use super::errors::ProfileServerError;
use super::validation::{ValidatedQuery, ValidatedXmlBody};
use super::xml::SetProfileDataXml;

use super::util::{check_realm_is_configured, get_realm, get_player, make_account_model};
use super::util::ACCOUNT_COLUMNS;
use super::params::SetProfileParams;

use entity::{Account, AccountModel, AccountActiveModel, AccountColumn};

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
                // construct account active model from player xml
                let account = make_account_model(realm.id, player_xml);
                // add account to vec of accounts to update in bulk insert many
                accounts_to_update.push(account);
            }
        }
    }

    // update accounts models in cache
    tracing::debug!("inserting/updating accounts in cache...");
    for account in accounts_to_update.iter() {
        let account_model: AccountModel = account.clone().try_into().unwrap();
        let hash = account_model.hash;
        let arc_model = Arc::new(account_model);
        state.cache.accounts.insert((realm.id, hash), arc_model).await;
    }
    // insert many active model accounts with on_conflict to update
    tracing::debug!("inserting account model(s) into db...");
    // todo: need to check that this is "atomic" (making insert many sql) and doesn't need a transaction
    let res = Account::insert_many(accounts_to_update)
                        .on_conflict(
                            OnConflict::columns([AccountColumn::RealmId, AccountColumn::Hash])
                                        .update_columns(ACCOUNT_COLUMNS)
                                        .to_owned())
                        .exec(&state.db).await?;

    tracing::debug!("inserted accounts into db, last insert = ({},{})", res.last_insert_id.0, res.last_insert_id.1);

    // todo: need to make proper <data ok="1"/> response here with headers etc
    Ok((StatusCode::OK).into_response())
}

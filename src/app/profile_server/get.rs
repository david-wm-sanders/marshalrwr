use axum::{extract::State, response::Html};
use axum_macros::debug_handler;

use super::errors::ProfileServerError;
use super::super::{state::AppState, validated_query::ValidatedQuery};
use super::util::{check_realm_is_configured, digest_ok, get_realm, get_player,
                  get_player_from_db, get_account_from_db};

use super::params::GetProfileParams;

#[debug_handler]
pub async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Result<Html<String>, ProfileServerError> {
    // check that the realm has been configured, see fn comments for more detail
    check_realm_is_configured(&state, &params.realm)?;
    
    // get the realm
    let realm = get_realm(&state, &params.realm, &params.realm_digest).await?;
    // verify the realm digest
    if !digest_ok(&params.realm_digest, &realm.digest) {
        tracing::error!("digest provided for realm '{}' incorrect", &params.realm);
        return Err(ProfileServerError::RealmDigestIncorrect(String::from(&params.realm), String::from(&params.realm_digest)));
    }

    // tracing::info!("checking db for player '{}' in '{}' realm", &params.username, &params.realm);
    // let opt_player = get_player_from_db(&state.db, params.hash).await?;
    // let opt_account = get_account_from_db(&state.db, realm.id, params.hash).await?;
    // match (opt_player, opt_account) {
    // // requires unstable library feature? maybe gotta use "futures" crate instead for now
    // // match future::join!(get_player_from_db(&state.db, params.hash), get_account_from_db(&state.db, realm.id, params.hash)).await? {
    //     (None, None) => {
    //         tracing::info!("player '{}' not found in db, enlisting them (pending checks)", &params.username);
    //         // todo: run complex validation on username here :D
    //         // todo: create player and then init profile for player in realm
    //     },
    //     (Some(player), None) => {
    //         // todo: player was created (by get) previously but no set, resend init
    //     },
    //     (Some(player), Some(account)) => {
    //         // todo: the player has some account for this realm, send it to them
    //     }
    //     (None, Some(_)) => unreachable!("no player but some account wtf")
    // }

    tracing::info!("acquiring documentation for player '{}'", &params.username);
    let opt_player = get_player(&state, &params).await?;
    tracing::debug!("{opt_player:#?}");
    match opt_player {
        None => {
            tracing::debug!("None player");
            // todo: make player init profile magic
        },
        Some(player) => {
            tracing::debug!("Some player");
            // todo: verify player rid and sid
            // we have a player, try to get_account
        }
    }

    let s = format!("{params:#?} {state:#?}");
    Ok(Html(s))
}
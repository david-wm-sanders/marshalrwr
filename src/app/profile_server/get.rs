use axum::{extract::State, response::Html};
use axum_macros::debug_handler;

use serde::Deserialize;
use validator::Validate;

use surrealdb::{Datastore, Session, Error, sql::Value};

use super::super::{state::AppState, validated_query::ValidatedQuery};
use super::validation::{validate_get_profile_params, validate_username, RE_HEX_STR};
use super::PROFILES_SESSION;


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
pub async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Html<String> {
    let s = format!("{params:#?} {state:#?}");
    // get own ref to the surrealdb datastore
    // let ds = state.db.datastore.clone();
    let db = state.db.clone();
    let statement: &str = "SELECT * FROM realms WHERE realm_name = $realm_name;";
    let mut vars: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    vars.insert("realm_name".into(), params.realm.into());
    let results = db.query(&PROFILES_SESSION, statement).await.unwrap();
    // let responses = match ds.execute(statement, &PROFILES_SESSION, Some(vars), false).await {
    //     Ok(vr) => vr,
    //     Err(err) => {
    //         tracing::error!("surrealdb error: {err}");
    //         vec![]
    //     }
    // };
    
    // todo: perform any additional validation that requires app state
    Html(s)
}

// let responses = self.datastore.execute(statement, session, None, false).await?;
// let mut results = Vec::new();
// for response in responses {
//     results.push(response.result?.first());
// }
// Ok(results)
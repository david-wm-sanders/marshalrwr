use axum::{extract::State, response::Html};
use axum_macros::debug_handler;

use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use serde::Deserialize;
use validator::Validate;

use crate::app::errors::ServerError;

use super::super::{state::AppState, validated_query::ValidatedQuery};
use super::validation::{validate_get_profile_params, validate_username, RE_HEX_STR};
use entity::Realm;

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
pub async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Result<Html<String>, ServerError> {
    let s = format!("{params:#?} {state:#?}");

    let realm = Realm::find().filter(entity::realm::Column::Name.eq(&params.realm)).one(&state.db).await?;
    match realm {
        Some(r) => {
            tracing::debug!("Found existing realm");
            // todo:: check the model digest against params.digest
        }
        None => {
            tracing::debug!("No realm '{}'", params.realm)
            // todo:: make realm?
        }
    }
    
    Ok(Html(s))
}
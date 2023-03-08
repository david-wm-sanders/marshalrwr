use serde::Deserialize;
use validator::Validate;

use super::validation::{validate_get_profile_params, validate_username, RE_HEX_STR};

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_get_profile_params"))]
pub struct GetProfileParams {
    #[validate(range(min = 1, max = "u32::MAX"))]
    pub hash: i64,
    #[validate(length(min = 1, max = 32))]
    #[validate(non_control_character)]
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[validate(length(equal = 64))]
    #[validate(regex(path = "RE_HEX_STR", code = "rid not hexadecimal"))]
    pub rid: String,
    #[validate(range(min = 1, max = "u32::MAX"))]
    pub sid: i64,
    #[validate(length(min = 1, max = 32))]
    pub realm: String,
    #[validate(length(equal = 64))]
    #[validate(regex(path = "RE_HEX_STR", code = "realm digest not hexadecimal"))]
    pub realm_digest: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SetProfileParams {
    #[validate(length(min = 1, max = 32))]
    pub realm: String,
    #[validate(length(equal = 64))]
    #[validate(regex(path = "RE_HEX_STR", code = "realm digest not hexadecimal"))]
    pub realm_digest: String,
}

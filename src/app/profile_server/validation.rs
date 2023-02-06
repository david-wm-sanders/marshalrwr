use lazy_static::lazy_static;
use validator::ValidationError;
use regex::Regex;

use crate::GetProfileParams;
use super::util::rwr1_hash_username;

lazy_static! {
    pub static ref RE_HEX_STR: Regex = Regex::new(r"^([0-9A-Fa-f]{2})+$").unwrap();
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains("  ") {
        return Err(ValidationError::new("username contains multiple consecutive spaces"));
    }
    if username.starts_with(' ') {
        return Err(ValidationError::new("username starts with a space"));
    }
    if username.ends_with(' ') {
        return Err(ValidationError::new("username ends with a space"));
    }
    // todo: check against blocklist?
    // todo: check for weird characters that aren't control but correspond to weird things in default rwr latin font
    Ok(())
}

pub fn validate_get_profile_params(params: &GetProfileParams) -> Result<(), ValidationError> {
    // calculate int hash from string username and confirm they match
    if params.hash != rwr1_hash_username(params.username.as_str()) {
        return Err(ValidationError::new("hash not for given username"));
    }
    // todo: validate steam account associated with sid has license?!
    // this would be better as a one-time check during account creation (as it would require costly call to Steam APIs)
    Ok(())
}
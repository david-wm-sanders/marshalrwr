use std::hash::Hasher;

use lazy_static::lazy_static;
use regex::Regex;
use validator::ValidationError;

use crate::GetProfileParams;

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

// # by Xe-No
// def get_file_hash(name):
//     lst = [ord(x) for x in name]
//     d1 = 0
//     lst.reverse()
//     for i, element in enumerate(lst):
//         d1 += element * (33**i)
//     d2 = 33**len(lst) * 5381
//     return (d1+d2) % (2**32)

// derived from a python implementation^ by Xe-No
// this seems to overcomplicate things a bit (and won't work w/o big integers),
// a search for `"5381" "33" hash` indicated that this could be djb2 (http://www.cse.yorku.ca/~oz/hash.html)
// a Djb2a hasher was thus devised (with inspiration from some crates that provide various versions of djb2 hash implementations)
// in this implementation, even though the server is x64, we ensure to use u32 (as the rwr client is 32-bit)
pub fn rwr1_hash_username(username: &str) -> u64 {
    let mut djb2a_hasher = Djb2aHash32::default();
    djb2a_hasher.write(username.as_bytes());
    djb2a_hasher.finish()
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

struct Djb2aHash32 {
    v: u32
}

impl Default for Djb2aHash32 {
    fn default() -> Self {
        Self { v: 5381 }
    }
}

impl Hasher for Djb2aHash32 {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.v = (self.v << 5)
                     .wrapping_add(self.v)
                     .wrapping_add(b as u32);
        }
    }

    fn finish(&self) -> u64 {
        self.v as u64
    }
}
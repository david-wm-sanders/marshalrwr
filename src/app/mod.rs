pub mod errors;
pub mod signalling;
pub mod state;
pub mod validated_query;
pub mod profile_server;

use lazy_static::lazy_static;

// lazy_static! {
//     pub static ref DB_DEFAULT_URL: &'static str = "sqlite:://classified.db";
// }

pub static DB_DEFAULT_URL: &str = "sqlite://classified.db";
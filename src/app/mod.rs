pub mod errors;
pub mod signalling;
pub mod state;
pub mod hasher;
pub mod validated_query;
pub mod profile_server;

pub static DB_DEFAULT_URL: &str = "sqlite://classified.db";
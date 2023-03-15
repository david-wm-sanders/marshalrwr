pub mod config;
pub mod errors;
pub mod hasher;
pub mod profile_server;
pub mod signalling;
pub mod state;
pub mod validated_query;

pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub static DB_DEFAULT_URL: &str = "sqlite://classified.db";

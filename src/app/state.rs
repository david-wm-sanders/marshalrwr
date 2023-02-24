use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use moka::future::Cache;

use entity::{RealmModel, PlayerModel, AccountModel};
use crate::AppConfiguration;

pub struct CacheManager {
    pub realm_cache: Cache<String, RealmModel>,
    pub player_cache: Cache<i64, PlayerModel>,
    pub account_cache: Cache<(i32, i64), AccountModel>,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self {
            realm_cache: Cache::builder().name("realms")
                                         .max_capacity(32)
                                         .build(),
            player_cache: Cache::builder().name("players")
                                         .max_capacity(256)
                                         .build(),
            account_cache: Cache::builder().name("accounts")
                                         .max_capacity(256)
                                         .build(),
        }    
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfiguration,
    pub db: DatabaseConnection,
    pub realm_cache: Arc<RwLock<HashMap<String, RealmModel>>>,
    pub player_cache: Arc<RwLock<HashMap<i64, PlayerModel>>>,
}

impl AppState {
    pub fn new(app_config: AppConfiguration, db_conn: DatabaseConnection) -> Self {
        Self { config: app_config, db: db_conn,
               realm_cache: Arc::new(RwLock::new(HashMap::new())),
               player_cache: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ config: {:#?} }}", self.config)
    }
}
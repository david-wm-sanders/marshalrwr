use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::Duration;

use sea_orm::DatabaseConnection;
use moka::future::Cache;

use entity::{RealmModel, PlayerModel, AccountModel};
use crate::AppConfiguration;

#[derive(Clone)]
pub struct CacheManager {
    pub realms: Cache<String, Arc<RealmModel>>,
    pub players: Cache<i64, Arc<PlayerModel>>,
    pub accounts: Cache<(i32, i64), Arc<AccountModel>>,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self {
            realms: Cache::builder().name("realms")
                                         .max_capacity(32)
                                         .time_to_live(Duration::from_secs(60*60))
                                         .time_to_idle(Duration::from_secs(15*60))
                                         .build(),
            players: Cache::builder().name("players")
                                         .max_capacity(256)
                                         .time_to_live(Duration::from_secs(30*60))
                                         .time_to_idle(Duration::from_secs(15*60))
                                         .build(),
            accounts: Cache::builder().name("accounts")
                                         .max_capacity(256)
                                         .time_to_live(Duration::from_secs(30*60))
                                         .time_to_idle(Duration::from_secs(15*60))
                                         .build(),
        }    
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfiguration,
    pub db: DatabaseConnection,
    pub cache: CacheManager,
    // pub realm_cache: Arc<RwLock<HashMap<String, RealmModel>>>,
    // pub player_cache: Arc<RwLock<HashMap<i64, PlayerModel>>>,
}

impl AppState {
    pub fn new(app_config: AppConfiguration, db_conn: DatabaseConnection) -> Self {
        Self { config: app_config, db: db_conn,
               cache: CacheManager::default(),
               /* realm_cache: Arc::new(RwLock::new(HashMap::new())),
               player_cache: Arc::new(RwLock::new(HashMap::new())) */ }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ config: {:#?} }}", self.config)
    }
}
use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::AppConfiguration;
use entity::{AccountModel, PlayerModel, RealmModel};

#[derive(Clone)]
pub struct CacheManager {
    pub realms: Cache<String, Arc<RwLock<RealmModel>>>,
    pub players: Cache<i64, Arc<PlayerModel>>,
    pub accounts: Cache<(i32, i64), Arc<AccountModel>>,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self {
            realms:
                Cache::builder()
                    .name("realms")
                    .max_capacity(32)
                    .time_to_live(Duration::from_secs(60*60))
                    .time_to_idle(Duration::from_secs(15*60))
                    .eviction_listener_with_queued_delivery_mode(|key, value: Arc<RwLock<RealmModel>>, removal_cause| {
                        let value = value.blocking_read();
                        match removal_cause {
                            moka::notification::RemovalCause::Expired =>
                                tracing::debug!("realm '{}' [{}] expired and was evicted from the cache", &key, value.id),
                            moka::notification::RemovalCause::Explicit =>
                                tracing::debug!("realm '{}' [{}] was explictly removed from the cache", &key, value.id),
                            moka::notification::RemovalCause::Replaced =>
                                tracing::debug!("realm '{}' [{}] was replaced in the cache", &key, value.id),
                            moka::notification::RemovalCause::Size =>
                                tracing::debug!("realm '{}' [{}] was evicted from the cache due to size constraints", &key, value.id)
                        }
                        drop(value);
                    })
                    .build(),
            players:
                Cache::builder()
                    .name("players")
                    .max_capacity(256)
                    .time_to_live(Duration::from_secs(30*60))
                    .time_to_idle(Duration::from_secs(15*60))
                    .eviction_listener_with_queued_delivery_mode(|_key, value: Arc<PlayerModel>, removal_cause| {
                        match removal_cause {
                            moka::notification::RemovalCause::Expired =>
                                tracing::debug!("player '{}' [hash:{}, sid:{}] expired and was evicted from the cache", value.username, value.hash, value.sid),
                            moka::notification::RemovalCause::Explicit =>
                                tracing::debug!("player '{}' [hash:{}, sid:{}] was explictly removed from the cache", value.username, value.hash, value.sid),
                            moka::notification::RemovalCause::Replaced =>
                                tracing::debug!("player '{}' [hash:{}, sid:{}] was replaced in the cache", value.username, value.hash, value.sid),
                            moka::notification::RemovalCause::Size =>
                                tracing::debug!("player '{}' [hash:{}, sid:{}] was evicted from the cache due to size constraints", value.username, value.hash, value.sid)
                            }
                    })
                    .build(),
            accounts:
                Cache::builder()
                    .name("accounts")
                    .max_capacity(256)
                    .time_to_live(Duration::from_secs(30*60))
                    .time_to_idle(Duration::from_secs(15*60))
                    .eviction_listener_with_queued_delivery_mode(|key: Arc<(i32, i64)>, _value: Arc<AccountModel>, removal_cause| {
                        match removal_cause {
                            moka::notification::RemovalCause::Expired =>
                                tracing::debug!("account '({}, {})' expired and was evicted from the cache", key.0, key.1),
                            moka::notification::RemovalCause::Explicit =>
                                tracing::debug!("account '({}, {})' was explictly removed from the cache", key.0, key.1),
                            moka::notification::RemovalCause::Replaced =>
                                tracing::debug!("account '({}, {})' was replaced in the cache", key.0, key.1),
                            moka::notification::RemovalCause::Size =>
                                tracing::debug!("account '({}, {})' was evicted from the cache due to size constraints", key.0, key.1)
                        }
                    })
                    .build(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfiguration,
    pub db: DatabaseConnection,
    pub cache: CacheManager,
}

impl AppState {
    pub fn new(app_config: AppConfiguration, db_conn: DatabaseConnection) -> Self {
        Self {
            config: app_config,
            db: db_conn,
            cache: CacheManager::default(),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ config: {:#?} }}", self.config)
    }
}

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use sea_orm::DatabaseConnection;

use entity::RealmModel;
use crate::AppConfiguration;


#[derive(Clone)]
pub struct AppState {
    pub config: AppConfiguration,
    pub db: DatabaseConnection,
    pub realm_cache: Arc<RwLock<HashMap<String, RealmModel>>>,
}

impl AppState {
    pub fn new(app_config: AppConfiguration, db_conn: DatabaseConnection) -> Self {
        Self { config: app_config, db: db_conn,
               realm_cache: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ config: {:#?} }}", self.config)
    }
}
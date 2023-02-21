use sea_orm::DatabaseConnection;

use crate::AppConfiguration;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfiguration,
    pub db: DatabaseConnection
}

impl AppState {
    pub fn new(app_config: AppConfiguration, db_conn: DatabaseConnection) -> Self {
        Self { config: app_config, db: db_conn }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ config: {:#?} }}", self.config)
    }
}
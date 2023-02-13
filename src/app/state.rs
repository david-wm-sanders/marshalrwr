use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection
}

impl AppState {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db: db_conn }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ ? }}")
    }
}
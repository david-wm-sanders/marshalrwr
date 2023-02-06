use std::sync::Arc;

use axum::extract::FromRef;
use surrealdb::{Datastore, Session, Error, sql::Value};

#[derive(Clone)]
pub struct AppState {
    pub db: DbState
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{ ? }}")
    }
}

#[derive(Clone)]
pub struct DbState {
    pub datastore: Arc<Datastore>,
}

impl AppState {
    pub async fn new(datastore: &str) -> Result<Self, Error> {
        let ds = Arc::new(Datastore::new(datastore).await?);
        Ok(Self { db: DbState { datastore: ds } } )
    }
}

impl FromRef<AppState> for DbState {
    fn from_ref(app_state: &AppState) -> DbState {
        app_state.db.clone()
    }
}
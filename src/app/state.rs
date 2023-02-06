use std::sync::Arc;

use axum::extract::FromRef;
use surrealdb::{Datastore, Session, Error, sql::Value};

// use super::profile_server::PROFILES_SESSION;

#[derive(Clone)]
pub struct AppState {
    pub db: DbState
}

impl AppState {
    pub async fn new(datastore: &str) -> Result<Self, Error> {
        let ds = Arc::new(Datastore::new(datastore).await?);
        Ok(Self { db: DbState { datastore: ds } } )
    }
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

impl DbState {
    pub async fn query(&self, session: &Session, statement: &str) -> Result<Vec<Value>, Error> {
        let responses = self.datastore.execute(statement, session, None, false).await?;
        let mut results = Vec::new();
        for response in responses {
            results.push(response.result?.first());
        }
        Ok(results)
    }
}

impl FromRef<AppState> for DbState {
    fn from_ref(app_state: &AppState) -> DbState {
        app_state.db.clone()
    }
}
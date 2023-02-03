use std::sync::Arc;

use axum::extract::FromRef;
use surrealdb::{Datastore, Session, Error, sql::Value};

#[derive(Clone)]
pub struct AppState {
    pub db: DbState
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ns: &str = self.db.session.ns.as_ref().unwrap();
        let db_name: &str = self.db.session.db.as_ref().unwrap();
        write!(f, "AppState {{ db: {ns}/{db_name} }}")
    }
}

#[derive(Clone)]
pub struct DbState {
    pub datastore: Arc<Datastore>,
    pub session: Session
}

impl AppState {
    pub async fn new(datastore: &str, namespace: &str, database: &str) -> Result<Self, Error> {
        let ds = Arc::new(Datastore::new(datastore).await?);
        let sesh = Session::for_db(namespace, database);
        Ok(Self { db: DbState { datastore: ds, session: sesh} } )
    }
}

impl FromRef<AppState> for DbState {
    fn from_ref(app_state: &AppState) -> DbState {
        app_state.db.clone()
    }
}
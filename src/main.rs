use std::net::SocketAddr;
use std::sync::Arc;

use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{Router, routing::{get, post},
           http::{StatusCode, request::Parts},
           extract::{Query, FromRequestParts,
                     rejection::QueryRejection,
                     State, FromRef},
           response::{Html, Response, IntoResponse}};
use tower_http::trace::TraceLayer;
use serde::{Deserialize, de::DeserializeOwned};
use validator::{Validate, ValidateArgs, ValidationError};
use async_trait::async_trait;
use thiserror::Error;
use surrealdb::{Datastore, Session, Error, sql::Value};

mod app;
use app::signalling::shutdown_signal;

#[tokio::main]
async fn main() {
    // setup tracing subscriber first and foremost
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "marshalrwr=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("setting up application state");
    let app_state = AppState::new("file://classified.db", "marshalrwr", "profiles").await.unwrap();

    // build our application with a route and add the tower-http tracing layer
    let application_router = Router::new()
        .route("/get_profile.php", get(rwr1_get_profile_handler))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(application_router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    
    // salute the fallen
    tracing::debug!("o7");
}

#[derive(Clone)]
struct AppState {
    pub db: DbState
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let db_name: &str = self.db.session.db.as_ref().unwrap();
        write!(f, "AppState {{ ds: {:#?} }}", db_name)
    }
}

#[derive(Clone)]
struct DbState {
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

#[derive(Debug, Deserialize, Validate)]
struct GetProfileParams {
    #[validate(range(min=1, max="u32::MAX"))]
    hash: u64,
    #[validate(length(min=1, max=32))]
    username: String,
    #[validate(length(equal=64))]
    rid: String,
    #[validate(range(min=1, max="u32::MAX"))]
    sid: u64,
    #[validate(length(min=1, max=32))]
    realm: String,
    #[validate(length(equal=64))]
    realm_digest: String
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<T>::from_request_parts(parts, state).await?;
        params.validate()?;
        // params.validate_args(state)?;
        Ok(ValidatedQuery(params))
    }
}

// fn validate_username(value: &str, arg: &mut AppState) -> Result<(), ValidationError> {
//     Err(ValidationError::new("bad name"))
// }

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection)
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{}]", self).replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ServerError::AxumQueryRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Html<String> {
    // params.validate_args(&state);
    Html(format!("{:#?}\n{:#?}", params, state))
}


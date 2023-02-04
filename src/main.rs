use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{Router, routing::{get, post},
           extract::State,
           response::Html};
use tower_http::trace::TraceLayer;
use serde::Deserialize;
use validator::{Validate, ValidationError};

use surrealdb::{Datastore, Session, Error, sql::Value};

mod app;
use app::signalling::shutdown_signal;
use app::state::AppState;
use app::validated_query::ValidatedQuery;
use app::util::{validate_username, validate_get_profile_params, RE_HEX_STR};

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

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function="validate_get_profile_params"))]
pub struct GetProfileParams {
    #[validate(range(min=1, max="u32::MAX"))]
    hash: u64,
    #[validate(length(min=1, max=32))]
    #[validate(non_control_character)]
    #[validate(custom(function="validate_username"))]
    username: String,
    #[validate(length(equal=64))]
    #[validate(regex(path="RE_HEX_STR", code="rid not hexadecimal"))]
    rid: String,
    #[validate(range(min=1, max="u32::MAX"))]
    sid: u64,
    #[validate(length(min=1, max=32))]
    realm: String,
    #[validate(length(equal=64))]
    #[validate(regex(path="RE_HEX_STR", code="realm digest not hexadecimal"))]
    realm_digest: String
}

async fn rwr1_get_profile_handler(State(state): State<AppState>, ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Html<String> {
    let s = format!("{params:#?} {state:#?}");
    // todo: perform any additional validation that requires app state
    Html(s)
}


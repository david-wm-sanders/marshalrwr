use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{Router, routing::{get, post},
           extract::State,
           response::Html};
use tower_http::trace::TraceLayer;
use serde::Deserialize;
use validator::{Validate, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;
use surrealdb::{Datastore, Session, Error, sql::Value};

mod app;
use app::signalling::shutdown_signal;
use app::state::AppState;
use app::validated_query::ValidatedQuery;

lazy_static! {
    static ref RE_HEX_STR: Regex = Regex::new(r"^([0-9A-Fa-f]{2})+$").unwrap();
}

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

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains("  ") {
        return Err(ValidationError::new("username contains multiple consecutive spaces"));
    }
    if username.starts_with(' ') {
        return Err(ValidationError::new("username starts with a space"));
    }
    if username.ends_with(' ') {
        return Err(ValidationError::new("username ends with a space"));
    }
    // todo: check against blocklist?
    // todo: check for weird characters that aren't control but correspond to weird things in default rwr latin font
    Ok(())
}

fn validate_get_profile_params(params: &GetProfileParams) -> Result<(), ValidationError> {
    // todo: calculate int hash from string username and confirm they match
    let hash = params.hash;
    let username = params.username.as_str();
    let calculated_hash = rwr1_hash_username(username);
    Ok(())
}

// # by Xe-No
// def get_file_hash(name):
//     lst = [ord(x) for x in name]
//     d1 = 0
//     lst.reverse()
//     for i, element in enumerate(lst):
//         d1 += element * (33**i)
//     d2 = 33**len(lst) * 5381
//     return (d1+d2) % (2**32)

// derived from a python implementation^ by Xe-No
// this seems to overcomplicate things a bit, a search for "5381 hash" seems to indicate that this could be djb2 (http://www.cse.yorku.ca/~oz/hash.html)
// maybe also now improved djb2.1(?) with XOR?!
// guess i will have to try both xd
fn rwr1_hash_username(username: &str) -> u32 {
    // iterators op?
    let h = 5381u64;
    let rev_ords = username.chars().map(|c| c as u32).rev().collect::<Vec<u32>>();
    0
}

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function="validate_get_profile_params"))]
struct GetProfileParams {
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


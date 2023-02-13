use std::net::SocketAddr;

use sea_orm::Database;
use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{Router, routing::{get, post}};
use tower_http::trace::TraceLayer;
use anyhow;

// use surrealdb::{Datastore, Session, Error, sql::Value};

mod app;
use app::signalling::shutdown_signal;
use app::state::AppState;
use app::profile_server::get::rwr1_get_profile_handler;
use app::DB_DEFAULT_URL;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup tracing subscriber first and foremost
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "marshalrwr=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("setting up application state");
    let db_connection = Database::connect(format!("{DB_DEFAULT_URL}?mode=rwc")).await?;
    let app_state = AppState::new("file://classified.db").await.unwrap();

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
    Ok(())
}




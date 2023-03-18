use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::Database;
use tower_http::trace::TraceLayer;

mod app;
use app::config::AppConfiguration;
use app::profile_server::{get::rwr1_get_profile_handler, set::rwr1_set_profile_handler};
use app::signalling::shutdown_signal;
use app::state::AppState;
use app::tracing::init_tracing_subscriber;
use app::{DB_DEFAULT_URL, VERSION};

use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();

    tracing::info!("starting marshalrwr [v{}]", VERSION.unwrap_or("n/a"));
    tracing::info!("loading configuration...");
    let app_config = AppConfiguration::build()?;
    tracing::debug!("{app_config:?}");

    tracing::debug!("setting up application state...");
    let db_connection = Database::connect(format!("{DB_DEFAULT_URL}?mode=rwc")).await?;

    tracing::info!("performing migrations (if any)... :D");
    Migrator::up(&db_connection, None).await?;

    let app_state = AppState::new(app_config, db_connection);

    // build our application with a route and add the tower-http tracing layer
    let application_router = Router::new()
        .route("/get_profile.php", get(rwr1_get_profile_handler))
        .route("/set_profile.php", post(rwr1_set_profile_handler))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}...", addr);
    axum::Server::bind(&addr)
        .serve(application_router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    // salute the fallen
    tracing::info!("o7");
    Ok(())
}
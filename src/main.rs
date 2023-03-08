use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    routing::{get, post},
    Router,
};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
use app::profile_server::{get::rwr1_get_profile_handler, set::rwr1_set_profile_handler};
use app::signalling::shutdown_signal;
use app::state::AppState;
use app::{DB_DEFAULT_URL, VERSION};

use migration::{Migrator, MigratorTrait};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfiguration {
    realms: Vec<String>,
    ps_allowed_ips: Vec<IpAddr>,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        AppConfiguration {
            realms: vec![],
            ps_allowed_ips: vec![IpAddr::from_str("127.0.0.1").unwrap()],
        }
    }
}

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

    tracing::info!("starting marshalrwr [v{}]", VERSION.unwrap_or("n/a"));
    tracing::info!("loading configuration...");
    let app_config: AppConfiguration =
        Figment::from(Serialized::defaults(AppConfiguration::default()))
            .merge(Toml::file("marshalrwr.toml"))
            .merge(Env::prefixed("MRWR_"))
            .extract()?;
    tracing::debug!("{app_config:?}");

    tracing::debug!("setting up application state...");
    let db_connection = Database::connect(format!("{DB_DEFAULT_URL}?mode=rwc")).await?;

    tracing::info!("performing migrations... :D");
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

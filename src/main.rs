use std::net::SocketAddr;

use sea_orm::Database;
use serde::{Serialize, Deserialize};
use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{Router, routing::{get, post}};
use tower_http::trace::TraceLayer;
use figment::{Figment, providers::{Format, Toml, Env, Serialized}};

mod app;
use app::signalling::shutdown_signal;
use app::state::AppState;
use app::profile_server::get::rwr1_get_profile_handler;
use app::DB_DEFAULT_URL;

use migration::{Migrator, MigratorTrait};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfiguration {
    realms: Vec<String>
}

impl Default for AppConfiguration {
    fn default() -> Self {
        AppConfiguration {
            realms: vec![]   
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
        
    tracing::info!("loading configuration");
    let app_config: AppConfiguration = Figment::from(Serialized::defaults(AppConfiguration::default()))
                                                .merge(Toml::file("marshalrwr.toml"))
                                                .merge(Env::prefixed("MRWR_"))
                                                .extract()?;
    tracing::debug!("{app_config:?}");

    tracing::debug!("setting up application state");
    let db_connection = Database::connect(format!("{DB_DEFAULT_URL}?mode=rwc")).await?;
    
    tracing::info!("performing migrations :D");
    Migrator::up(&db_connection, None).await?;
    
    let app_state = AppState::new(app_config, db_connection);

    // build our application with a route and add the tower-http tracing layer
    let application_router = Router::new()
        .route("/get_profile.php", get(rwr1_get_profile_handler))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(application_router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    
    // salute the fallen
    tracing::info!("o7");
    Ok(())
}




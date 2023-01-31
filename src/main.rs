use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{response::Html, routing::get, Router};
use tower_http::trace::TraceLayer;

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

    // build our application with a route and add the tower-http tracing layer
    let application_router = Router::new()
        .route("/", get(handler))
        .route("/get_profile.php", get(rwr1_get_profile_php_handler))
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

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn rwr1_get_profile_php_handler() -> Html<&'static str> {
    Html("get_profile.php test")
}
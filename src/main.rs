use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{response::Html, routing::get, Router};
use tower_http::trace::TraceLayer;
use tokio::signal;


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
    let app = Router::new()
        .route("/", get(handler)).layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(gs_sig())
        .await
        .unwrap();
    
    // salute the fallen
    tracing::debug!("o7");
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn gs_sig() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed ctrl-c handler installation")  
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed signal::unix::SignalKind::terminate handler installation")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    tracing::debug!("shutdown signal received, commencing a graceful death :D");
}
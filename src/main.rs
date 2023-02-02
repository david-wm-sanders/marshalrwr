use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use axum::{Router, routing::{get, post},
           http::{StatusCode, request::Parts},
           extract::{Query, FromRequestParts,
                     rejection::QueryRejection},
           response::{Html, Response, IntoResponse}};
use tower_http::trace::TraceLayer;
use serde::{Deserialize, de::DeserializeOwned};
use validator::Validate;
use async_trait::async_trait;
use thiserror::Error;

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
        .route("/get_profile.php", get(rwr1_get_profile_handler))
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

#[derive(Debug, Deserialize, Validate)]
struct GetProfileParams {
    hash: u32,
    username: String,
    #[validate(length(equal = 64))]
    rid: String,
    sid: u32,
    realm: String,
    #[validate(length(equal = 64))]
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
        Ok(ValidatedQuery(params))
    }
}

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

async fn rwr1_get_profile_handler(ValidatedQuery(params): ValidatedQuery<GetProfileParams>) -> Html<String> {
    Html(format!("{:#?}", params))
}


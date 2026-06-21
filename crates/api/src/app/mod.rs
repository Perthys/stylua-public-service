mod health;
mod routes;

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::{
    catch_panic::CatchPanicLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer,
};


pub fn build() -> Router {
    let built_routes = routes::build();
    let health_route = health::health;

    Router::new()
        .route("/health", get(health_route))
        .merge(built_routes)
        .layer(RequestBodyLimitLayer::new(256 * 1024))
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(5)))
        .layer(CatchPanicLayer::new())
}

mod health;
mod routes;

use axum::{Router, routing::get};

pub fn build() -> Router {
    let built_routes = routes::build();
    let health_route = health::health;

    let router = Router::new()
        .route("/health", get(health_route))
        .merge(built_routes);

    router
}

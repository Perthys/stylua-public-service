mod healthz;
mod routes;

use axum::{Router, routing::get};

pub fn build() -> Router {
    let built_routes = routes::build();
    let healthz_route = healthz::healthz;

    let router = Router::new()
        .route("/healthz", get(healthz_route))
        .merge(built_routes);

    router
}

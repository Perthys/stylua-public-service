mod v1;

use axum::routing::Router;

pub fn build() -> Router {
    let built_v1 = v1::build();

    return Router::new().nest("/v1", built_v1);
}

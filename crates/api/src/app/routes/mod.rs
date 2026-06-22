mod v1;

use axum::Router;

use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let built_v1 = v1::build();

    Router::new().nest("/v1", built_v1)
}

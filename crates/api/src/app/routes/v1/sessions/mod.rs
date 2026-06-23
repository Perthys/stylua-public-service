mod create_session;
mod delete_session;
mod get_result;
mod get_session;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub fn build() -> Router<AppState> {
    Router::new()
        .route("/sessions", post(create_session::handler))
        .route(
            "/sessions/{id}",
            get(get_session::handler).delete(delete_session::handler),
        )
        .route("/sessions/{id}/result", get(get_result::handler))
}

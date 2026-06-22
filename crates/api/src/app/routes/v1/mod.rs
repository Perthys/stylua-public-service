mod format;

use axum::{Router, routing::post};

use crate::state::AppState;

pub fn build() -> Router<AppState> {
    Router::new().route("/format", post(format::format))
}

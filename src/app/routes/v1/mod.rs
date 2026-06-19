mod format;

use axum::{Router, routing::post};

pub fn build() -> axum::Router {
    let built_format = format::format;

    return Router::new().route("/format", post(built_format));
}

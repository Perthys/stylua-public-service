use axum::{Json, extract::State, response::Response};
use serde::Deserialize;
use serde_json::Value;

use crate::jobs;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateRequest {
    code: String,
    #[serde(default)]
    config: Value,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRequest>,
) -> Response {
    let mut connection = state.store.clone();
    jobs::enqueue(&mut connection, payload.code, payload.config).await
}

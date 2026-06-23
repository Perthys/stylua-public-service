use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use shared::store;

use crate::state::AppState;

pub async fn handler(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    let mut connection = state.store.clone();

    match store::get_session(&mut connection, &session_id).await {
        Ok(Some(data)) => match serde_json::from_str::<Value>(&data) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "corrupt session").into_response(),
        },
        Ok(None) => (StatusCode::NOT_FOUND, "session not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response(),
    }
}

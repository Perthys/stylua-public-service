use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use shared::store;

use crate::state::AppState;

pub async fn handler(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    let mut connection = state.store.clone();

    let data = match store::get_session(&mut connection, &session_id).await {
        Ok(Some(data)) => data,
        Ok(None) => return (StatusCode::NOT_FOUND, "session not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response(),
    };

    let session: Value = match serde_json::from_str(&data) {
        Ok(session) => session,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "corrupt session").into_response(),
    };

    match session.get("status").and_then(Value::as_str) {
        Some("completed") => session
            .get("formatted")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
            .into_response(),
        Some("failed") => {
            let error = session
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("formatting failed");
            (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()).into_response()
        }
        _ => (StatusCode::CONFLICT, "result not ready").into_response(),
    }
}

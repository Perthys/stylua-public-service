use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shared::store;

use crate::state::AppState;

pub async fn handler(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    let mut connection = state.store.clone();

    match store::delete_session(&mut connection, &session_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response(),
    }
}

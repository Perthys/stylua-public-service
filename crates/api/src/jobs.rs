use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use shared::Job;
use shared::store::{self, Store};
use uuid::Uuid;

use std::env::var;

pub async fn enqueue(connection: &mut Store, code: String, config: Value) -> Response {
    let max_queue = var("MAX_QUEUE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1000);

    let queued = store::queue_len(connection).await.unwrap_or(0);
    if queued >= max_queue {
        return (StatusCode::SERVICE_UNAVAILABLE, "server at capacity").into_response();
    }

    let session_id = Uuid::new_v4().to_string();
    let job = Job {
        session_id: session_id.clone(),
        code,
        config,
    };

    let payload = match serde_json::to_string(&job) {
        Ok(payload) => payload,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "serialize error").into_response(),
    };

    if store::enqueue(connection, &payload).await.is_err() {
        return (StatusCode::SERVICE_UNAVAILABLE, "queue unavailable").into_response();
    }

    let session = json!({ "status": "queued" }).to_string();
    store::set_session(connection, &session_id, &session)
        .await
        .ok();

    (
        StatusCode::ACCEPTED,
        Json(json!({ "status": "queued", "session_id": session_id })),
    )
        .into_response()
}

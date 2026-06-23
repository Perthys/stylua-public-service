use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use shared::store;
use stylua_lib::{Config, OutputVerification, format_code};

use crate::jobs;
use crate::state::AppState;

const MAX_CODE_SIZE: usize = 256 * 1024;
const LEASE_TTL: usize = 30;

#[derive(Deserialize)]
pub struct FormatRequest {
    code: String,
    #[serde(default)]
    config: Value,
}

pub async fn format(State(state): State<AppState>, Json(payload): Json<FormatRequest>) -> Response {
    let FormatRequest { code, config } = payload;

    if code.len() > MAX_CODE_SIZE {
        return (StatusCode::PAYLOAD_TOO_LARGE, "code exceeds maximum size").into_response();
    }

    let limit = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let mut connection = state.store.clone();

    match store::acquire(&mut connection, limit, LEASE_TTL).await {
        Ok(Some(permit_id)) => {
            let response = format_inline(code, config).await;
            store::release(&mut connection, &permit_id).await.ok();
            response
        }
        Ok(None) => jobs::enqueue(&mut connection, code, config).await,
        Err(_) => format_inline(code, config).await,
    }
}

async fn format_inline(code: String, config: Value) -> Response {
    let result = tokio::task::spawn_blocking(move || {
        let config: Config = serde_json::from_value(config).unwrap_or_default();
        format_code(&code, config, None, OutputVerification::None)
            .map(|formatted| {
                let changed = formatted != code;
                (formatted, changed)
            })
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok((formatted, changed))) => (
            StatusCode::OK,
            Json(json!({ "status": "completed", "formatted": formatted, "changed": changed })),
        )
            .into_response(),
        Ok(Err(message)) => (StatusCode::BAD_REQUEST, message).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response(),
    }
}

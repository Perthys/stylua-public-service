use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use stylua_lib::{Config, OutputVerification, format_code};

use std::env::var;

#[derive(Deserialize)]
pub struct FormatRequest {
    code: String,
    #[serde(default)]
    config: Config,
}

#[derive(Serialize)]
pub struct FormatResponse {
    formatted: String,
    changed: bool,
}

pub async fn format(
    Json(payload): Json<FormatRequest>,
) -> Result<Json<FormatResponse>, (StatusCode, String)> {
    let max_size = var("MAX_CODE_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10 * 1024); 

    let FormatRequest { code, config } = payload;

    if code.len() > max_size {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Code size exceeds the maximum allowed size of {} bytes", max_size),
        ));
    }

    let result = tokio::task::spawn_blocking(move || {
        match format_code(&code, config, None, OutputVerification::None) {
            Ok(formatted) => {
                let changed = formatted != code;
                Ok(Json(FormatResponse { formatted, changed }))
            }
            Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
        }
    })
    .await;

    match result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(e),
        Err(e) => {
            println!("Error in formatting task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ))
        }
    }
}

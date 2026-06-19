use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use stylua_lib::{Config, OutputVerification, format_code};

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
    let FormatRequest { code, config } = payload;

    match format_code(&code, config, None, OutputVerification::None) {
        Ok(formatted) => {
            let changed = formatted != code;
            Ok(Json(FormatResponse { formatted, changed }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

use std::env::var;
use std::time::Duration;

use dotenvy::dotenv;
use serde_json::{Value, json};
use shared::Job;
use shared::store::{self, Store};
use stylua_lib::{Config, OutputVerification, format_code};
use tokio::time::timeout;

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const FORMAT_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() {
    dotenv().ok();

    let redis_url = var("REDIS_URL").expect("REDIS_URL must be set");
    let mut connection = store::connect(&redis_url)
        .await
        .expect("redis didn't connect");

    println!("worker: I AM ALIVE");

    loop {
        match store::dequeue(&mut connection).await {
            Ok(Some(payload)) => process(&mut connection, &payload).await,
            Ok(None) => tokio::time::sleep(POLL_INTERVAL).await,
            Err(error) => {
                eprintln!("worker: dequeue error: {error}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process(connection: &mut Store, payload: &str) {
    let job: Job = match serde_json::from_str(payload) {
        Ok(job) => job,
        Err(error) => {
            eprintln!("worker: skipping malformed job: {error}");
            return;
        }
    };

    let processing = json!({ "status": "processing" }).to_string();
    store::set_session(connection, &job.session_id, &processing)
        .await
        .ok();

    let outcome = format_job(job.code, job.config).await;

    store::set_session(connection, &job.session_id, &outcome.to_string())
        .await
        .ok();
}

async fn format_job(code: String, config: Value) -> Value {
    let task = tokio::task::spawn_blocking(move || {
        let config: Config = serde_json::from_value(config).unwrap_or_default();
        format_code(&code, config, None, OutputVerification::None)
            .map(|formatted| {
                let changed = formatted != code;
                (formatted, changed)
            })
            .map_err(|error| error.to_string())
    });

    match timeout(FORMAT_TIMEOUT, task).await {
        Ok(Ok(Ok((formatted, changed)))) => {
            json!({ "status": "completed", "formatted": formatted, "changed": changed })
        }
        Ok(Ok(Err(message))) => json!({ "status": "failed", "error": message }),
        Ok(Err(_)) => json!({ "status": "failed", "error": "internal error" }),
        Err(_) => json!({ "status": "failed", "error": "formatting timed out" }),
    }
}

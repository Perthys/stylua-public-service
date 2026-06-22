use std::env::var;
use std::time::Duration;

use dotenvy::dotenv;
use serde_json::json;
use shared::Job;
use shared::store::{self, Store};
use stylua_lib::{Config, OutputVerification, format_code};

const POLL_INTERVAL: Duration = Duration::from_millis(200);

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

    let config: Config = serde_json::from_value(job.config).unwrap_or_default();
    let outcome = match format_code(&job.code, config, None, OutputVerification::None) {
        Ok(formatted) => {
            let changed = formatted != job.code;
            json!({ "status": "completed", "formatted": formatted, "changed": changed })
        }
        Err(error) => json!({ "status": "failed", "error": error.to_string() }),
    };

    store::set_session(connection, &job.session_id, &outcome.to_string())
        .await
        .ok();
}

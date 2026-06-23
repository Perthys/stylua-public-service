use std::env::var;
use std::time::Duration;

use dotenvy::dotenv;
use serde_json::{Value, json};
use shared::Job;
use shared::store::{self, Store};
use stylua_lib::{Config, OutputVerification, format_code};
use tokio::time::timeout;

fn env_u64(key: &str, default: u64) -> u64 {
    var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let redis_url = var("REDIS_URL").expect("REDIS_URL must be set");
    let mut connection = store::connect(&redis_url)
        .await
        .expect("redis didn't connect");

    let visibility_timeout = env_u64("VISIBILITY_TIMEOUT_SECS", 30) as usize;

    let poll_interval = Duration::from_millis(env_u64("POLL_INTERVAL_MS", 200));
    let format_timeout = Duration::from_secs(env_u64("FORMAT_TIMEOUT_SECS", 5));
    let reap_interval = Duration::from_secs(env_u64("REAP_INTERVAL_SECS", 10));

    println!("worker: I AM ALIVE");

    tokio::spawn(reaper_loop(connection.clone(), reap_interval));

    loop {
        match store::consume(&mut connection, visibility_timeout).await {
            Ok(Some(payload)) => {
                process(&mut connection, &payload, format_timeout).await;
                store::ack(&mut connection, &payload).await.ok();
            }
            Ok(None) => tokio::time::sleep(poll_interval).await,
            Err(error) => {
                eprintln!("worker: consume error: {error}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn reaper_loop(mut connection: Store, reap_interval: Duration) {
    loop {
        tokio::time::sleep(reap_interval).await;
        match store::reap(&mut connection).await {
            Ok(reaped) if reaped > 0 => eprintln!("worker: requeued {reaped} stuck job(s)"),
            Ok(_) => {}
            Err(error) => eprintln!("worker: reap error: {error}"),
        }
    }
}

async fn process(connection: &mut Store, payload: &str, format_timeout: Duration) {
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

    let outcome = format_job(job.code, job.config, format_timeout).await;

    store::set_session(connection, &job.session_id, &outcome.to_string())
        .await
        .ok();
}

async fn format_job(code: String, config: Value, format_timeout: Duration) -> Value {
    let task = tokio::task::spawn_blocking(move || {
        let config: Config = serde_json::from_value(config).unwrap_or_default();
        format_code(&code, config, None, OutputVerification::None)
            .map(|formatted| {
                let changed = formatted != code;
                (formatted, changed)
            })
            .map_err(|error| error.to_string())
    });

    match timeout(format_timeout, task).await {
        Ok(Ok(Ok((formatted, changed)))) => {
            json!({ "status": "completed", "formatted": formatted, "changed": changed })
        }
        Ok(Ok(Err(message))) => json!({ "status": "failed", "error": message }),
        Ok(Err(_)) => json!({ "status": "failed", "error": "internal error" }),
        Err(_) => json!({ "status": "failed", "error": "formatting timed out" }),
    }
}

mod app;
mod state;

use axum::serve::serve;
use dotenvy::dotenv;
use state::AppState;
use std::env::var;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let redis_url = var("REDIS_URL").expect("REDIS_URL must be set");
    let store = shared::store::connect(&redis_url)
        .await
        .expect("Failed to connect to Redis");

    let app = app::build(AppState { store });

    let port = var("PORT").unwrap_or_else(|_| "3000".to_string());
    let address = format!("0.0.0.0:{port}");

    let listener = TcpListener::bind(&address).await.unwrap();

    println!("Listening on {address}");
    serve(listener, app).await.expect("explosion");
}

mod app;
use axum::serve::serve;

use tokio::net::TcpListener;

use std::env::var;

use dotenvy::dotenv;

#[tokio::main]

async fn main() {
    dotenv().ok();

    let app = app::build();

    let port = var("PORT").unwrap_or_else(|_| "3000".to_string());
    let address = format!("0.0.0.0:{port}");

    let listener = TcpListener::bind(&address).await.unwrap();

    println!("Listening on {address}");
    serve(listener, app).await.expect("explosion");
}

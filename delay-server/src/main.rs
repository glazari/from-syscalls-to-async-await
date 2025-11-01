//! Delay Server
//!
//! Minimal Axum server: GET / sleeps 100ms then responds "hello".

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::{net::TcpListener, time::{sleep, Duration}};

async fn root() -> &'static str {
    println!("Received request, sleeping for 100ms...");
    sleep(Duration::from_millis(100)).await;
    "hello"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
}

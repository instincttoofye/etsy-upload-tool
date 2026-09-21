mod models;
mod routes;
mod services;

use axum::{
    routing::get,
    Router,
};

use routes::health::health;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3020")
        .await
        .unwrap();

    println!("InstinctivePiping Etsy backend running on port 3020");

    axum::serve(listener, app)
        .await
        .unwrap();
}
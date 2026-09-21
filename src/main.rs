mod models;
mod routes;
mod services;

use axum::{
    routing::{get, post},
    Router,
};

use routes::{
    health::health,
    listings::create_listing,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/health", get(health))
        .route("/listings", post(create_listing));
    let port = std::env::var("PORT")
    .unwrap_or_else(|_| "3020".to_string());

let address = format!("0.0.0.0:{port}");

let listener = tokio::net::TcpListener::bind(&address)
    .await
    .unwrap();

println!("InstinctivePiping Etsy backend running on {address}");

axum::serve(listener, app)
    .await
    .unwrap();
}
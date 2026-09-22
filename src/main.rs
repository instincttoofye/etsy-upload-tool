mod models;
mod routes;
mod services;
mod state;

use axum::{
    routing::{get, post},
    Router,
};

use routes::{
    etsy_auth::{
        etsy_auth,
        etsy_callback,
    },
    health::health,
    listings::create_listing,
    etsy_images::test_upload_image,
};
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = AppState::new();

    let app = Router::new()
        .route("/health", get(health))
        .route("/listings", post(create_listing))
        .route("/etsy/auth", get(etsy_auth))
        .route("/etsy/callback", get(etsy_callback))
        .route("/etsy/test-image", post(test_upload_image))
        .with_state(state);
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
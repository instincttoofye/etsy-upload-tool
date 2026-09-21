use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::services::etsy::get_my_shop;

pub async fn test_etsy_auth() -> impl IntoResponse {
    match get_my_shop().await {
        Ok(shop) => {
            println!(
                "Connected Etsy shop: {} ({})",
                shop.shop_name,
                shop.shop_id
            );

            (
                StatusCode::OK,
                format!(
                    "Connected to Etsy shop '{}' - shop_id: {}",
                    shop.shop_name,
                    shop.shop_id
                ),
            )
                .into_response()
        }

        Err(error) => {
            eprintln!("Etsy shop lookup failed: {error}");

            (
                StatusCode::BAD_GATEWAY,
                format!("Etsy shop lookup failed: {error}"),
            )
                .into_response()
        }
    }
}
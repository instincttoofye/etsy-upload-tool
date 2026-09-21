use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::services::etsy::test_authenticated_request;

pub async fn test_etsy_auth() -> impl IntoResponse {
    match test_authenticated_request().await {
        Ok(body) => (
            StatusCode::OK,
            body,
        )
            .into_response(),

        Err(error) => {
            eprintln!("Etsy auth test failed: {error}");

            (
                StatusCode::BAD_GATEWAY,
                "Etsy authenticated request failed",
            )
                .into_response()
        }
    }
}
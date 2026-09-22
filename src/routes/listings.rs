use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    models::listing::CreateListingRequest,
    services::etsy::create_draft_listing,
};

pub async fn create_listing(
    Json(listing): Json<CreateListingRequest>,
) -> impl IntoResponse {
    println!(
        "Creating Etsy draft: {}",
        listing.title
    );

    match create_draft_listing(&listing).await {
        Ok(draft) => {
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "success": true,
                    "listing_id": draft.listing_id,
                    "title": draft.title,
                    "state": draft.state,
                })),
            )
                .into_response()
        }

        Err(error) => {
            eprintln!(
                "Failed to create Etsy draft: {error}"
            );

            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "success": false,
                    "error": error.to_string(),
                })),
            )
                .into_response()
        }
    }
}
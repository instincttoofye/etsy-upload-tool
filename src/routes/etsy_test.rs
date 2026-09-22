use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::services::etsy::get_existing_listing;

pub async fn test_etsy_auth() -> impl IntoResponse {
    match get_existing_listing().await {
        Ok(listing) => {
            println!(
                "Reference Etsy listing: {} ({})",
                listing.title,
                listing.listing_id
            );

            (
                StatusCode::OK,
                format!(
                    "Listing: {}\n\
                     listing_id: {}\n\
                     taxonomy_id: {:?}\n\
                     who_made: {:?}\n\
                     when_made: {:?}",
                    listing.title,
                    listing.listing_id,
                    listing.taxonomy_id,
                    listing.who_made,
                    listing.when_made,
                ),
            )
                .into_response()
        }

        Err(error) => {
            eprintln!("Etsy listing lookup failed: {error}");

            (
                StatusCode::BAD_GATEWAY,
                format!(
                    "Etsy listing lookup failed: {error}"
                ),
            )
                .into_response()
        }
    }
}
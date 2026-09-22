use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::services::etsy::inspect_reference_listing;

pub async fn test_etsy_auth() -> impl IntoResponse {
    match inspect_reference_listing().await {
        Ok((shipping_profile_id, readiness_state_id)) => {
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
            eprintln!(
                "Reference listing inspection failed: {error}"
            );

            (
                StatusCode::BAD_GATEWAY,
                format!(
                    "Reference listing inspection failed: {error}"
                ),
            )
                .into_response()
        }
    }
}
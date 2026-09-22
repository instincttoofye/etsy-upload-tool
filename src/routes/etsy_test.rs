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
                    "taxonomy_id: 1647\n\
                     shipping_profile_id: {shipping_profile_id}\n\
                     readiness_state_id: {readiness_state_id}"
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
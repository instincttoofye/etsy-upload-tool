use axum::Json;

use crate::models::listing::CreateListingRequest;

pub async fn create_listing(
    Json(listing): Json<CreateListingRequest>,
) -> Json<CreateListingRequest> {
    println!("{:#?}", listing);

    Json(listing)
}
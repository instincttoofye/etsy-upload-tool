use crate::models::listing::CreateListingRequest;

use super::{
    client::EtsyClient,
    shops::get_my_shop,
};

#[derive(Debug, serde::Serialize)]
struct EtsyCreateDraftRequest {
    quantity: u32,
    title: String,
    description: String,
    price: f64,

    who_made: String,
    when_made: String,

    taxonomy_id: u64,
    shipping_profile_id: u64,
    readiness_state_id: u64,

    materials: String,

    item_weight: f64,
    item_length: f64,
    item_width: f64,
    item_height: f64,

    item_weight_unit: String,
    item_dimensions_unit: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyDraftListing {
    pub listing_id: u64,
    pub title: String,
    pub state: String,
}

pub async fn create_draft_listing(
    listing: &CreateListingRequest,
) -> Result<EtsyDraftListing, Box<dyn std::error::Error>> {
    let shop = get_my_shop().await?;
    let etsy = EtsyClient::new().await?;

    let payload = EtsyCreateDraftRequest {
        quantity: 1,

        title: listing.title.clone(),
        description: listing.description.clone(),
        price: listing.price,

        who_made: "i_did".to_string(),
        when_made: "2020_2026".to_string(),

        taxonomy_id: 1647,
        shipping_profile_id: 315704763375,
        readiness_state_id: 1517708374509,

        materials: listing.materials.join(","),

        item_weight: listing.package_dimensions.weight_oz,
        item_length: listing.package_dimensions.length,
        item_width: listing.package_dimensions.width,
        item_height: listing.package_dimensions.height,

        item_weight_unit: "oz".to_string(),
        item_dimensions_unit: "in".to_string(),
    };

    let url = format!(
        "https://api.etsy.com/v3/application/shops/{}/listings?legacy=false",
        shop.shop_id
    );

    let response = etsy
        .client
        .post(url)
        .header("x-api-key", &etsy.api_key)
        .bearer_auth(&etsy.access_token)
        .form(&payload)
        .send()
        .await
        .map_err(|error| {
            format!(
                "Failed to build/send Etsy draft request: {error:#?}"
            )
        })?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(
            format!(
                "Failed to create Etsy draft: {status} - {body}"
            )
            .into(),
        );
    }

    let draft =
        serde_json::from_str::<EtsyDraftListing>(&body)?;

    println!(
        "Created Etsy draft '{}' ({})",
        draft.title,
        draft.listing_id
    );

    Ok(draft)
}
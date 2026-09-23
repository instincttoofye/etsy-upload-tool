use crate::{
    models::{
        etsy::listing::{
            EtsyCreateDraftRequest,
            EtsyDraftListing,
        },
        listing::{
            CreateListingRequest,
            ProductType,
        },
    },
};

use super::{
    client::EtsyClient,
    shops::get_my_shop,
};

struct EtsyListingConfig {
    taxonomy_id: u64,
    shop_section_id: u64,
    shipping_profile_id: u64,
    readiness_state_id: u64,
    return_policy_id: u64,
}

fn listing_config(
    product_type: &ProductType,
) -> Result<EtsyListingConfig, Box<dyn std::error::Error + Send + Sync>> {
    match product_type {
        ProductType::Pipe => {
            Ok(EtsyListingConfig {
                taxonomy_id: 1647,
                shop_section_id: 60450910,
                shipping_profile_id: 315704763375,
                readiness_state_id: 1517708374509,
                return_policy_id: 1516472977734,
            })
        }

        ProductType::Tamper => {
            Ok(EtsyListingConfig {
                taxonomy_id: 1866,
                shop_section_id: 60470571,
                shipping_profile_id: 315704763375,
                readiness_state_id: 1517708374509,
                return_policy_id: 1516472977734,
            })
        }

        ProductType::Ashtray => {
            Err(
                "Ashtray listings are not configured yet"
                    .into()
            )
        }
    }
}

pub async fn create_draft_listing(
    listing: &CreateListingRequest,
) -> Result<EtsyDraftListing, Box<dyn std::error::Error + Send + Sync>> {
    let shop = get_my_shop().await?;
    let etsy = EtsyClient::new().await?;

    let config = listing_config(
        &listing.product_type
    )?;

    let payload = EtsyCreateDraftRequest {
        quantity: 1,

        title: listing.title.clone(),
        description: listing.description.clone(),
        price: listing.price,

        who_made: "i_did".to_string(),
        when_made: "2020_2026".to_string(),

        taxonomy_id: config.taxonomy_id,
        shop_section_id: config.shop_section_id,
        shipping_profile_id: config.shipping_profile_id,
        readiness_state_id: config.readiness_state_id,
        return_policy_id: config.return_policy_id,

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



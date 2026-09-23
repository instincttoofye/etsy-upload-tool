use super::client::EtsyClient;

use crate::models::etsy::shop::{
    EtsyMeResponse,
    EtsyShopResponse,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EtsyShopSectionsResponse {
    pub count: u32,
    pub results: Vec<EtsyShopSection>,
}

#[derive(Debug, Deserialize)]
pub struct EtsyShopSection {
    pub shop_section_id: u64,
    pub title: String,
    pub rank: u32,
    pub user_id: u64,
    pub active_listing_count: u32,
}

pub async fn get_shop_sections(
    shop_id: u64,
) -> Result<EtsyShopSectionsResponse, Box<dyn std::error::Error + Send + Sync>> {
    let etsy = EtsyClient::new().await?;

    let url = format!(
        "https://api.etsy.com/v3/application/shops/{shop_id}/sections"
    );

    let response = etsy
        .client
        .get(url)
        .header("x-api-key", &etsy.api_key)
        .bearer_auth(&etsy.access_token)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(
            format!(
                "Failed to retrieve Etsy shop sections: {status} - {body}"
            )
            .into(),
        );
    }

    let sections =
        serde_json::from_str::<EtsyShopSectionsResponse>(&body)?;

    Ok(sections)
}

pub async fn get_my_shop(
) -> Result<EtsyShopResponse, Box<dyn std::error::Error + Send + Sync>> {
    let etsy = EtsyClient::new().await?;

    let response = etsy
        .client
        .get("https://api.etsy.com/v3/application/users/me")
        .header("x-api-key", &etsy.api_key)
        .bearer_auth(&etsy.access_token)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await?;

        return Err(
            format!(
                "Failed to retrieve Etsy user: {status} - {body}"
            )
            .into(),
        );
    }

    let me = response
        .json::<EtsyMeResponse>()
        .await?;

    let url = format!(
        "https://api.etsy.com/v3/application/users/{}/shops",
        me.user_id
    );

    let response = etsy
        .client
        .get(url)
        .header("x-api-key", &etsy.api_key)
        .bearer_auth(&etsy.access_token)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await?;

        return Err(
            format!(
                "Failed to retrieve Etsy shop: {status} - {body}"
            )
            .into(),
        );
    }

    let shop = response
        .json::<EtsyShopResponse>()
        .await?;

    Ok(shop)
}
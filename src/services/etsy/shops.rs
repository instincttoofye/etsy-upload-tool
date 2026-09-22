use super::client::EtsyClient;

use crate::models::etsy::shop::{
    EtsyMeResponse,
    EtsyShopResponse,
};

pub async fn get_my_shop(
) -> Result<EtsyShopResponse, Box<dyn std::error::Error>> {
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
use std::path::Path;

use reqwest::multipart::{
    Form,
    Part,
};

use super::client::EtsyClient;

#[derive(Debug, serde::Deserialize)]
pub struct EtsyListingImage {
    pub listing_image_id: u64,
    pub listing_id: u64,
    pub rank: u32,
}

pub async fn upload_listing_image(
    shop_id: u64,
    listing_id: u64,
    image_path: &Path,
) -> Result<EtsyListingImage, Box<dyn std::error::Error + Send + Sync>> {
    let etsy = EtsyClient::new().await?;

    let file_name = image_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("listing-image.jpg")
        .to_string();

    let image_bytes = tokio::fs::read(image_path).await?;

    let image_part = Part::bytes(image_bytes)
        .file_name(file_name);

    let form = Form::new()
        .part("image", image_part);

    let url = format!(
        "https://api.etsy.com/v3/application/shops/{shop_id}/listings/{listing_id}/images"
    );

    let response = etsy
        .client
        .post(url)
        .header("x-api-key", &etsy.api_key)
        .bearer_auth(&etsy.access_token)
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(
            format!(
                "Failed to upload Etsy listing image: {status} - {body}"
            )
            .into(),
        );
    }

    let image =
        serde_json::from_str::<EtsyListingImage>(&body)?;

    println!(
        "Uploaded Etsy listing image {} to listing {}",
        image.listing_image_id,
        listing_id
    );

    Ok(image)
}
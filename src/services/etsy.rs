use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::fs;

use crate::routes::etsy_auth::{
    EtsyTokenResponse,
    StoredEtsyTokens,
};

const TOKEN_PATH: &str = "/data/etsy_tokens.json";

#[derive(Debug, serde::Deserialize)]
struct EtsyMeResponse {
    user_id: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyShopResponse {
    pub shop_id: u64,
    pub user_id: u64,
    pub shop_name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyListingsResponse {
    pub count: u64,
    pub results: Vec<EtsyListing>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyListing {
    pub listing_id: u64,
    pub title: String,
    pub taxonomy_id: Option<u64>,

    pub who_made: Option<String>,
    pub when_made: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyShippingBatchResponse {
    pub results: Vec<EtsyListingShipping>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyListingShipping {
    pub listing_id: u64,
    pub shipping_profile: Option<EtsyShippingProfile>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyShippingProfile {
    pub shipping_profile_id: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyInventoryBatchResponse {
    pub results: Vec<EtsyListingInventoryResult>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyListingInventoryResult {
    pub listing_id: u64,
    pub inventory: Option<EtsyInventory>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyInventory {
    pub products: Vec<EtsyProduct>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyProduct {
    pub offerings: Vec<EtsyOffering>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyOffering {
    pub readiness_state_id: Option<u64>,
}

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
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyDraftListing {
    pub listing_id: u64,
    pub title: String,
    pub state: String,
}

pub async fn inspect_reference_listing(
) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let access_token = get_valid_access_token().await?;

    let keystring = std::env::var("ETSY_KEYSTRING")?;
    let shared_secret = std::env::var("ETSY_SHARED_SECRET")?;

    let api_key = format!("{keystring}:{shared_secret}");

    let listing_id = 4579334215_u64;

    let client = reqwest::Client::new();

    // Shipping profile
    let shipping_response = client
        .get(
            "https://api.etsy.com/v3/application/listings/batch/shipping"
        )
        .query(&[
            ("listing_ids", listing_id.to_string()),
        ])
        .header("x-api-key", &api_key)
        .bearer_auth(&access_token)
        .send()
        .await?;

    let shipping_status = shipping_response.status();

    if !shipping_status.is_success() {
        let body = shipping_response.text().await?;

        return Err(
            format!(
                "Shipping lookup failed: {shipping_status} - {body}"
            )
            .into()
        );
    }

    let shipping =
        shipping_response
            .json::<EtsyShippingBatchResponse>()
            .await?;

    let shipping_profile_id = shipping
        .results
        .first()
        .and_then(|result| result.shipping_profile.as_ref())
        .map(|profile| profile.shipping_profile_id)
        .ok_or("No shipping profile found")?;

    // Inventory / processing profile
    let inventory_response = client
        .get(
            "https://api.etsy.com/v3/application/listings/batch/inventory"
        )
        .query(&[
            ("listing_ids", listing_id.to_string()),
        ])
        .header("x-api-key", &api_key)
        .bearer_auth(&access_token)
        .send()
        .await?;

    let inventory_status = inventory_response.status();

    if !inventory_status.is_success() {
        let body = inventory_response.text().await?;

        return Err(
            format!(
                "Inventory lookup failed: {inventory_status} - {body}"
            )
            .into()
        );
    }

    let inventory =
        inventory_response
            .json::<EtsyInventoryBatchResponse>()
            .await?;

    let readiness_state_id = inventory
        .results
        .first()
        .and_then(|result| result.inventory.as_ref())
        .and_then(|inventory| inventory.products.first())
        .and_then(|product| product.offerings.first())
        .and_then(|offering| offering.readiness_state_id)
        .ok_or("No readiness state found")?;

    Ok((
        shipping_profile_id,
        readiness_state_id,
    ))
}

pub async fn get_existing_listing(
) -> Result<EtsyListing, Box<dyn std::error::Error>> {
    let shop = get_my_shop().await?;
    let access_token = get_valid_access_token().await?;

    let keystring = std::env::var("ETSY_KEYSTRING")?;
    let shared_secret = std::env::var("ETSY_SHARED_SECRET")?;

    let api_key = format!("{keystring}:{shared_secret}");

    let url = format!(
        "https://api.etsy.com/v3/application/shops/{}/listings",
        shop.shop_id
    );

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .query(&[
            ("state", "active"),
            ("limit", "1"),
        ])
        .header("x-api-key", api_key)
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await?;

        return Err(
            format!(
                "Failed to retrieve Etsy listings: {status} - {body}"
            )
            .into()
        );
    }

    let listings =
        response.json::<EtsyListingsResponse>().await?;

    listings
        .results
        .into_iter()
        .next()
        .ok_or_else(|| "No active Etsy listings found".into())
}

pub async fn get_my_shop(
) -> Result<EtsyShopResponse, Box<dyn std::error::Error>> {
    let access_token = get_valid_access_token().await?;

    let keystring = std::env::var("ETSY_KEYSTRING")?;
    let shared_secret = std::env::var("ETSY_SHARED_SECRET")?;

    let api_key = format!("{keystring}:{shared_secret}");

    let client = reqwest::Client::new();

    // First: find the authenticated Etsy user.
    let response = client
        .get("https://api.etsy.com/v3/application/users/me")
        .header("x-api-key", &api_key)
        .bearer_auth(&access_token)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await?;

        return Err(
            format!(
                "Failed to retrieve Etsy user: {status} - {body}"
            )
            .into()
        );
    }

    let me = response
        .json::<EtsyMeResponse>()
        .await?;

    // Then: retrieve the shop owned by that user.
    let url = format!(
        "https://api.etsy.com/v3/application/users/{}/shops",
        me.user_id
    );

    let response = client
        .get(url)
        .header("x-api-key", api_key)
        .bearer_auth(&access_token)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await?;

        return Err(
            format!(
                "Failed to retrieve Etsy shop: {status} - {body}"
            )
            .into()
        );
    }

    let shop = response
        .json::<EtsyShopResponse>()
        .await?;

    Ok(shop)
}

pub async fn save_tokens(
    response: EtsyTokenResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let stored_tokens = StoredEtsyTokens {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        token_type: response.token_type,
        scope: response.scope,
        expires_at: now + response.expires_in,
    };

    if let Some(parent) = Path::new(TOKEN_PATH).parent() {
        fs::create_dir_all(parent).await?;
    }

    let json =
        serde_json::to_string_pretty(&stored_tokens)?;

    fs::write(TOKEN_PATH, json).await?;

    Ok(())
}

pub async fn load_tokens(
) -> Result<StoredEtsyTokens, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(TOKEN_PATH).await?;

    let tokens =
        serde_json::from_str::<StoredEtsyTokens>(&json)?;

    Ok(tokens)
}

pub async fn get_valid_access_token(
) -> Result<String, Box<dyn std::error::Error>> {
    let tokens = load_tokens().await?;

    if token_is_expired(&tokens)? {
        println!("Etsy access token expired. Refreshing...");

        let refreshed =
            refresh_tokens(&tokens.refresh_token).await?;

        return Ok(refreshed.access_token);
    }

    Ok(tokens.access_token)
}

pub async fn test_authenticated_request(
) -> Result<String, Box<dyn std::error::Error>> {
    let access_token = get_valid_access_token().await?;

    let keystring = std::env::var("ETSY_KEYSTRING")?;
    let shared_secret = std::env::var("ETSY_SHARED_SECRET")?;

    let api_key = format!("{keystring}:{shared_secret}");

    let client = reqwest::Client::new();

    let response = client
        .get("https://api.etsy.com/v3/application/listings")
        .query(&[
            ("state", "active"),
            ("limit", "1"),
        ])
        .header("x-api-key", api_key)
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(
            format!(
                "Authenticated Etsy request failed: {status} - {body}"
            )
            .into()
        );
    }

    Ok(body)
}

fn current_timestamp(
) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
    )
}

fn token_is_expired(
    tokens: &StoredEtsyTokens,
) -> Result<bool, Box<dyn std::error::Error>> {
    let now = current_timestamp()?;

    // Refresh a minute early instead of riding the exact expiration.
    Ok(now >= tokens.expires_at.saturating_sub(60))
}

async fn refresh_tokens(
    refresh_token: &str,
) -> Result<StoredEtsyTokens, Box<dyn std::error::Error>> {
    let keystring = std::env::var("ETSY_KEYSTRING")?;

    let client = reqwest::Client::new();

    let response = client
        .post("https://api.etsy.com/v3/public/oauth/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", keystring.as_str()),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;

        return Err(
            format!(
                "Etsy token refresh failed: {status} - {body}"
            )
            .into()
        );
    }

    let response =
        response.json::<EtsyTokenResponse>().await?;

    let now = current_timestamp()?;

    let tokens = StoredEtsyTokens {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        token_type: response.token_type,
        scope: response.scope,
        expires_at: now + response.expires_in,
    };

    let json = serde_json::to_string_pretty(&tokens)?;

    fs::write(TOKEN_PATH, json).await?;

    println!("Etsy access token refreshed successfully");

    Ok(tokens)
}
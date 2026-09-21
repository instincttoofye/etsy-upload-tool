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
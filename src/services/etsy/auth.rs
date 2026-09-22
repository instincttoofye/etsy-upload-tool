use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::fs;

use crate::models::etsy::auth::{
    EtsyTokenResponse,
    StoredEtsyTokens,
};

const TOKEN_PATH: &str = "/data/etsy_tokens.json";

pub async fn save_tokens(
    response: EtsyTokenResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = current_timestamp()?;

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
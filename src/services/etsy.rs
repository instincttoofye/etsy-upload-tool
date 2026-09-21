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
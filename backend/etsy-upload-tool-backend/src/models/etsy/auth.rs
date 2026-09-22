use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct EtsyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredEtsyTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_at: u64,
}
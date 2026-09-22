use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};

use base64::{
    engine::general_purpose::URL_SAFE_NO_PAD,
    Engine,
};

use rand::Rng;
use sha2::{Digest, Sha256};
use url::Url;

use crate::{
    services::etsy::auth::save_tokens,
    state::AppState,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EtsyCallbackQuery {
    pub code: String,
    pub state: String,
}

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

pub async fn etsy_auth(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let keystring = match std::env::var("ETSY_KEYSTRING") {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ETSY_KEYSTRING is not configured",
            )
                .into_response();
        }
    };

    let redirect_uri = match std::env::var("ETSY_REDIRECT_URI") {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ETSY_REDIRECT_URI is not configured",
            )
                .into_response();
        }
    };

    // Generate 32 cryptographically random bytes.
    let mut verifier_bytes = [0u8; 32];
    rand::rng().fill(&mut verifier_bytes);

    // URL-safe Base64 without padding gives us a valid
    // 43-character PKCE verifier.
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

    // SHA256(verifier) -> URL-safe Base64
    let code_challenge =
        URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));

    // Generate a separate random value for OAuth state.
    let mut state_bytes = [0u8; 32];
    rand::rng().fill(&mut state_bytes);

    let state = URL_SAFE_NO_PAD.encode(state_bytes);

    let mut auth_url =
        Url::parse("https://www.etsy.com/oauth/connect").unwrap();

    auth_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &keystring)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", "listings_r listings_w shops_r")
        .append_pair("state", &state)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256");

        app_state
        .pending_oauth
        .lock()
        .await
        .insert(state.clone(), code_verifier);

    Redirect::temporary(auth_url.as_str()).into_response()
}

pub async fn etsy_callback(
    State(app_state): State<AppState>,
    Query(query): Query<EtsyCallbackQuery>,
) -> impl IntoResponse {
    let code_verifier = {
        let mut pending_oauth =
            app_state.pending_oauth.lock().await;

        match pending_oauth.remove(&query.state) {
            Some(verifier) => verifier,

            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Invalid or expired OAuth state",
                )
                    .into_response();
            }
        }
    };

    let keystring = match std::env::var("ETSY_KEYSTRING") {
        Ok(value) => value,

        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ETSY_KEYSTRING is not configured",
            )
                .into_response();
        }
    };

    let redirect_uri =
        match std::env::var("ETSY_REDIRECT_URI") {
            Ok(value) => value,

            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ETSY_REDIRECT_URI is not configured",
                )
                    .into_response();
            }
        };

    let client = reqwest::Client::new();

    let response = match client
        .post("https://api.etsy.com/v3/public/oauth/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", keystring.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", query.code.as_str()),
            ("code_verifier", code_verifier.as_str()),
        ])
        .send()
        .await
    {
        Ok(response) => response,

        Err(error) => {
            eprintln!("Failed to contact Etsy token endpoint: {error}");

            return (
                StatusCode::BAD_GATEWAY,
                "Failed to contact Etsy",
            )
                .into_response();
        }
    };

    if !response.status().is_success() {
        let status = response.status();

        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown Etsy error".to_string());

        eprintln!(
            "Etsy token exchange failed: {status} - {body}"
        );

        return (
            StatusCode::BAD_GATEWAY,
            "Etsy rejected the token exchange",
        )
            .into_response();
    }

    let token_response =
        match response.json::<EtsyTokenResponse>().await {
            Ok(tokens) => tokens,

            Err(error) => {
                eprintln!("Failed to decode Etsy tokens: {error}");

                return (
                    StatusCode::BAD_GATEWAY,
                    "Invalid response from Etsy",
                )
                    .into_response();
            }
        };

        let scope = token_response.scope.clone();
let expires_in = token_response.expires_in;

if let Err(error) = save_tokens(token_response).await {
    eprintln!("Failed to persist Etsy tokens: {error}");

    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Etsy authorized successfully, but token storage failed",
    )
        .into_response();
}

println!("Etsy OAuth successful!");
println!("Granted scopes: {scope}");
println!("Expires in: {expires_in}");


    /*
        TEMPORARY.

        Do NOT print the actual access_token or refresh_token.
        We'll persist those properly next.
    */

    (
        StatusCode::OK,
        "Etsy authorization successful. You can close this window.",
    )
        .into_response()
}
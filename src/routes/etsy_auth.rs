use axum::{
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

pub async fn etsy_auth() -> impl IntoResponse {
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
        .append_pair("scope", "listings_r listings_w")
        .append_pair("state", &state)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256");

    /*
        TEMPORARY.

        We need these for the callback we're building next.

        Do NOT leave the verifier printed once OAuth is finished.
    */
    println!("OAuth state: {state}");
    println!("PKCE verifier: {code_verifier}");

    Redirect::temporary(auth_url.as_str()).into_response()
}
pub struct EtsyClient {
    pub client: reqwest::Client,
    pub access_token: String,
    pub api_key: String,
}

impl EtsyClient {
    pub async fn new(
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let access_token =
            super::auth::get_valid_access_token().await?;

        let keystring =
            std::env::var("ETSY_KEYSTRING")?;

        let shared_secret =
            std::env::var("ETSY_SHARED_SECRET")?;

        let api_key =
            format!("{keystring}:{shared_secret}");

        Ok(Self {
            client: reqwest::Client::new(),
            access_token,
            api_key,
        })
    }
}
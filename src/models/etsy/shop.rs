#[derive(Debug, serde::Deserialize)]
pub struct EtsyMeResponse {
   pub user_id: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyShopResponse {
    pub shop_id: u64,
    pub user_id: u64,
    pub shop_name: String,
}
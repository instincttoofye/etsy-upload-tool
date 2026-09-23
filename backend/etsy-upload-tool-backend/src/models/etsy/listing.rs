#[derive(Debug, serde::Serialize)]
pub struct EtsyCreateDraftRequest {
    pub quantity: u32,
    pub title: String,
    pub description: String,
    pub price: f64,

    pub who_made: String,
    pub when_made: String,

    pub taxonomy_id: u64,
    pub shop_section_id: u64,
    pub shipping_profile_id: u64,
    pub readiness_state_id: u64,
    pub return_policy_id: u64,

    pub materials: String,
    pub tags: String,

    pub item_weight: f64,
    pub item_length: f64,
    pub item_width: f64,
    pub item_height: f64,

    pub item_weight_unit: String,
    pub item_dimensions_unit: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EtsyDraftListing {
    pub listing_id: u64,
    pub title: String,
    pub state: String,
}
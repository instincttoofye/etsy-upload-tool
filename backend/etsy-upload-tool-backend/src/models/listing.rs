use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Pipe,
    Tamper,
    Ashtray,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipeDimensions {
    pub overall_length: f64,
    pub bowl_height: f64,
    pub chamber_diameter: f64,
    pub chamber_depth: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessoryDimensions {
    pub length: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDimensions {
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub weight_oz: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "dimensions")]
pub enum ProductDimensions {
    Pipe(PipeDimensions),
    Accessory(AccessoryDimensions),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateListingRequest {
    pub product_type: ProductType,

    pub title: String,
    pub price: f64,
    pub description: String,
    pub materials: Vec<String>,
    pub tags: Vec<String>,

    pub product_dimensions: ProductDimensions,
    pub package_dimensions: PackageDimensions,
}
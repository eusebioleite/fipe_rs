use serde::Deserialize;

// Types
#[derive(Debug, Deserialize)]
pub struct Types {
    pub id: i32,
    pub description: String,
}

// References
#[derive(Debug, Deserialize)]
pub struct References {
    pub id: i32,
    pub description: String,
    pub ref_date: String,
    pub fipe: String,
}

#[derive(Debug, Deserialize)]
pub struct ReferencesResponse {
    #[serde(rename = "Codigo")]
    pub codigo: i32,
    #[serde(rename = "Mes")]
    pub mes: String,
}

// Brands
#[derive(Debug, Deserialize)]
pub struct Brands {
    pub id: i32,
    pub description: String,
    pub fipe: String,
    pub ref_id: String,
    pub ref_description: String,
    pub type_id: i32,
    pub type_description: String,
}

// Models
pub struct Models {
    pub id: i32,
    pub description: String,
    pub fipe: String,
    pub ref_id: String,
    pub ref_description: String,
    pub type_id: i32,
    pub type_description: String,
    pub brand_id: String,
    pub brand_description: String,
}

pub struct ModelsReplicate {
    pub id: i32,
    pub description: String,
    pub ref_description: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    #[serde(rename = "Modelos")]
    pub model: Vec<FipeModels>,
}

#[derive(Debug, Deserialize)]
pub struct FipeModels {
    #[serde(rename = "Label")]
    pub label: String,
    #[serde(rename = "Value")]
    pub value: i32,
}

// Generic
#[derive(Debug, Deserialize)]
pub struct FipeStruct {
    #[serde(rename = "Label")]
    pub label: String,
    #[serde(rename = "Value")]
    pub value: String,
}

// Utilities
#[derive(Debug, Deserialize)]
pub struct Status {
    pub db_status: String,
    pub last_update: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RowCount {
    pub brands_rowcount: i32,
    pub models_rowcount: i32,
    pub years_rowcount: i32,
    pub vehicles_rowcount: i32,
}

pub struct Count {
    pub count: i32,
}

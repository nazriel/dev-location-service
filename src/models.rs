// src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "MaxResults")]
    pub max_results: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    #[serde(rename = "Results")]
    pub results: Vec<ResultItem>,
    #[serde(rename = "Summary")]
    pub summary: SearchSummary,
}

#[derive(Debug, Serialize)]
pub struct SearchSummary {
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "MaxResults")]
    pub max_results: i32,
    #[serde(rename = "ResultCount")]
    pub result_count: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Place {
    #[serde(rename = "Label")]
    pub label: String,
    #[serde(rename = "Geometry")]
    pub geometry: Geometry,
    #[serde(rename = "PlaceId")]
    pub place_id: String,
    #[serde(rename = "Relevance")]
    pub relevance: f64,
    #[serde(rename = "Interpolated")]
    pub interpolated: Option<bool>,
    #[serde(rename = "Country")]
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Geometry {
    #[serde(rename = "Point")]
    pub point: Vec<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResultItem {
    #[serde(rename = "Place")]
    pub place: Place,
    #[serde(rename = "Relevance")]
    pub relevance: f64,
}

#[derive(Debug, Deserialize)]
pub struct NominatimAddress {
    pub country_code: String,
}

// OpenStreetMap Nominatim response structures
#[derive(Debug, Deserialize)]
pub struct NominatimPlace {
    pub display_name: String,
    pub lat: String,
    pub lon: String,
    pub place_id: i64,
    pub importance: Option<f64>,
    pub address: NominatimAddress,
}

// YAML configuration structures
#[derive(Debug, Deserialize)]
pub struct PlacesConfig {
    pub places: Vec<PredefinedPlace>,
    pub metadata: Option<ConfigMetadata>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PredefinedPlace {
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigMetadata {
    pub version: Option<String>,
    pub description: Option<String>,
    pub last_updated: Option<String>,
}

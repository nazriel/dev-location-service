// src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "Text")]
    pub text: Option<String>,
    #[serde(rename = "Position")]
    pub position: Option<[f64; 2]>,
    #[serde(rename = "MaxResults")]
    pub max_results: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::SearchRequest;

    #[test]
    fn accepts_a_valid_position_without_text() {
        let request = SearchRequest {
            text: None,
            position: Some([21.0122, 52.2297]),
            max_results: Some(1),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn rejects_ambiguous_or_invalid_requests() {
        let both = SearchRequest {
            text: Some("Warszawa".to_string()),
            position: Some([21.0122, 52.2297]),
            max_results: None,
        };
        let invalid_position = SearchRequest {
            text: None,
            position: Some([181.0, 52.2297]),
            max_results: None,
        };
        let blank_text = SearchRequest {
            text: Some("   ".to_string()),
            position: None,
            max_results: None,
        };

        assert!(both.validate().is_err());
        assert!(invalid_position.validate().is_err());
        assert!(blank_text.validate().is_err());
    }
}

impl SearchRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        match (&self.text, self.position) {
            (Some(text), None) if !text.trim().is_empty() => Ok(()),
            (Some(_), None) => Err("Text must not be empty"),
            (None, Some([longitude, latitude]))
                if longitude.is_finite()
                    && latitude.is_finite()
                    && (-180.0..=180.0).contains(&longitude)
                    && (-90.0..=90.0).contains(&latitude) =>
            {
                Ok(())
            }
            (None, Some(_)) => Err("Position must contain a valid [longitude, latitude] pair"),
            (Some(_), Some(_)) => Err("provide either Text or Position, not both"),
            (None, None) => Err("either Text or Position is required"),
        }
    }
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
    #[serde(rename = "Text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "Position", skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 2]>,
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

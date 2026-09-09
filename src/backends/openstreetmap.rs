use super::LocationBackend;
use crate::models::{Geometry, NominatimPlace, Place, SearchRequest};
use async_trait::async_trait;
use celes::Country;
use std::str::FromStr;

pub struct OpenStreetMapBackend {
    client: reqwest::Client,
    base_url: String,
}

impl OpenStreetMapBackend {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://nominatim.openstreetmap.org".to_string(),
        }
    }
}

#[async_trait]
impl LocationBackend for OpenStreetMapBackend {
    async fn search(
        &self,
        request: &SearchRequest,
    ) -> Result<Vec<Place>, Box<dyn std::error::Error>> {
        let max_results = request.max_results.unwrap_or(10);
        let url;
        let params;

        match (&request.text, request.position) {
            (Some(text), None) => {
                url = format!("{}/search", self.base_url);
                params = vec![
                    ("q", text.clone()),
                    ("format", "json".to_string()),
                    ("limit", max_results.to_string()),
                    ("accept-language", "en".to_string()),
                    ("addressdetails", "1".to_string()),
                ];
            }
            (None, Some([lon, lat])) => {
                url = format!("{}/reverse", self.base_url);
                params = vec![
                    ("lon", lon.to_string()),
                    ("lat", lat.to_string()),
                    ("format", "json".to_string()),
                    ("accept-language", "en".to_string()),
                    ("addressdetails", "1".to_string()),
                ];
            }
            _ => {
                return Err("request must contain exactly one of Text or Position".into());
            }
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("User-Agent", "LocationServiceMock/1.0")
            .send()
            .await?;

        let response_body: serde_json::Value = response.json().await?;
        let nominatim_places: Vec<NominatimPlace> = if request.position.is_some() {
            if response_body.get("error").is_some() {
                Vec::new()
            } else {
                vec![serde_json::from_value(response_body)?]
            }
        } else {
            serde_json::from_value(response_body)?
        };

        let results = nominatim_places
            .into_iter()
            .map(|nom_place| {
                let lat: f64 = nom_place.lat.parse().unwrap_or(0.0);
                let lon: f64 = nom_place.lon.parse().unwrap_or(0.0);
                let relevance = nom_place.importance.unwrap_or(0.5);
                Place {
                    label: nom_place.display_name,
                    geometry: Geometry {
                        point: vec![lon, lat],
                    },
                    place_id: nom_place.place_id.to_string(),
                    relevance,
                    interpolated: Some(false),
                    country: match Country::from_str(nom_place.address.country_code.as_str()) {
                        Ok(country) => Some(country.alpha3.to_string()),
                        Err(_) => None,
                    },
                }
            })
            .collect();

        Ok(results)
    }
}

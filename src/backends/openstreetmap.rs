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
        let max_results_string = max_results.to_string();
        let url = format!("{}/search", self.base_url);
        let params = vec![
            ("q", request.text.as_str()),
            ("format", "json"),
            ("limit", &max_results_string),
            ("accept-language", "en"),
            ("addressdetails", "1"),
        ];

        let request = self
            .client
            .get(&url)
            .query(&params)
            .header("User-Agent", "LocationServiceMock/1.0");

        let response = request.send().await?;
        let nominatim_places: Vec<NominatimPlace> = response.json().await?;

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

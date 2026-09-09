// src/backends/predefined.rs
use super::LocationBackend;
use crate::models::{Geometry, Place, PlacesConfig, PredefinedPlace, SearchRequest};
use async_trait::async_trait;
use rand::seq::SliceRandom;
use std::path::Path;
use strsim::levenshtein;
use tracing::{info, warn};
use uuid::Uuid;

pub struct PredefinedBackend {
    places: Vec<PredefinedPlace>,
}

impl PredefinedBackend {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;
        let config: PlacesConfig = serde_yaml::from_str(&file_content)?;

        info!(
            places = config.places.len(),
            msg = "Loaded places from configuration file"
        );

        if let Some(metadata) = &config.metadata {
            if let Some(description) = &metadata.description {
                info!(description = %description, msg = "Places config description");
            }
            if let Some(version) = &metadata.version {
                info!(version = %version, msg = "Places config version");
            }
            if let Some(last_updated) = &metadata.last_updated {
                info!(last_updated = %last_updated, msg = "Places config last updated");
            }
        }

        Ok(Self {
            places: config.places,
        })
    }

    // Fallback method with default places (for backward compatibility)
    pub fn with_defaults() -> Self {
        let places = vec![
            PredefinedPlace {
                label: "Warsaw, Poland".to_string(),
                latitude: 52.2297,
                longitude: 21.0122,
                country: Some("Poland".to_string()),
                city: Some("Warsaw".to_string()),
            },
            PredefinedPlace {
                label: "Berlin, Germany".to_string(),
                latitude: 52.5200,
                longitude: 13.4050,
                country: Some("Germany".to_string()),
                city: Some("Berlin".to_string()),
            },
        ];

        warn!(msg = "Using default fallback places (configuration file not found)");

        Self { places }
    }

    fn calculate_relevance(&self, query: &str, place: &PredefinedPlace) -> f64 {
        let query_lower = query.to_lowercase();
        let label_lower = place.label.to_lowercase();

        // Check for exact substring matches first
        if label_lower.contains(&query_lower) || query_lower.contains(&label_lower) {
            return 0.9;
        }

        // Check individual fields for matches
        let mut best_score: f64 = 0.0;

        // Check against label
        let label_distance = levenshtein(&query_lower, &label_lower);
        let label_max_len = query.len().max(place.label.len());
        if label_max_len > 0 {
            let label_score = 1.0 - (label_distance as f64 / label_max_len as f64);
            best_score = best_score.max(label_score);
        }

        // Check against city
        if let Some(city) = &place.city {
            let city_lower = city.to_lowercase();
            if city_lower.contains(&query_lower) || query_lower.contains(&city_lower) {
                best_score = best_score.max(0.8);
            } else {
                let city_distance = levenshtein(&query_lower, &city_lower);
                let city_max_len = query.len().max(city.len());
                if city_max_len > 0 {
                    let city_score = 1.0 - (city_distance as f64 / city_max_len as f64);
                    best_score = best_score.max(city_score * 0.7);
                }
            }
        }

        // Check against country
        if let Some(country) = &place.country {
            let country_lower = country.to_lowercase();
            if country_lower.contains(&query_lower) || query_lower.contains(&country_lower) {
                best_score = best_score.max(0.6);
            }
        }

        best_score.into()
    }

    fn distance_km(position: [f64; 2], place: &PredefinedPlace) -> f64 {
        let [longitude, latitude] = position;
        let latitude_delta = (place.latitude - latitude).to_radians();
        let longitude_delta = (place.longitude - longitude).to_radians();
        let latitude_radians = latitude.to_radians();
        let place_latitude_radians = place.latitude.to_radians();
        let a = (latitude_delta / 2.0).sin().powi(2)
            + latitude_radians.cos()
                * place_latitude_radians.cos()
                * (longitude_delta / 2.0).sin().powi(2);

        6_371.0 * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
    }
}

#[async_trait]
impl LocationBackend for PredefinedBackend {
    async fn search(
        &self,
        request: &SearchRequest,
    ) -> Result<Vec<Place>, Box<dyn std::error::Error>> {
        let max_results = request.max_results.unwrap_or(10) as usize;
        let mut scored_places: Vec<(PredefinedPlace, f64)> = match (&request.text, request.position)
        {
            (Some(query), None) => self
                .places
                .iter()
                .map(|place| (place.clone(), self.calculate_relevance(query, place)))
                .collect(),
            (None, Some(position)) => self
                .places
                .iter()
                .map(|place| {
                    let distance = Self::distance_km(position, place);
                    (place.clone(), 1.0 / (1.0 + distance))
                })
                .collect(),
            _ => {
                return Err("request must contain exactly one of Text or Position".into());
            }
        };

        // Sort by relevance (highest first).
        scored_places.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Text searches retain the mock's fallback behavior when no place matches.
        if request.text.is_some() && (scored_places.is_empty() || scored_places[0].1 < 0.3) {
            let mut rng = rand::rng();
            let mut random_places = self.places.clone();
            random_places.shuffle(&mut rng);
            scored_places = random_places
                .into_iter()
                .take(max_results)
                .map(|place| (place, 0.5))
                .collect();
        }

        let results = scored_places
            .into_iter()
            .take(max_results)
            .map(|(place, relevance)| Place {
                label: place.label.clone(),
                geometry: Geometry {
                    point: vec![place.longitude, place.latitude],
                },
                place_id: Uuid::new_v4().to_string(),
                relevance,
                interpolated: Some(false), // No interpolation for predefined places
                country: place.country.clone(),
            })
            .collect();

        Ok(results)
    }
}

// src/backends/mod.rs
use crate::models::{Place, SearchRequest};
use async_trait::async_trait;

pub mod openstreetmap;
pub mod predefined;

#[async_trait]
pub trait LocationBackend {
    async fn search(
        &self,
        request: &SearchRequest,
    ) -> Result<Vec<Place>, Box<dyn std::error::Error>>;
}

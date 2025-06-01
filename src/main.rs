mod backends;
mod models;

use actix_web::{App, HttpResponse, HttpServer, Result, web};
use backends::{
    LocationBackend, openstreetmap::OpenStreetMapBackend, predefined::PredefinedBackend,
};
use models::{ResultItem, SearchRequest, SearchResponse, SearchSummary};
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_appender::{self, non_blocking};

#[derive(Debug, Clone)]
enum BackendType {
    Predefined,
    OpenStreetMap,
}

impl BackendType {
    fn from_env() -> Result<Self, String> {
        let backend_name = env::var("LOCATION_BACKEND")
            .unwrap_or_else(|_| "predefined".to_string())
            .to_lowercase();

        match backend_name.as_str() {
            "predefined" | "pred" => Ok(BackendType::Predefined),
            "openstreetmap" | "osm" | "nominatim" => Ok(BackendType::OpenStreetMap),
            _ => Err(format!(
                "Invalid backend '{}'. Supported backends: predefined, openstreetmap",
                backend_name
            )),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            BackendType::Predefined => "predefined",
            BackendType::OpenStreetMap => "openstreetmap",
        }
    }
}

struct AppState {
    backend: Arc<dyn LocationBackend + Send + Sync>,
    backend_type: BackendType,
}

#[derive(Deserialize)]
struct SearchPathParams {
    search_index: Option<String>,
}

impl Default for SearchPathParams {
    fn default() -> Self {
        SearchPathParams {
            search_index: Some("index-name".to_string()),
        }
    }
}

async fn search_place_index(
    data: web::Data<AppState>,
    request: web::Json<SearchRequest>,
    path: web::Path<SearchPathParams>,
) -> Result<HttpResponse> {
    let index_name = match path.search_index {
        Some(ref params) => params.clone(),
        None => "default-index".to_string(),
    };

    info!(
        msg = "Received search request",
        text = request.text.as_str(),
        max_results = request.max_results.unwrap_or(10),
        index_name = index_name
    );

    match data.backend.search(&request).await {
        Ok(results) => {
            let response = SearchResponse {
                summary: SearchSummary {
                    text: request.text.clone(),
                    max_results: request.max_results.unwrap_or(10),
                    result_count: results.len() as i32,
                },
                results: results
                    .into_iter()
                    .map(|place| ResultItem {
                        place,
                        relevance: 1.0,
                    })
                    .collect(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!(error = %e, msg = "Backend error");
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": format!("Backend search failed: {}", e)
            })))
        }
    }
}

async fn health(data: web::Data<AppState>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "backend": data.backend_type.as_str(),
        "version": "0.1.0"
    })))
}

async fn info(data: web::Data<AppState>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "Location Service Mock",
        "current_backend": data.backend_type.as_str(),
        "available_backends": ["predefined", "openstreetmap"],
        "endpoints": {
            "search": "POST /search",
            "health": "GET /health",
            "info": "GET /info"
        },
        "environment_variables": {
            "LOCATION_BACKEND": "Backend to use (predefined|openstreetmap)",
            "PLACES_CONFIG_PATH": "Path to YAML configuration file (predefined backend only)"
        }
    })))
}

fn create_backend(
    backend_type: &BackendType,
) -> Result<Arc<dyn LocationBackend + Send + Sync>, Box<dyn std::error::Error>> {
    match backend_type {
        BackendType::Predefined => {
            let config_path =
                env::var("PLACES_CONFIG_PATH").unwrap_or_else(|_| "data/places.yaml".to_string());

            info!(
                msg = "📂 Looking for places configuration",
                path = config_path.as_str()
            );

            let backend = match PredefinedBackend::from_file(&config_path) {
                Ok(backend) => backend,
                Err(e) => {
                    error!(error = %e, msg = "Failed to load places from config");
                    warn!(msg = "Falling back to default places");
                    PredefinedBackend::with_defaults()
                }
            };

            Ok(Arc::new(backend))
        }
        BackendType::OpenStreetMap => Ok(Arc::new(OpenStreetMapBackend::new())),
    }
}

async fn setup_logging(non_blocking: non_blocking::NonBlocking) {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(non_blocking)
        // .json()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .init();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up JSON structured logging
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    setup_logging(non_blocking).await;

    // Determine backend from environment variable
    let backend_type = match BackendType::from_env() {
        Ok(backend) => backend,
        Err(e) => {
            error!(error = %e, msg = "Configuration error");
            std::process::exit(1);
        }
    };

    let backend = match create_backend(&backend_type) {
        Ok(backend) => backend,
        Err(e) => {
            error!(error = %e, msg = "Failed to initialize backend");
            std::process::exit(1);
        }
    };

    let app_state = web::Data::new(AppState {
        backend,
        backend_type: backend_type.clone(),
    });

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    // Replace println! with info! for structured logging
    info!(msg = "Starting Location Service Mock");
    info!(msg = "Backend selected", backend = backend_type.as_str());
    info!(msg = "Server address", server = %format!("http://localhost:{}", port));
    info!(msg = "Available endpoints", endpoints = ?["POST /search", "GET /health", "GET /info"]);
    info!(msg = "Environment variables", location_backend = backend_type.as_str(), port = %port);
    if matches!(backend_type, BackendType::Predefined) {
        let config_path =
            env::var("PLACES_CONFIG_PATH").unwrap_or_else(|_| "data/places.yaml".to_string());
        info!(msg = "PLACES_CONFIG_PATH set", config_path = %config_path);
    }

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route(
                "/places/v0/indexes/{search_index}/search/text",
                web::post().to(search_place_index),
            )
            .route("/search", web::post().to(search_place_index))
            .route("/health", web::get().to(health))
            .route("/info", web::get().to(info))
    })
    .bind(&bind_address)?
    .run()
    .await
}

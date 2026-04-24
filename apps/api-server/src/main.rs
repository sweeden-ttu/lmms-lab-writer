use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use url::Url;
use chrono::Utc;
use tracing::{info, error};

#[derive(Clone)]
struct AppState {
    data_dir: PathBuf,
}

#[derive(Deserialize, Debug)]
struct IngestPayload {
    url: String,
    title: String,
    html_blocks: Vec<String>,
}

#[derive(Serialize)]
struct IngestResponse {
    status: String,
    message: String,
    saved_path: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Determine the base data directory: ~/.lmms-lab-writer/data
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let data_dir = home_dir.join(".lmms-lab-writer").join("data");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).await.expect("Failed to create data directory");
    }

    let state = AppState { data_dir };

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/ingest", post(handle_ingest))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3222));
    info!("Background API Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Sanitizes a string for use as a directory or file name
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

async fn handle_ingest(
    State(state): State<AppState>,
    Json(payload): Json<IngestPayload>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    info!("Received payload for URL: {}", payload.url);

    // Parse domain
    let parsed_url = match Url::parse(&payload.url) {
        Ok(u) => u,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid URL".to_string())),
    };

    let domain = parsed_url.host_str().unwrap_or("unknown_domain").to_string();
    let sanitized_domain = sanitize_name(&domain);
    let sanitized_title = sanitize_name(&payload.title);

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    
    // Create directory: ~/.lmms-lab-writer/data/<domain>/<title>
    let target_dir = state.data_dir.join(&sanitized_domain).join(&sanitized_title);
    
    if let Err(e) = fs::create_dir_all(&target_dir).await {
        error!("Failed to create directory {:?}: {}", target_dir, e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create directory".to_string()));
    }

    let filename = format!("{}.html", timestamp);
    let file_path = target_dir.join(&filename);

    // Combine HTML blocks
    let mut full_content = String::new();
    full_content.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    full_content.push_str(&format!("<title>{}</title>\n", payload.title));
    full_content.push_str(&format!("<meta name=\"source\" content=\"{}\">\n", payload.url));
    full_content.push_str("</head>\n<body>\n");
    
    for block in payload.html_blocks {
        full_content.push_str(&block);
        full_content.push_str("\n<hr>\n");
    }
    
    full_content.push_str("</body>\n</html>");

    // Write to file
    if let Err(e) = fs::write(&file_path, full_content).await {
        error!("Failed to write file {:?}: {}", file_path, e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file".to_string()));
    }

    info!("Successfully saved to {:?}", file_path);

    Ok(Json(IngestResponse {
        status: "success".to_string(),
        message: "HTML blocks ingested and saved".to_string(),
        saved_path: Some(file_path.to_string_lossy().to_string()),
    }))
}

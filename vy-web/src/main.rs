use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use vy_core::config::{VyConfig, default_config_path, load_or_create_config};

#[derive(Clone)]
struct AppState {
    config: Arc<VyConfig>,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    conversation_id: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    conversation_id: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load Vy configuration
    let config_path =
        default_config_path().ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;
    let config = load_or_create_config(&config_path)?;
    let state = AppState {
        config: Arc::new(config),
    };

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/chat", post(chat))
        .layer(CorsLayer::permissive()) // Configure CORS for web frontend
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Determine port from environment or default to 3001
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    log::info!("Starting Vy web server on {}", addr);
    println!("🚀 Vy web server starting on http://{}", addr);
    println!("📊 Health check: http://{}/health", addr);
    println!("💬 Chat API: http://{}/api/chat", addr);
    println!("\nPress Ctrl+C to stop");

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    log::info!("Received chat request: {}", request.message);

    // Build a new Vy instance for each request (stateless for now)
    let mut vy_core = match vy_core::builder::build_openai_vy(&state.config).await {
        Ok(core) => core,
        Err(e) => {
            log::error!("Failed to create Vy instance: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get response from Vy core
    let response = match vy_core.send_message(&request.message).await {
        Ok(reply) => reply,
        Err(e) => {
            log::error!("Error processing message: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Generate or use provided conversation ID
    let conversation_id = request
        .conversation_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    Ok(Json(ChatResponse {
        response,
        conversation_id,
    }))
}

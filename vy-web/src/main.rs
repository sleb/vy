use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use vy_core::config::{VyConfig, default_system_prompt};
use vy_core::vector_memory::VectorMemoryConfig;

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

#[derive(Parser)]
#[command(name = "vy-web")]
#[command(about = "Vy web API server")]
struct Args {
    /// Port to bind to
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

fn load_config_from_env() -> anyhow::Result<VyConfig> {
    // Helper function to get required env var
    let get_required_env = |key: &str| -> anyhow::Result<String> {
        std::env::var(key)
            .map_err(|_| anyhow::anyhow!("Missing required environment variable: {}", key))
    };

    // Helper function to get optional env var
    let get_optional_env =
        |key: &str| -> Option<String> { std::env::var(key).ok().filter(|s| !s.is_empty()) };

    let llm_api_key = get_required_env("VY_LLM_API_KEY")?;

    let config = VyConfig {
        llm_api_key: llm_api_key.clone(),
        google_api_key: get_required_env("VY_GOOGLE_API_KEY")?,
        google_search_engine_id: get_required_env("VY_GOOGLE_SEARCH_ENGINE_ID")?,
        llm_model_id: get_optional_env("VY_LLM_MODEL_ID")
            .unwrap_or_else(|| "gpt-4o-mini".to_string()),
        memory_model_id: get_optional_env("VY_MEMORY_MODEL_ID")
            .unwrap_or_else(|| "gpt-4o-mini".to_string()),
        memory_similarity_model_id: get_optional_env("VY_MEMORY_SIMILARITY_MODEL_ID")
            .unwrap_or_else(|| "gpt-4o-mini".to_string()),
        system_prompt: get_optional_env("VY_SYSTEM_PROMPT")
            .unwrap_or_else(|| default_system_prompt()),
        default_chat_mode: get_optional_env("VY_DEFAULT_CHAT_MODE")
            .unwrap_or_else(|| "cli".to_string()),
        vector_memory: VectorMemoryConfig {
            qdrant_url: get_optional_env("VY_QDRANT_URL")
                .unwrap_or_else(|| "http://localhost:6333".to_string()),
            qdrant_api_key: get_optional_env("VY_QDRANT_API_KEY"),
            collection_name: get_optional_env("VY_COLLECTION_NAME")
                .unwrap_or_else(|| "vy_memories".to_string()),
            openai_api_key: get_optional_env("VY_VECTOR_MEMORY_OPENAI_API_KEY")
                .unwrap_or_else(|| llm_api_key.clone()),
            embedding_model: get_optional_env("VY_EMBEDDING_MODEL")
                .unwrap_or_else(|| "text-embedding-3-small".to_string()),
        },
    };

    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Parse command line arguments for port and host only
    let args = Args::parse();

    // Override port and host from environment if available
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(args.port);
    let host = std::env::var("HOST").unwrap_or(args.host);

    // Load configuration from environment variables
    let config = load_config_from_env()?;
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

    // Use port and host from environment/args
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

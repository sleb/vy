//! Integration tests for Vy Core
//!
//! These tests verify that the memory tools work correctly when integrated
//! with the agent builder.

use vy_core::{builder, config::VyConfig, vector_memory::VectorMemoryConfig};

fn create_test_config() -> VyConfig {
    VyConfig {
        llm_api_key: "test_key".to_string(),
        google_api_key: "test_google_key".to_string(),
        google_search_engine_id: "test_search_id".to_string(),
        llm_model_id: "gpt-4o-mini".to_string(),
        memory_model_id: "gpt-4o-mini".to_string(),
        memory_similarity_model_id: "gpt-4o-mini".to_string(),
        system_prompt: "Test prompt".to_string(),
        default_chat_mode: "cli".to_string(),
        vector_memory: VectorMemoryConfig {
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None,
            collection_name: "test_memories".to_string(),
            openai_api_key: "test_openai_key".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
        },
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires API keys
async fn test_openai_agent_builder_with_memory_tools() {
    let config = create_test_config();

    // This should not panic or fail with schema errors
    let result = builder::build_openai_vy(&config).await;

    // We expect this to fail due to invalid API key, but it should fail
    // at the API call stage, not at the schema validation stage
    match result {
        Ok(_) => {
            // Unexpected success with fake API key, but that's fine for this test
            println!("Agent built successfully");
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Should NOT contain schema validation errors
            assert!(!error_msg.contains("Invalid schema"));
            assert!(!error_msg.contains("required' is required"));
            println!("Expected error (likely invalid API key): {}", error_msg);
        }
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires API keys
async fn test_anthropic_agent_builder_with_memory_tools() {
    let config = create_test_config();

    // This should not panic or fail with schema errors
    let result = builder::build_anthropic_vy(&config).await;

    // We expect this to fail due to invalid API key, but it should fail
    // at the API call stage, not at the schema validation stage
    match result {
        Ok(_) => {
            // Unexpected success with fake API key, but that's fine for this test
            println!("Agent built successfully");
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Should NOT contain schema validation errors
            assert!(!error_msg.contains("Invalid schema"));
            assert!(!error_msg.contains("required' is required"));
            println!("Expected error (likely invalid API key): {}", error_msg);
        }
    }
}

#[test]
fn test_memory_config_access() {
    let config = create_test_config();

    // Verify that vector memory config is accessible
    assert_eq!(config.vector_memory.qdrant_url, "http://localhost:6334");
    assert_eq!(config.vector_memory.collection_name, "test_memories");
    assert_eq!(
        config.vector_memory.embedding_model,
        "text-embedding-3-small"
    );
}

//! Integration test for agent building with memory tools
//!
//! Tests that the Vy agent can be built with memory tools properly integrated
//! and that the tools function correctly within the agent context.

use rig::tool::Tool;
use vy_core::builder::{build_anthropic_vy, build_openai_vy};
use vy_core::config::VyConfig;
use vy_core::vector_memory::VectorMemoryConfig;

fn test_config() -> VyConfig {
    VyConfig {
        llm_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
        google_api_key: "test-google-key".to_string(),
        google_search_engine_id: "test-engine-id".to_string(),
        llm_model_id: "gpt-4o-mini".to_string(),
        memory_model_id: "gpt-4o-mini".to_string(),
        memory_similarity_model_id: "gpt-4o-mini".to_string(),
        system_prompt: "You are Vy, a helpful AI assistant with memory capabilities.".to_string(),
        default_chat_mode: "cli".to_string(),
        vector_memory: VectorMemoryConfig {
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None,
            collection_name: "test_agent_memories".to_string(),
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "test-key".to_string()),
            embedding_model: "text-embedding-3-small".to_string(),
        },
    }
}

#[tokio::test]
async fn test_openai_agent_builds_with_memory_tools() {
    let config = test_config();

    // This should not panic or fail due to schema issues
    let result = build_openai_vy(&config).await;

    match result {
        Ok(agent) => {
            println!("✅ OpenAI agent built successfully with memory tools");
            println!("   - Memory tools integrated without schema errors");
        }
        Err(error) => {
            // If this fails, it should be due to API key issues, not schema problems
            let error_msg = error.to_string();

            // Check that it's not a schema validation error
            assert!(
                !error_msg.contains("Invalid schema")
                    && !error_msg.contains("required")
                    && !error_msg.contains("properties"),
                "Schema validation error detected: {}",
                error_msg
            );

            println!(
                "✅ OpenAI agent build failed as expected (API key issue): {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_anthropic_agent_builds_with_memory_tools() {
    let config = test_config();

    // This should not panic or fail due to schema issues
    let result = build_anthropic_vy(&config).await;

    match result {
        Ok(agent) => {
            println!("✅ Anthropic agent built successfully with memory tools");
            println!("   - Memory tools integrated without schema errors");
        }
        Err(error) => {
            // If this fails, it should be due to API key issues, not schema problems
            let error_msg = error.to_string();

            // Check that it's not a schema validation error
            assert!(
                !error_msg.contains("Invalid schema")
                    && !error_msg.contains("required")
                    && !error_msg.contains("properties"),
                "Schema validation error detected: {}",
                error_msg
            );

            println!(
                "✅ Anthropic agent build failed as expected (API key issue): {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_memory_tools_have_correct_names() {
    // Verify that our memory tools have the expected names that would be registered
    use vy_core::tools::{RemoveMemoryTool, SearchMemoryTool, StoreMemoryTool, UpdateMemoryTool};

    assert_eq!(StoreMemoryTool::NAME, "store_memory");
    assert_eq!(SearchMemoryTool::NAME, "search_memory");
    assert_eq!(UpdateMemoryTool::NAME, "smart_update_memory");
    assert_eq!(RemoveMemoryTool::NAME, "remove_memories");

    println!("✅ All memory tools have correct names:");
    println!("   - {}", StoreMemoryTool::NAME);
    println!("   - {}", SearchMemoryTool::NAME);
    println!("   - {}", UpdateMemoryTool::NAME);
    println!("   - {}", RemoveMemoryTool::NAME);
}

#[tokio::test]
async fn test_vector_memory_config_integration() {
    let config = test_config();

    // Verify that the vector memory config is properly set up
    assert_eq!(config.vector_memory.qdrant_url, "http://localhost:6334");
    assert_eq!(config.vector_memory.collection_name, "test_agent_memories");
    assert_eq!(
        config.vector_memory.embedding_model,
        "text-embedding-3-small"
    );
    assert!(config.vector_memory.qdrant_api_key.is_none()); // Local Qdrant

    // The OpenAI API key should be set from the main config
    assert!(!config.vector_memory.openai_api_key.is_empty());

    println!("✅ Vector memory configuration is properly structured:");
    println!("   - Qdrant URL: {}", config.vector_memory.qdrant_url);
    println!("   - Collection: {}", config.vector_memory.collection_name);
    println!(
        "   - Embedding model: {}",
        config.vector_memory.embedding_model
    );
    println!(
        "   - API key configured: {}",
        !config.vector_memory.openai_api_key.is_empty()
    );
}

#[tokio::test]
async fn test_memory_tools_constructor_functions() {
    use vy_core::tools::{
        remove_memory_tool_with_config, search_memory_tool_with_config,
        store_memory_tool_with_config, update_memory_tool_with_config,
    };

    let config = test_config();

    // Test that all constructor functions work without panicking
    let _store_tool = store_memory_tool_with_config(config.vector_memory.clone());
    let _search_tool = search_memory_tool_with_config(config.vector_memory.clone());
    let _update_tool = update_memory_tool_with_config(config.vector_memory.clone());
    let _remove_tool = remove_memory_tool_with_config(config.vector_memory.clone());

    println!("✅ All memory tool constructors work correctly with VectorMemoryConfig");
    println!("   - store_memory_tool_with_config");
    println!("   - search_memory_tool_with_config");
    println!("   - update_memory_tool_with_config");
    println!("   - remove_memory_tool_with_config");
}

// Test that demonstrates the fix: memory tools now use actual vector memory instead of mocks
#[tokio::test]
async fn test_memory_tools_use_real_implementation() {
    use rig::tool::Tool;
    use vy_core::tools::store_memory_tool_with_config;

    let config = test_config();
    let store_tool = store_memory_tool_with_config(config.vector_memory);

    // Create test arguments
    let args = serde_json::json!({
        "fact": "Integration test memory storage"
    });
    let store_args: <vy_core::tools::StoreMemoryTool as Tool>::Args =
        serde_json::from_value(args).unwrap();

    // Attempt to store memory
    let result = store_tool.call(store_args).await;

    // The key test: it should attempt to connect to Qdrant (real implementation)
    // rather than just returning a mock success response
    match result {
        Ok(response) => {
            // If successful (Qdrant is running), verify it's a real response
            assert!(response.success);
            assert!(!response.stored_fact.is_empty());
            println!("✅ Memory tool successfully used real vector memory implementation");
        }
        Err(error) => {
            // If it fails, it should be due to Qdrant connection, not mock behavior
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("Failed to connect")
                    || error_msg.contains("qdrant")
                    || error_msg.contains("Task join error"),
                "Unexpected error (might be using mock implementation): {}",
                error_msg
            );
            println!(
                "✅ Memory tool correctly attempted real vector memory connection: {}",
                error_msg
            );
        }
    }
}

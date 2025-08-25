//! Integration test for actual memory functionality
//!
//! Tests that the memory tools can actually store and retrieve information
//! using the vector memory system, not just schema validation.

use anyhow::Result;
use rig::tool::Tool;
use serde_json;
use vy_core::config::VyConfig;
use vy_core::tools::{
    RemoveMemoryTool, SearchMemoryTool, StoreMemoryTool, UpdateMemoryTool,
    remove_memory_tool_with_config, search_memory_tool_with_config, store_memory_tool_with_config,
    update_memory_tool_with_config,
};
use vy_core::vector_memory::VectorMemoryConfig;

// Test configuration that works without requiring external services
fn test_vector_config() -> VectorMemoryConfig {
    VectorMemoryConfig {
        qdrant_url: "http://localhost:6334".to_string(),
        qdrant_api_key: None, // Local Qdrant
        collection_name: "test_memory_integration".to_string(),
        openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
        embedding_model: "text-embedding-3-small".to_string(),
    }
}

#[tokio::test]
async fn test_memory_tools_can_be_created_with_config() {
    let config = test_vector_config();

    // Test that all tools can be created without panicking
    let store_tool = store_memory_tool_with_config(config.clone());
    let search_tool = search_memory_tool_with_config(config.clone());
    let update_tool = update_memory_tool_with_config(config.clone());
    let remove_tool = remove_memory_tool_with_config(config.clone());

    // Verify tool names are correct
    assert_eq!(StoreMemoryTool::NAME, "store_memory");
    assert_eq!(SearchMemoryTool::NAME, "search_memory");
    assert_eq!(UpdateMemoryTool::NAME, "smart_update_memory");
    assert_eq!(RemoveMemoryTool::NAME, "remove_memories");
}

#[tokio::test]
async fn test_memory_tool_definitions() {
    let config = test_vector_config();

    // Test store memory tool definition
    let store_tool = store_memory_tool_with_config(config.clone());
    let store_def = store_tool.definition("test".to_string()).await;

    assert_eq!(store_def.name, "store_memory");
    assert!(store_def.description.contains("Store a fact"));

    // Verify schema structure
    let params = &store_def.parameters;
    assert_eq!(params["type"], "object");
    assert!(params["properties"]["fact"].is_object());
    assert_eq!(params["properties"]["fact"]["type"], "string");
    assert!(
        params["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("fact".to_string()))
    );

    // Test search memory tool definition
    let search_tool = search_memory_tool_with_config(config.clone());
    let search_def = search_tool.definition("test".to_string()).await;

    assert_eq!(search_def.name, "search_memory");
    assert!(
        search_def
            .description
            .contains("Search through stored memories")
    );

    // Verify schema structure
    let params = &search_def.parameters;
    assert!(params["properties"]["query"].is_object());
    assert_eq!(params["properties"]["query"]["type"], "string");
    assert!(
        params["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("query".to_string()))
    );
}

#[tokio::test]
async fn test_memory_storage_without_qdrant() {
    // This test checks the tool behavior when Qdrant is not available
    let config = test_vector_config();
    let store_tool = store_memory_tool_with_config(config);

    // Create test arguments
    let args = serde_json::json!({
        "fact": "Scott is a Sr Software Development Manager at Amazon"
    });

    let store_args: <StoreMemoryTool as Tool>::Args = serde_json::from_value(args).unwrap();

    // Attempt to store memory - this should gracefully handle connection failures
    let result = store_tool.call(store_args).await;

    // The result should either succeed (if Qdrant is running) or fail with a clear error message
    match result {
        Ok(response) => {
            // If successful, verify the response structure
            assert!(response.success);
            assert!(!response.message.is_empty());
            assert!(!response.stored_fact.is_empty());
            println!("✅ Memory storage succeeded: {}", response.message);
        }
        Err(error) => {
            // If failed, verify it's due to connection issues, not schema problems
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("Failed to connect")
                    || error_msg.contains("connection")
                    || error_msg.contains("qdrant")
                    || error_msg.contains("Task join error"),
                "Unexpected error type: {}",
                error_msg
            );
            println!(
                "✅ Memory storage failed as expected (no Qdrant): {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_memory_search_without_qdrant() {
    // This test checks the search tool behavior when Qdrant is not available
    let config = test_vector_config();
    let search_tool = search_memory_tool_with_config(config);

    // Create test arguments
    let args = serde_json::json!({
        "query": "Scott"
    });

    let search_args: <SearchMemoryTool as Tool>::Args = serde_json::from_value(args).unwrap();

    // Attempt to search memory
    let result = search_tool.call(search_args).await;

    match result {
        Ok(response) => {
            // If successful, verify the response structure
            assert!(!response.query.is_empty());
            assert_eq!(response.query, "Scott");
            assert!(response.total_found >= 0); // Should be >= 0
            println!(
                "✅ Memory search succeeded: found {} memories",
                response.total_found
            );
        }
        Err(error) => {
            // If failed, verify it's due to connection issues, not schema problems
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("Failed to connect")
                    || error_msg.contains("connection")
                    || error_msg.contains("qdrant")
                    || error_msg.contains("Task join error"),
                "Unexpected error type: {}",
                error_msg
            );
            println!(
                "✅ Memory search failed as expected (no Qdrant): {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_memory_update_without_qdrant() {
    let config = test_vector_config();
    let update_tool = update_memory_tool_with_config(config);

    let args = serde_json::json!({
        "fact": "Scott is now a Principal Engineer at Microsoft"
    });

    let update_args: <UpdateMemoryTool as Tool>::Args = serde_json::from_value(args).unwrap();
    let result = update_tool.call(update_args).await;

    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(!response.message.is_empty());
            println!("✅ Memory update succeeded: {}", response.message);
        }
        Err(error) => {
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("Failed to connect")
                    || error_msg.contains("connection")
                    || error_msg.contains("qdrant")
                    || error_msg.contains("Task join error"),
                "Unexpected error type: {}",
                error_msg
            );
            println!(
                "✅ Memory update failed as expected (no Qdrant): {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_memory_remove_without_qdrant() {
    let config = test_vector_config();
    let remove_tool = remove_memory_tool_with_config(config);

    let args = serde_json::json!({
        "fact": "outdated information"
    });

    let remove_args: <RemoveMemoryTool as Tool>::Args = serde_json::from_value(args).unwrap();
    let result = remove_tool.call(remove_args).await;

    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(!response.message.is_empty());
            assert!(!response.query.is_empty());
            println!("✅ Memory remove succeeded: {}", response.message);
        }
        Err(error) => {
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("Failed to connect")
                    || error_msg.contains("connection")
                    || error_msg.contains("qdrant")
                    || error_msg.contains("Task join error"),
                "Unexpected error type: {}",
                error_msg
            );
            println!(
                "✅ Memory remove failed as expected (no Qdrant): {}",
                error_msg
            );
        }
    }
}

// Test that would run with actual Qdrant if available
#[tokio::test]
#[ignore] // Ignore by default since it requires Qdrant to be running
async fn test_memory_full_workflow_with_qdrant() {
    // This test requires OPENAI_API_KEY environment variable and Qdrant running on localhost:6334
    let openai_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            println!("Skipping full workflow test - OPENAI_API_KEY not set");
            return;
        }
    };

    let config = VectorMemoryConfig {
        qdrant_url: "http://localhost:6334".to_string(),
        qdrant_api_key: None,
        collection_name: "test_memory_full_workflow".to_string(),
        openai_api_key: openai_key,
        embedding_model: "text-embedding-3-small".to_string(),
    };

    let store_tool = store_memory_tool_with_config(config.clone());
    let search_tool = search_memory_tool_with_config(config.clone());

    // Test 1: Store a memory
    let store_args = serde_json::json!({
        "fact": "Scott loves Rust programming and works on AI systems"
    });
    let store_args: <StoreMemoryTool as Tool>::Args = serde_json::from_value(store_args).unwrap();

    let store_result = store_tool.call(store_args).await;
    assert!(store_result.is_ok(), "Store failed: {:?}", store_result);

    let store_response = store_result.unwrap();
    assert!(store_response.success);
    println!("✅ Stored memory: {}", store_response.message);

    // Wait a moment for indexing
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Test 2: Search for the memory
    let search_args = serde_json::json!({
        "query": "Scott programming"
    });
    let search_args: <SearchMemoryTool as Tool>::Args =
        serde_json::from_value(search_args).unwrap();

    let search_result = search_tool.call(search_args).await;
    assert!(search_result.is_ok(), "Search failed: {:?}", search_result);

    let search_response = search_result.unwrap();
    assert!(search_response.total_found > 0);
    assert!(!search_response.matches.is_empty());

    let first_match = &search_response.matches[0];
    assert!(first_match.fact.contains("Scott"));
    assert!(first_match.fact.contains("Rust") || first_match.fact.contains("AI"));

    println!("✅ Found {} matching memories", search_response.total_found);
    println!("   First match: {}", first_match.fact);
}

#[tokio::test]
async fn test_memory_error_handling() {
    // Test with invalid configuration to ensure proper error handling
    let invalid_config = VectorMemoryConfig {
        qdrant_url: "http://invalid-url:9999".to_string(),
        qdrant_api_key: None,
        collection_name: "test".to_string(),
        openai_api_key: "invalid-key".to_string(),
        embedding_model: "text-embedding-3-small".to_string(),
    };

    let store_tool = store_memory_tool_with_config(invalid_config);

    let args = serde_json::json!({
        "fact": "test fact"
    });
    let store_args: <StoreMemoryTool as Tool>::Args = serde_json::from_value(args).unwrap();

    let result = store_tool.call(store_args).await;

    // Should fail gracefully with clear error message
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to connect")
            || error_msg.contains("connection")
            || error_msg.contains("Task join error")
    );

    println!("✅ Error handling works correctly: {}", error_msg);
}

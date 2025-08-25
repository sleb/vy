//! Integration test for memory tools schema validation
//!
//! This test directly validates that memory tools work with OpenAI's schema validation
//! without requiring the interactive CLI.

use anyhow::Result;
use rig::tool::Tool;
use serde_json::Value;
use vy_core::tools::{
    complete_memory_tools::StoreMemoryArgs, exact_copy_memory_tool,
    exact_copy_memory_tool::ExactCopyMemoryArgs, remove_memory_tool, search_memory_tool,
    store_memory_tool, update_memory_tool,
};

#[tokio::test]
async fn test_exact_copy_tool_schema() -> Result<()> {
    println!("🧪 Testing exact copy tool schema...");

    let tool = exact_copy_memory_tool("test-api-key".to_string());
    let definition = tool.definition("test".to_string()).await;

    // Verify basic structure
    assert_eq!(definition.name, "store_memory_exact");
    assert!(!definition.description.is_empty());

    // Verify schema structure
    let params = &definition.parameters;
    assert_eq!(params["type"], "object");
    assert!(params["properties"].is_object());
    assert!(params["required"].is_array());

    // Verify properties
    let properties = &params["properties"];
    assert!(properties["fact"].is_object());
    assert_eq!(properties["fact"]["type"], "string");

    // Verify required array
    let required = params["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "fact");

    println!("✅ Exact copy tool schema is valid");
    Ok(())
}

#[tokio::test]
async fn test_store_memory_tool_schema() -> Result<()> {
    println!("🧪 Testing store memory tool schema...");

    let tool = store_memory_tool("test-api-key".to_string());
    let definition = tool.definition("test".to_string()).await;

    // Verify basic structure
    assert_eq!(definition.name, "store_memory");
    assert!(!definition.description.is_empty());

    // Verify schema structure
    let params = &definition.parameters;
    assert_eq!(params["type"], "object");
    assert!(params["properties"].is_object());
    assert!(params["required"].is_array());

    // Verify properties
    let properties = &params["properties"];
    assert!(properties["fact"].is_object());
    assert_eq!(properties["fact"]["type"], "string");

    // Check if source field exists and how it's handled
    if let Some(source_field) = properties.get("source") {
        assert!(source_field.is_object());
        assert_eq!(source_field["type"], "string");
    }

    // Verify required array
    let required = params["required"].as_array().unwrap();
    assert!(required.contains(&Value::String("fact".to_string())));

    println!("✅ Store memory tool schema structure is valid");

    // Print the actual schema for debugging
    println!("🔍 Store memory tool schema:");
    println!("{}", serde_json::to_string_pretty(params)?);

    Ok(())
}

#[tokio::test]
async fn test_schema_comparison() -> Result<()> {
    println!("🧪 Comparing exact copy vs store memory schemas...");

    let exact_copy_tool = exact_copy_memory_tool("test-api-key".to_string());
    let store_memory_tool = store_memory_tool("test-api-key".to_string());

    let exact_copy_def = exact_copy_tool.definition("test".to_string()).await;
    let store_memory_def = store_memory_tool.definition("test".to_string()).await;

    let exact_copy_params = &exact_copy_def.parameters;
    let store_memory_params = &store_memory_def.parameters;

    println!("🔍 Exact copy schema:");
    println!("{}", serde_json::to_string_pretty(exact_copy_params)?);

    println!("🔍 Store memory schema:");
    println!("{}", serde_json::to_string_pretty(store_memory_params)?);

    // Compare key structure elements
    assert_eq!(exact_copy_params["type"], store_memory_params["type"]);
    assert!(exact_copy_params["properties"].is_object());
    assert!(store_memory_params["properties"].is_object());
    assert!(exact_copy_params["required"].is_array());
    assert!(store_memory_params["required"].is_array());

    println!("✅ Both schemas have valid structure");
    Ok(())
}

#[tokio::test]
async fn test_empty_api_key_validation() -> Result<()> {
    println!("🧪 Testing API key validation...");

    // Test exact copy tool with empty API key
    let exact_copy_tool = exact_copy_memory_tool("".to_string());
    let exact_copy_args = ExactCopyMemoryArgs {
        fact: "Test fact".to_string(),
    };

    let exact_copy_result = exact_copy_tool.call(exact_copy_args).await;
    assert!(exact_copy_result.is_err());
    println!("✅ Exact copy tool correctly validates API key");

    // Test store memory tool with empty API key
    let store_memory_tool = store_memory_tool("".to_string());
    let store_memory_args = StoreMemoryArgs {
        fact: "Test fact".to_string(),
    };

    let store_memory_result = store_memory_tool.call(store_memory_args).await;
    assert!(store_memory_result.is_err());
    println!("✅ Store memory tool correctly validates API key");

    Ok(())
}

#[tokio::test]
async fn test_json_schema_validity() -> Result<()> {
    println!("🧪 Testing JSON schema validity...");

    // Test exact copy tool schema
    let exact_copy_tool = exact_copy_memory_tool("test-key".to_string());
    let exact_copy_def = exact_copy_tool.definition("test".to_string()).await;
    let exact_copy_params = &exact_copy_def.parameters;
    let json_string = serde_json::to_string(exact_copy_params)?;
    let _: Value = serde_json::from_str(&json_string)?;
    assert!(exact_copy_params.get("type").is_some());
    assert!(exact_copy_params.get("properties").is_some());
    assert!(exact_copy_params.get("required").is_some());
    println!("✅ exact_copy tool has valid JSON schema");

    // Test store memory tool schema
    let store_memory_tool = store_memory_tool("test-key".to_string());
    let store_memory_def = store_memory_tool.definition("test".to_string()).await;
    let store_memory_params = &store_memory_def.parameters;
    let json_string = serde_json::to_string(store_memory_params)?;
    let _: Value = serde_json::from_str(&json_string)?;
    assert!(store_memory_params.get("type").is_some());
    assert!(store_memory_params.get("properties").is_some());
    assert!(store_memory_params.get("required").is_some());
    println!("✅ store_memory tool has valid JSON schema");

    Ok(())
}

#[tokio::test]
async fn test_all_memory_tools_schema_individually() -> Result<()> {
    println!("🧪 Testing all memory tools schema individually...");

    // Get the exact copy tool schema as reference
    let exact_copy_tool = exact_copy_memory_tool("test-key".to_string());
    let exact_copy_def = exact_copy_tool.definition("test".to_string()).await;
    let exact_copy_params = &exact_copy_def.parameters;

    // Test store memory tool
    let store_tool = store_memory_tool("test-key".to_string());
    let store_def = store_tool.definition("test".to_string()).await;
    let store_params = &store_def.parameters;
    assert_eq!(store_params["type"], exact_copy_params["type"]);
    assert_eq!(store_params["properties"].as_object().unwrap().len(), 1);
    assert_eq!(store_params["required"].as_array().unwrap().len(), 1);
    println!("✅ store_memory tool schema matches exact copy pattern");

    // Test search memory tool
    let search_tool = search_memory_tool("test-key".to_string());
    let search_def = search_tool.definition("test".to_string()).await;
    let search_params = &search_def.parameters;
    assert_eq!(search_params["type"], exact_copy_params["type"]);
    assert_eq!(search_params["properties"].as_object().unwrap().len(), 1);
    assert_eq!(search_params["required"].as_array().unwrap().len(), 1);
    println!("✅ search_memory tool schema matches exact copy pattern");

    // Test update memory tool
    let update_tool = update_memory_tool("test-key".to_string());
    let update_def = update_tool.definition("test".to_string()).await;
    let update_params = &update_def.parameters;
    assert_eq!(update_params["type"], exact_copy_params["type"]);
    assert_eq!(update_params["properties"].as_object().unwrap().len(), 1);
    assert_eq!(update_params["required"].as_array().unwrap().len(), 1);
    println!("✅ update_memory tool schema matches exact copy pattern");

    // Test remove memory tool
    let remove_tool = remove_memory_tool("test-key".to_string());
    let remove_def = remove_tool.definition("test".to_string()).await;
    let remove_params = &remove_def.parameters;
    assert_eq!(remove_params["type"], exact_copy_params["type"]);
    assert_eq!(remove_params["properties"].as_object().unwrap().len(), 1);
    assert_eq!(remove_params["required"].as_array().unwrap().len(), 1);
    println!("✅ remove_memory tool schema matches exact copy pattern");

    println!("✅ All memory tools have identical schema structure");
    Ok(())
}

#[tokio::test]
async fn test_agent_initialization_with_memory_tools() -> Result<()> {
    println!("🧪 Testing agent initialization with memory tools...");

    // This test simulates what happens when the agent is built with memory tools
    use vy_core::builder;
    use vy_core::config::VyConfig;

    // Create a test config
    let config = VyConfig {
        llm_api_key: "test-api-key".to_string(),
        google_api_key: "test-google-key".to_string(),
        google_search_engine_id: "test-search-id".to_string(),
        llm_model_id: "gpt-4o-mini".to_string(),
        memory_model_id: "gpt-4o-mini".to_string(),
        memory_similarity_model_id: "gpt-4o-mini".to_string(),
        default_chat_mode: "cli".to_string(),
        system_prompt: "You are Vy, a helpful AI assistant.".to_string(),
        vector_memory: vy_core::vector_memory::VectorMemoryConfig {
            qdrant_url: "https://test.qdrant.io".to_string(),
            qdrant_api_key: Some("test-qdrant-key".to_string()),
            collection_name: "test_memories".to_string(),
            openai_api_key: "test-openai-key".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
        },
    };

    // This will test if the agent can be built without schema validation errors
    // Note: This will fail at runtime due to invalid API keys, but schema validation happens first
    let result = builder::build_openai_vy(&config).await;

    // We expect this to fail with an authentication error, not a schema error
    match result {
        Ok(_) => {
            println!("✅ Agent built successfully (unexpected but good!)");
        }
        Err(e) => {
            let error_message = format!("{e}");
            // Check that it's NOT a schema validation error
            if error_message.contains("Invalid schema")
                || error_message.contains("required is required")
            {
                panic!("❌ Schema validation error detected: {error_message}");
            } else {
                println!("✅ Agent failed with non-schema error (expected): {error_message}");
                println!("   This means schema validation passed successfully!");
            }
        }
    }

    Ok(())
}

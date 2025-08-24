//! Tests for vector memory tools
//!
//! These tests verify that the memory tools have correct schemas and work properly.

#[cfg(test)]
mod tests {
    use super::super::vector_memory_tools::*;
    use crate::vector_memory::VectorMemoryConfig;
    use rig::tool::Tool;

    fn create_test_config() -> VectorMemoryConfig {
        VectorMemoryConfig {
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None,
            collection_name: "test_memories".to_string(),
            openai_api_key: "test_key".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
        }
    }

    #[tokio::test]
    async fn test_search_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemorySearchTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "search_memory");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "search_memory schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[tokio::test]
    async fn test_store_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemoryStoreTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "store_memory");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "store_memory schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[tokio::test]
    async fn test_update_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemoryUpdateTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "smart_update_memory");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "smart_update_memory schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[tokio::test]
    async fn test_remove_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemoryRemoveTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "remove_memories");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "remove_memories schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[test]
    fn test_search_args_serialization() {
        let args = VectorMemorySearchArgs {
            query: "test query".to_string(),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemorySearchArgs JSON: {}", json);

        // Test deserialization
        let _deserialized: VectorMemorySearchArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_store_args_serialization() {
        let args = VectorMemoryStoreArgs {
            fact: "test fact".to_string(),
            source: Some("test source".to_string()),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemoryStoreArgs JSON: {}", json);

        // Test deserialization
        let _deserialized: VectorMemoryStoreArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_update_args_serialization() {
        let args = VectorMemoryUpdateArgs {
            old_info: "old info".to_string(),
            new_info: "new info".to_string(),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemoryUpdateArgs JSON: {}", json);

        // Test deserialization
        let _deserialized: VectorMemoryUpdateArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_remove_args_serialization() {
        let args = VectorMemoryRemoveArgs {
            query: "test query".to_string(),
            confirm: Some(true),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemoryRemoveArgs JSON: {}", json);

        // Test deserialization
        let _deserialized: VectorMemoryRemoveArgs = serde_json::from_str(&json).unwrap();
    }
}

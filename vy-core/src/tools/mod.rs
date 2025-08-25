//! Tools for the Vy AI assistant
//!
//! This module provides various tools that can be used by the Vy agent,
//! including Google search, nutrition analysis, and vector memory operations.

pub mod complete_memory_tools;
pub mod exact_copy_memory_tool;
pub mod google_search;
pub mod nutrition_analysis;
pub mod test_memory_tool;
pub mod vector_memory_tools;

pub use complete_memory_tools::{
    RemoveMemoryTool, SearchMemoryTool, StoreMemoryTool, UpdateMemoryTool, remove_memory_tool,
    remove_memory_tool_with_config, search_memory_tool, search_memory_tool_with_config,
    store_memory_tool, store_memory_tool_with_config, update_memory_tool,
    update_memory_tool_with_config,
};
pub use exact_copy_memory_tool::{ExactCopyMemoryTool, exact_copy_memory_tool};
pub use google_search::GoogleSearchTool;
pub use nutrition_analysis::NutritionAnalysisTool;
pub use test_memory_tool::{TestMemoryTool, test_memory_tool};
pub use vector_memory_tools::{
    VectorMemoryRemoveTool, VectorMemorySearchTool, VectorMemoryStoreTool, VectorMemoryUpdateTool,
    vector_memory_remove_tool, vector_memory_search_tool, vector_memory_store_tool,
    vector_memory_update_tool,
};

/// Create a Google search tool instance
pub fn google_search(api_key: String, search_engine_id: String) -> GoogleSearchTool {
    GoogleSearchTool::new(api_key, search_engine_id)
}

/// Create a nutrition analysis tool instance
pub fn nutrition_analysis_tool(api_key: String) -> NutritionAnalysisTool {
    NutritionAnalysisTool::new(api_key)
}

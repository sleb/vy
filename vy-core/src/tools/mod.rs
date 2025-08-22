//! Tools for the Vy AI assistant
//!
//! This module provides various tools that can be used by the Vy agent,
//! including Google search, memory management, and nutrition analysis.

pub mod google_search;
pub mod memory;
pub mod memory_remove;
pub mod memory_store;
pub mod memory_update;
pub mod nutrition_analysis;
pub mod smart_memory_update;

pub use google_search::GoogleSearchTool;
pub use memory::MemoryTool;
pub use memory_remove::MemoryRemoveTool;
pub use memory_store::MemoryStoreTool;
pub use memory_update::MemoryUpdateTool;
pub use nutrition_analysis::NutritionAnalysisTool;
pub use smart_memory_update::SmartMemoryUpdateTool;

/// Create a Google search tool instance
pub fn google_search(api_key: String, search_engine_id: String) -> GoogleSearchTool {
    GoogleSearchTool::new(api_key, search_engine_id)
}

/// Create a simple memory search tool instance
pub fn simple_memory_tool() -> MemoryTool {
    MemoryTool
}

/// Create a memory store tool instance
pub fn memory_store_tool() -> MemoryStoreTool {
    MemoryStoreTool
}

/// Create a memory remove tool instance
pub fn memory_remove_tool() -> MemoryRemoveTool {
    MemoryRemoveTool
}

/// Create a memory update tool instance
pub fn memory_update_tool() -> MemoryUpdateTool {
    MemoryUpdateTool
}

/// Create a smart memory update tool instance
pub fn smart_memory_update_tool() -> SmartMemoryUpdateTool {
    SmartMemoryUpdateTool
}

/// Create a nutrition analysis tool instance
pub fn nutrition_analysis_tool(api_key: String) -> NutritionAnalysisTool {
    NutritionAnalysisTool::new(api_key)
}

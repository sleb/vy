//! Tools for the Vy AI assistant
//!
//! This module provides various tools that can be used by the Vy agent,
//! including Google search and nutrition analysis.

pub mod google_search;
pub mod nutrition_analysis;
// TODO: Re-enable vector memory tools after fixing Sync issues
// pub mod vector_memory_tools;

pub use google_search::GoogleSearchTool;
pub use nutrition_analysis::NutritionAnalysisTool;

/// Create a Google search tool instance
pub fn google_search(api_key: String, search_engine_id: String) -> GoogleSearchTool {
    GoogleSearchTool::new(api_key, search_engine_id)
}

/// Create a nutrition analysis tool instance
pub fn nutrition_analysis_tool(api_key: String) -> NutritionAnalysisTool {
    NutritionAnalysisTool::new(api_key)
}

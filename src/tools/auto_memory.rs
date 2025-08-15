use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AutoMemoryError(String);

impl std::fmt::Display for AutoMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AutoMemoryError {}

#[derive(Debug, Deserialize)]
pub struct AutoMemoryArgs {
    pub user_message: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AutoMemoryResponse {
    pub should_store: bool,
    pub extracted_facts: Vec<String>,
    pub memory_type: String,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPriority {
    High,   // Always store (identity, employment, relationships)
    Medium, // Context dependent (preferences, opinions)
    Low,    // Usually ignore unless explicit (temporary info)
}

pub struct AutoMemoryTool;

impl Default for AutoMemoryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoMemoryTool {
    pub fn new() -> Self {
        Self
    }

    /// Analyze if a message contains memory-worthy information
    /// Since we now use LLM-based extraction, we assume all non-trivial messages are worth analyzing
    pub fn should_analyze_message(message: &str) -> bool {
        // Skip very short messages or pure punctuation/whitespace
        let cleaned = message.trim();
        cleaned.len() > 3
            && !cleaned
                .chars()
                .all(|c| c.is_ascii_punctuation() || c.is_whitespace())
    }

    /// Determine the priority of potential memory information
    /// With LLM-based extraction, we default to medium priority and let the LLM decide
    fn classify_memory_priority(_message: &str) -> MemoryPriority {
        // Since we're using LLM-based analysis, we assume medium priority
        // and let the LLM determine what's actually important
        MemoryPriority::Medium
    }

    /// Extract memory type from message content
    /// With LLM-based extraction, we default to general and let the LLM categorize
    fn determine_memory_type(_message: &str) -> String {
        "general".to_string()
    }

    /// Calculate confidence score based on message clarity and specificity
    /// With LLM-based extraction, we use a fixed confidence and let the LLM decide
    fn calculate_confidence(_message: &str, priority: &MemoryPriority) -> f32 {
        match priority {
            MemoryPriority::High => 0.9,
            MemoryPriority::Medium => 0.7,
            MemoryPriority::Low => 0.3,
        }
    }

    async fn analyze_for_memories(
        &self,
        message: &str,
        _context: Option<&str>,
    ) -> Result<AutoMemoryResponse, AutoMemoryError> {
        // Check if message should be analyzed
        if !Self::should_analyze_message(message) {
            return Ok(AutoMemoryResponse {
                should_store: false,
                extracted_facts: vec![],
                memory_type: "none".to_string(),
                confidence: 0.0,
                reasoning: "Message doesn't contain obvious memory-worthy information".to_string(),
            });
        }

        // Classify priority and extract basic facts
        let priority = Self::classify_memory_priority(message);
        let memory_type = Self::determine_memory_type(message);
        let confidence = Self::calculate_confidence(message, &priority);

        // No fact extraction needed since we now use LLM-based analysis in the main chat flow
        let extracted_facts = Vec::new();

        let should_store = match priority {
            MemoryPriority::High => confidence >= 0.6,
            MemoryPriority::Medium => confidence >= 0.7,
            MemoryPriority::Low => false, // Require explicit request
        };

        let reasoning = format!(
            "Priority: {:?}, Type: {}, Confidence: {:.2}. {}",
            priority,
            memory_type,
            confidence,
            if should_store {
                "Automatic storage recommended"
            } else {
                "Manual storage required or information not suitable for automatic collection"
            }
        );

        Ok(AutoMemoryResponse {
            should_store,
            extracted_facts,
            memory_type,
            confidence,
            reasoning,
        })
    }
}

impl Tool for AutoMemoryTool {
    const NAME: &'static str = "analyze_memory_potential";

    type Error = AutoMemoryError;
    type Args = AutoMemoryArgs;
    type Output = AutoMemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Automatically analyze user messages to detect memory-worthy information. This tool helps decide when to store information without explicit user requests, focusing on important personal details like identity, employment, relationships, and preferences.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "user_message": {
                        "type": "string",
                        "description": "The user's message to analyze for memory-worthy content"
                    },
                    "context": {
                        "type": ["string", "null"],
                        "description": "Optional conversation context to help with analysis"
                    }
                },
                "required": ["user_message"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.analyze_for_memories(&args.user_message, args.context.as_deref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_analyze_message() {
        // With simplified logic, most messages are considered worth analyzing
        assert!(AutoMemoryTool::should_analyze_message(
            "Hi, my name is John"
        ));
        assert!(AutoMemoryTool::should_analyze_message("I work at Google"));
        assert!(AutoMemoryTool::should_analyze_message("I love coffee"));
        assert!(AutoMemoryTool::should_analyze_message(
            "What's the weather like?"
        ));
        assert!(AutoMemoryTool::should_analyze_message(
            "Can you help me with math?"
        ));
        // Only very short or empty messages are rejected
        assert!(!AutoMemoryTool::should_analyze_message("Hi"));
        assert!(!AutoMemoryTool::should_analyze_message("?"));
    }

    #[test]
    fn test_memory_priority_classification() {
        // With simplified logic, all priorities default to Medium
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("My name is Alice"),
            MemoryPriority::Medium
        );
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("I work at Microsoft"),
            MemoryPriority::Medium
        );
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("I like pizza"),
            MemoryPriority::Medium
        );
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("Today I feel tired"),
            MemoryPriority::Medium
        );
    }

    #[test]
    fn test_memory_type_determination() {
        // With simplified logic, all types default to "general"
        assert_eq!(
            AutoMemoryTool::determine_memory_type("I work at Apple"),
            "general"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("My name is Bob"),
            "general"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("I live in Seattle"),
            "general"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("I love chocolate"),
            "general"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("My wife is great"),
            "general"
        );
    }

    #[test]
    fn test_confidence_calculation() {
        let high_priority = MemoryPriority::High;
        let medium_priority = MemoryPriority::Medium;
        let low_priority = MemoryPriority::Low;

        // Confidence is now fixed based on priority only
        assert_eq!(
            AutoMemoryTool::calculate_confidence("My name is John", &high_priority),
            0.9
        );
        assert_eq!(
            AutoMemoryTool::calculate_confidence("I might work somewhere", &medium_priority),
            0.7
        );
        assert_eq!(
            AutoMemoryTool::calculate_confidence("Today I'm tired", &low_priority),
            0.3
        );
    }

    #[tokio::test]
    async fn test_memory_analysis() {
        let tool = AutoMemoryTool::new();

        let args = AutoMemoryArgs {
            user_message: "Hi, my name is Scott and I work at Amazon".to_string(),
            context: None,
        };

        let result = tool
            .analyze_for_memories(&args.user_message, args.context.as_deref())
            .await
            .unwrap();

        assert!(result.should_store);
        // extract_facts is now deprecated and returns empty
        assert!(result.extracted_facts.is_empty());
        assert_eq!(result.memory_type, "general"); // All types default to general
        assert_eq!(result.confidence, 0.7); // Medium priority = 0.7 confidence
    }

    #[tokio::test]
    async fn test_no_memory_content() {
        let tool = AutoMemoryTool::new();

        let args = AutoMemoryArgs {
            user_message: "What's the weather like today?".to_string(),
            context: None,
        };

        let result = tool
            .analyze_for_memories(&args.user_message, args.context.as_deref())
            .await
            .unwrap();

        // With simplified logic, most messages are now considered worth storing
        assert!(result.should_store);
        assert!(result.extracted_facts.is_empty());
        assert_eq!(result.confidence, 0.7); // Medium priority confidence
    }
}

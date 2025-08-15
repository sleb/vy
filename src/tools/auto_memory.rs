use crate::simple_memory::{SimpleMemory, default_memory_file};
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

impl AutoMemoryError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

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
    pub fn should_analyze_message(message: &str) -> bool {
        let memory_triggers = [
            // Identity indicators
            "i am",
            "i'm",
            "my name",
            "call me",
            "i go by",
            // Employment indicators
            "i work",
            "my job",
            "i'm employed",
            "my career",
            "my company",
            "i started",
            "i quit",
            "i got hired",
            "promoted to",
            // Location indicators
            "i live",
            "i moved",
            "i'm from",
            "my address",
            "my home",
            "i relocated",
            "based in",
            // Relationship indicators
            "my wife",
            "my husband",
            "my partner",
            "my kids",
            "my children",
            "my family",
            "married to",
            "divorced from",
            "dating",
            // Preference indicators
            "i like",
            "i love",
            "i hate",
            "i prefer",
            "my favorite",
            "i enjoy",
            "i can't stand",
            "i'm into",
            // Life changes
            "i just",
            "i recently",
            "i used to",
            "now i",
            "i changed",
            "i decided",
            "i bought",
            "i sold",
            "i started",
            // Personal details
            "my birthday",
            "i was born",
            "my age",
            "i have",
            "i own",
            "my hobby",
            "i play",
            "i study",
            "i learned",
        ];

        let message_lower = message.to_lowercase();
        memory_triggers
            .iter()
            .any(|trigger| message_lower.contains(trigger))
    }

    /// Determine the priority of potential memory information
    fn classify_memory_priority(message: &str) -> MemoryPriority {
        let message_lower = message.to_lowercase();

        // High priority: identity, employment, major life changes
        let high_priority_indicators = [
            "my name",
            "i am",
            "i work",
            "my job",
            "i live",
            "my wife",
            "my husband",
            "my children",
            "my kids",
            "i moved",
            "i got married",
            "i divorced",
            "i started working",
            "i quit",
            "promoted to",
            "my company",
        ];

        // Low priority: temporary, mood, daily activities
        let low_priority_indicators = [
            "today i",
            "right now",
            "currently",
            "at the moment",
            "feeling",
            "this morning",
            "tonight",
            "yesterday",
            "planning to",
            "thinking about",
            "wondering if",
        ];

        if high_priority_indicators
            .iter()
            .any(|indicator| message_lower.contains(indicator))
        {
            MemoryPriority::High
        } else if low_priority_indicators
            .iter()
            .any(|indicator| message_lower.contains(indicator))
        {
            MemoryPriority::Low
        } else {
            MemoryPriority::Medium
        }
    }

    /// Extract memory type from message content
    fn determine_memory_type(message: &str) -> String {
        let message_lower = message.to_lowercase();

        if message_lower.contains("work")
            || message_lower.contains("job")
            || message_lower.contains("company")
            || message_lower.contains("career")
        {
            "employment".to_string()
        } else if message_lower.contains("name")
            || message_lower.contains("i am")
            || message_lower.contains("i'm")
        {
            "identity".to_string()
        } else if message_lower.contains("live")
            || message_lower.contains("home")
            || message_lower.contains("address")
            || message_lower.contains("from")
        {
            "location".to_string()
        } else if message_lower.contains("like")
            || message_lower.contains("love")
            || message_lower.contains("hate")
            || message_lower.contains("prefer")
            || message_lower.contains("favorite")
        {
            "preference".to_string()
        } else if message_lower.contains("wife")
            || message_lower.contains("husband")
            || message_lower.contains("family")
            || message_lower.contains("children")
            || message_lower.contains("married")
        {
            "relationship".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Calculate confidence score based on message clarity and specificity
    fn calculate_confidence(message: &str, priority: &MemoryPriority) -> f32 {
        let mut confidence: f32 = match priority {
            MemoryPriority::High => 0.9,
            MemoryPriority::Medium => 0.7,
            MemoryPriority::Low => 0.3,
        };

        let message_lower = message.to_lowercase();

        // Boost confidence for clear statements
        if message_lower.contains("my name is")
            || message_lower.contains("i work at")
            || message_lower.contains("i live in")
        {
            confidence += 0.1;
        }

        // Reduce confidence for vague statements
        if message_lower.contains("maybe")
            || message_lower.contains("might")
            || message_lower.contains("probably")
            || message_lower.contains("thinking")
        {
            confidence -= 0.2;
        }

        confidence.clamp(0.0, 1.0)
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

        // Use existing extraction logic from SimpleMemory
        let memory_file = default_memory_file()
            .map_err(|e| AutoMemoryError::new(format!("Failed to get memory file path: {e}")))?;
        let temp_memory = SimpleMemory::new(memory_file);
        let extracted_facts = temp_memory.extract_facts(message);

        let should_store = match priority {
            MemoryPriority::High => confidence > 0.6,
            MemoryPriority::Medium => confidence > 0.7,
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
        assert!(AutoMemoryTool::should_analyze_message(
            "Hi, my name is John"
        ));
        assert!(AutoMemoryTool::should_analyze_message("I work at Google"));
        assert!(AutoMemoryTool::should_analyze_message("I love coffee"));
        assert!(!AutoMemoryTool::should_analyze_message(
            "What's the weather like?"
        ));
        assert!(!AutoMemoryTool::should_analyze_message(
            "Can you help me with math?"
        ));
    }

    #[test]
    fn test_memory_priority_classification() {
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("My name is Alice"),
            MemoryPriority::High
        );
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("I work at Microsoft"),
            MemoryPriority::High
        );
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("I like pizza"),
            MemoryPriority::Medium
        );
        assert_eq!(
            AutoMemoryTool::classify_memory_priority("Today I feel tired"),
            MemoryPriority::Low
        );
    }

    #[test]
    fn test_memory_type_determination() {
        assert_eq!(
            AutoMemoryTool::determine_memory_type("I work at Apple"),
            "employment"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("My name is Bob"),
            "identity"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("I live in Seattle"),
            "location"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("I love chocolate"),
            "preference"
        );
        assert_eq!(
            AutoMemoryTool::determine_memory_type("My wife is great"),
            "relationship"
        );
    }

    #[test]
    fn test_confidence_calculation() {
        let high_priority = MemoryPriority::High;
        let medium_priority = MemoryPriority::Medium;

        // Clear statements should have high confidence
        assert!(AutoMemoryTool::calculate_confidence("My name is John", &high_priority) > 0.9);

        // Vague statements should have lower confidence
        assert!(
            AutoMemoryTool::calculate_confidence("I might work somewhere", &medium_priority) < 0.6
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
        assert!(!result.extracted_facts.is_empty());
        assert_eq!(result.memory_type, "employment"); // "work" is detected first in the logic
        assert!(result.confidence > 0.8);
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

        assert!(!result.should_store);
        assert!(result.extracted_facts.is_empty());
        assert_eq!(result.confidence, 0.0);
    }
}

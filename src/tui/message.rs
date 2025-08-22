use chrono::{DateTime, Utc};

/// Types of messages in the chat interface
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Error,
    Thinking,
}

/// A single chat message in the TUI
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub message_type: MessageType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(message_type: MessageType, content: String) -> Self {
        Self {
            message_type,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn user(content: String) -> Self {
        Self::new(MessageType::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(MessageType::Assistant, content)
    }

    pub fn system(content: String) -> Self {
        Self::new(MessageType::System, content)
    }

    pub fn error(content: String) -> Self {
        Self::new(MessageType::Error, content)
    }

    pub fn thinking(content: String) -> Self {
        Self::new(MessageType::Thinking, content)
    }

    /// Get a display prefix for the message based on its type
    pub fn get_prefix(&self) -> &'static str {
        match self.message_type {
            MessageType::User => "💬 You: ",
            MessageType::Assistant => "🤖 Vy: ",
            MessageType::Error => "❌ Error: ",
            MessageType::Thinking => "",
            MessageType::System => "ℹ️  ",
        }
    }
}

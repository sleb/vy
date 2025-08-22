use crate::tui::input::InputHandler;
use crate::tui::message::{ChatMessage, MessageType};
use rig::completion::Message;

/// Application state for the TUI
pub struct App {
    pub model_id: String,
    pub messages: Vec<ChatMessage>,
    pub conversation_history: Vec<Message>,
    pub input_handler: InputHandler,
    pub scroll_offset: usize,
    pub in_help_mode: bool,
}

impl App {
    pub fn new(model_id: String) -> Self {
        let mut app = Self {
            model_id: model_id.clone(),
            messages: Vec::new(),
            conversation_history: Vec::new(),
            input_handler: InputHandler::new(),
            scroll_offset: 0,
            in_help_mode: false,
        };

        // Add welcome message
        app.add_message(ChatMessage::new(
            MessageType::System,
            format!(
                "🤖 Welcome to Vy TUI - {model_id}! Type your message and press Enter to chat."
            ),
        ));
        app.add_message(ChatMessage::new(
            MessageType::System,
            "Press F1 for help, Esc to exit.".to_string(),
        ));

        app
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        // Auto-scroll to bottom when new message is added
        if self.messages.len() > 10 {
            self.scroll_offset = self.messages.len().saturating_sub(10);
        }
    }

    pub fn remove_thinking_messages(&mut self) {
        self.messages
            .retain(|msg| msg.message_type != MessageType::Thinking);
    }

    pub fn scroll_messages_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_messages_down(&mut self) {
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_messages_page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    pub fn scroll_messages_page_down(&mut self) {
        let max_scroll = self.messages.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + 10).min(max_scroll);
    }
}

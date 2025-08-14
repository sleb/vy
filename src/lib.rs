use anyhow::{Context, Result};
use rig::{cli_chatbot::cli_chatbot, completion::Chat};

pub struct Vy<A: Chat> {
    agent: A,
}

impl<A: Chat> Vy<A> {
    pub fn new(agent: A) -> Self {
        Self { agent }
    }

    pub async fn chat(self) -> Result<()> {
        cli_chatbot(self.agent)
            .await
            .context("Failed to start chatbot")?;

        Ok(())
    }
}

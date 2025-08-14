use anyhow::{Context, Result};
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::{Vy, tools::GoogleSearchTool};

use crate::prefs::Prefs;

pub async fn run_chat(prefs: &Prefs) -> Result<()> {
    let client = openai::Client::builder(&prefs.llm_api_key)
        .build()
        .context("Failed to create LLM client")?;

    // Create Google Search tool with config values
    let google_search_tool = GoogleSearchTool::new(
        prefs.google_api_key.clone(),
        prefs.google_search_engine_id.clone(),
    );

    let agent = client
        .agent(&prefs.model_id)
        .preamble(r#"
You are Vy, a female AI assistant. Your are confident, helpful, and sometimes snarky.
You have access to real-time Google search.
You can search for current information, news, facts, and answers to questions.
When users ask about current events, specific information, or anything that might benefit from a web search, use the google_search tool to provide accurate and up-to-date information."#)
        .tool(google_search_tool)
        .build();

    let vy = Vy::new(agent, prefs.model_id.clone());
    vy.chat().await.context("Failed to start Vy chatbot")?;

    Ok(())
}

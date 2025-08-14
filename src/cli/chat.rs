use anyhow::{Context, Result};
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::{Vy, tools::GoogleSearchTool};

use crate::prefs::Prefs;

pub async fn run_chat(prefs: &Prefs) -> Result<()> {
    let client = openai::Client::builder(&prefs.llm_api_key)
        .build()
        .context("Failed to create LLM client")?;

    // Check if this is gpt-5-mini which has tool calling issues
    let is_gpt5_mini = prefs.model_id == "gpt-5-mini";

    let agent = if is_gpt5_mini {
        // For gpt-5-mini, disable tools due to compatibility issues
        client
            .agent(&prefs.model_id)
            .preamble(r#"
You are Vy, a female AI assistant. Your are confident, helpful, and sometimes snarky.
Note: Google search functionality is temporarily unavailable with gpt-5-mini due to tool calling compatibility issues.
This is expected to be resolved in future updates. You can still help with general questions, coding, analysis,
and other tasks using your training data. For real-time information, consider switching to gpt-4o or gpt-4o-mini."#)
            .build()
    } else {
        // For other models, use tools normally
        let google_search_tool = GoogleSearchTool::new(
            prefs.google_api_key.clone(),
            prefs.google_search_engine_id.clone(),
        );

        client
            .agent(&prefs.model_id)
            .preamble(r#"
You are Vy, a female AI assistant. Your are confident, helpful, and sometimes snarky.
You have access to real-time Google search.
You can search for current information, news, facts, and answers to questions.
When users ask about current events, specific information, or anything that might benefit from a web search, use the google_search tool to provide accurate and up-to-date information."#)
            .tool(google_search_tool)
            .build()
    };

    let vy = Vy::new(agent, prefs.model_id.clone());
    vy.chat().await.context("Failed to start Vy chatbot")?;

    Ok(())
}

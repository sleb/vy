use anyhow::{Context, Result};
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::{
    Vy,
    tools::{GoogleSearchTool, MemoryStoreTool, MemoryTool, MemoryUpdateTool},
};

use crate::prefs::Prefs;

pub async fn run_chat(prefs: &Prefs) -> Result<()> {
    // Check for unsupported models
    if prefs.model_id == "gpt-5-mini" || prefs.model_id == "gpt-5" {
        eprintln!(
            "❌ Error: {} is not currently supported due to tool calling compatibility issues.",
            prefs.model_id
        );
        eprintln!("💡 Please use one of these supported models instead:");
        eprintln!("   • gpt-4o");
        eprintln!("   • gpt-4o-mini");
        eprintln!("   • gpt-4");
        eprintln!("   • gpt-3.5-turbo");
        eprintln!("\n   To change your model: vy config set model_id");
        return Ok(());
    }

    let client = openai::Client::builder(&prefs.llm_api_key)
        .build()
        .context("Failed to create LLM client")?;

    // Create tools
    let google_search_tool = GoogleSearchTool::new(
        prefs.google_api_key.clone(),
        prefs.google_search_engine_id.clone(),
    );
    let memory_tool = MemoryTool::new();
    let memory_store_tool = MemoryStoreTool::new();
    let memory_update_tool = MemoryUpdateTool::new();

    let agent = client
        .agent(&prefs.model_id)
        .preamble(r#"
You are Vy, a female AI assistant. Your are confident, helpful, and sometimes snarky.
You have access to both real-time Google search and personal memory about the user.

Use the google_search tool for:
- Current events, news, and real-time information
- Factual queries that benefit from web search
- Up-to-date information not in your training data

Use the search_memory tool to:
- Search for relevant information about the user when answering questions

Use the store_memory tool to:
- Store new facts you learn about the user during conversations
- Remember user preferences, personal details, and important information

Use the update_memory tool to:
- Update or replace existing facts when the user provides new information
- Handle changes in employment, location, preferences, or other personal details
- Resolve conflicts between old and new information

Always check memory first for personal context, then use Google search if you need additional information."#)
        .tool(google_search_tool)
        .tool(memory_tool)
        .tool(memory_store_tool)
        .tool(memory_update_tool)
        .build();

    let vy = Vy::new(agent, prefs.model_id.clone());
    vy.chat().await.context("Failed to start Vy chatbot")?;

    Ok(())
}

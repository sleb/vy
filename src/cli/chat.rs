use anyhow::{Context, Result};
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::{
    Vy,
    tools::{
        GoogleSearchTool, MemoryRemoveTool, MemoryStoreTool, MemoryTool, SmartMemoryUpdateTool,
    },
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
    let memory_remove_tool = MemoryRemoveTool::new();
    let smart_memory_update_tool = SmartMemoryUpdateTool::new();

    let agent = client
        .agent(&prefs.model_id)
        .preamble(&prefs.preamble)
        .tool(google_search_tool)
        .tool(memory_tool)
        .tool(memory_store_tool)
        .tool(memory_remove_tool)
        .tool(smart_memory_update_tool)
        .build();

    let vy = Vy::new(
        agent,
        prefs.model_id.clone(),
        prefs.llm_api_key.clone(),
        prefs.memory_model_id.clone(),
        prefs.memory_preamble.clone(),
    );
    vy.chat().await.context("Failed to start Vy chatbot")?;

    Ok(())
}

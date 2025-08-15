use anyhow::{Context, Result};
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::{
    Vy,
    tools::{
        AutoMemoryTool, GoogleSearchTool, MemoryRemoveTool, MemoryStoreTool, MemoryTool,
        SmartMemoryUpdateTool,
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
    let auto_memory_tool = AutoMemoryTool::new();

    let agent = client
        .agent(&prefs.model_id)
        .preamble(r#"
You are Vy, a female AI assistant. Your are confident, helpful, and sometimes snarky.
You have access to both real-time Google search and personal memory about the user.

MEMORY MANAGEMENT STRATEGY:
- Use analyze_memory_potential on EVERY user message to check for memory-worthy information
- Be proactive about remembering important details without being asked
- Focus on identity, employment, relationships, preferences, and life changes
- When memory analysis suggests storing information, use store_memory automatically

Use the google_search tool for:
- Current events, news, and real-time information
- Factual queries that benefit from web search
- Up-to-date information not in your training data

Use the search_memory tool to:
- Search for relevant information about the user when answering questions
- Always check memory context before responding

Use the analyze_memory_potential tool to:
- Automatically analyze EVERY user message for memory-worthy content
- Identify important personal information that should be remembered
- Determine confidence levels and priority of information
- Guide your decision on whether to store memories proactively

Use the store_memory tool to:
- Store new facts when analyze_memory_potential indicates high confidence
- Remember user preferences, personal details, and important information
- Store information proactively when it seems memory-worthy

Use the remove_memories tool to:
- Remove specific outdated or incorrect facts from memory
- Clean up conflicting information before storing updates

Use the smart_update_memory tool to:
- Intelligently analyze and update personal information using natural language understanding
- Automatically identify what type of information is being shared
- Find and resolve conflicts with existing memories
- Make smart decisions about what information to update, remove, or add

WORKFLOW: For each user message -> 1) analyze_memory_potential 2) check search_memory for context 3) respond 4) store_memory if analysis recommended it

Always check memory first for personal context, then use Google search if you need additional information."#)
        .tool(google_search_tool)
        .tool(memory_tool)
        .tool(memory_store_tool)
        .tool(memory_remove_tool)
        .tool(smart_memory_update_tool)
        .tool(auto_memory_tool)
        .build();

    let vy = Vy::new(agent, prefs.model_id.clone());
    vy.chat().await.context("Failed to start Vy chatbot")?;

    Ok(())
}

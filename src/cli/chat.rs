use anyhow::{Context, Result};
use rig::{client::completion::CompletionClientDyn, completion::Prompt, providers::openai};
use vy::{
    Vy,
    simple_memory::{SimpleMemory, default_memory_file},
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

    // Load memory context and enhance preamble
    let enhanced_preamble = load_memory_enhanced_preamble().await?;

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
        .preamble(&enhanced_preamble)
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
    );
    vy.chat().await.context("Failed to start Vy chatbot")?;

    Ok(())
}

/// Load relevant memories and enhance the preamble with user context
async fn load_memory_enhanced_preamble() -> Result<String> {
    let base_preamble = crate::prefs::default_preamble();
    // Get memory file path
    let memory_file = match default_memory_file() {
        Ok(path) => path,
        Err(e) => {
            log::debug!("Failed to get memory file path: {e}");
            return Ok(base_preamble.to_string());
        }
    };

    // Load existing memory
    let mut memory = SimpleMemory::new(memory_file);
    if let Err(e) = memory.load().await {
        log::debug!("Failed to load existing memories: {e}");
        return Ok(base_preamble.to_string());
    }

    // If no memories exist, return base preamble
    if memory.entry_count() == 0 {
        return Ok(base_preamble.to_string());
    }

    // Load preferences to get API key and model for intelligent context extraction
    let prefs = match get_prefs_for_memory_context().await {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Failed to load prefs for context extraction: {e}");
            // Fall back to basic search if we can't load prefs
            return fallback_memory_search(&memory).await;
        }
    };

    // Use LLM to intelligently determine what context is relevant
    let relevant_memories = match extract_relevant_context(&memory, &prefs).await {
        Ok(memories) => memories,
        Err(e) => {
            log::debug!("Failed to extract intelligent context: {e}, falling back to basic search");
            // Fall back to basic search if LLM fails
            return fallback_memory_search(&memory).await;
        }
    };

    if relevant_memories.is_empty() {
        return Ok(base_preamble.to_string());
    }

    let memory_context = format!(
        r#"

IMPORTANT USER CONTEXT (from previous conversations):
{}

RESPONSE GUIDELINES:
- When asked about tasks/todos, provide clean bulleted lists
- Be aware of temporal context - don't mention outdated appointments
- For "what's on my todo list" queries, format as actionable bullet points
- Prioritize recent and current information over old entries"#,
        relevant_memories
            .iter()
            .map(|fact| format!("• {}", fact))
            .collect::<Vec<_>>()
            .join("\n")
    );

    log::debug!(
        "Loaded {} relevant memories into preamble context",
        relevant_memories.len()
    );

    Ok(format!("{}{}", base_preamble, memory_context))
}

/// Extract relevant context using LLM analysis with temporal and relevance awareness
async fn extract_relevant_context(
    memory: &SimpleMemory,
    prefs: &crate::prefs::Prefs,
) -> Result<Vec<String>> {
    use rig::providers::openai;

    let client = openai::Client::builder(&prefs.llm_api_key).build()?;

    // Get all memories with timestamps for analysis
    let all_entries = memory.get_all_entries();

    if all_entries.is_empty() {
        return Ok(Vec::new());
    }

    let today = chrono::Utc::now().date_naive();

    // Create enriched memory text with temporal context
    let memories_text = all_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let entry_date = entry.timestamp.date_naive();
            let days_ago = (today - entry_date).num_days();
            let temporal_note = if days_ago == 0 {
                " [TODAY]"
            } else if days_ago == 1 {
                " [YESTERDAY]"
            } else if days_ago <= 7 {
                &format!(" [{}D AGO]", days_ago)
            } else {
                " [OLD]"
            };
            format!("{}. {}{}", i + 1, entry.fact, temporal_note)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let analysis_prompt = format!(
        r#"You are about to start a conversation with a user. Below are facts from previous conversations with temporal indicators.

Select the most relevant and current facts for providing personalized responses. Prioritize:
- Current/recent information over old entries
- Core identity (name, role, location) - always relevant
- Active tasks, appointments, and commitments
- Family/personal relationships
- Professional context
- Recent preferences and activities

TEMPORAL GUIDANCE:
- [TODAY]/[YESTERDAY] entries are highly relevant
- [OLD] entries (>7 days) should only be selected if they contain core identity info
- Avoid outdated appointments, meetings, or time-sensitive tasks

Available facts with temporal context:
{}

Return ONLY a JSON array of the most relevant fact numbers (1-{}), like: [1, 3, 7, 12]
Limit to maximum 8 facts, prioritizing recent and permanent information."#,
        memories_text,
        all_entries.len()
    );

    let agent = client
        .agent(&prefs.memory_model_id)
        .preamble(&crate::prefs::default_memory_preamble())
        .build();

    let response = agent.prompt(&analysis_prompt).await?;

    // Parse the response to get selected fact indices
    let response_cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_end_matches("```")
        .trim();

    let selected_indices: Vec<usize> =
        serde_json::from_str(response_cleaned).unwrap_or_else(|_| {
            log::debug!("Failed to parse LLM response, using temporal fallback selection");
            // Fallback: prioritize recent entries and core identity
            all_entries
                .iter()
                .enumerate()
                .filter_map(|(idx, entry)| {
                    let entry_date = entry.timestamp.date_naive();
                    let days_ago = (today - entry_date).num_days();
                    let is_core_identity = entry.fact.to_lowercase().contains("scott")
                        || entry.fact.to_lowercase().contains("amazon")
                        || entry.fact.to_lowercase().contains("live");
                    if days_ago <= 2 || is_core_identity {
                        Some(idx + 1)
                    } else {
                        None
                    }
                })
                .take(8)
                .collect()
        });

    // Convert indices to actual facts (adjusting for 1-based indexing)
    let relevant_memories = selected_indices
        .iter()
        .filter_map(|&idx| {
            if idx > 0 && idx <= all_entries.len() {
                Some(all_entries[idx - 1].fact.clone())
            } else {
                None
            }
        })
        .collect();

    Ok(relevant_memories)
}

/// Fallback memory search using basic text matching for common terms
async fn fallback_memory_search(memory: &SimpleMemory) -> Result<String> {
    let search_terms = vec![
        "name",
        "work",
        "job",
        "live",
        "family",
        "son",
        "daughter",
        "wife",
        "husband",
        "appointment",
        "meeting",
    ];

    let mut relevant_memories = Vec::new();
    for term in search_terms {
        let results = memory.search(term);
        for entry in results {
            if !relevant_memories
                .iter()
                .any(|existing| existing == &entry.fact)
            {
                relevant_memories.push(entry.fact.clone());
            }
        }
    }

    relevant_memories.truncate(8);

    if relevant_memories.is_empty() {
        return Ok(crate::prefs::default_preamble());
    }

    let memory_context = format!(
        "\n\nIMPORTANT USER CONTEXT (from previous conversations):\n{}",
        relevant_memories
            .iter()
            .map(|fact| format!("• {}", fact))
            .collect::<Vec<_>>()
            .join("\n")
    );

    log::debug!(
        "Used fallback search, loaded {} memories",
        relevant_memories.len()
    );

    Ok(format!(
        "{}{}",
        crate::prefs::default_preamble(),
        memory_context
    ))
}

/// Load preferences for memory context extraction
async fn get_prefs_for_memory_context() -> Result<crate::prefs::Prefs> {
    let project_dirs = directories::ProjectDirs::from("vy", "", "")
        .ok_or_else(|| anyhow::anyhow!("Could not determine user directories"))?;
    let config_path = project_dirs.config_dir().join("prefs.toml");
    crate::prefs::load_prefs(&config_path)
}

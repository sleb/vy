//! CLI commands for the simple memory system

use anyhow::Result;
use clap::Subcommand;
use std::io::Write;

use crate::simple_memory::{SimpleMemory, default_memory_file};

#[derive(Debug, Clone, Subcommand)]
pub enum SimpleMemoryCommand {
    /// Add a fact manually to memory
    Add {
        /// The fact to remember
        fact: String,
    },
    /// List all stored memories
    List,
    /// Search memories for a specific term
    Search {
        /// Search query
        query: String,
    },
    /// Show memory statistics
    Stats,
    /// Delete a specific memory entry by number
    Delete {
        /// The entry number to delete (as shown in 'list' command)
        index: usize,
        /// Skip confirmation prompt
        #[clap(long)]
        yes: bool,
    },
    /// Clear all memories (use with caution!)
    Clear {
        /// Confirm the clear operation
        #[clap(long)]
        confirm: bool,
    },
    /// Test fact extraction on some text
    Extract {
        /// Text to extract facts from
        text: String,
    },
    /// Consolidate and optimize memory entries by removing duplicates and combining related information
    Vacuum {
        /// Skip confirmation prompt
        #[clap(long)]
        yes: bool,
    },
}

impl SimpleMemoryCommand {
    pub async fn run(self) -> Result<()> {
        let memory_file = default_memory_file()?;
        let mut memory = SimpleMemory::new(memory_file);
        memory.load().await?;

        match self {
            SimpleMemoryCommand::Add { fact } => {
                // Load API key for LLM-based extraction
                // Load preferences for API key and memory settings
                let prefs = std::env::var("OPENAI_API_KEY")
                    .map(|key| {
                        // Use environment API key with default memory settings
                        crate::prefs::Prefs {
                            llm_api_key: key,
                            google_api_key: String::new(),
                            google_search_engine_id: String::new(),
                            model_id: "gpt-3.5-turbo".to_string(),
                            preamble: crate::prefs::default_preamble(),
                            memory_model_id: "gpt-4".to_string(),
                            memory_similarity_model_id: "gpt-3.5-turbo".to_string(),
                            memory_preamble: crate::prefs::default_memory_preamble(),
                        }
                    })
                    .or_else(|_| {
                        // Try to load from config file
                        if let Some(proj_dirs) = directories::ProjectDirs::from("vy", "", "") {
                            let config_path = proj_dirs.config_dir().join("prefs.toml");
                            crate::prefs::load_prefs(&config_path)
                        } else {
                            Err(anyhow::anyhow!("No config directory"))
                        }
                    });

                match prefs {
                    Ok(prefs) if !prefs.llm_api_key.is_empty() => {
                        memory
                            .learn_from_input(
                                &fact,
                                "manual".to_string(),
                                &prefs.llm_api_key,
                                &prefs.memory_model_id,
                                &prefs.memory_preamble,
                            )
                            .await?;
                        println!("✅ Added fact to memory: {fact}");
                    }
                    _ => {
                        println!(
                            "❌ No OpenAI API key found. Set OPENAI_API_KEY environment variable or run 'vy config set llm_api_key'"
                        );
                        return Ok(());
                    }
                }
            }
            SimpleMemoryCommand::List => {
                let display = memory.list_all();
                println!("{display}");
            }
            SimpleMemoryCommand::Search { query } => {
                let results = memory.search(&query);
                if results.is_empty() {
                    println!("No memories found matching '{query}'");
                } else {
                    println!("Found {} memories matching '{query}':\n", results.len());
                    for (i, entry) in results.iter().enumerate() {
                        println!(
                            "{}. [{}] {}\n   Source: {}\n",
                            i + 1,
                            entry.timestamp.format("%Y-%m-%d %H:%M"),
                            entry.fact,
                            entry.source
                        );
                    }
                }
            }
            SimpleMemoryCommand::Delete { index, yes } => {
                // First, let's show what we're about to delete
                if let Some(entry) = memory.get_entry_by_index(index) {
                    println!(
                        "About to delete entry #{}:\n[{}] {}\nSource: {}\n",
                        index,
                        entry.timestamp.format("%Y-%m-%d %H:%M"),
                        entry.fact,
                        entry.source
                    );

                    if !yes {
                        print!("Are you sure you want to delete this entry? (y/N): ");
                        std::io::stdout().flush().unwrap();
                        let mut confirmation = String::new();
                        std::io::stdin().read_line(&mut confirmation).unwrap();
                        let confirmation = confirmation.trim().to_lowercase();

                        if confirmation != "y" && confirmation != "yes" {
                            println!("❌ Deletion cancelled.");
                            return Ok(());
                        }
                    }

                    match memory.delete_by_index(index).await? {
                        Some(deleted_entry) => {
                            println!("✅ Deleted entry: {}", deleted_entry.fact);
                        }
                        None => {
                            println!("❌ Entry #{index} not found.");
                        }
                    }
                } else {
                    println!(
                        "❌ Entry #{index} not found. Use 'vy remember list' to see all entries."
                    );
                }
            }
            SimpleMemoryCommand::Stats => {
                let stats = memory.stats();
                println!("{}", stats.to_display_string());
            }
            SimpleMemoryCommand::Clear { confirm } => {
                if !confirm {
                    println!("⚠️  This will delete ALL memories. Use --confirm to proceed.");
                    return Ok(());
                }

                memory.clear().await?;
                println!("🗑️  All memories have been cleared.");
            }
            SimpleMemoryCommand::Extract { text: _ } => {
                println!(
                    "❌ Extract command has been removed. Use the chat interface for LLM-based memory extraction instead."
                );
            }
            SimpleMemoryCommand::Vacuum { yes } => {
                let entries_before = memory.entry_count();

                if !yes {
                    print!(
                        "⚠️  This will consolidate {} memory entries. Continue? [y/N]: ",
                        entries_before
                    );
                    std::io::stdout().flush()?;

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    let input = input.trim().to_lowercase();

                    if input != "y" && input != "yes" {
                        println!("❌ Vacuum cancelled.");
                        return Ok(());
                    }
                }

                // Load preferences for API key and memory settings
                let prefs = std::env::var("OPENAI_API_KEY")
                    .map(|key| {
                        // Use environment API key with default memory settings
                        crate::prefs::Prefs {
                            llm_api_key: key,
                            google_api_key: String::new(),
                            google_search_engine_id: String::new(),
                            model_id: "gpt-3.5-turbo".to_string(),
                            preamble: crate::prefs::default_preamble(),
                            memory_model_id: "gpt-4".to_string(),
                            memory_similarity_model_id: "gpt-3.5-turbo".to_string(),
                            memory_preamble: crate::prefs::default_memory_preamble(),
                        }
                    })
                    .or_else(|_| {
                        // Try to load from config file
                        if let Some(proj_dirs) = directories::ProjectDirs::from("vy", "", "") {
                            let config_path = proj_dirs.config_dir().join("prefs.toml");
                            crate::prefs::load_prefs(&config_path)
                        } else {
                            Err(anyhow::anyhow!("No config directory"))
                        }
                    });

                match prefs {
                    Ok(prefs) if !prefs.llm_api_key.is_empty() => {
                        println!(
                            "🧹 Consolidating {} memory entries with LLM analysis...",
                            entries_before
                        );
                        memory
                            .vacuum(&prefs.llm_api_key, &prefs.memory_similarity_model_id)
                            .await?;
                    }
                    _ => {
                        eprintln!("❌ Error: No API key found for LLM analysis.");
                        eprintln!("💡 Please set your API key:");
                        eprintln!("   • Environment variable: export OPENAI_API_KEY=your_key");
                        eprintln!("   • Or configure it: vy config set llm_api_key");
                        return Ok(());
                    }
                }

                memory.save().await?;

                let entries_after = memory.entry_count();
                let removed = entries_before - entries_after;

                if removed > 0 {
                    println!(
                        "✅ Vacuum completed! Removed {} redundant entries.",
                        removed
                    );
                    println!("📊 Memory entries: {} → {}", entries_before, entries_after);
                } else {
                    println!("✅ Vacuum completed! No redundant entries found.");
                }
            }
        }

        Ok(())
    }
}

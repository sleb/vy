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
                let api_key = std::env::var("OPENAI_API_KEY").or_else(|_| {
                    // Try to load from config file
                    if let Some(proj_dirs) = directories::ProjectDirs::from("vy", "", "") {
                        let config_path = proj_dirs.config_dir().join("prefs.toml");
                        match crate::prefs::load_prefs(&config_path) {
                            Ok(prefs) => Ok(prefs.llm_api_key),
                            Err(_) => Err("No API key found".to_string()),
                        }
                    } else {
                        Err("No config directory".to_string())
                    }
                });

                match api_key {
                    Ok(key) if !key.is_empty() => {
                        memory
                            .learn_from_input(&fact, "manual".to_string(), &key)
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

                // Try to load API key from environment or config
                let api_key = std::env::var("OPENAI_API_KEY").or_else(|_| {
                    // Try to load from config file
                    if let Some(proj_dirs) = directories::ProjectDirs::from("vy", "", "") {
                        let config_path = proj_dirs.config_dir().join("prefs.toml");
                        match crate::prefs::load_prefs(&config_path) {
                            Ok(prefs) => Ok(prefs.llm_api_key),
                            Err(_) => Err("No API key found".to_string()),
                        }
                    } else {
                        Err("No config directory".to_string())
                    }
                });

                match api_key {
                    Ok(key) if !key.is_empty() => {
                        println!(
                            "🧹 Consolidating {} memory entries with LLM analysis...",
                            entries_before
                        );
                        memory.vacuum(&key).await?;
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

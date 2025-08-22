//! CLI commands for the memory system

use anyhow::Result;
use clap::Subcommand;
use std::io::Write;

use vy_core::memory::{Memory, default_memory_file};

#[derive(Debug, Clone, Subcommand)]
pub enum MemoryCommand {
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

impl MemoryCommand {
    pub async fn run(self) -> Result<()> {
        let memory_file = default_memory_file()?;
        let mut memory = Memory::new(memory_file);
        memory.load().await?;

        match self {
            MemoryCommand::Add { fact } => {
                // Load API key for LLM-based extraction
                // Load preferences for API key and memory settings
                let prefs = if let Some(proj_dirs) = directories::ProjectDirs::from("vy", "", "") {
                    let config_path = proj_dirs.config_dir().join("prefs.toml");
                    vy_core::config::load_config(&config_path)
                        .map_err(|_| anyhow::anyhow!(
                            "Configuration file not found. Please run 'vy config init' to set up all required configuration"
                        ))?
                } else {
                    anyhow::bail!("No config directory available")
                };

                if !prefs.llm_api_key.is_empty() {
                    memory
                        .learn_from_input(
                            &fact,
                            "manual".to_string(),
                            &prefs.llm_api_key,
                            &prefs.memory_model_id,
                        )
                        .await?;
                    println!("✅ Added fact to memory: {fact}");
                } else {
                    println!(
                        "❌ No OpenAI API key found. Please run 'vy config init' to set up configuration"
                    );
                    return Ok(());
                }
            }
            MemoryCommand::List => {
                let display = memory.list_all();
                println!("{display}");
            }
            MemoryCommand::Search { query } => {
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
            MemoryCommand::Delete { index, yes } => {
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
            MemoryCommand::Clear { confirm } => {
                if !confirm {
                    println!("⚠️  This will delete ALL memories. Use --confirm to proceed.");
                    return Ok(());
                }

                memory.clear().await?;
                println!("🗑️  All memories have been cleared.");
            }
            MemoryCommand::Stats => {
                let stats = memory.stats();
                println!("{}", stats.to_display_string());
            }
            MemoryCommand::Extract { text: _ } => {
                println!(
                    "❌ Extract command has been removed. Use the chat interface for LLM-based memory extraction instead."
                );
            }
            MemoryCommand::Vacuum { yes: _ } => {
                println!("❌ Vacuum command is temporarily disabled in this version.");
            }
        }

        Ok(())
    }
}

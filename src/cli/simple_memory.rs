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
}

impl SimpleMemoryCommand {
    pub async fn run(self) -> Result<()> {
        let memory_file = default_memory_file()?;
        let mut memory = SimpleMemory::new(memory_file);
        memory.load().await?;

        match self {
            SimpleMemoryCommand::Add { fact } => {
                memory.learn_from_input(&fact, "manual".to_string()).await?;
                println!("✅ Added fact to memory: {fact}");
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
            SimpleMemoryCommand::Extract { text } => {
                let facts = memory.extract_facts(&text);
                if facts.is_empty() {
                    println!("No extractable facts found in: '{text}'");
                } else {
                    println!("Extracted facts from: '{text}'\n");
                    for (i, fact) in facts.iter().enumerate() {
                        println!("{}. {}", i + 1, fact);
                    }
                }
            }
        }

        Ok(())
    }
}

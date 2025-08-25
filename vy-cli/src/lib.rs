use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;

use vy_core::config::{VyConfig, load_config, load_or_create_config};

pub mod chat;
pub mod config;

pub use config::ConfigAction;

static DEFAULT_PREFS_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

fn default_config_path() -> Option<&'static Path> {
    DEFAULT_PREFS_PATH
        .get_or_init(|| {
            directories::ProjectDirs::from("vy", "", "")
                .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
        })
        .as_deref()
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Optional config path
    #[clap(long, value_parser)]
    config_path: Option<PathBuf>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Start the chatbot (default: from config, use --tui/--cli to override)
    Chat {
        /// Use TUI (Terminal User Interface) mode - modern full-screen interface with real-time chat
        #[clap(long)]
        tui: bool,
        /// Use CLI mode - classic text-based interface
        #[clap(long)]
        cli: bool,
    },
    /// Manage configuration values
    Config {
        /// Edit the preferences file in your default editor
        #[clap(long)]
        edit: bool,
        #[clap(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Manage memories (facts Vy remembers about you)
    #[clap(name = "mem")]
    Memory {
        #[clap(subcommand)]
        action: MemoryAction,
    },
}

#[derive(Debug, clap::Subcommand)]
enum MemoryAction {
    /// Store a new memory/fact
    Create {
        /// The fact or information to remember
        fact: String,
    },
    /// List all stored memories
    List,
    /// Search memories by query
    Search {
        /// Search query to find relevant memories
        query: String,
        /// Number of results to return (default: 10)
        #[clap(short = 'n', long, default_value = "10")]
        limit: usize,
    },
    /// Remove memories matching a query
    Remove {
        /// Query to find memories to remove
        query: String,
        /// Skip confirmation prompt
        #[clap(short = 'y', long)]
        yes: bool,
    },
    /// Show memory statistics
    Stats,
    /// Test memory system connection
    Test,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Chat { tui, cli } => {
                let config = self.load_config()?;

                // Determine which mode to use
                let use_tui = if *tui && *cli {
                    // Both flags specified - show error
                    anyhow::bail!(
                        "Cannot specify both --tui and --cli flags. Choose one or rely on default configuration."
                    );
                } else if *tui {
                    true
                } else if *cli {
                    false
                } else {
                    // No explicit flag, use configuration default
                    config.default_chat_mode == "tui"
                };

                if use_tui {
                    vy_tui::run_tui(&config).await
                } else {
                    chat::run_chat(&config).await
                }
            }
            Commands::Config { edit, action } => {
                let config_path = self
                    .config_path
                    .as_deref()
                    .or(default_config_path())
                    .context("Please specify a config path via --config-path")?;

                if *edit {
                    self.run_config_edit(config_path).await
                } else if let Some(action) = action {
                    config::run_config(action, config_path, |path| self.load_config_strict(path))
                } else {
                    // Default to list when no action is specified
                    config::run_config(&config::ConfigAction::List, config_path, |path| {
                        self.load_config_strict(path)
                    })
                }
            }
            Commands::Memory { action } => {
                let config = self.load_config()?;
                self.run_memory_command(action, &config).await
            }
        }
    }

    fn load_config(&self) -> Result<VyConfig> {
        let config_path = self
            .config_path
            .as_deref()
            .or(default_config_path())
            .context("Please specify a config path via --config-path")?;
        load_or_create_config(config_path)
    }

    fn load_config_strict(&self, config_path: &Path) -> Result<VyConfig> {
        debug!("config_path: {config_path:?}");

        let config = load_config(config_path)
            .with_context(|| {
                format!(
                    "Failed to load configuration. Make sure the config file exists or use 'vy config init' to create it.\nExpected location: {}",
                    config_path.display()
                )
            })?;
        debug!("config: {config:?}");

        Ok(config)
    }

    async fn run_config_edit(&self, config_path: &Path) -> Result<()> {
        // Ensure config file exists
        if !config_path.exists() {
            anyhow::bail!(
                "Configuration file not found at: {}\n💡 Run 'vy config init' to set up all required configuration first",
                config_path.display()
            );
        }

        // Open in default editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let status = std::process::Command::new(&editor)
            .arg(config_path)
            .status()
            .with_context(|| format!("Failed to open editor: {editor}"))?;

        if !status.success() {
            anyhow::bail!("Editor exited with non-zero status");
        }

        println!("✅ Configuration file edited successfully!");

        Ok(())
    }

    async fn run_memory_command(&self, action: &MemoryAction, config: &VyConfig) -> Result<()> {
        use vy_core::memory::MemoryEntry;
        use vy_core::vector_memory::VectorMemory;

        // Create vector memory instance with full config
        let mut vector_config = config.vector_memory.clone();
        vector_config.openai_api_key = config.llm_api_key.clone();

        println!("🧠 Connecting to memory system...");
        let vector_memory = match VectorMemory::new(vector_config).await {
            Ok(memory) => {
                println!("✅ Connected successfully!\n");
                memory
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to memory system: {e}");
                eprintln!("\n💡 Troubleshooting:");
                eprintln!("   • Check your internet connection");
                eprintln!("   • Verify Qdrant Cloud credentials: vy config list");
                eprintln!("   • Test the connection: vy mem test");
                anyhow::bail!("Memory system unavailable");
            }
        };

        match action {
            MemoryAction::Create { fact } => {
                println!("💾 Storing memory: {fact}");
                let memory_entry = MemoryEntry::new(
                    fact.clone(),
                    format!("manual_{}", chrono::Utc::now().timestamp()),
                );

                match vector_memory.store_memory(&memory_entry).await {
                    Ok(id) => {
                        println!("✅ Memory stored successfully!");
                        println!("   ID: {id}");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to store memory: {e}");
                    }
                }
            }

            MemoryAction::List => {
                println!("📋 Retrieving all memories...");
                match vector_memory.get_all_memories().await {
                    Ok(memories) => {
                        if memories.is_empty() {
                            println!("📭 No memories found.");
                        } else {
                            println!("Found {} memories:\n", memories.len());
                            for (i, memory) in memories.iter().enumerate() {
                                println!("{}. {}", i + 1, memory.fact);
                                println!(
                                    "   Source: {} | {}",
                                    memory.source,
                                    memory.timestamp.format("%Y-%m-%d %H:%M:%S")
                                );
                                println!();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to retrieve memories: {e}");
                    }
                }
            }

            MemoryAction::Search { query, limit } => {
                println!("🔍 Searching for: '{query}'");
                match vector_memory.search_memories(query, *limit).await {
                    Ok(results) => {
                        if results.is_empty() {
                            println!("📭 No matching memories found.");
                        } else {
                            println!("Found {} matching memories:\n", results.len());
                            for (i, memory) in results.iter().enumerate() {
                                println!("{}. {}", i + 1, memory.fact);
                                println!(
                                    "   Source: {} | {}",
                                    memory.source,
                                    memory.timestamp.format("%Y-%m-%d %H:%M:%S")
                                );
                                println!();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Search failed: {e}");
                    }
                }
            }

            MemoryAction::Remove { query, yes } => {
                // First, search for matching memories
                println!("🔍 Finding memories matching: '{query}'");
                let matches = match vector_memory.search_memories(query, 10).await {
                    Ok(results) => results,
                    Err(e) => {
                        eprintln!("❌ Search failed: {e}");
                        return Ok(());
                    }
                };

                if matches.is_empty() {
                    println!("📭 No matching memories found to remove.");
                    return Ok(());
                }

                // Show what will be removed
                println!("Found {} matching memories:", matches.len());
                for (i, memory) in matches.iter().enumerate() {
                    println!("  {}. {}", i + 1, memory.fact);
                }

                // Confirm deletion unless --yes flag is used
                if !yes {
                    print!("\n🗑️  Remove these memories? [y/N]: ");
                    std::io::Write::flush(&mut std::io::stdout())?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if !input.trim().to_lowercase().starts_with('y') {
                        println!("❌ Removal cancelled.");
                        return Ok(());
                    }
                }

                // Remove memories by timestamp (delete each individually)
                let total_matches = matches.len();
                println!("🗑️  Removing {total_matches} memories...");
                let mut removed_count = 0;
                for memory in &matches {
                    match vector_memory.delete_memory(memory.timestamp).await {
                        Ok(true) => removed_count += 1,
                        Ok(false) => {
                            println!(
                                "⚠️  Memory not found (may have been already deleted): {}",
                                memory.fact
                            );
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to remove memory '{}': {}", memory.fact, e);
                        }
                    }
                }
                println!("✅ Removed {removed_count} out of {total_matches} memories!");
            }

            MemoryAction::Stats => {
                println!("📊 Getting memory statistics...");
                match vector_memory.get_stats().await {
                    Ok(stats) => {
                        println!("Memory System Statistics:");
                        println!("{}", stats.to_display_string());
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to get statistics: {e}");
                    }
                }
            }

            MemoryAction::Test => {
                println!("🧪 Testing memory system connection...");

                // Test 1: Basic stats
                match vector_memory.get_stats().await {
                    Ok(stats) => {
                        println!("✅ Connection test passed!");
                        println!("   Total memories: {}", stats.total_entries);
                    }
                    Err(e) => {
                        println!("❌ Connection test failed: {e}");
                        return Ok(());
                    }
                }

                // Test 2: Store and retrieve a test memory
                let test_fact = format!(
                    "Connection test at {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
                );
                let test_memory =
                    MemoryEntry::new(test_fact.clone(), "connection_test".to_string());

                println!("\n🧪 Testing memory storage...");
                match vector_memory.store_memory(&test_memory).await {
                    Ok(_) => {
                        println!("✅ Test memory stored successfully!");

                        // Try to search for it
                        match vector_memory.search_memories("connection test", 1).await {
                            Ok(results) => {
                                if !results.is_empty() {
                                    println!("✅ Test memory retrieved successfully!");
                                    println!("   🎉 Memory system is working correctly!");
                                } else {
                                    println!("⚠️  Test memory stored but not found in search");
                                }
                            }
                            Err(e) => {
                                println!("⚠️  Test memory stored but search failed: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Test memory storage failed: {e}");
                    }
                }
            }
        }

        Ok(())
    }
}

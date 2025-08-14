//! Memory management commands for Vy CLI
//!
//! This module provides commands for interacting with Vy's long-term memory system,
//! including viewing, searching, and managing stored memories.

use anyhow::{Result, anyhow};
use clap::Subcommand;
use directories::ProjectDirs;
use rig::providers::openai::{Client, EmbeddingModel};
use std::path::PathBuf;
use vy::memory::{
    EmbeddingProvider, Memory, MemoryManager, MemoryQuery, MemoryStats, MemoryType,
    MockEmbeddingProvider, RigEmbeddingProvider, SqliteMemoryStore,
};

#[derive(Subcommand, Debug)]
pub enum MemoryCommand {
    /// Show memory statistics
    Stats,
    /// List stored memories
    List {
        /// Memory type filter
        #[arg(short, long)]
        memory_type: Option<String>,
        /// Maximum number of memories to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Search memories
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "5")]
        limit: usize,
        /// Minimum similarity threshold
        #[arg(short, long, default_value = "0.6")]
        threshold: f32,
    },
    /// Add a memory manually
    Add {
        /// Memory type (fact, opinion, personal, etc.)
        #[arg(short, long, default_value = "fact")]
        memory_type: String,
        /// Memory content
        content: String,
        /// Entities involved (comma-separated)
        #[arg(short, long)]
        entities: Option<String>,
        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// Delete a memory by ID
    Delete {
        /// Memory ID to delete
        id: String,
    },
    /// Clear all memories
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Export memories to JSON
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show memory details by ID
    Show {
        /// Memory ID
        id: String,
    },
}

impl MemoryCommand {
    pub async fn run(self, api_key: Option<String>) -> Result<()> {
        let (mut store, embeddings) = Self::create_memory_manager(api_key).await?;

        match self {
            MemoryCommand::Stats => self.show_stats(&store, &embeddings).await?,
            MemoryCommand::List { memory_type, limit } => {
                self.list_memories(&store, &embeddings, memory_type, limit)
                    .await?
            }
            MemoryCommand::Search {
                query,
                limit,
                threshold,
            } => {
                self.search_memories(&store, &embeddings, query, limit, threshold)
                    .await?
            }
            MemoryCommand::Add {
                memory_type,
                content,
                entities,
                tags,
            } => {
                self.add_memory(
                    &mut store,
                    &embeddings,
                    memory_type,
                    content,
                    entities,
                    tags,
                )
                .await?
            }
            MemoryCommand::Delete { id } => self.delete_memory(&mut store, &embeddings, id).await?,
            MemoryCommand::Clear { yes } => {
                self.clear_memories(&mut store, &embeddings, yes).await?
            }
            MemoryCommand::Export { output } => {
                self.export_memories(&store, &embeddings, output).await?
            }
            MemoryCommand::Show { id } => self.show_memory(&store, &embeddings, id).await?,
        }

        Ok(())
    }

    async fn create_memory_manager(
        api_key: Option<String>,
    ) -> Result<(SqliteMemoryStore, Box<dyn EmbeddingProvider>)> {
        // Get data directory
        let data_dir = ProjectDirs::from("com", "vy", "vy")
            .ok_or_else(|| anyhow!("Could not determine data directory"))?
            .data_dir()
            .to_path_buf();

        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("memories.db");

        // Create storage
        let store = SqliteMemoryStore::new(&db_path).await?;

        // Create embedding provider
        let embeddings: Box<dyn EmbeddingProvider> = if let Some(key) = api_key {
            let client = Client::new(&key);
            let model = client.embedding_model(EmbeddingModel::Text3Small);
            Box::new(RigEmbeddingProvider::new(model, 1536))
        } else {
            // Use mock provider if no API key
            println!("⚠️  No API key provided, using mock embeddings (limited functionality)");
            Box::new(MockEmbeddingProvider::new(256))
        };

        Ok((store, embeddings))
    }

    async fn show_stats(
        &self,
        store: &SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
    ) -> Result<()> {
        let manager = MemoryManager::new(store, embeddings.as_ref());
        let stats = manager.get_memory_stats().await?;

        println!("🧠 Memory Statistics");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 Total Memories: {}", stats.total_memories);
        println!();

        if stats.total_memories > 0 {
            println!("🗂️  Memory Types:");
            println!("   📋 Facts:         {}", stats.fact_count);
            println!("   💭 Opinions:      {}", stats.opinion_count);
            println!("   👤 Personal:      {}", stats.personal_count);
            println!("   🔗 Relationships: {}", stats.relationship_count);
            println!("   💬 Conversations: {}", stats.conversation_count);
            println!("   📚 Knowledge:     {}", stats.knowledge_count);
        } else {
            println!("📭 No memories stored yet.");
            println!("💡 Try chatting with Vy to automatically create memories,");
            println!("   or use 'vy memory add' to manually add one.");
        }

        Ok(())
    }

    async fn list_memories(
        &self,
        store: &SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        memory_type_filter: Option<String>,
        limit: usize,
    ) -> Result<()> {
        let manager = MemoryManager::new(store, embeddings.as_ref());
        let memory_type = if let Some(type_str) = memory_type_filter {
            Some(vec![Self::parse_memory_type(&type_str)?])
        } else {
            None
        };

        let query = MemoryQuery {
            memory_types: memory_type,
            limit,
            ..Default::default()
        };

        let memories = manager.search_memories(&query).await?;

        if memories.is_empty() {
            println!("📭 No memories found matching your criteria.");
            return Ok(());
        }

        println!("📚 Memories ({} found):", memories.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, memory) in memories.iter().enumerate() {
            let type_icon = Self::memory_type_icon(&memory.memory_type);
            let confidence_bar = Self::confidence_bar(memory.confidence);

            println!(
                "{}. {} [{}] {} {}",
                i + 1,
                type_icon,
                &memory.id[..8],
                confidence_bar,
                memory.created_at.format("%Y-%m-%d %H:%M")
            );

            // Truncate long content
            let content = if memory.content.len() > 80 {
                format!("{}...", &memory.content[..77])
            } else {
                memory.content.clone()
            };
            println!("   {}", content);

            if !memory.entities.is_empty() {
                println!("   🏷️  Entities: {}", memory.entities.join(", "));
            }
            if !memory.tags.is_empty() {
                println!("   📌 Tags: {}", memory.tags.join(", "));
            }

            if i < memories.len() - 1 {
                println!();
            }
        }

        Ok(())
    }

    async fn search_memories(
        &self,
        store: &SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        query: String,
        limit: usize,
        threshold: f32,
    ) -> Result<()> {
        let manager = MemoryManager::new(store, embeddings.as_ref());
        println!("🔍 Searching for: \"{}\"", query);

        let memory_query = MemoryQuery {
            content: query,
            limit,
            similarity_threshold: threshold,
            ..Default::default()
        };

        let memories = manager.search_memories(&memory_query).await?;

        if memories.is_empty() {
            println!("📭 No memories found matching your search.");
            println!("💡 Try adjusting your search terms or lowering the threshold.");
            return Ok(());
        }

        println!("📚 Found {} relevant memories:", memories.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, memory) in memories.iter().enumerate() {
            let type_icon = Self::memory_type_icon(&memory.memory_type);
            let confidence_bar = Self::confidence_bar(memory.confidence);

            println!(
                "{}. {} {} {} {}",
                i + 1,
                type_icon,
                confidence_bar,
                &memory.id[..8],
                memory.created_at.format("%Y-%m-%d")
            );
            println!("   {}", memory.content);

            if !memory.entities.is_empty() {
                println!("   🏷️  {}", memory.entities.join(", "));
            }

            if i < memories.len() - 1 {
                println!();
            }
        }

        Ok(())
    }

    async fn add_memory(
        &self,
        store: &mut SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        memory_type: String,
        content: String,
        entities: Option<String>,
        tags: Option<String>,
    ) -> Result<()> {
        let mut manager = MemoryManager::new(store, embeddings.as_ref());
        let parsed_type = Self::parse_memory_type(&memory_type)?;

        let entity_vec = entities
            .map(|e| e.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let tag_vec = tags
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let memory = Memory::new(parsed_type, content, entity_vec).with_tags(tag_vec);

        manager.add_memory(memory.clone()).await?;

        let type_icon = Self::memory_type_icon(&memory.memory_type);
        println!("✅ {} Memory added successfully!", type_icon);
        println!("   ID: {}", memory.id);
        println!("   Content: {}", memory.content);

        if !memory.entities.is_empty() {
            println!("   Entities: {}", memory.entities.join(", "));
        }
        if !memory.tags.is_empty() {
            println!("   Tags: {}", memory.tags.join(", "));
        }

        Ok(())
    }

    async fn delete_memory(
        &self,
        store: &mut SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        id: String,
    ) -> Result<()> {
        let manager = MemoryManager::new(store, embeddings.as_ref());
        // First, try to show the memory to be deleted
        if let Ok(Some(memory)) = manager.store.get_memory(&id).await {
            let type_icon = Self::memory_type_icon(&memory.memory_type);
            println!("🗑️  About to delete memory:");
            println!("   {} [{}] {}", type_icon, &memory.id[..8], memory.content);

            print!("Are you sure? (y/N): ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("❌ Deletion cancelled.");
                return Ok(());
            }
        }

        let deleted = manager.store.delete_memory(&id).await?;

        if deleted {
            println!("✅ Memory deleted successfully.");
        } else {
            println!("❌ Memory with ID '{}' not found.", id);
        }

        Ok(())
    }

    async fn clear_memories(
        &self,
        store: &mut SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        yes: bool,
    ) -> Result<()> {
        let mut manager = MemoryManager::new(store, embeddings.as_ref());
        if !yes {
            print!("⚠️  This will delete ALL memories. Are you sure? (y/N): ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("❌ Clear operation cancelled.");
                return Ok(());
            }
        }

        manager.store.clear_memories().await?;
        println!("✅ All memories cleared.");

        Ok(())
    }

    async fn export_memories(
        &self,
        store: &SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        output: Option<PathBuf>,
    ) -> Result<()> {
        let manager = MemoryManager::new(store, embeddings.as_ref());
        let memories = manager.store.get_all_memories().await?;
        let json = serde_json::to_string_pretty(&memories)?;

        match output {
            Some(path) => {
                std::fs::write(&path, json)?;
                println!(
                    "✅ Exported {} memories to {}",
                    memories.len(),
                    path.display()
                );
            }
            None => {
                println!("{}", json);
            }
        }

        Ok(())
    }

    async fn show_memory(
        &self,
        store: &SqliteMemoryStore,
        embeddings: &Box<dyn EmbeddingProvider>,
        id: String,
    ) -> Result<()> {
        let manager = MemoryManager::new(store, embeddings.as_ref());
        let memory = manager.store.get_memory(&id).await?;

        match memory {
            Some(memory) => {
                let type_icon = Self::memory_type_icon(&memory.memory_type);
                let confidence_bar = Self::confidence_bar(memory.confidence);

                println!("🔍 Memory Details");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📄 ID: {}", memory.id);
                println!("{} Type: {:?}", type_icon, memory.memory_type);
                println!(
                    "{} Confidence: {:.1}%",
                    confidence_bar,
                    memory.confidence * 100.0
                );
                println!(
                    "📅 Created: {}",
                    memory.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!(
                    "📝 Updated: {}",
                    memory.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!();
                println!("💬 Content:");
                println!("   {}", memory.content);

                if !memory.entities.is_empty() {
                    println!();
                    println!("🏷️  Entities: {}", memory.entities.join(", "));
                }

                if !memory.tags.is_empty() {
                    println!("📌 Tags: {}", memory.tags.join(", "));
                }

                if !memory.metadata.is_empty() {
                    println!();
                    println!("🔧 Metadata:");
                    for (key, value) in &memory.metadata {
                        if key != "embedding" {
                            // Don't show embeddings
                            println!("   {}: {}", key, value);
                        }
                    }
                    if memory.metadata.contains_key("embedding") {
                        println!(
                            "   embedding: [vector data - {} dimensions]",
                            memory
                                .metadata
                                .get("embedding")
                                .and_then(|e| serde_json::from_str::<Vec<f32>>(e).ok())
                                .map(|v| v.len())
                                .unwrap_or(0)
                        );
                    }
                }
            }
            None => {
                println!("❌ Memory with ID '{}' not found.", id);
            }
        }

        Ok(())
    }

    fn parse_memory_type(type_str: &str) -> Result<MemoryType> {
        match type_str.to_lowercase().as_str() {
            "fact" => Ok(MemoryType::Fact),
            "opinion" => Ok(MemoryType::Opinion),
            "personal" => Ok(MemoryType::Personal),
            "relationship" => Ok(MemoryType::Relationship),
            "conversation" => Ok(MemoryType::Conversation),
            "knowledge" => Ok(MemoryType::Knowledge),
            _ => Err(anyhow!(
                "Invalid memory type. Valid types: fact, opinion, personal, relationship, conversation, knowledge"
            )),
        }
    }

    fn memory_type_icon(memory_type: &MemoryType) -> &'static str {
        match memory_type {
            MemoryType::Fact => "📋",
            MemoryType::Opinion => "💭",
            MemoryType::Personal => "👤",
            MemoryType::Relationship => "🔗",
            MemoryType::Conversation => "💬",
            MemoryType::Knowledge => "📚",
        }
    }

    fn confidence_bar(confidence: f32) -> String {
        let filled = (confidence * 10.0) as usize;
        let empty = 10 - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

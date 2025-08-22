//! Vector Memory Demo
//!
//! This example demonstrates how to use Vy's vector memory system with Qdrant.
//!
//! Prerequisites:
//! 1. Start a local Qdrant instance: `docker run -p 6334:6334 qdrant/qdrant`
//! 2. Set your OpenAI API key: `export OPENAI_API_KEY=your_key_here`
//! 3. Run with: `cargo run --example vector_memory_demo`

use anyhow::Result;
use vy_core::memory::MemoryEntry;
use vy_core::vector_memory::{VectorMemory, VectorMemoryConfig};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("🚀 Vector Memory Demo Starting...\n");

    // Configure vector memory
    let vector_config = VectorMemoryConfig {
        qdrant_url: "http://localhost:6334".to_string(),
        qdrant_api_key: None, // For local Qdrant instance
        collection_name: "vy_demo_memories".to_string(),
        openai_api_key: std::env::var("OPENAI_API_KEY")
            .expect("Please set OPENAI_API_KEY environment variable"),
        embedding_model: "text-embedding-3-small".to_string(),
    };

    println!("📡 Connecting to Qdrant at {}...", vector_config.qdrant_url);

    // Initialize vector memory
    let vector_memory = match VectorMemory::new(vector_config).await {
        Ok(memory) => {
            println!("✅ Connected to Qdrant successfully!\n");
            memory
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to Qdrant: {}", e);
            eprintln!("💡 Make sure Qdrant is running: docker run -p 6334:6334 qdrant/qdrant");
            return Err(e);
        }
    };

    // Clear any existing demo data
    println!("🧹 Clearing existing demo memories...");
    vector_memory.clear_all().await?;
    println!("✅ Cleared!\n");

    // Demo memories to store
    let demo_memories = vec![
        MemoryEntry::new(
            "User works as a Senior Software Engineer at Google".to_string(),
            "demo_conversation_1".to_string(),
        ),
        MemoryEntry::new(
            "User loves hiking in the mountains on weekends".to_string(),
            "demo_conversation_2".to_string(),
        ),
        MemoryEntry::new(
            "User has a meeting with the product team on Friday at 2pm".to_string(),
            "demo_conversation_3".to_string(),
        ),
        MemoryEntry::new(
            "User is learning Rust programming language".to_string(),
            "demo_conversation_4".to_string(),
        ),
        MemoryEntry::new(
            "User prefers coffee over tea and visits the local café daily".to_string(),
            "demo_conversation_5".to_string(),
        ),
    ];

    // Store memories
    println!("💾 Storing demo memories...");
    for memory in &demo_memories {
        match vector_memory.store_memory(memory).await {
            Ok(id) => println!("  ✅ Stored: {} (ID: {})", memory.fact, id),
            Err(e) => println!("  ❌ Failed to store memory: {}", e),
        }
    }
    println!();

    // Demonstrate semantic search
    let search_queries = vec![
        "software engineering job",
        "outdoor activities",
        "upcoming meetings",
        "programming languages",
        "coffee preferences",
        "weekend plans",
    ];

    println!("🔍 Demonstrating semantic search...\n");

    for query in search_queries {
        println!("🔎 Searching for: '{}'", query);

        match vector_memory.search_memories(query, 3).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("  📭 No results found\n");
                } else {
                    for (i, memory) in results.iter().enumerate() {
                        println!("  {}. {}", i + 1, memory.fact);
                        println!(
                            "     Source: {} | Time: {}",
                            memory.source,
                            memory.timestamp.format("%Y-%m-%d %H:%M:%S")
                        );
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("  ❌ Search failed: {}\n", e);
            }
        }
    }

    // Get memory statistics
    println!("📊 Memory Statistics:");
    match vector_memory.get_stats().await {
        Ok(stats) => {
            println!("{}", stats.to_display_string());
        }
        Err(e) => {
            println!("❌ Failed to get stats: {}", e);
        }
    }

    println!("\n🎉 Demo completed successfully!");
    println!("\n💡 Try searching for different queries to see semantic similarity in action!");
    println!("💡 The vector database can find related concepts even with different wording!");

    Ok(())
}

//! Memory types for Vy
//!
//! Provides basic memory entry types for compatibility with vector memory system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: DateTime<Utc>,
    pub fact: String,
    pub source: String, // What conversation this came from
}

impl MemoryEntry {
    pub fn new(fact: String, source: String) -> Self {
        Self {
            timestamp: Utc::now(),
            fact,
            source,
        }
    }
}

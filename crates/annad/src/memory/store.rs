//! Memory storage and persistence.

use anyhow::Result;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::types::{EpisodicMemory, SemanticMemory};

const EPISODIC_MEMORY_FILE: &str = "/var/lib/anna/memory/episodic.json";
const SEMANTIC_MEMORY_FILE: &str = "/var/lib/anna/memory/semantic.json";

/// Complete memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    pub episodic: EpisodicMemory,
    pub semantic: SemanticMemory,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            episodic: EpisodicMemory::default(),
            semantic: SemanticMemory::default(),
        }
    }
}

impl MemoryStore {
    /// Load memory from disk
    pub fn load() -> Self {
        let episodic = Self::load_episodic().unwrap_or_default();
        let semantic = Self::load_semantic().unwrap_or_default();

        Self { episodic, semantic }
    }

    /// Save memory to disk
    pub fn save(&self) -> Result<()> {
        self.save_episodic()?;
        self.save_semantic()?;
        Ok(())
    }

    fn load_episodic() -> Result<EpisodicMemory> {
        let path = PathBuf::from(EPISODIC_MEMORY_FILE);
        if !path.exists() {
            return Ok(EpisodicMemory::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let memory: EpisodicMemory = serde_json::from_str(&content)?;
        debug!("Loaded {} episodic memories", memory.interactions.len());
        Ok(memory)
    }

    fn load_semantic() -> Result<SemanticMemory> {
        let path = PathBuf::from(SEMANTIC_MEMORY_FILE);
        if !path.exists() {
            return Ok(SemanticMemory::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let memory: SemanticMemory = serde_json::from_str(&content)?;
        debug!("Loaded {} semantic facts", memory.facts.len());
        Ok(memory)
    }

    fn save_episodic(&self) -> Result<()> {
        let path = PathBuf::from(EPISODIC_MEMORY_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.episodic)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn save_semantic(&self) -> Result<()> {
        let path = PathBuf::from(SEMANTIC_MEMORY_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.semantic)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get context for answering a query
    pub fn get_relevant_context(&self, query: &str) -> String {
        let mut context = String::new();

        // Find similar past interactions
        let similar = self.episodic.find_similar(query);
        if !similar.is_empty() {
            context.push_str("\n## Past Experience:\n");
            context.push_str("I remember similar situations:\n");

            for (i, interaction) in similar.iter().enumerate() {
                if i >= 3 { break; } // Only show top 3

                context.push_str(&format!("- Previous query: \"{}\"\n",
                    interaction.user_query.chars().take(100).collect::<String>()));

                if let Some(ref outcome) = interaction.outcome {
                    match outcome {
                        super::types::InteractionOutcome::Success { .. } => {
                            context.push_str("  Result: ✓ Solved successfully\n");
                        }
                        super::types::InteractionOutcome::Failure { reason } => {
                            context.push_str(&format!("  Result: ✗ Failed: {}\n", reason));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Add relevant facts
        let query_keywords: Vec<&str> = query.split_whitespace().collect();
        let relevant_facts: Vec<_> = self.semantic.facts.iter()
            .filter(|fact| {
                query_keywords.iter().any(|keyword| {
                    fact.statement.to_lowercase().contains(&keyword.to_lowercase())
                }) && fact.confidence > 0.6
            })
            .take(5)
            .collect();

        if !relevant_facts.is_empty() {
            context.push_str("\n## What I Know:\n");
            for fact in relevant_facts {
                context.push_str(&format!("- {} (confidence: {:.0}%)\n",
                    fact.statement, fact.confidence * 100.0));
            }
        }

        context
    }
}

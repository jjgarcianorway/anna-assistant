//! Anna stats persistence.
//! v0.2.7: Initial implementation - tracks RPG stats across sessions

use crate::status::RpgStats;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::debug;

/// Stats file path
fn stats_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".anna/stats.json")
    } else {
        PathBuf::from("/var/lib/anna/stats.json")
    }
}

/// Persistent stats that survive daemon restarts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistentStats {
    /// RPG stats
    pub rpg: RpgStats,
    /// When stats file was created
    pub created_at: Option<String>,
    /// Last updated
    pub updated_at: Option<String>,
    /// Total response times for average calculation
    pub total_response_time_ms: u64,
}

impl PersistentStats {
    /// Load stats from disk
    pub fn load() -> Result<Self> {
        let path = stats_path();
        if !path.exists() {
            debug!("No stats file found, creating new");
            let mut stats = Self::default();
            stats.created_at = Some(chrono::Utc::now().to_rfc3339());
            stats.rpg.installed_at = Some(chrono::Utc::now().to_rfc3339());
            stats.rpg.reliability = 1.0; // Start with 100% reliability
            stats.rpg.title = RpgStats::get_title(0);
            return Ok(stats);
        }

        let content = fs::read_to_string(&path)?;
        let stats: Self = serde_json::from_str(&content)?;
        Ok(stats)
    }

    /// Save stats to disk
    pub fn save(&self) -> Result<()> {
        let path = stats_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut stats = self.clone();
        stats.updated_at = Some(chrono::Utc::now().to_rfc3339());

        let content = serde_json::to_string_pretty(&stats)?;
        fs::write(&path, content)?;
        debug!("Stats saved to {:?}", path);
        Ok(())
    }

    /// Record a question answered
    pub fn record_answer(&mut self, response_time_ms: u64, answer_type: AnswerType) {
        self.rpg.total_questions += 1;

        match answer_type {
            AnswerType::Instant => self.rpg.instant_answers += 1,
            AnswerType::Memory => self.rpg.memory_answers += 1,
            AnswerType::Llm => self.rpg.llm_answers += 1,
        }

        // Update response times
        self.total_response_time_ms += response_time_ms;
        self.rpg.avg_response_ms = self.total_response_time_ms / self.rpg.total_questions;

        if self.rpg.fastest_response_ms == 0 || response_time_ms < self.rpg.fastest_response_ms {
            self.rpg.fastest_response_ms = response_time_ms;
        }
        if response_time_ms > self.rpg.slowest_response_ms {
            self.rpg.slowest_response_ms = response_time_ms;
        }

        // Recalculate XP
        self.rpg.calculate_xp();
    }

    /// Record a recipe learned
    pub fn record_recipe_learned(&mut self) {
        self.rpg.recipes_learned += 1;
        self.rpg.calculate_xp();
    }

    /// Update uptime
    pub fn update_uptime(&mut self, session_uptime_secs: u64) {
        self.rpg.total_uptime_secs += session_uptime_secs;
    }

    /// Get current RPG stats
    pub fn get_rpg_stats(&self) -> RpgStats {
        self.rpg.clone()
    }
}

/// Type of answer provided
#[derive(Debug, Clone, Copy)]
pub enum AnswerType {
    /// Fast-path or instant error response
    Instant,
    /// From memory/recipes
    Memory,
    /// Required LLM processing
    Llm,
}

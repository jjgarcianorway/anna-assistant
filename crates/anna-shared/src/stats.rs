//! Anna stats persistence.
//! v0.2.7: Initial implementation - tracks RPG stats across sessions
//! v0.3.21: Truthful stats contract - all numbers backed by audit trail

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
    /// Create fresh stats for new installation or reset.
    /// v0.3.28: Single source of truth for baseline stats - used by both
    /// load() (when no file exists) and reset_stats() (when clearing).
    /// This ensures XP baseline consistency per truth contract.
    pub fn fresh() -> Self {
        Self {
            rpg: RpgStats {
                reliability: 1.0, // Start with 100% reliability
                title: RpgStats::get_title(0), // "Novice Apprentice"
                installed_at: Some(chrono::Utc::now().to_rfc3339()),
                ..RpgStats::default()
            },
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: None,
            total_response_time_ms: 0,
        }
    }

    /// Load stats from disk
    pub fn load() -> Result<Self> {
        let path = stats_path();
        if !path.exists() {
            debug!("No stats file found, creating fresh");
            return Ok(Self::fresh());
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

        // v0.3.21: Log to audit trail for truthfulness
        let _ = StatsAudit::log(StatsAuditEntry::new(
            StatsEventType::AnswerProvided { answer_type },
            serde_json::json!({ "response_time_ms": response_time_ms }),
        ));
    }

    /// Record a recipe learned
    pub fn record_recipe_learned(&mut self) {
        self.rpg.recipes_learned += 1;
        self.rpg.calculate_xp();

        // v0.3.21: Log to audit trail
        let _ = StatsAudit::log(StatsAuditEntry::new(
            StatsEventType::RecipeLearned,
            serde_json::json!({}),
        ));
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnswerType {
    /// Fast-path or instant error response
    Instant,
    /// From memory/recipes
    Memory,
    /// Required LLM processing
    Llm,
}

// =============================================================================
// v0.3.21: Truthful Stats Contract
// =============================================================================

/// Audit trail path
fn audit_trail_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".anna/stats_audit.jsonl")
    } else {
        PathBuf::from("/var/lib/anna/stats_audit.jsonl")
    }
}

/// v0.3.21: Single audit event for stats verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsAuditEntry {
    /// Event timestamp (RFC3339)
    pub timestamp: String,
    /// Event type
    pub event_type: StatsEventType,
    /// Associated data
    pub data: serde_json::Value,
}

/// v0.3.21: Types of events that affect stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsEventType {
    /// Question asked
    QuestionAsked,
    /// Answer provided
    AnswerProvided { answer_type: AnswerType },
    /// Recipe learned
    RecipeLearned,
    /// Session started
    SessionStarted,
    /// Session ended
    SessionEnded { uptime_secs: u64 },
    /// Stats reset
    StatsReset { reason: String },
}

impl StatsAuditEntry {
    /// Create new audit entry
    pub fn new(event_type: StatsEventType, data: serde_json::Value) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
            data,
        }
    }
}

/// v0.3.21: Stats audit trail for truthfulness verification
pub struct StatsAudit;

impl StatsAudit {
    /// Append an audit entry
    pub fn log(entry: StatsAuditEntry) -> Result<()> {
        use std::io::Write;
        let path = audit_trail_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let line = serde_json::to_string(&entry)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Read all audit entries
    pub fn read_all() -> Result<Vec<StatsAuditEntry>> {
        use std::io::BufRead;
        let path = audit_trail_path();

        if !path.exists() {
            return Ok(vec![]);
        }

        let file = fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<StatsAuditEntry>(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Verify stats match audit trail
    pub fn verify_stats(stats: &PersistentStats) -> StatsVerification {
        let entries = Self::read_all().unwrap_or_default();

        let mut audit_questions: u64 = 0;
        let mut audit_instant: u64 = 0;
        let mut audit_memory: u64 = 0;
        let mut audit_llm: u64 = 0;
        let mut audit_recipes: u32 = 0;

        for entry in &entries {
            match &entry.event_type {
                StatsEventType::QuestionAsked => audit_questions += 1,
                StatsEventType::AnswerProvided { answer_type } => match answer_type {
                    AnswerType::Instant => audit_instant += 1,
                    AnswerType::Memory => audit_memory += 1,
                    AnswerType::Llm => audit_llm += 1,
                },
                StatsEventType::RecipeLearned => audit_recipes += 1,
                StatsEventType::StatsReset { .. } => {
                    // Reset counters on reset event
                    audit_questions = 0;
                    audit_instant = 0;
                    audit_memory = 0;
                    audit_llm = 0;
                    audit_recipes = 0;
                }
                _ => {}
            }
        }

        // Verify against reported stats
        let total_answers = audit_instant + audit_memory + audit_llm;
        let mut discrepancies = Vec::new();

        if stats.rpg.total_questions != total_answers {
            discrepancies.push(format!(
                "total_questions: reported {} vs audited {}",
                stats.rpg.total_questions, total_answers
            ));
        }
        if stats.rpg.instant_answers != audit_instant {
            discrepancies.push(format!(
                "instant_answers: reported {} vs audited {}",
                stats.rpg.instant_answers, audit_instant
            ));
        }
        if stats.rpg.memory_answers != audit_memory {
            discrepancies.push(format!(
                "memory_answers: reported {} vs audited {}",
                stats.rpg.memory_answers, audit_memory
            ));
        }
        if stats.rpg.llm_answers != audit_llm {
            discrepancies.push(format!(
                "llm_answers: reported {} vs audited {}",
                stats.rpg.llm_answers, audit_llm
            ));
        }
        if stats.rpg.recipes_learned != audit_recipes {
            discrepancies.push(format!(
                "recipes_learned: reported {} vs audited {}",
                stats.rpg.recipes_learned, audit_recipes
            ));
        }

        StatsVerification {
            verified: discrepancies.is_empty(),
            audit_entries: entries.len(),
            discrepancies,
        }
    }

    /// Get audit trail summary
    pub fn summary() -> AuditSummary {
        let entries = Self::read_all().unwrap_or_default();
        let path = audit_trail_path();
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        AuditSummary {
            entry_count: entries.len(),
            file_size_bytes: file_size,
            first_entry: entries.first().map(|e| e.timestamp.clone()),
            last_entry: entries.last().map(|e| e.timestamp.clone()),
        }
    }

    /// Rotate audit trail (keep last N entries)
    pub fn rotate(keep_count: usize) -> Result<usize> {
        let entries = Self::read_all()?;
        if entries.len() <= keep_count {
            return Ok(0);
        }

        let removed = entries.len() - keep_count;
        let to_keep: Vec<_> = entries.into_iter().skip(removed).collect();

        // Rewrite the file
        let path = audit_trail_path();
        let content: String = to_keep
            .iter()
            .map(|e| serde_json::to_string(e).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, content)?;

        Ok(removed)
    }
}

/// v0.3.21: Result of stats verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsVerification {
    /// Whether stats match audit trail
    pub verified: bool,
    /// Number of audit entries checked
    pub audit_entries: usize,
    /// List of discrepancies found
    pub discrepancies: Vec<String>,
}

/// v0.3.21: Audit trail summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Number of audit entries
    pub entry_count: usize,
    /// File size in bytes
    pub file_size_bytes: u64,
    /// Timestamp of first entry
    pub first_entry: Option<String>,
    /// Timestamp of last entry
    pub last_entry: Option<String>,
}

/// v0.3.21: XP formula documentation
pub mod xp_formula {
    //! XP Formula Documentation - Truthful Stats Contract
    //!
    //! The XP formula is transparent and verifiable. All values are derived
    //! from real events tracked in the audit trail.
    //!
    //! ## Formula Components
    //!
    //! 1. **Questions XP** (0-50): `log2(total_questions) * 10`
    //!    - Each doubling of questions adds ~10 XP
    //!    - 1 question = 0 XP, 2 = 10, 4 = 20, 8 = 30, 16 = 40, 32 = 50
    //!
    //! 2. **Efficiency Bonus** (0-20): `(instant + memory) / total * 20`
    //!    - Rewards answering without LLM
    //!    - 100% efficiency = 20 XP bonus
    //!
    //! 3. **Recipe Bonus** (0-20): `min(recipes_learned, 20)`
    //!    - 1 XP per recipe, capped at 20
    //!
    //! 4. **Reliability Multiplier** (0.5-1.0): `0.5 + reliability * 0.5`
    //!    - Low reliability (0%) = 0.5x multiplier
    //!    - High reliability (100%) = 1.0x multiplier
    //!
    //! ## Final Formula
    //!
    //! ```text
    //! XP = min(100, (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult)
    //! ```
    //!
    //! ## Titles by XP
    //!
    //! | XP Range | Title |
    //! |----------|-------|
    //! | 0-4 | Novice Apprentice |
    //! | 5-9 | Eager Learner |
    //! | 10-19 | Junior Technician |
    //! | 20-29 | Curious Explorer |
    //! | 30-39 | Competent Assistant |
    //! | 40-49 | Skilled Operator |
    //! | 50-59 | Senior Specialist |
    //! | 60-69 | Expert Analyst |
    //! | 70-79 | Master Troubleshooter |
    //! | 80-89 | IT Sage |
    //! | 90-94 | System Whisperer |
    //! | 95-99 | Arch Wizard |
    //! | 100 | Omniscient Oracle |

    /// Calculate XP from components (for verification)
    pub fn calculate_xp(
        total_questions: u64,
        instant_answers: u64,
        memory_answers: u64,
        recipes_learned: u32,
        reliability: f32,
    ) -> u32 {
        // Questions XP: logarithmic scaling
        let questions_xp = if total_questions > 0 {
            (total_questions as f64).log2() * 10.0
        } else {
            0.0
        };

        // Efficiency bonus
        let efficiency = if total_questions > 0 {
            (instant_answers + memory_answers) as f64 / total_questions as f64
        } else {
            0.0
        };
        let efficiency_bonus = efficiency * 20.0;

        // Recipe bonus (capped at 20)
        let recipe_bonus = (recipes_learned as f64).min(20.0);

        // Reliability multiplier
        let reliability_mult = 0.5 + (reliability as f64 * 0.5);

        // Final calculation
        let raw_xp = (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult;
        (raw_xp as u32).min(100)
    }

    /// Explain XP calculation for transparency
    pub fn explain_xp(
        total_questions: u64,
        instant_answers: u64,
        memory_answers: u64,
        recipes_learned: u32,
        reliability: f32,
    ) -> String {
        let questions_xp = if total_questions > 0 {
            (total_questions as f64).log2() * 10.0
        } else {
            0.0
        };

        let efficiency = if total_questions > 0 {
            (instant_answers + memory_answers) as f64 / total_questions as f64
        } else {
            0.0
        };
        let efficiency_bonus = efficiency * 20.0;

        let recipe_bonus = (recipes_learned as f64).min(20.0);
        let reliability_mult = 0.5 + (reliability as f64 * 0.5);

        let raw_xp = (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult;
        let final_xp = (raw_xp as u32).min(100);

        format!(
            "XP Breakdown:\n\
             - Questions ({} total): {:.1} XP (log2 * 10)\n\
             - Efficiency ({:.0}% solved alone): {:.1} XP\n\
             - Recipes ({} learned): {:.1} XP\n\
             - Reliability ({:.0}%): {:.2}x multiplier\n\
             - Raw: ({:.1} + {:.1} + {:.1}) * {:.2} = {:.1}\n\
             - Final: {} XP",
            total_questions, questions_xp,
            efficiency * 100.0, efficiency_bonus,
            recipes_learned, recipe_bonus,
            reliability * 100.0, reliability_mult,
            questions_xp, efficiency_bonus, recipe_bonus, reliability_mult, raw_xp,
            final_xp
        )
    }
}

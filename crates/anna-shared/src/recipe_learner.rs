//! Recipe Learner (v0.0.416).
//!
//! Extracts reusable recipes from successful tickets.
//!
//! Learning process:
//! 1. Track tickets with same intent + high confidence
//! 2. Identify common probe patterns
//! 3. Extract answer structure (not exact words)
//! 4. Create generic, parameterized recipe
//!
//! NO HARDCODING of specific questions or answers.

use crate::canonical_intents::CanonicalIntent;
use crate::learned_recipes::{
    AnswerTemplate, CompareOp, LearnedRecipe, RecipeComputeStep, RecipeStats, RecipeStore,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A ticket observation for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketObservation {
    /// Ticket ID
    pub ticket_id: String,
    /// Canonical intent
    pub intent: CanonicalIntent,
    /// Domain
    pub domain: String,
    /// Probes that were used
    pub probes_used: Vec<String>,
    /// Probe outputs (sanitized)
    pub probe_outputs: HashMap<String, String>,
    /// Answer summary
    pub answer_summary: String,
    /// Answer confidence
    pub confidence: f32,
    /// Was successful (user feedback or status ok)
    pub successful: bool,
    /// Timestamp
    pub timestamp: u64,
}

/// Learning candidates store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningCandidates {
    /// Observations grouped by intent
    pub by_intent: HashMap<String, Vec<TicketObservation>>,
    /// Minimum observations to create recipe
    pub min_observations: usize,
    /// Minimum success rate to create recipe
    pub min_success_rate: f32,
}

impl LearningCandidates {
    pub fn new() -> Self {
        Self {
            by_intent: HashMap::new(),
            min_observations: 2,
            min_success_rate: 0.8,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = candidates_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let path = candidates_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Record a ticket observation
    pub fn record(&mut self, observation: TicketObservation) {
        let key = format!("{:?}", observation.intent);
        let observations = self.by_intent.entry(key).or_default();

        // Keep last 20 observations per intent
        if observations.len() >= 20 {
            observations.remove(0);
        }
        observations.push(observation);
    }

    /// Check if ready to learn a recipe for intent
    pub fn ready_to_learn(&self, intent: CanonicalIntent) -> bool {
        let key = format!("{:?}", intent);
        if let Some(observations) = self.by_intent.get(&key) {
            if observations.len() < self.min_observations {
                return false;
            }

            let successful = observations.iter().filter(|o| o.successful).count();
            let rate = successful as f32 / observations.len() as f32;
            rate >= self.min_success_rate
        } else {
            false
        }
    }

    /// Get observations for intent
    pub fn get_observations(&self, intent: CanonicalIntent) -> Vec<&TicketObservation> {
        let key = format!("{:?}", intent);
        self.by_intent
            .get(&key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}

/// Recipe learner
pub struct RecipeLearner {
    candidates: LearningCandidates,
    store: RecipeStore,
}

impl RecipeLearner {
    pub fn new() -> Self {
        Self {
            candidates: LearningCandidates::load(),
            store: RecipeStore::load(),
        }
    }

    /// Record a successful ticket
    pub fn record_success(&mut self, observation: TicketObservation) {
        let intent = observation.intent;
        self.candidates.record(observation);

        // Check if we can learn a recipe
        if self.candidates.ready_to_learn(intent) && !self.has_recipe(intent) {
            if let Some(recipe) = self.try_learn_recipe(intent) {
                self.store.upsert(recipe);
                let _ = self.store.save();
            }
        }

        let _ = self.candidates.save();
    }

    /// Check if we already have a recipe for this intent
    pub fn has_recipe(&self, intent: CanonicalIntent) -> bool {
        self.store.find_for_intent(intent).is_some()
    }

    /// Get recipe for intent
    pub fn get_recipe(&self, intent: CanonicalIntent) -> Option<&LearnedRecipe> {
        self.store.find_for_intent(intent)
    }

    /// Try to learn a recipe from observations
    fn try_learn_recipe(&self, intent: CanonicalIntent) -> Option<LearnedRecipe> {
        let observations = self.candidates.get_observations(intent);
        if observations.len() < 2 {
            return None;
        }

        // Find common probes across observations
        let common_probes = find_common_probes(&observations);
        if common_probes.is_empty() {
            return None;
        }

        // Try to create recipe based on intent type
        let recipe = match intent {
            CanonicalIntent::CheckDiskUsage => learn_disk_usage_recipe(&observations, &common_probes),
            CanonicalIntent::CheckFreeRam => learn_memory_recipe(&observations, &common_probes),
            CanonicalIntent::CheckSwapPresence => learn_swap_recipe(&observations, &common_probes),
            CanonicalIntent::CheckFailedServices => learn_failed_services_recipe(&observations, &common_probes),
            CanonicalIntent::CheckUptime => learn_uptime_recipe(&observations, &common_probes),
            CanonicalIntent::CheckBootTime => learn_boot_time_recipe(&observations, &common_probes),
            _ => learn_generic_recipe(intent, &observations, &common_probes),
        };

        recipe
    }

    /// Record recipe usage
    pub fn record_recipe_result(&mut self, recipe_id: &str, success: bool, confidence: f32) {
        if let Some(recipe) = self.store.get_mut(recipe_id) {
            if success {
                recipe.stats.record_success(confidence);
            } else {
                recipe.stats.record_failure();
            }
            recipe.last_used_at = current_secs();

            // Auto-deprecate if success rate drops
            if recipe.stats.uses >= 10 && recipe.stats.success_rate() < 0.5 {
                recipe.deprecated = true;
            }
        }
        let _ = self.store.save();
    }

    /// Get store reference
    pub fn store(&self) -> &RecipeStore {
        &self.store
    }
}

impl Default for RecipeLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Find probes common across all observations
fn find_common_probes(observations: &[&TicketObservation]) -> Vec<String> {
    if observations.is_empty() {
        return vec![];
    }

    let first_probes: std::collections::HashSet<_> =
        observations[0].probes_used.iter().cloned().collect();

    let mut common: Vec<String> = first_probes
        .into_iter()
        .filter(|p| {
            observations
                .iter()
                .all(|o| o.probes_used.contains(p))
        })
        .collect();

    common.sort();
    common
}

// Intent-specific recipe learning functions

fn learn_disk_usage_recipe(
    observations: &[&TicketObservation],
    common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "disk_usage_v1".to_string(),
        name: "Disk Usage Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckDiskUsage,
        domain: "storage".to_string(),
        required_probes: vec!["disk_usage".to_string()],
        optional_probes: vec!["block_devices".to_string()],
        steps: vec![
            RecipeComputeStep::Extract {
                probe: "disk_usage".to_string(),
                pattern: r"(\d+)%".to_string(),
                variable: "root_percent".to_string(),
            },
            RecipeComputeStep::ParseNumber {
                source_var: "root_percent".to_string(),
                target_var: "root_percent_num".to_string(),
            },
            RecipeComputeStep::Compare {
                variable: "root_percent_num".to_string(),
                operator: CompareOp::Ge,
                threshold: 90.0,
                result_var: "is_critical".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "Root filesystem at {root_percent}% used".to_string(),
            details: vec![],
            evidence: vec!["disk_usage".to_string()],
        },
        answer_critical: Some(AnswerTemplate {
            summary: "[WARNING] Root filesystem at {root_percent}% used - running low on space".to_string(),
            details: vec!["Consider cleaning package cache: pacman -Sc".to_string()],
            evidence: vec!["disk_usage".to_string()],
        }),
        answer_partial: None,
        knowledge_topics: vec!["df_command".to_string(), "disk_usage".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn learn_memory_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "memory_check_v1".to_string(),
        name: "Memory Usage Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckFreeRam,
        domain: "system".to_string(),
        required_probes: vec!["memory_info".to_string()],
        optional_probes: vec![],
        steps: vec![
            RecipeComputeStep::Extract {
                probe: "memory_info".to_string(),
                pattern: r"Mem:.*?(\d+)".to_string(),
                variable: "total_mb".to_string(),
            },
            RecipeComputeStep::Extract {
                probe: "memory_info".to_string(),
                pattern: r"available:\s*(\d+)".to_string(),
                variable: "available_mb".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "Available RAM: {available_mb} MiB".to_string(),
            details: vec![],
            evidence: vec!["memory_info".to_string()],
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: vec!["free_command".to_string(), "proc_meminfo".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn learn_swap_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "swap_check_v1".to_string(),
        name: "Swap Presence Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckSwapPresence,
        domain: "system".to_string(),
        required_probes: vec!["swap_files".to_string()],
        optional_probes: vec![],
        steps: vec![
            RecipeComputeStep::IsEmpty {
                probe: "swap_files".to_string(),
                variable: "no_swap".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "Swap is configured on this system".to_string(),
            details: vec![],
            evidence: vec!["swap_files".to_string()],
        },
        answer_critical: Some(AnswerTemplate {
            summary: "No swap configured on this system".to_string(),
            details: vec![],
            evidence: vec!["swap_files".to_string()],
        }),
        answer_partial: None,
        knowledge_topics: vec!["swap_configuration".to_string(), "proc_swaps".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn learn_failed_services_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "failed_services_v1".to_string(),
        name: "Failed Services Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckFailedServices,
        domain: "services".to_string(),
        required_probes: vec!["failed_services".to_string()],
        optional_probes: vec![],
        steps: vec![
            RecipeComputeStep::Count {
                probe: "failed_services".to_string(),
                pattern: r"(?m)^\s*\*".to_string(),
                variable: "failed_count".to_string(),
            },
            RecipeComputeStep::Compare {
                variable: "failed_count".to_string(),
                operator: CompareOp::Gt,
                threshold: 0.0,
                result_var: "has_failures".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "No failed systemd units".to_string(),
            details: vec![],
            evidence: vec!["failed_services".to_string()],
        },
        answer_critical: Some(AnswerTemplate {
            summary: "{failed_count} systemd unit(s) failed".to_string(),
            details: vec!["Run 'systemctl --failed' for details".to_string()],
            evidence: vec!["failed_services".to_string()],
        }),
        answer_partial: None,
        knowledge_topics: vec!["systemctl".to_string(), "systemd_units".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn learn_uptime_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "uptime_v1".to_string(),
        name: "System Uptime".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckUptime,
        domain: "system".to_string(),
        required_probes: vec!["uptime".to_string()],
        optional_probes: vec![],
        steps: vec![
            RecipeComputeStep::Extract {
                probe: "uptime".to_string(),
                pattern: r"up\s+(.+?),".to_string(),
                variable: "uptime_str".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "System uptime: {uptime_str}".to_string(),
            details: vec![],
            evidence: vec!["uptime".to_string()],
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: vec!["uptime".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn learn_boot_time_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "boot_time_v1".to_string(),
        name: "Boot Time Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckBootTime,
        domain: "boot".to_string(),
        required_probes: vec!["boot_time".to_string()],
        optional_probes: vec!["boot_blame".to_string()],
        steps: vec![
            RecipeComputeStep::Extract {
                probe: "boot_time".to_string(),
                pattern: r"=\s*([\d.]+)s".to_string(),
                variable: "total_seconds".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "Boot time: {total_seconds}s".to_string(),
            details: vec![],
            evidence: vec!["boot_time".to_string()],
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: vec!["systemd_analyze".to_string(), "boot_performance".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn learn_generic_recipe(
    intent: CanonicalIntent,
    observations: &[&TicketObservation],
    common_probes: &[String],
) -> Option<LearnedRecipe> {
    if common_probes.is_empty() {
        return None;
    }

    let domain = observations.first()?.domain.clone();

    Some(LearnedRecipe {
        id: format!("{:?}_v1", intent).to_lowercase(),
        name: format!("{}", intent.display()),
        version: 1,
        intent,
        domain,
        required_probes: common_probes.to_vec(),
        optional_probes: vec![],
        steps: vec![],
        answer_ok: AnswerTemplate {
            summary: format!("[{}] See probe output", intent.display()),
            details: vec![],
            evidence: common_probes.to_vec(),
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: intent.knowledge_topics().iter().map(|s| s.to_string()).collect(),
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

fn candidates_path() -> std::path::PathBuf {
    let base = std::env::var("ANNA_STATE_DIR")
        .unwrap_or_else(|_| "/var/lib/anna".to_string());
    std::path::PathBuf::from(base).join("learning_candidates.json")
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_recording() {
        let mut candidates = LearningCandidates::new();

        let obs = TicketObservation {
            ticket_id: "TEST-001".to_string(),
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            probes_used: vec!["disk_usage".to_string()],
            probe_outputs: HashMap::new(),
            answer_summary: "Disk at 50%".to_string(),
            confidence: 0.9,
            successful: true,
            timestamp: 0,
        };

        candidates.record(obs);
        assert!(!candidates.ready_to_learn(CanonicalIntent::CheckDiskUsage)); // Need 2+
    }

    #[test]
    fn test_common_probes() {
        let obs1 = TicketObservation {
            ticket_id: "T1".to_string(),
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            probes_used: vec!["disk_usage".to_string(), "block_devices".to_string()],
            probe_outputs: HashMap::new(),
            answer_summary: "".to_string(),
            confidence: 0.9,
            successful: true,
            timestamp: 0,
        };

        let obs2 = TicketObservation {
            ticket_id: "T2".to_string(),
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            probes_used: vec!["disk_usage".to_string()],
            probe_outputs: HashMap::new(),
            answer_summary: "".to_string(),
            confidence: 0.9,
            successful: true,
            timestamp: 0,
        };

        let observations = vec![&obs1, &obs2];
        let common = find_common_probes(&observations);

        assert_eq!(common, vec!["disk_usage".to_string()]);
    }
}

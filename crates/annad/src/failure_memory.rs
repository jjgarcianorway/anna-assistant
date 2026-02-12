//! Failure Memory - Anna remembers how she fixed problems and auto-applies next time.
//!
//! Philosophy: If it failed before and I fixed it, just fix it again automatically!

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

const FAILURE_DB: &str = "/var/lib/anna/failure_memory.json";
const MIN_OCCURRENCES_FOR_AUTO: usize = 2; // Auto-fix after 2nd occurrence
const MAX_AUTO_FIX_AGE_DAYS: i64 = 30; // Only auto-fix if solution worked in last 30 days

/// A remembered failure and its solution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMemory {
    /// Failure signature (normalized error/issue)
    pub signature: String,
    /// Human-readable description
    pub description: String,
    /// Occurrences of this failure
    pub occurrences: Vec<FailureOccurrence>,
    /// Solutions that worked
    pub working_solutions: Vec<Solution>,
    /// Whether auto-fix is enabled
    pub auto_fix_enabled: bool,
    /// User preference for this specific failure
    pub user_preference: AutoFixPreference,
}

/// A specific occurrence of a failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureOccurrence {
    pub timestamp: DateTime<Utc>,
    pub error_details: String,
    pub solution_applied: Option<String>,
    pub solution_successful: bool,
}

/// A solution that worked for this failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    /// Solution ID
    pub id: String,
    /// Commands to execute
    pub commands: Vec<String>,
    /// Human description of what this does
    pub description: String,
    /// Times this solution worked
    pub success_count: usize,
    /// Times this solution failed
    pub failure_count: usize,
    /// Last time this solution worked
    pub last_success: Option<DateTime<Utc>>,
    /// Success rate (0.0-1.0)
    pub success_rate: f32,
}

/// User's preference for auto-fixing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AutoFixPreference {
    /// Auto-fix silently
    AutoFixSilent,
    /// Auto-fix but notify user
    AutoFixNotify,
    /// Always ask for permission
    AlwaysAsk,
    /// Never auto-fix this specific issue
    NeverAutoFix,
}

/// Failure memory database.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FailureDatabase {
    pub failures: HashMap<String, FailureMemory>,
    pub global_auto_fix: bool,
}

impl FailureDatabase {
    /// Load from disk.
    pub fn load() -> Self {
        let path = PathBuf::from(FAILURE_DB);
        if !path.exists() {
            return Self {
                failures: HashMap::new(),
                global_auto_fix: true, // Enabled by default
            };
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(FAILURE_DB);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Record a failure occurrence.
    pub fn record_failure(&mut self, error: &str, details: &str) -> String {
        let signature = generate_signature(error);

        if let Some(memory) = self.failures.get_mut(&signature) {
            memory.occurrences.push(FailureOccurrence {
                timestamp: Utc::now(),
                error_details: details.to_string(),
                solution_applied: None,
                solution_successful: false,
            });
        } else {
            let memory = FailureMemory {
                signature: signature.clone(),
                description: error.to_string(),
                occurrences: vec![FailureOccurrence {
                    timestamp: Utc::now(),
                    error_details: details.to_string(),
                    solution_applied: None,
                    solution_successful: false,
                }],
                working_solutions: Vec::new(),
                auto_fix_enabled: true,
                user_preference: AutoFixPreference::AutoFixNotify,
            };
            self.failures.insert(signature.clone(), memory);
        }

        signature
    }

    /// Record a successful solution.
    pub fn record_solution(
        &mut self,
        signature: &str,
        commands: Vec<String>,
        description: String,
    ) {
        if let Some(memory) = self.failures.get_mut(signature) {
            let solution_id = format!("{:x}", md5::compute(commands.join(";")));

            // Update last occurrence as fixed
            if let Some(last_occurrence) = memory.occurrences.last_mut() {
                last_occurrence.solution_applied = Some(solution_id.clone());
                last_occurrence.solution_successful = true;
            }

            // Update or add solution
            if let Some(solution) = memory
                .working_solutions
                .iter_mut()
                .find(|s| s.id == solution_id)
            {
                solution.success_count += 1;
                solution.last_success = Some(Utc::now());
                solution.success_rate = solution.success_count as f32
                    / (solution.success_count + solution.failure_count) as f32;
            } else {
                memory.working_solutions.push(Solution {
                    id: solution_id,
                    commands,
                    description,
                    success_count: 1,
                    failure_count: 0,
                    last_success: Some(Utc::now()),
                    success_rate: 1.0,
                });
            }
        }
    }

    /// Check if auto-fix should be applied for this failure.
    pub fn should_auto_fix(&self, signature: &str) -> Option<&Solution> {
        if !self.global_auto_fix {
            return None;
        }

        let memory = self.failures.get(signature)?;

        if memory.user_preference == AutoFixPreference::NeverAutoFix {
            return None;
        }

        if memory.user_preference == AutoFixPreference::AlwaysAsk
            && memory.occurrences.len() < MIN_OCCURRENCES_FOR_AUTO
        {
            return None;
        }

        // Find best solution (highest success rate, recent success)
        memory
            .working_solutions
            .iter()
            .filter(|s| {
                s.success_rate > 0.7
                    && s.last_success
                        .map(|dt| (Utc::now() - dt).num_days() < MAX_AUTO_FIX_AGE_DAYS)
                        .unwrap_or(false)
            })
            .max_by(|a, b| {
                a.success_rate
                    .partial_cmp(&b.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get auto-fix action for a failure.
    pub fn get_auto_fix_action(&self, signature: &str) -> Option<AutoFixAction> {
        let memory = self.failures.get(signature)?;
        let solution = self.should_auto_fix(signature)?;

        let action_type = match memory.user_preference {
            AutoFixPreference::AutoFixSilent => AutoFixActionType::Silent,
            AutoFixPreference::AutoFixNotify => AutoFixActionType::Notify,
            _ => AutoFixActionType::Ask,
        };

        Some(AutoFixAction {
            failure_description: memory.description.clone(),
            solution: solution.clone(),
            action_type,
            occurrence_count: memory.occurrences.len(),
        })
    }
}

/// Auto-fix action to take.
#[derive(Debug, Clone)]
pub struct AutoFixAction {
    pub failure_description: String,
    pub solution: Solution,
    pub action_type: AutoFixActionType,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoFixActionType {
    Silent,  // Fix without telling user
    Notify,  // Fix and notify user
    Ask,     // Ask user first
}

/// Generate failure signature from error message.
fn generate_signature(error: &str) -> String {
    let normalized = error
        .to_lowercase()
        .replace(|c: char| c.is_numeric(), "N") // Replace numbers
        .replace(|c: char| c.is_whitespace(), "_");

    // Extract key parts (service names, error types)
    let key_words: Vec<&str> = normalized
        .split('_')
        .filter(|w| w.len() > 3) // Skip short words
        .take(5) // First 5 meaningful words
        .collect();

    format!("{:x}", md5::compute(key_words.join("_")))
}

/// Check if this error matches a known failure.
pub fn check_for_known_failure(error: &str) -> Option<AutoFixAction> {
    let db = FailureDatabase::load();
    let signature = generate_signature(error);

    db.get_auto_fix_action(&signature)
}

/// Record a failure for future reference.
pub fn record_failure(error: &str, details: &str) -> String {
    let mut db = FailureDatabase::load();
    let signature = db.record_failure(error, details);

    if let Err(e) = db.save() {
        warn!("Failed to save failure memory: {}", e);
    }

    signature
}

/// Record that a solution worked.
pub fn record_successful_fix(signature: &str, commands: Vec<String>, description: String) {
    let mut db = FailureDatabase::load();
    db.record_solution(signature, commands, description);

    if let Err(e) = db.save() {
        info!("Recorded successful fix for {}", signature);
        if let Err(e) = db.save() {
            warn!("Failed to save failure memory: {}", e);
        }
    }
}

/// Apply auto-fix if available.
pub async fn apply_auto_fix(action: &AutoFixAction) -> Result<String> {
    info!(
        "Applying auto-fix for: {} (occurred {} times)",
        action.failure_description, action.occurrence_count
    );

    let mut results = Vec::new();

    for cmd in &action.solution.commands {
        match crate::core_loop::execute_command(cmd) {
            Ok(output) => results.push(format!("✓ {}", cmd)),
            Err(e) => {
                warn!("Auto-fix command failed: {}", e);
                results.push(format!("✗ {}: {}", cmd, e));
            }
        }
    }

    let result = results.join("\n");

    match action.action_type {
        AutoFixActionType::Silent => {
            info!("Auto-fix applied silently");
        }
        AutoFixActionType::Notify => {
            let notification = format!(
                "Auto-Fixed: {}\n\nSolution: {}\n\n{}\n\nOccurred {} times before, fix applied automatically.",
                action.failure_description,
                action.solution.description,
                result,
                action.occurrence_count
            );
            crate::telegram::notifier::push_notification(&notification);
        }
        AutoFixActionType::Ask => {
            // This shouldn't happen if we checked should_auto_fix properly
            warn!("Auto-fix applied but should have asked first");
        }
    }

    Ok(result)
}

/// Format auto-fix suggestion for user.
pub fn format_auto_fix_suggestion(action: &AutoFixAction) -> String {
    format!(
        "I've seen this problem {} times before and fixed it successfully.\n\n\
        Issue: {}\n\
        Solution: {}\n\
        Success rate: {:.0}%\n\n\
        Should I:\n\
        1. Fix it now (same solution as before)\n\
        2. Fix automatically next time (silent)\n\
        3. Fix automatically but notify me\n\
        4. Always ask for this specific issue\n\
        5. Never auto-fix this issue\n\n\
        What would you like?",
        action.occurrence_count,
        action.failure_description,
        action.solution.description,
        action.solution.success_rate * 100.0
    )
}

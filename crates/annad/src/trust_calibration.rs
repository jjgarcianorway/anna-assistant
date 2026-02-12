//! User Trust Calibration - Adjust autonomy based on user's comfort level.
//!
//! Philosophy: Earn trust gradually. Start cautious, become autonomous as trust builds.
//! NO HARDCODING: Evidence-based trust adjustment, not arbitrary levels.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Trust calibration state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustState {
    /// Overall trust level (0.0-1.0)
    pub trust_level: f32,
    /// Per-category trust levels
    pub category_trust: std::collections::HashMap<String, CategoryTrust>,
    /// User interaction history
    pub interaction_count: u32,
    /// Days since first interaction
    pub days_active: i64,
    /// Last updated
    pub last_updated: DateTime<Utc>,
    /// First interaction date
    pub first_interaction: DateTime<Utc>,
}

/// Trust level for a specific category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTrust {
    pub category: String,
    pub trust_level: f32, // 0.0-1.0
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub user_interventions: u32, // Times user stopped/modified Anna's action
    pub autonomy_level: AutonomyLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutonomyLevel {
    /// Ask for everything
    Cautious,
    /// Ask for risky operations only
    Moderate,
    /// Execute safe operations automatically
    Autonomous,
    /// Fully autonomous (high trust)
    FullyAutonomous,
}

impl Default for TrustState {
    fn default() -> Self {
        Self {
            trust_level: 0.3, // Start cautious
            category_trust: std::collections::HashMap::new(),
            interaction_count: 0,
            days_active: 0,
            last_updated: Utc::now(),
            first_interaction: Utc::now(),
        }
    }
}

impl TrustState {
    /// Load from disk.
    pub fn load() -> Self {
        let path = Self::storage_path();

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(mut state) = serde_json::from_str::<Self>(&contents) {
                // Update days_active
                state.days_active = (Utc::now() - state.first_interaction).num_days();
                return state;
            }
        }

        Self::default()
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("/var/lib/anna/trust_state.json")
    }

    /// Record an interaction.
    pub fn record_interaction(&mut self) {
        self.interaction_count += 1;
        self.days_active = (Utc::now() - self.first_interaction).num_days();
        self.last_updated = Utc::now();
    }

    /// Record successful operation.
    pub fn record_success(&mut self, category: &str) {
        let trust = self
            .category_trust
            .entry(category.to_string())
            .or_insert_with(|| CategoryTrust {
                category: category.to_string(),
                trust_level: 0.3,
                successful_operations: 0,
                failed_operations: 0,
                user_interventions: 0,
                autonomy_level: AutonomyLevel::Cautious,
            });

        trust.successful_operations += 1;
        trust.update_trust_level();
        let trust_level = trust.trust_level;

        self.update_overall_trust();
        self.last_updated = Utc::now();

        info!(
            "Trust calibration: {} success (trust: {:.0}%)",
            category,
            trust_level * 100.0
        );

        let _ = self.save();
    }

    /// Record failed operation.
    pub fn record_failure(&mut self, category: &str) {
        let trust = self
            .category_trust
            .entry(category.to_string())
            .or_insert_with(|| CategoryTrust {
                category: category.to_string(),
                trust_level: 0.3,
                successful_operations: 0,
                failed_operations: 0,
                user_interventions: 0,
                autonomy_level: AutonomyLevel::Cautious,
            });

        trust.failed_operations += 1;
        trust.update_trust_level();
        let trust_level = trust.trust_level;

        self.update_overall_trust();
        self.last_updated = Utc::now();

        info!(
            "Trust calibration: {} failure (trust: {:.0}%)",
            category,
            trust_level * 100.0
        );

        let _ = self.save();
    }

    /// Record user intervention (user stopped/modified action).
    pub fn record_intervention(&mut self, category: &str) {
        let trust = self
            .category_trust
            .entry(category.to_string())
            .or_insert_with(|| CategoryTrust {
                category: category.to_string(),
                trust_level: 0.3,
                successful_operations: 0,
                failed_operations: 0,
                user_interventions: 0,
                autonomy_level: AutonomyLevel::Cautious,
            });

        trust.user_interventions += 1;
        trust.update_trust_level();
        let trust_level = trust.trust_level;

        self.update_overall_trust();
        self.last_updated = Utc::now();

        info!(
            "Trust calibration: {} intervention (trust: {:.0}%)",
            category,
            trust_level * 100.0
        );

        let _ = self.save();
    }

    /// Update overall trust level.
    fn update_overall_trust(&mut self) {
        if self.category_trust.is_empty() {
            return;
        }

        let avg_trust: f32 = self.category_trust.values().map(|t| t.trust_level).sum::<f32>()
            / self.category_trust.len() as f32;

        // Weight by experience (more interactions = more stable trust)
        let experience_weight = (self.interaction_count as f32 / 100.0).min(1.0);

        // Blend with historical trust
        self.trust_level = (avg_trust * experience_weight) + (self.trust_level * (1.0 - experience_weight));

        // Adjust for time active (longer usage = higher base trust)
        if self.days_active > 30 {
            self.trust_level = (self.trust_level + 0.1).min(1.0);
        }
    }

    /// Get recommended autonomy level for a category.
    pub fn get_autonomy_level(&self, category: &str) -> AutonomyLevel {
        if let Some(trust) = self.category_trust.get(category) {
            trust.autonomy_level
        } else {
            // No history for this category, use overall trust
            if self.trust_level > 0.8 {
                AutonomyLevel::Autonomous
            } else if self.trust_level > 0.5 {
                AutonomyLevel::Moderate
            } else {
                AutonomyLevel::Cautious
            }
        }
    }

    /// Should ask user before taking action?
    pub fn should_ask_permission(&self, category: &str, action_risk: RiskLevel) -> bool {
        let autonomy = self.get_autonomy_level(category);

        match autonomy {
            AutonomyLevel::Cautious => true, // Always ask
            AutonomyLevel::Moderate => action_risk != RiskLevel::Safe, // Ask for non-safe actions
            AutonomyLevel::Autonomous => action_risk == RiskLevel::High, // Ask only for high-risk
            AutonomyLevel::FullyAutonomous => false, // Never ask (user opted in)
        }
    }

    /// Get trust adjustment recommendation.
    pub fn get_trust_recommendation(&self) -> Option<String> {
        // Check if any category has high trust and could be more autonomous
        for (category, trust) in &self.category_trust {
            if trust.trust_level > 0.85 && trust.autonomy_level != AutonomyLevel::FullyAutonomous {
                return Some(format!(
                    "You've had {} successful {} operations with no failures. Would you like me to be more autonomous with {}?",
                    trust.successful_operations, category, category
                ));
            }
        }

        // Check overall trust increase
        if self.trust_level > 0.75 && self.interaction_count > 50 {
            return Some(format!(
                "After {} interactions over {} days with {:.0}% trust level, would you like me to be more autonomous overall?",
                self.interaction_count, self.days_active, self.trust_level * 100.0
            ));
        }

        None
    }
}

impl CategoryTrust {
    /// Update trust level based on performance.
    fn update_trust_level(&mut self) {
        let total_ops = self.successful_operations + self.failed_operations;

        if total_ops == 0 {
            return;
        }

        let success_rate = self.successful_operations as f32 / total_ops as f32;

        // Calculate base trust from success rate
        let mut trust = success_rate;

        // Penalize for user interventions
        if self.user_interventions > 0 {
            let intervention_penalty = (self.user_interventions as f32 / total_ops as f32) * 0.5;
            trust -= intervention_penalty;
        }

        // Require minimum operations for high trust
        if total_ops < 5 {
            trust = trust.min(0.5); // Cap at 50% until proven
        } else if total_ops < 10 {
            trust = trust.min(0.7); // Cap at 70% until well-proven
        }

        self.trust_level = trust.max(0.0).min(1.0);

        // Update autonomy level based on trust
        self.autonomy_level = if self.trust_level > 0.85 && total_ops >= 10 {
            AutonomyLevel::FullyAutonomous
        } else if self.trust_level > 0.70 && total_ops >= 5 {
            AutonomyLevel::Autonomous
        } else if self.trust_level > 0.50 {
            AutonomyLevel::Moderate
        } else {
            AutonomyLevel::Cautious
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,      // No risk (read-only, safe cleanup)
    Low,       // Minimal risk (restart service, safe config changes)
    Medium,    // Moderate risk (package updates, file edits)
    High,      // High risk (system-wide changes, data deletion)
}

/// Get trust report for user.
pub fn get_trust_report() -> String {
    let state = TrustState::load();

    let mut report = format!(
        "Trust Calibration Report\n\n\
        Overall Trust: {:.0}%\n\
        Interactions: {}\n\
        Days Active: {}\n\n",
        state.trust_level * 100.0,
        state.interaction_count,
        state.days_active
    );

    if !state.category_trust.is_empty() {
        report.push_str("Category Trust Levels:\n");

        let mut categories: Vec<_> = state.category_trust.values().collect();
        categories.sort_by(|a, b| b.trust_level.partial_cmp(&a.trust_level).unwrap());

        for trust in categories {
            report.push_str(&format!(
                "  {} - {:.0}% trust ({:?})\n    {} successful, {} failed, {} interventions\n",
                trust.category,
                trust.trust_level * 100.0,
                trust.autonomy_level,
                trust.successful_operations,
                trust.failed_operations,
                trust.user_interventions
            ));
        }
    }

    if let Some(rec) = state.get_trust_recommendation() {
        report.push_str(&format!("\nRecommendation:\n  {}\n", rec));
    }

    report
}

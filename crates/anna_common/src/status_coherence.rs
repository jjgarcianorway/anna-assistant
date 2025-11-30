//! Status Coherence Module v3.9.0
//!
//! Provides honest, consistent status information by cross-validating
//! all data stores (XP, Telemetry, XP Events) and detecting inconsistencies.
//!
//! ## Problem Statement
//!
//! Anna has three separate data stores that can get out of sync:
//! 1. XpStore (`/var/lib/anna/xp/xp_store.json`) - contains `total_questions`, levels, XP
//! 2. XpLog (`/var/lib/anna/knowledge/stats/xp_events.jsonl`) - timestamped events for 24h metrics
//! 3. Telemetry (`/var/log/anna/telemetry.jsonl`) - performance data
//!
//! This module provides:
//! - Coherent status data that doesn't contradict itself
//! - Clear explanations when data is inconsistent or missing
//! - Reset history tracking to explain why data might be missing
//!
//! ## Consistency Rules
//!
//! 1. If `total_questions > 0`, we must either:
//!    - Have XP events from the last 24h, OR
//!    - Explain: "XP events older than 24h have been pruned" or "Events from older sessions"
//!
//! 2. If `total_questions > 0`, we must either:
//!    - Have telemetry data, OR
//!    - Explain: "Telemetry only tracks events since v2.x" or "After reset"
//!
//! 3. LLM Autoprovision status must match actual benchmark state

use crate::experience_reset::ExperiencePaths;
use crate::llm_provision::LlmSelection;
use crate::telemetry::TelemetryReader;
use crate::xp_log::{Metrics24h, XpLog};
use crate::xp_track::XpStore;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Reset History (v3.9.0)
// ============================================================================

/// Reset history file location
pub const RESET_HISTORY_FILE: &str = "/var/lib/anna/reset_history.json";

/// A recorded reset event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetEvent {
    /// ISO timestamp of when reset occurred
    pub timestamp: String,
    /// Type of reset: "experience" or "factory"
    pub reset_type: String,
    /// Anna version at time of reset
    pub version: String,
    /// Questions answered before reset
    pub questions_before: u64,
    /// XP before reset
    pub xp_before: u64,
}

/// Reset history storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResetHistory {
    pub resets: Vec<ResetEvent>,
}

impl ResetHistory {
    /// Load from disk (returns empty if file doesn't exist)
    pub fn load() -> Self {
        let path = PathBuf::from(RESET_HISTORY_FILE);
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(RESET_HISTORY_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)
    }

    /// Record a reset event
    pub fn record_reset(
        reset_type: &str,
        questions_before: u64,
        xp_before: u64,
    ) -> std::io::Result<()> {
        let mut history = Self::load();
        history.resets.push(ResetEvent {
            timestamp: Utc::now().to_rfc3339(),
            reset_type: reset_type.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            questions_before,
            xp_before,
        });
        // Keep only last 10 resets
        if history.resets.len() > 10 {
            history.resets = history.resets.split_off(history.resets.len() - 10);
        }
        history.save()
    }

    /// Get the most recent reset, if any
    pub fn last_reset(&self) -> Option<&ResetEvent> {
        self.resets.last()
    }

    /// Check if there was a reset in the last 24 hours
    pub fn reset_in_last_24h(&self) -> bool {
        if let Some(last) = self.last_reset() {
            if let Ok(ts) = DateTime::parse_from_rfc3339(&last.timestamp) {
                let cutoff = Utc::now() - chrono::Duration::hours(24);
                return ts.with_timezone(&Utc) > cutoff;
            }
        }
        false
    }
}

// ============================================================================
// Coherent Status Data
// ============================================================================

/// Coherent status snapshot that cross-validates all data sources
#[derive(Debug, Clone)]
pub struct CoherentStatus {
    // XP Store data
    pub total_questions: u64,
    pub anna_xp: u64,
    pub anna_level: u8,
    pub xp_store_exists: bool,

    // XP Log (24h metrics)
    pub xp_events_24h: Metrics24h,
    pub xp_log_exists: bool,

    // Telemetry
    pub telemetry_exists: bool,
    pub telemetry_has_data: bool,
    pub telemetry_question_count: u64,

    // LLM Autoprovision
    pub autoprovision_ran: bool,
    pub models_selected: bool,

    // Reset history
    pub recent_reset: Option<ResetEvent>,
    pub was_recently_reset: bool,

    // Consistency analysis
    pub inconsistencies: Vec<StatusInconsistency>,
}

/// A detected inconsistency in status data
#[derive(Debug, Clone)]
pub struct StatusInconsistency {
    pub code: InconsistencyCode,
    pub explanation: String,
}

/// Types of inconsistencies we can detect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InconsistencyCode {
    /// Questions > 0 but no telemetry
    QuestionsWithoutTelemetry,
    /// Questions > 0 but no 24h XP events
    QuestionsWithoutXpEvents,
    /// XP > 0 but total_questions = 0
    XpWithoutQuestions,
    /// Models selected but "not yet run"
    ModelsWithoutBenchmark,
    /// Telemetry says more questions than XP store
    TelemetryXpMismatch,
}

impl CoherentStatus {
    /// Capture current status from all data sources
    pub fn capture() -> Self {
        let paths = ExperiencePaths::default();

        // Load XP Store
        let xp_store = XpStore::load();
        let xp_store_exists = paths.xp_store_file().exists();

        // Load XP Log
        let xp_log = XpLog::new();
        let xp_log_path = PathBuf::from(crate::xp_log::XP_LOG_DIR)
            .join(crate::xp_log::XP_LOG_FILE);
        let xp_log_exists = xp_log_path.exists();
        let xp_events_24h = xp_log.metrics_24h();

        // Load Telemetry
        let telemetry_reader = TelemetryReader::default_path();
        let telemetry_exists = PathBuf::from(crate::telemetry::TELEMETRY_FILE).exists();
        let telemetry_has_data = telemetry_reader.has_data();
        let telemetry_complete = telemetry_reader.complete_summary(100);
        let telemetry_question_count = telemetry_complete.lifetime.total;

        // Load LLM Selection
        let llm_selection = LlmSelection::load();
        let autoprovision_ran = !llm_selection.last_benchmark.is_empty()
            || llm_selection.junior_score > 0.0;
        let models_selected = !llm_selection.junior_model.is_empty()
            && llm_selection.junior_model != "auto";

        // Load Reset History
        let reset_history = ResetHistory::load();
        let recent_reset = reset_history.last_reset().cloned();
        let was_recently_reset = reset_history.reset_in_last_24h();

        let mut status = Self {
            total_questions: xp_store.anna_stats.total_questions,
            anna_xp: xp_store.anna.xp,
            anna_level: xp_store.anna.level,
            xp_store_exists,
            xp_events_24h,
            xp_log_exists,
            telemetry_exists,
            telemetry_has_data,
            telemetry_question_count,
            autoprovision_ran,
            models_selected,
            recent_reset,
            was_recently_reset,
            inconsistencies: vec![],
        };

        // Analyze consistency
        status.analyze_consistency();
        status
    }

    /// Analyze data for inconsistencies and add explanations
    fn analyze_consistency(&mut self) {
        // Check: Questions > 0 but no telemetry data
        if self.total_questions > 0 && !self.telemetry_has_data {
            if self.was_recently_reset {
                self.inconsistencies.push(StatusInconsistency {
                    code: InconsistencyCode::QuestionsWithoutTelemetry,
                    explanation: "Telemetry cleared by recent reset. Questions count persists from XP store.".to_string(),
                });
            } else {
                self.inconsistencies.push(StatusInconsistency {
                    code: InconsistencyCode::QuestionsWithoutTelemetry,
                    explanation: "Telemetry file missing or empty. Questions count from XP store (older data).".to_string(),
                });
            }
        }

        // Check: Questions > 0 but no 24h XP events
        if self.total_questions > 0 && self.xp_events_24h.total_events == 0 {
            if self.was_recently_reset {
                self.inconsistencies.push(StatusInconsistency {
                    code: InconsistencyCode::QuestionsWithoutXpEvents,
                    explanation: "XP events cleared by recent reset. Questions count persists from XP store.".to_string(),
                });
            } else {
                self.inconsistencies.push(StatusInconsistency {
                    code: InconsistencyCode::QuestionsWithoutXpEvents,
                    explanation: "No XP events in last 24 hours (events older than 24h are pruned).".to_string(),
                });
            }
        }

        // Check: XP > 0 but questions = 0 (shouldn't happen normally)
        if self.anna_xp > 0 && self.total_questions == 0 {
            self.inconsistencies.push(StatusInconsistency {
                code: InconsistencyCode::XpWithoutQuestions,
                explanation: "XP accumulated but no questions recorded (possible data migration).".to_string(),
            });
        }

        // Check: Models selected but autoprovision shows "not yet run"
        if self.models_selected && !self.autoprovision_ran {
            self.inconsistencies.push(StatusInconsistency {
                code: InconsistencyCode::ModelsWithoutBenchmark,
                explanation: "Models are configured but benchmark hasn't run yet.".to_string(),
            });
        }

        // Check: Telemetry question count doesn't match XP store
        if self.telemetry_has_data && self.telemetry_question_count > 0 {
            let diff = (self.telemetry_question_count as i64 - self.total_questions as i64).abs();
            // Allow some variance (telemetry may miss some events)
            if diff > 5 && diff as u64 > self.total_questions / 10 {
                self.inconsistencies.push(StatusInconsistency {
                    code: InconsistencyCode::TelemetryXpMismatch,
                    explanation: format!(
                        "Telemetry ({}) and XP store ({}) question counts differ.",
                        self.telemetry_question_count, self.total_questions
                    ),
                });
            }
        }
    }

    /// Get explanation for why telemetry might be missing
    pub fn telemetry_explanation(&self) -> Option<&str> {
        for inc in &self.inconsistencies {
            if inc.code == InconsistencyCode::QuestionsWithoutTelemetry {
                return Some(&inc.explanation);
            }
        }
        None
    }

    /// Get explanation for why 24h XP events might be missing
    pub fn xp_events_explanation(&self) -> Option<&str> {
        for inc in &self.inconsistencies {
            if inc.code == InconsistencyCode::QuestionsWithoutXpEvents {
                return Some(&inc.explanation);
            }
        }
        None
    }

    /// Is the data consistent (no major issues)?
    pub fn is_consistent(&self) -> bool {
        self.inconsistencies.is_empty()
    }

    /// Is this a fresh install (no data at all)?
    pub fn is_fresh_install(&self) -> bool {
        self.total_questions == 0
            && self.anna_xp == 0
            && !self.telemetry_has_data
            && self.xp_events_24h.total_events == 0
    }
}

// ============================================================================
// Status Message Helpers
// ============================================================================

/// Generate an honest telemetry status message
pub fn telemetry_status_message(status: &CoherentStatus) -> String {
    if status.telemetry_has_data {
        return String::new(); // No special message needed
    }

    if status.is_fresh_install() {
        return "No telemetry yet. Ask a few questions to gather data.".to_string();
    }

    if let Some(explanation) = status.telemetry_explanation() {
        return explanation.to_string();
    }

    if status.was_recently_reset {
        return "Telemetry cleared by recent reset.".to_string();
    }

    "No telemetry data available.".to_string()
}

/// Generate an honest 24h XP events message
pub fn xp_events_status_message(status: &CoherentStatus) -> String {
    if status.xp_events_24h.total_events > 0 {
        return String::new(); // No special message needed
    }

    if status.is_fresh_install() {
        return "No XP events yet. Ask a few questions to start earning XP.".to_string();
    }

    if let Some(explanation) = status.xp_events_explanation() {
        return explanation.to_string();
    }

    if status.was_recently_reset {
        return "XP event log cleared by recent reset.".to_string();
    }

    // Default: explain the 24h window
    if status.total_questions > 0 {
        return "No activity in the last 24 hours. XP events older than 24h are pruned.".to_string();
    }

    "No XP events in the last 24 hours.".to_string()
}

/// Generate autoprovision status
pub fn autoprovision_status_message(status: &CoherentStatus) -> &'static str {
    if status.autoprovision_ran {
        if status.models_selected {
            "enabled"
        } else {
            "ran but no models selected"
        }
    } else if status.models_selected {
        "models configured, benchmark pending"
    } else {
        "not yet run"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_history_serialization() {
        let event = ResetEvent {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            reset_type: "experience".to_string(),
            version: "3.9.0".to_string(),
            questions_before: 100,
            xp_before: 5000,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("experience"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_inconsistency_detection_questions_without_telemetry() {
        // Simulate status with questions but no telemetry
        let mut status = CoherentStatus {
            total_questions: 100,
            anna_xp: 5000,
            anna_level: 5,
            xp_store_exists: true,
            xp_events_24h: Metrics24h::default(),
            xp_log_exists: true,
            telemetry_exists: true,
            telemetry_has_data: false,
            telemetry_question_count: 0,
            autoprovision_ran: true,
            models_selected: true,
            recent_reset: None,
            was_recently_reset: false,
            inconsistencies: vec![],
        };

        status.analyze_consistency();

        // Should detect QuestionsWithoutTelemetry
        assert!(status.inconsistencies.iter().any(|i|
            i.code == InconsistencyCode::QuestionsWithoutTelemetry
        ));
    }

    #[test]
    fn test_fresh_install_detection() {
        let status = CoherentStatus {
            total_questions: 0,
            anna_xp: 0,
            anna_level: 1,
            xp_store_exists: false,
            xp_events_24h: Metrics24h::default(),
            xp_log_exists: false,
            telemetry_exists: false,
            telemetry_has_data: false,
            telemetry_question_count: 0,
            autoprovision_ran: false,
            models_selected: false,
            recent_reset: None,
            was_recently_reset: false,
            inconsistencies: vec![],
        };

        assert!(status.is_fresh_install());
    }

    #[test]
    fn test_consistent_status() {
        let mut status = CoherentStatus {
            total_questions: 10,
            anna_xp: 500,
            anna_level: 2,
            xp_store_exists: true,
            xp_events_24h: Metrics24h {
                total_events: 5,
                xp_gained: 50,
                ..Default::default()
            },
            xp_log_exists: true,
            telemetry_exists: true,
            telemetry_has_data: true,
            telemetry_question_count: 10,
            autoprovision_ran: true,
            models_selected: true,
            recent_reset: None,
            was_recently_reset: false,
            inconsistencies: vec![],
        };

        status.analyze_consistency();
        assert!(status.is_consistent());
    }

    #[test]
    fn test_telemetry_message_fresh_install() {
        let status = CoherentStatus {
            total_questions: 0,
            anna_xp: 0,
            anna_level: 1,
            xp_store_exists: false,
            xp_events_24h: Metrics24h::default(),
            xp_log_exists: false,
            telemetry_exists: false,
            telemetry_has_data: false,
            telemetry_question_count: 0,
            autoprovision_ran: false,
            models_selected: false,
            recent_reset: None,
            was_recently_reset: false,
            inconsistencies: vec![],
        };

        let msg = telemetry_status_message(&status);
        assert!(msg.contains("Ask a few questions"));
    }

    #[test]
    fn test_xp_events_message_after_reset() {
        let mut status = CoherentStatus {
            total_questions: 100,
            anna_xp: 0,
            anna_level: 1,
            xp_store_exists: true,
            xp_events_24h: Metrics24h::default(),
            xp_log_exists: false,
            telemetry_exists: true,
            telemetry_has_data: false,
            telemetry_question_count: 0,
            autoprovision_ran: true,
            models_selected: true,
            recent_reset: Some(ResetEvent {
                timestamp: Utc::now().to_rfc3339(),
                reset_type: "experience".to_string(),
                version: "3.9.0".to_string(),
                questions_before: 100,
                xp_before: 5000,
            }),
            was_recently_reset: true,
            inconsistencies: vec![],
        };

        status.analyze_consistency();
        let msg = xp_events_status_message(&status);
        assert!(msg.contains("reset") || msg.contains("cleared"));
    }
}

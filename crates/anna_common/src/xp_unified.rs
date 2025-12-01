//! Unified XP Recording Module v1.1.0
//!
//! CANONICAL source for recording XP events in real-world runs.
//! Ensures both XpLog (24h metrics) and XpStore (LLM agent stats) are updated.
//!
//! ## Problem Solved
//!
//! Previously, XP recording was split across two systems with different update paths:
//! - Brain path: Updated XpStore only (missing from 24h metrics)
//! - LLM path: Updated XpLog only in daemon, XpStore only in annactl
//! - Result: "No XP events in 24 hours" and "good_plans=0" despite real activity
//!
//! ## Solution
//!
//! Single unified function `record_xp` that:
//! 1. Creates XpEvent from type and question
//! 2. Appends to XpLog (for 24h metrics display)
//! 3. Updates XpStore (for LLM agent stats display)
//! 4. Returns formatted log line for debug output

use crate::xp_events::{XpEvent, XpEventType};
use crate::xp_log::XpLog;
use crate::xp_track::XpStore;

/// Unified XP recorder that updates both XpLog and XpStore
pub struct UnifiedXpRecorder {
    xp_log: XpLog,
}

impl Default for UnifiedXpRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedXpRecorder {
    /// Create new recorder with default paths
    pub fn new() -> Self {
        Self {
            xp_log: XpLog::new(),
        }
    }

    /// Create with custom XpLog (for testing)
    pub fn with_log(xp_log: XpLog) -> Self {
        Self { xp_log }
    }

    /// Record an XP event - updates BOTH XpLog AND XpStore
    /// Returns formatted log line for debug output
    pub fn record(&self, event_type: XpEventType, question: &str) -> String {
        self.record_with_context(event_type, question, None, None)
    }

    /// Record an XP event with optional command and context
    pub fn record_with_context(
        &self,
        event_type: XpEventType,
        question: &str,
        command: Option<&str>,
        context: Option<&str>,
    ) -> String {
        // 1. Create the event
        let mut event = XpEvent::new(event_type.clone(), question);
        if let Some(cmd) = command {
            event = event.with_command(cmd);
        }
        if let Some(ctx) = context {
            event = event.with_context(ctx);
        }

        // 2. Append to XpLog (for 24h metrics)
        if let Err(e) = self.xp_log.append(&event) {
            // v3.10.0: Clearer error message for permission issues
            let err_str = e.to_string();
            if err_str.contains("Permission denied") || err_str.contains("os error 13") {
                // Don't spam permission errors - they happen on every Brain answer
                // The issue is directory ownership, fix with reinstall
                tracing::debug!("XP log permission denied (reinstall to fix): {}", e);
            } else {
                eprintln!("[!] Failed to append XP event to log: {}", e);
            }
        }

        // 3. Update XpStore (for LLM agent stats)
        let mut xp_store = XpStore::load();
        let log_line = match &event_type {
            // Anna events
            XpEventType::BrainSelfSolve => xp_store.anna_self_solve(question),
            XpEventType::BrainPartialSolve => xp_store.anna_brain_helped(),
            XpEventType::LlmTimeoutFallback => xp_store.anna_timeout(),
            XpEventType::LowReliabilityRefusal => xp_store.anna_refusal_correct(),

            // Junior events
            XpEventType::JuniorCleanProposal => {
                xp_store.junior_plan_good(command.unwrap_or(""))
            }
            XpEventType::JuniorBadCommand | XpEventType::JuniorWrongDomain => {
                xp_store.junior_plan_bad()
            }

            // Senior events
            XpEventType::SeniorGreenApproval => {
                // Extract score from context if available
                let score = context
                    .and_then(|c| c.strip_prefix("score="))
                    .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
                    .map(|s| s / 100.0)
                    .unwrap_or(0.9);
                // v4.3.1: Also increment total_questions for LLM answers
                // This was missing, causing XpStore.total_questions to only count Brain answers
                xp_store.anna_stats.llm_answers += 1;
                xp_store.anna_stats.total_questions += 1;
                xp_store.senior_approve_correct(score)
            }
            XpEventType::SeniorRepeatedFix => {
                // v4.3.1: Also increment total_questions for fix-and-accept LLM answers
                xp_store.anna_stats.llm_answers += 1;
                xp_store.anna_stats.total_questions += 1;
                xp_store.senior_fix_accept_good()
            }

            // Neutral/other events - just format the event
            XpEventType::StablePatternDetected | XpEventType::UnstablePatternPenalized => {
                event.format_log()
            }
        };

        // 4. CRITICAL: Save XpStore to disk (fixes "No XP events in 24 hours" bug)
        if let Err(e) = xp_store.save() {
            // v3.10.0: Quieter error for permission issues
            let err_str = e.to_string();
            if err_str.contains("Permission denied") || err_str.contains("os error 13") {
                tracing::debug!("XP store permission denied (reinstall to fix): {}", e);
            } else {
                eprintln!("[!] Failed to save XP store: {}", e);
            }
        }

        log_line
    }

    /// Record Brain self-solve event (simple hardware questions)
    pub fn brain_self_solve(&self, question: &str, command: &str) -> String {
        self.record_with_context(
            XpEventType::BrainSelfSolve,
            question,
            Some(command),
            None,
        )
    }

    /// Record Brain partial solve (helped LLM)
    pub fn brain_partial_solve(&self, question: &str) -> String {
        self.record(XpEventType::BrainPartialSolve, question)
    }

    /// Record Junior clean proposal (approved by Senior)
    pub fn junior_clean_proposal(&self, question: &str, command: &str) -> String {
        self.record_with_context(
            XpEventType::JuniorCleanProposal,
            question,
            Some(command),
            None,
        )
    }

    /// Record Junior bad command
    pub fn junior_bad_command(&self, question: &str, command: &str, error: &str) -> String {
        self.record_with_context(
            XpEventType::JuniorBadCommand,
            question,
            Some(command),
            Some(error),
        )
    }

    /// Record Senior green approval (>= 90%)
    pub fn senior_green_approval(&self, question: &str, score: f64) -> String {
        self.record_with_context(
            XpEventType::SeniorGreenApproval,
            question,
            None,
            Some(&format!("score={:.0}%", score * 100.0)),
        )
    }

    /// Record Senior fix and accept
    pub fn senior_fix_and_accept(&self, question: &str) -> String {
        self.record(XpEventType::SeniorRepeatedFix, question)
    }

    /// Record LLM timeout fallback
    pub fn llm_timeout(&self, question: &str, elapsed_ms: u64) -> String {
        self.record_with_context(
            XpEventType::LlmTimeoutFallback,
            question,
            None,
            Some(&format!("elapsed={}ms", elapsed_ms)),
        )
    }

    /// Record low reliability refusal
    pub fn low_reliability_refusal(&self, question: &str) -> String {
        self.record(XpEventType::LowReliabilityRefusal, question)
    }
}

// ============================================================================
// Convenience functions for direct use without creating a recorder
// ============================================================================

/// Record a Brain self-solve event (convenience function)
pub fn record_brain_self_solve(question: &str, command: &str) -> String {
    UnifiedXpRecorder::new().brain_self_solve(question, command)
}

/// Record a Junior clean proposal event (convenience function)
pub fn record_junior_proposal(question: &str, command: &str) -> String {
    UnifiedXpRecorder::new().junior_clean_proposal(question, command)
}

/// Record a Junior bad command event (convenience function)
pub fn record_junior_bad_command(question: &str, command: &str, error: &str) -> String {
    UnifiedXpRecorder::new().junior_bad_command(question, command, error)
}

/// Record a Senior green approval event (convenience function)
pub fn record_senior_approval(question: &str, score: f64) -> String {
    UnifiedXpRecorder::new().senior_green_approval(question, score)
}

/// Record a Senior fix and accept event (convenience function)
pub fn record_senior_fix_accept(question: &str) -> String {
    UnifiedXpRecorder::new().senior_fix_and_accept(question)
}

/// Record an LLM timeout event (convenience function)
pub fn record_llm_timeout(question: &str, elapsed_ms: u64) -> String {
    UnifiedXpRecorder::new().llm_timeout(question, elapsed_ms)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::path::PathBuf;

    fn create_test_recorder() -> (UnifiedXpRecorder, PathBuf) {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();
        let xp_log = XpLog::with_path(path.clone());
        (UnifiedXpRecorder::with_log(xp_log), path)
    }

    #[test]
    fn test_brain_self_solve_updates_both() {
        let (recorder, log_path) = create_test_recorder();

        let result = recorder.brain_self_solve("How many cores?", "lscpu");
        assert!(result.contains("+5 XP") || result.contains("self_solve"));

        // Check XpLog was updated
        let xp_log = XpLog::with_path(log_path);
        let events = xp_log.read_recent(10);
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, "BrainSelfSolve");
    }

    #[test]
    fn test_junior_clean_proposal() {
        let (recorder, _) = create_test_recorder();
        let result = recorder.junior_clean_proposal("What CPU?", "lscpu -J");
        assert!(result.contains("XP") || result.contains("plan"));
    }

    #[test]
    fn test_senior_green_approval() {
        let (recorder, _) = create_test_recorder();
        let result = recorder.senior_green_approval("What's my RAM?", 0.95);
        assert!(result.contains("XP") || result.contains("approve"));
    }

    #[test]
    fn test_llm_timeout() {
        let (recorder, _) = create_test_recorder();
        let result = recorder.llm_timeout("Slow question", 30000);
        assert!(result.contains("timeout") || result.contains("XP"));
    }

    #[test]
    fn test_convenience_functions() {
        // Just ensure they don't panic
        let _ = record_brain_self_solve("test", "cmd");
        let _ = record_junior_proposal("test", "cmd");
        let _ = record_junior_bad_command("test", "cmd", "error");
        let _ = record_senior_approval("test", 0.9);
        let _ = record_senior_fix_accept("test");
        let _ = record_llm_timeout("test", 1000);
    }
}

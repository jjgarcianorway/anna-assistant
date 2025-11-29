//! XP Events v1.0.0 - CANONICAL SOURCE
//!
//! XP event types and values. See `docs/architecture.md` Section 6.
//!
//! Events track Anna, Junior, and Senior performance for trust-based routing.
//!
//! ## Architecture Note (v1.0.0)
//!
//! This module is the CANONICAL source for XP event types and base values.
//! Other XP-related modules should import from here:
//! - `xp_track.rs`: Uses these events for XpStore tracking
//! - `rpg_display.rs`: Provides display/formatting for these events
//! - `progression/`: Legacy module, deprecated in favor of this
//!
//! ## XP Event Categories
//!
//! | Event Type              | Base XP | Trust Delta | Agent   |
//! |-------------------------|---------|-------------|---------|
//! | BrainSelfSolve          | +15     | +0.02       | Anna    |
//! | BrainPartialSolve       | +8      | +0.01       | Anna    |
//! | JuniorCleanProposal     | +10     | +0.02       | Junior  |
//! | SeniorGreenApproval     | +12     | +0.02       | Senior  |
//! | StablePatternDetected   | +20     | +0.03       | Anna    |
//! | JuniorBadCommand        | -8      | -0.05       | Junior  |
//! | JuniorWrongDomain       | -5      | -0.03       | Junior  |
//! | SeniorRepeatedFix       | -10     | -0.05       | Senior  |
//! | LlmTimeoutFallback      | -5      | -0.03       | Anna    |
//! | UnstablePatternPenalized| -12     | -0.05       | Anna    |
//! | LowReliabilityRefusal   | -3      | -0.02       | Anna    |

use serde::{Deserialize, Serialize};

/// v0.85.0 XP event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum XpEventType {
    // ========== Positive events (increase XP) ==========
    /// Brain solved question without LLM
    BrainSelfSolve,
    /// Brain provided partial answer that helped Junior
    BrainPartialSolve,
    /// Junior proposed command that Senior approved without changes
    JuniorCleanProposal,
    /// Senior approved answer as Green (>=90%)
    SeniorGreenApproval,
    /// A stable pattern was detected and learned
    StablePatternDetected,

    // ========== Negative events (decrease XP) ==========
    /// Junior proposed a command that failed
    JuniorBadCommand,
    /// Junior proposed a command in wrong domain
    JuniorWrongDomain,
    /// Senior had to fix the same error type repeatedly
    SeniorRepeatedFix,
    /// LLM timed out, had to fallback
    LlmTimeoutFallback,
    /// An unstable pattern was detected
    UnstablePatternPenalized,
    /// Answer was refused due to low reliability
    LowReliabilityRefusal,
}

impl XpEventType {
    /// Get the base XP change for this event type
    pub fn base_xp(&self) -> i32 {
        match self {
            // Positive
            Self::BrainSelfSolve => 15,       // Big reward for no LLM usage
            Self::BrainPartialSolve => 8,     // Smaller reward for helping
            Self::JuniorCleanProposal => 10,  // Good Junior work
            Self::SeniorGreenApproval => 12,  // High quality answer
            Self::StablePatternDetected => 20, // Learning reward

            // Negative
            Self::JuniorBadCommand => -8,
            Self::JuniorWrongDomain => -5,
            Self::SeniorRepeatedFix => -10,
            Self::LlmTimeoutFallback => -5,
            Self::UnstablePatternPenalized => -12,
            Self::LowReliabilityRefusal => -3,
        }
    }

    /// Is this a positive event?
    pub fn is_positive(&self) -> bool {
        self.base_xp() > 0
    }
}

/// An XP event with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpEvent {
    /// Event type
    pub event_type: XpEventType,
    /// Question that triggered this event
    pub question: String,
    /// Command involved (if any)
    pub command: Option<String>,
    /// Additional context
    pub context: Option<String>,
    /// Timestamp (unix seconds)
    pub timestamp: u64,
    /// Calculated XP change
    pub xp_change: i32,
}

impl XpEvent {
    pub fn new(event_type: XpEventType, question: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self {
            xp_change: event_type.base_xp(),
            event_type,
            question: question.to_string(),
            command: None,
            context: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    pub fn with_context(mut self, ctx: &str) -> Self {
        self.context = Some(ctx.to_string());
        self
    }

    /// Apply multiplier to XP change
    pub fn with_multiplier(mut self, mult: f64) -> Self {
        self.xp_change = (self.xp_change as f64 * mult) as i32;
        self
    }

    /// Format as log line
    pub fn format_log(&self) -> String {
        let sign = if self.xp_change >= 0 { "+" } else { "" };
        let cmd_str = self.command.as_ref()
            .map(|c| format!(" cmd={}", c))
            .unwrap_or_default();
        format!(
            "ANNA_XP event={:?} xp={}{}{} q=\"{}\"",
            self.event_type,
            sign,
            self.xp_change,
            cmd_str,
            self.question
        )
    }
}

/// XP calculator with v0.85.0 rules
pub struct XpCalculatorV85 {
    /// Recent bad command count (for SeniorRepeatedFix detection)
    recent_fixes: u32,
    /// Recent pattern successes
    pattern_successes: u32,
}

impl Default for XpCalculatorV85 {
    fn default() -> Self {
        Self::new()
    }
}

impl XpCalculatorV85 {
    pub fn new() -> Self {
        Self {
            recent_fixes: 0,
            pattern_successes: 0,
        }
    }

    /// Process Brain self-solve
    pub fn brain_self_solve(&self, question: &str, command: &str) -> XpEvent {
        XpEvent::new(XpEventType::BrainSelfSolve, question)
            .with_command(command)
    }

    /// Process Brain partial solve
    pub fn brain_partial_solve(&self, question: &str) -> XpEvent {
        XpEvent::new(XpEventType::BrainPartialSolve, question)
    }

    /// Process Junior clean proposal (approved without changes)
    pub fn junior_clean_proposal(&self, question: &str, command: &str) -> XpEvent {
        XpEvent::new(XpEventType::JuniorCleanProposal, question)
            .with_command(command)
    }

    /// Process Junior bad command
    pub fn junior_bad_command(&self, question: &str, command: &str, error: &str) -> XpEvent {
        XpEvent::new(XpEventType::JuniorBadCommand, question)
            .with_command(command)
            .with_context(error)
    }

    /// Process Senior green approval
    pub fn senior_green_approval(&self, question: &str, score: f64) -> XpEvent {
        XpEvent::new(XpEventType::SeniorGreenApproval, question)
            .with_context(&format!("score={:.0}%", score * 100.0))
    }

    /// Process Senior repeated fix
    pub fn senior_repeated_fix(&mut self, question: &str) -> XpEvent {
        self.recent_fixes += 1;
        let mult = if self.recent_fixes > 3 { 1.5 } else { 1.0 };
        XpEvent::new(XpEventType::SeniorRepeatedFix, question)
            .with_multiplier(mult)
    }

    /// Process LLM timeout
    pub fn llm_timeout(&self, question: &str, elapsed_ms: u64) -> XpEvent {
        XpEvent::new(XpEventType::LlmTimeoutFallback, question)
            .with_context(&format!("elapsed={}ms", elapsed_ms))
    }

    /// Process stable pattern detection
    pub fn stable_pattern(&mut self, question: &str, pattern_key: &str) -> XpEvent {
        self.pattern_successes += 1;
        XpEvent::new(XpEventType::StablePatternDetected, question)
            .with_context(pattern_key)
    }

    /// Process unstable pattern
    pub fn unstable_pattern(&self, question: &str, pattern_key: &str) -> XpEvent {
        XpEvent::new(XpEventType::UnstablePatternPenalized, question)
            .with_context(pattern_key)
    }

    /// Reset counters (call after each session)
    pub fn reset(&mut self) {
        self.recent_fixes = 0;
        self.pattern_successes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_event_types() {
        assert!(XpEventType::BrainSelfSolve.is_positive());
        assert!(!XpEventType::JuniorBadCommand.is_positive());
    }

    #[test]
    fn test_xp_event_creation() {
        let event = XpEvent::new(XpEventType::BrainSelfSolve, "test question")
            .with_command("lscpu");
        assert_eq!(event.xp_change, 15);
        assert_eq!(event.command, Some("lscpu".to_string()));
    }

    #[test]
    fn test_xp_event_multiplier() {
        let event = XpEvent::new(XpEventType::SeniorRepeatedFix, "test")
            .with_multiplier(1.5);
        assert_eq!(event.xp_change, -15); // -10 * 1.5
    }

    #[test]
    fn test_xp_calculator() {
        let calc = XpCalculatorV85::new();
        let event = calc.brain_self_solve("How many cores?", "lscpu");
        assert_eq!(event.xp_change, 15);
    }

    #[test]
    fn test_xp_log_format() {
        let event = XpEvent::new(XpEventType::BrainSelfSolve, "test")
            .with_command("lscpu");
        let log = event.format_log();
        assert!(log.contains("ANNA_XP"));
        assert!(log.contains("+15"));
        assert!(log.contains("cmd=lscpu"));
    }
}

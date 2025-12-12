//! Stats and UI Components (Part 5) - v0.0.443.
//!
//! Clean stats driven by ticket state machine:
//! - Breakdown by outcome (answered, parse_error, etc.)
//! - No XP gamification by default (--fun flag)
//!
//! Clean dialogs:
//! - Single question, enumerated choices
//! - Always allow Cancel
//! - Never duplicate
//! - --plain and --json modes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ticket_integrity::outcome::TicketOutcome;

/// Clean stats (no gamification by default).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanStats {
    /// Total tickets.
    pub total: u64,
    /// Breakdown by outcome.
    pub by_outcome: HashMap<TicketOutcome, u64>,
    /// Average response time (ms).
    pub avg_response_ms: u64,
    /// Period (e.g., "today", "this_week").
    pub period: String,
}

impl CleanStats {
    /// Create empty stats.
    pub fn new(period: &str) -> Self {
        Self {
            period: period.to_string(),
            ..Default::default()
        }
    }

    /// Record a ticket.
    pub fn record(&mut self, outcome: TicketOutcome, response_ms: u64) {
        *self.by_outcome.entry(outcome).or_insert(0) += 1;
        self.total += 1;

        // Update average (rolling)
        let old_total = if self.total > 1 { self.total - 1 } else { 1 };
        self.avg_response_ms = (self.avg_response_ms * old_total + response_ms) / self.total;
    }

    /// Get count for outcome.
    pub fn count(&self, outcome: TicketOutcome) -> u64 {
        self.by_outcome.get(&outcome).copied().unwrap_or(0)
    }

    /// Get answered count.
    pub fn answered(&self) -> u64 {
        self.count(TicketOutcome::Answered)
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.answered() as f64 / self.total as f64
        }
    }

    /// Format for display (clean, no gamification).
    pub fn display(&self) -> String {
        let rate = self.success_rate() * 100.0;

        format!(
            "Stats ({})\n\
             ─────────────────────────\n\
             Total tickets:     {}\n\
             Answered:          {} ({:.0}%)\n\
             Parse errors:      {}\n\
             Probe errors:      {}\n\
             Clarification:     {}\n\
             Cancelled:         {}\n\
             Internal errors:   {}\n\
             ─────────────────────────\n\
             Avg response:      {}ms",
            self.period,
            self.total,
            self.answered(),
            rate,
            self.count(TicketOutcome::ParseError),
            self.count(TicketOutcome::ProbeError),
            self.count(TicketOutcome::ClarificationPending),
            self.count(TicketOutcome::Cancelled),
            self.count(TicketOutcome::InternalError),
            self.avg_response_ms
        )
    }

    /// Format for JSON output.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Format for plain output (minimal).
    pub fn display_plain(&self) -> String {
        format!(
            "total={} answered={} parse_errors={} probe_errors={} success_rate={:.1}%",
            self.total,
            self.answered(),
            self.count(TicketOutcome::ParseError),
            self.count(TicketOutcome::ProbeError),
            self.success_rate() * 100.0
        )
    }

    /// Format with gamification (--fun flag only).
    pub fn display_fun(&self) -> String {
        let rate = self.success_rate();
        let rank = match rate {
            r if r >= 0.95 => "🏆 Legendary",
            r if r >= 0.85 => "⭐ Expert",
            r if r >= 0.70 => "📈 Competent",
            r if r >= 0.50 => "📚 Learning",
            _ => "🌱 Beginner",
        };

        let xp = (self.answered() * 10) as u64;

        format!(
            "{}\n\n\
             {} Rank: {}\n\
             {} XP: {}\n\
             {} Streak: {} tickets",
            self.display(),
            "🎮",
            rank,
            "✨",
            xp,
            "🔥",
            self.answered()
        )
    }
}

/// Output mode for CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Normal formatted output.
    Normal,
    /// Plain text (minimal).
    Plain,
    /// JSON.
    Json,
    /// With gamification.
    Fun,
}

impl OutputMode {
    /// Parse from flags.
    pub fn from_flags(plain: bool, json: bool, fun: bool) -> Self {
        if json {
            Self::Json
        } else if plain {
            Self::Plain
        } else if fun {
            Self::Fun
        } else {
            Self::Normal
        }
    }
}

// ========== Dialog Components ==========

/// A clean dialog question.
#[derive(Debug, Clone)]
pub struct DialogQuestion {
    /// Question text.
    pub question: String,
    /// Choices.
    pub choices: Vec<DialogChoice>,
    /// Allow cancel?
    pub allow_cancel: bool,
    /// Allow other input?
    pub allow_other: bool,
}

/// A dialog choice.
#[derive(Debug, Clone)]
pub struct DialogChoice {
    /// Choice key (for selection).
    pub key: String,
    /// Display label.
    pub label: String,
    /// Value to return if selected.
    pub value: String,
}

impl DialogChoice {
    /// Create new choice.
    pub fn new(key: &str, label: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    /// Create numbered choice.
    pub fn numbered(num: usize, label: &str, value: &str) -> Self {
        Self::new(&num.to_string(), label, value)
    }
}

impl DialogQuestion {
    /// Create new question.
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            choices: Vec::new(),
            allow_cancel: true,
            allow_other: false,
        }
    }

    /// Add choice.
    pub fn choice(mut self, label: &str, value: &str) -> Self {
        let num = self.choices.len() + 1;
        self.choices.push(DialogChoice::numbered(num, label, value));
        self
    }

    /// Allow other input.
    pub fn with_other(mut self) -> Self {
        self.allow_other = true;
        self
    }

    /// Disallow cancel.
    pub fn no_cancel(mut self) -> Self {
        self.allow_cancel = false;
        self
    }

    /// Format for display (normal).
    pub fn display(&self) -> String {
        let mut output = format!("{}\n", self.question);

        for choice in &self.choices {
            output.push_str(&format!("  {}) {}\n", choice.key, choice.label));
        }

        if self.allow_other {
            output.push_str("  9) Something else (type it)\n");
        }

        if self.allow_cancel {
            output.push_str("  0) Cancel\n");
        }

        output
    }

    /// Format for plain display.
    pub fn display_plain(&self) -> String {
        let choices: Vec<_> = self.choices.iter().map(|c| c.label.as_str()).collect();
        format!("{} [{}]", self.question, choices.join("/"))
    }

    /// Format for JSON.
    pub fn to_json(&self) -> Result<String, String> {
        let obj = serde_json::json!({
            "question": self.question,
            "choices": self.choices.iter().map(|c| {
                serde_json::json!({
                    "key": c.key,
                    "label": c.label,
                    "value": c.value
                })
            }).collect::<Vec<_>>(),
            "allow_cancel": self.allow_cancel,
            "allow_other": self.allow_other
        });
        serde_json::to_string(&obj).map_err(|e| e.to_string())
    }

    /// Parse user input.
    pub fn parse_input(&self, input: &str) -> DialogResult {
        let input = input.trim();

        // Cancel
        if input == "0" && self.allow_cancel {
            return DialogResult::Cancelled;
        }

        // Other
        if input == "9" && self.allow_other {
            return DialogResult::Other;
        }

        // Match choice
        for choice in &self.choices {
            if input == choice.key {
                return DialogResult::Selected(choice.value.clone());
            }
        }

        // Invalid
        DialogResult::Invalid(input.to_string())
    }
}

/// Dialog result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResult {
    /// User selected a choice.
    Selected(String),
    /// User cancelled.
    Cancelled,
    /// User chose "other".
    Other,
    /// Invalid input.
    Invalid(String),
}

/// Confirmation dialog.
pub struct ConfirmDialog {
    /// Question text.
    pub question: String,
    /// Default answer.
    pub default: bool,
}

impl ConfirmDialog {
    /// Create new confirmation.
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            default: false,
        }
    }

    /// Set default to yes.
    pub fn default_yes(mut self) -> Self {
        self.default = true;
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let hint = if self.default { "[Y/n]" } else { "[y/N]" };
        format!("{} {}", self.question, hint)
    }

    /// Parse user input.
    pub fn parse_input(&self, input: &str) -> bool {
        let input = input.trim().to_lowercase();
        if input.is_empty() {
            return self.default;
        }
        input == "y" || input == "yes"
    }
}

/// Progress indicator.
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Current step.
    pub current: usize,
    /// Total steps.
    pub total: usize,
    /// Current step description.
    pub description: String,
}

impl ProgressIndicator {
    /// Create new indicator.
    pub fn new(total: usize) -> Self {
        Self {
            current: 0,
            total,
            description: String::new(),
        }
    }

    /// Advance to next step.
    pub fn advance(&mut self, description: &str) {
        self.current = (self.current + 1).min(self.total);
        self.description = description.to_string();
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let pct = if self.total > 0 {
            (self.current * 100) / self.total
        } else {
            100
        };
        format!(
            "[{}/{}] {} ({}%)",
            self.current, self.total, self.description, pct
        )
    }

    /// Format for plain display.
    pub fn display_plain(&self) -> String {
        format!("{}/{}: {}", self.current, self.total, self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_stats() {
        let mut stats = CleanStats::new("today");
        stats.record(TicketOutcome::Answered, 100);
        stats.record(TicketOutcome::Answered, 200);
        stats.record(TicketOutcome::ParseError, 50);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.answered(), 2);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_dialog_question() {
        let dialog = DialogQuestion::new("Which editor?")
            .choice("vim", "vim")
            .choice("nano", "nano")
            .with_other();

        let display = dialog.display();
        assert!(display.contains("Which editor"));
        assert!(display.contains("1) vim"));
        assert!(display.contains("2) nano"));
        assert!(display.contains("9) Something else"));
        assert!(display.contains("0) Cancel"));
    }

    #[test]
    fn test_dialog_parse() {
        let dialog = DialogQuestion::new("Choose")
            .choice("a", "a")
            .choice("b", "b");

        assert_eq!(
            dialog.parse_input("1"),
            DialogResult::Selected("a".to_string())
        );
        assert_eq!(
            dialog.parse_input("2"),
            DialogResult::Selected("b".to_string())
        );
        assert_eq!(dialog.parse_input("0"), DialogResult::Cancelled);
        assert!(matches!(dialog.parse_input("x"), DialogResult::Invalid(_)));
    }

    #[test]
    fn test_confirm_dialog() {
        let confirm = ConfirmDialog::new("Proceed?").default_yes();
        assert!(confirm.parse_input(""));
        assert!(confirm.parse_input("y"));
        assert!(!confirm.parse_input("n"));
    }

    #[test]
    fn test_progress_indicator() {
        let mut progress = ProgressIndicator::new(4);
        progress.advance("Step 1");

        assert_eq!(progress.current, 1);
        let display = progress.display();
        assert!(display.contains("[1/4]"));
        assert!(display.contains("Step 1"));
    }

    #[test]
    fn test_output_modes() {
        assert_eq!(
            OutputMode::from_flags(false, false, false),
            OutputMode::Normal
        );
        assert_eq!(
            OutputMode::from_flags(true, false, false),
            OutputMode::Plain
        );
        assert_eq!(OutputMode::from_flags(false, true, false), OutputMode::Json);
        assert_eq!(OutputMode::from_flags(false, false, true), OutputMode::Fun);
    }
}

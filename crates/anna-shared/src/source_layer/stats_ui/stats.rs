//! Stats components - CleanStats and OutputMode.

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

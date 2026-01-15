//! Timeline types for recording and replaying ticket activity.
//!
//! Phase 11: Human-Readable Internal Dialogue and Timeline Reconstruction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single entry in the dialogue timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEntry {
    /// Monotonic sequence number (for ordering).
    pub seq: u64,
    /// Timestamp when this entry occurred.
    pub timestamp: DateTime<Utc>,
    /// The kind of entry.
    pub kind: EntryKind,
    /// Whether this entry contains internal-only data (redacted by default).
    pub internal_only: bool,
}

/// The kind of timeline entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum EntryKind {
    /// Ticket created.
    TicketCreated {
        ticket_id: String,
        question: String,
        department: String,
    },
    /// Translator decision.
    TranslatorDecision {
        interpreted_as: String,
        confidence: f32,
        routed_to: String,
    },
    /// Specialist assigned.
    SpecialistAssigned {
        specialist_id: String,
        specialist_name: String,
        level: String,
        department: String,
    },
    /// Specialist action (probe, command, etc.).
    SpecialistAction {
        specialist_id: String,
        action_type: ActionType,
        description: String,
    },
    /// Escalation from junior to senior.
    Escalation {
        from_id: String,
        from_name: String,
        to_id: String,
        to_name: String,
        reason: String,
    },
    /// Recovery attempt.
    RecoveryAttempt {
        subsystem: String,
        attempt_num: u32,
        success: bool,
    },
    /// Resolution.
    Resolution {
        specialist_id: String,
        specialist_name: String,
        confidence: f32,
        learned_recipe: bool,
    },
    /// Failure.
    Failure {
        reason: String,
        specialist_id: Option<String>,
    },
    /// Internal note (for debugging, always redacted in non-debug).
    InternalNote {
        note: String,
    },
}

/// Type of action taken by a specialist.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// Running a system probe/command.
    Probe,
    /// Checking documentation.
    Documentation,
    /// Applying a recipe.
    Recipe,
    /// Analyzing output.
    Analysis,
    /// Other action.
    Other,
}

impl TimelineEntry {
    /// Create a new timeline entry.
    pub fn new(seq: u64, kind: EntryKind) -> Self {
        let internal_only = matches!(kind, EntryKind::InternalNote { .. });
        Self {
            seq,
            timestamp: Utc::now(),
            kind,
            internal_only,
        }
    }

    /// Mark this entry as internal-only.
    pub fn as_internal(mut self) -> Self {
        self.internal_only = true;
        self
    }

    /// Get a short description for display.
    pub fn short_description(&self) -> String {
        match &self.kind {
            EntryKind::TicketCreated { department, .. } => {
                format!("Ticket created for {}", department)
            }
            EntryKind::TranslatorDecision { routed_to, .. } => {
                format!("Routed to {}", routed_to)
            }
            EntryKind::SpecialistAssigned { specialist_name, level, .. } => {
                format!("{} ({}) assigned", specialist_name, level)
            }
            EntryKind::SpecialistAction { description, .. } => {
                description.clone()
            }
            EntryKind::Escalation { from_name, to_name, .. } => {
                format!("Escalated from {} to {}", from_name, to_name)
            }
            EntryKind::RecoveryAttempt { subsystem, success, .. } => {
                let status = if *success { "succeeded" } else { "failed" };
                format!("Recovery {} {}", subsystem, status)
            }
            EntryKind::Resolution { specialist_name, .. } => {
                format!("Resolved by {}", specialist_name)
            }
            EntryKind::Failure { reason, .. } => {
                format!("Failed: {}", reason)
            }
            EntryKind::InternalNote { note } => {
                format!("[internal] {}", truncate(note, 30))
            }
        }
    }
}

/// The complete dialogue timeline for a ticket.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogueTimeline {
    /// The ticket ID this timeline belongs to.
    pub ticket_id: String,
    /// The original question.
    pub question: String,
    /// Ordered entries (by seq).
    pub entries: Vec<TimelineEntry>,
    /// Next sequence number (internal use).
    #[serde(default)]
    pub next_seq: u64,
    /// Whether the timeline is complete (resolved or failed).
    pub complete: bool,
}

impl DialogueTimeline {
    /// Create a new timeline for a ticket.
    pub fn new(ticket_id: &str, question: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            question: question.to_string(),
            entries: Vec::new(),
            next_seq: 0,
            complete: false,
        }
    }

    /// Add an entry to the timeline.
    pub fn add(&mut self, kind: EntryKind) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.entries.push(TimelineEntry::new(seq, kind));
        seq
    }

    /// Add an internal-only entry.
    pub fn add_internal(&mut self, kind: EntryKind) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.entries.push(TimelineEntry::new(seq, kind).as_internal());
        seq
    }

    /// Mark the timeline as complete.
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Get entries, optionally filtering internal-only entries.
    pub fn entries_filtered(&self, include_internal: bool) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|e| include_internal || !e.internal_only)
            .collect()
    }

    /// Get the last entry.
    pub fn last_entry(&self) -> Option<&TimelineEntry> {
        self.entries.last()
    }

    /// Check if timeline is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Truncate a string for display.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_ordering() {
        let mut timeline = DialogueTimeline::new("CN-001", "test question");

        timeline.add(EntryKind::TicketCreated {
            ticket_id: "CN-001".to_string(),
            question: "test".to_string(),
            department: "System".to_string(),
        });
        timeline.add(EntryKind::SpecialistAssigned {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            level: "Junior".to_string(),
            department: "System".to_string(),
        });

        assert_eq!(timeline.entries[0].seq, 0);
        assert_eq!(timeline.entries[1].seq, 1);
    }

    #[test]
    fn test_internal_filtering() {
        let mut timeline = DialogueTimeline::new("CN-001", "test");

        timeline.add(EntryKind::TicketCreated {
            ticket_id: "CN-001".to_string(),
            question: "test".to_string(),
            department: "System".to_string(),
        });
        timeline.add_internal(EntryKind::InternalNote {
            note: "debug info".to_string(),
        });

        assert_eq!(timeline.entries_filtered(true).len(), 2);
        assert_eq!(timeline.entries_filtered(false).len(), 1);
    }

    #[test]
    fn test_complete_marking() {
        let mut timeline = DialogueTimeline::new("CN-001", "test");
        assert!(!timeline.complete);
        timeline.mark_complete();
        assert!(timeline.complete);
    }
}

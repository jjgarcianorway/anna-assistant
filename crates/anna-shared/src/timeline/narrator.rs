//! Narrator - Converts timeline entries to human-readable dialogue.
//!
//! The Translator acts as narrator, explaining what is happening without
//! exposing internal data structures or inventing facts.

use super::types::{ActionType, DialogueTimeline, EntryKind, TimelineEntry};

/// Dialogue line with speaker and message.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueLine {
    /// The speaker (e.g., "Anna", "James (Jr, System)").
    pub speaker: String,
    /// The recipient (e.g., "James", "Anna", None for narration).
    pub recipient: Option<String>,
    /// The message content.
    pub message: String,
    /// Timestamp offset from start (for display).
    pub offset_ms: u64,
}

impl DialogueLine {
    /// Create a narration line (no specific speaker/recipient).
    pub fn narration(message: &str, offset_ms: u64) -> Self {
        Self {
            speaker: "---".to_string(),
            recipient: None,
            message: message.to_string(),
            offset_ms,
        }
    }

    /// Create a dialogue line from speaker to recipient.
    pub fn dialogue(speaker: &str, recipient: &str, message: &str, offset_ms: u64) -> Self {
        Self {
            speaker: speaker.to_string(),
            recipient: Some(recipient.to_string()),
            message: message.to_string(),
            offset_ms,
        }
    }

    /// Format for display.
    pub fn format(&self) -> String {
        let time = format!("[{:.1}s]", self.offset_ms as f64 / 1000.0);
        match &self.recipient {
            Some(recip) => format!("{} {} -> {}: {}", time, self.speaker, recip, self.message),
            None => format!("{} {}", time, self.message),
        }
    }
}

/// Narrate a single timeline entry.
pub fn narrate_entry(entry: &TimelineEntry, start_ts: i64) -> Vec<DialogueLine> {
    let offset_ms = (entry.timestamp.timestamp_millis() - start_ts).max(0) as u64;

    match &entry.kind {
        EntryKind::TicketCreated { department, question, .. } => {
            vec![
                DialogueLine::narration("--- internal comms ---", offset_ms),
                DialogueLine::dialogue(
                    "Anna",
                    department,
                    &format!("New request: \"{}\"", truncate(question, 40)),
                    offset_ms,
                ),
            ]
        }

        EntryKind::TranslatorDecision { interpreted_as, confidence, routed_to } => {
            let conf_text = if *confidence >= 0.9 {
                "clear request"
            } else if *confidence >= 0.7 {
                "likely intent"
            } else {
                "uncertain, needs investigation"
            };
            vec![DialogueLine::dialogue(
                "Anna",
                routed_to,
                &format!("Interpreted as: {} ({})", interpreted_as, conf_text),
                offset_ms,
            )]
        }

        EntryKind::SpecialistAssigned { specialist_name, level, department, .. } => {
            let level_short = if level == "Junior" { "Jr" } else { "Sr" };
            vec![DialogueLine::dialogue(
                &format!("{} ({}, {})", specialist_name, level_short, department),
                "Anna",
                "I'll take this one.",
                offset_ms,
            )]
        }

        EntryKind::SpecialistAction { specialist_id, action_type, description } => {
            let action_verb = match action_type {
                ActionType::Probe => "Running",
                ActionType::Documentation => "Checking",
                ActionType::Recipe => "Applying",
                ActionType::Analysis => "Analyzing",
                ActionType::Other => "Working on",
            };
            vec![DialogueLine::narration(
                &format!("[{}] {} {}", specialist_id, action_verb, description),
                offset_ms,
            )]
        }

        EntryKind::Escalation { from_name, to_name, reason, .. } => {
            vec![
                DialogueLine::dialogue(from_name, to_name, "Need your expertise here.", offset_ms),
                DialogueLine::dialogue(from_name, to_name, &format!("Reason: {}", reason), offset_ms),
            ]
        }

        EntryKind::RecoveryAttempt { subsystem, attempt_num, success } => {
            let status = if *success { "recovered" } else { "still working on it" };
            vec![DialogueLine::narration(
                &format!("[recovery] {} attempt {} - {}", subsystem, attempt_num, status),
                offset_ms,
            )]
        }

        EntryKind::Resolution { specialist_name, confidence, learned_recipe, .. } => {
            let mut lines = vec![DialogueLine::dialogue(
                specialist_name,
                "Anna",
                &format!("Resolved with {:.0}% confidence.", confidence * 100.0),
                offset_ms,
            )];
            if *learned_recipe {
                lines.push(DialogueLine::dialogue(
                    "Anna",
                    specialist_name,
                    "Noted. I'll remember this for next time.",
                    offset_ms,
                ));
            }
            lines
        }

        EntryKind::Failure { reason, specialist_id } => {
            let speaker = specialist_id
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Anna");
            vec![DialogueLine::dialogue(
                speaker,
                "Anna",
                &format!("Could not resolve: {}", reason),
                offset_ms,
            )]
        }

        EntryKind::InternalNote { note } => {
            vec![DialogueLine::narration(&format!("[debug] {}", note), offset_ms)]
        }
    }
}

/// Narrate an entire timeline into dialogue lines.
pub fn narrate_timeline(timeline: &DialogueTimeline, include_internal: bool) -> Vec<DialogueLine> {
    if timeline.is_empty() {
        return vec![];
    }

    let start_ts = timeline.entries[0].timestamp.timestamp_millis();
    let mut lines = Vec::new();

    for entry in timeline.entries_filtered(include_internal) {
        lines.extend(narrate_entry(entry, start_ts));
    }

    lines
}

/// Format a complete timeline as readable text.
pub fn format_timeline(timeline: &DialogueTimeline, include_internal: bool) -> String {
    let lines = narrate_timeline(timeline, include_internal);
    lines.iter().map(|l| l.format()).collect::<Vec<_>>().join("\n")
}

/// Truncate string for display.
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
    use crate::timeline::types::DialogueTimeline;

    #[test]
    fn test_narrate_ticket_created() {
        let mut timeline = DialogueTimeline::new("CN-001", "how much disk space");
        timeline.add(EntryKind::TicketCreated {
            ticket_id: "CN-001".to_string(),
            question: "how much disk space".to_string(),
            department: "Storage".to_string(),
        });

        let lines = narrate_timeline(&timeline, false);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].message.contains("internal comms"));
        assert!(lines[1].message.contains("disk space"));
    }

    #[test]
    fn test_narrate_escalation() {
        let mut timeline = DialogueTimeline::new("CN-001", "test");
        timeline.add(EntryKind::Escalation {
            from_id: "net-jr".to_string(),
            from_name: "Michael".to_string(),
            to_id: "net-sr".to_string(),
            to_name: "Sarah".to_string(),
            reason: "Complex routing issue".to_string(),
        });

        let lines = narrate_timeline(&timeline, false);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].speaker, "Michael");
        assert_eq!(lines[0].recipient, Some("Sarah".to_string()));
    }

    #[test]
    fn test_internal_filtering_in_narration() {
        let mut timeline = DialogueTimeline::new("CN-001", "test");
        timeline.add(EntryKind::TicketCreated {
            ticket_id: "CN-001".to_string(),
            question: "test".to_string(),
            department: "System".to_string(),
        });
        timeline.add_internal(EntryKind::InternalNote {
            note: "debug info".to_string(),
        });

        let with_internal = narrate_timeline(&timeline, true);
        let without_internal = narrate_timeline(&timeline, false);

        assert!(with_internal.len() > without_internal.len());
    }

    #[test]
    fn test_format_timeline() {
        let mut timeline = DialogueTimeline::new("CN-001", "test");
        timeline.add(EntryKind::SpecialistAssigned {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            level: "Junior".to_string(),
            department: "System".to_string(),
        });

        let formatted = format_timeline(&timeline, false);
        assert!(formatted.contains("James"));
        assert!(formatted.contains("Jr"));
    }
}

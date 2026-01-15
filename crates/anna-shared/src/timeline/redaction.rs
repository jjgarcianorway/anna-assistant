//! Redaction - Rules for hiding internal-only data.
//!
//! Internal data is marked and redacted by default.
//! Debug mode may expose raw events.

use super::types::{DialogueTimeline, EntryKind, TimelineEntry};

/// Redaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RedactionMode {
    /// Normal mode - internal entries redacted.
    #[default]
    Normal,
    /// Debug mode - all entries visible.
    Debug,
}

/// Patterns that should be redacted from output.
const REDACTION_PATTERNS: &[(&str, &str)] = &[
    // Passwords in URLs
    (r"://[^:]+:[^@]+@", "://<credentials>@"),
    // API keys and tokens
    (r"(?i)(api[_-]?key|token|secret|password)=\S+", "$1=<redacted>"),
    // Home directory paths
    (r"/home/[^/\s]+", "/home/<user>"),
    // SSH key paths
    (r"\.ssh/[^\s]+", ".ssh/<key>"),
    // UUIDs (often internal IDs)
    (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "<uuid>"),
];

/// Redact sensitive patterns from text.
pub fn redact_text(text: &str) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in REDACTION_PATTERNS {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }
    result
}

/// Check if an entry should be visible in the given mode.
pub fn is_visible(entry: &TimelineEntry, mode: RedactionMode) -> bool {
    match mode {
        RedactionMode::Debug => true,
        RedactionMode::Normal => !entry.internal_only,
    }
}

/// Filter timeline entries by redaction mode.
pub fn filter_entries(timeline: &DialogueTimeline, mode: RedactionMode) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|e| is_visible(e, mode))
        .collect()
}

/// Redact an entry's content if needed.
pub fn redact_entry(entry: &TimelineEntry) -> TimelineEntry {
    let kind = match &entry.kind {
        EntryKind::SpecialistAction { specialist_id, action_type, description } => {
            EntryKind::SpecialistAction {
                specialist_id: specialist_id.clone(),
                action_type: action_type.clone(),
                description: redact_text(description),
            }
        }
        EntryKind::InternalNote { note } => {
            EntryKind::InternalNote {
                note: redact_text(note),
            }
        }
        EntryKind::Failure { reason, specialist_id } => {
            EntryKind::Failure {
                reason: redact_text(reason),
                specialist_id: specialist_id.clone(),
            }
        }
        other => other.clone(),
    };

    TimelineEntry {
        seq: entry.seq,
        timestamp: entry.timestamp,
        kind,
        internal_only: entry.internal_only,
    }
}

/// Redact all entries in a timeline.
pub fn redact_timeline(timeline: &DialogueTimeline) -> DialogueTimeline {
    DialogueTimeline {
        ticket_id: timeline.ticket_id.clone(),
        question: redact_text(&timeline.question),
        entries: timeline.entries.iter().map(redact_entry).collect(),
        next_seq: timeline.entries.len() as u64,
        complete: timeline.complete,
    }
}

/// Validate that no forbidden patterns appear in user-visible output.
pub fn validate_no_forbidden_patterns(text: &str) -> Result<(), Vec<String>> {
    let forbidden = [
        "sudo systemctl",
        "Run: sudo",
        "Try: sudo",
        "Execute: ",
        "Run this command",
    ];

    let mut violations = Vec::new();
    for pattern in &forbidden {
        if text.contains(pattern) {
            violations.push(format!("Found forbidden pattern: {}", pattern));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_password() {
        let text = "curl https://user:secret123@example.com";
        let redacted = redact_text(text);
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("<credentials>"));
    }

    #[test]
    fn test_redact_home_path() {
        let text = "Reading /home/johndoe/.config/file";
        let redacted = redact_text(text);
        assert!(!redacted.contains("johndoe"));
        assert!(redacted.contains("<user>"));
    }

    #[test]
    fn test_redact_api_key() {
        let text = "api_key=abc123secret token=xyz789";
        let redacted = redact_text(text);
        assert!(!redacted.contains("abc123"));
        assert!(redacted.contains("<redacted>"));
    }

    #[test]
    fn test_visibility_normal_mode() {
        let public_entry = TimelineEntry::new(0, EntryKind::TicketCreated {
            ticket_id: "CN-001".to_string(),
            question: "test".to_string(),
            department: "System".to_string(),
        });
        let internal_entry = TimelineEntry::new(1, EntryKind::InternalNote {
            note: "debug".to_string(),
        }).as_internal();

        assert!(is_visible(&public_entry, RedactionMode::Normal));
        assert!(!is_visible(&internal_entry, RedactionMode::Normal));
    }

    #[test]
    fn test_visibility_debug_mode() {
        let internal_entry = TimelineEntry::new(0, EntryKind::InternalNote {
            note: "debug".to_string(),
        }).as_internal();

        assert!(is_visible(&internal_entry, RedactionMode::Debug));
    }

    #[test]
    fn test_validate_forbidden_patterns() {
        assert!(validate_no_forbidden_patterns("normal text").is_ok());
        assert!(validate_no_forbidden_patterns("Run: sudo systemctl start").is_err());
        assert!(validate_no_forbidden_patterns("Try: sudo rm -rf").is_err());
    }

    #[test]
    fn test_redact_timeline() {
        let mut timeline = DialogueTimeline::new("CN-001", "check /home/john/file");
        timeline.add(EntryKind::SpecialistAction {
            specialist_id: "sys-jr".to_string(),
            action_type: super::super::types::ActionType::Probe,
            description: "cat /home/john/.ssh/id_rsa".to_string(),
        });

        let redacted = redact_timeline(&timeline);
        assert!(!redacted.question.contains("john"));
        assert!(!format!("{:?}", redacted.entries[0]).contains("john"));
    }
}

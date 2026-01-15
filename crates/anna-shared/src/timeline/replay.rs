//! Replay - Deterministic replay of completed timelines.
//!
//! A completed ticket can be replayed to produce the same dialogue.

use super::narrator::{narrate_timeline, DialogueLine};
use super::redaction::RedactionMode;
use super::types::DialogueTimeline;
use serde::{Deserialize, Serialize};

/// A replay session for a completed timeline.
#[derive(Debug, Clone)]
pub struct ReplaySession {
    /// The timeline to replay.
    timeline: DialogueTimeline,
    /// Current position in the dialogue.
    position: usize,
    /// Narrated dialogue lines.
    dialogue: Vec<DialogueLine>,
    /// Redaction mode.
    mode: RedactionMode,
}

impl ReplaySession {
    /// Create a replay session from a timeline.
    pub fn new(timeline: DialogueTimeline, mode: RedactionMode) -> Self {
        let include_internal = matches!(mode, RedactionMode::Debug);
        let dialogue = narrate_timeline(&timeline, include_internal);
        Self {
            timeline,
            position: 0,
            dialogue,
            mode,
        }
    }

    /// Get the next dialogue line.
    pub fn next(&mut self) -> Option<&DialogueLine> {
        if self.position < self.dialogue.len() {
            let line = &self.dialogue[self.position];
            self.position += 1;
            Some(line)
        } else {
            None
        }
    }

    /// Peek at the next line without advancing.
    pub fn peek(&self) -> Option<&DialogueLine> {
        self.dialogue.get(self.position)
    }

    /// Reset to the beginning.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Check if replay is complete.
    pub fn is_complete(&self) -> bool {
        self.position >= self.dialogue.len()
    }

    /// Get current position.
    pub fn current_position(&self) -> usize {
        self.position
    }

    /// Get total lines.
    pub fn total_lines(&self) -> usize {
        self.dialogue.len()
    }

    /// Get the underlying timeline.
    pub fn timeline(&self) -> &DialogueTimeline {
        &self.timeline
    }

    /// Get all dialogue lines.
    pub fn all_lines(&self) -> &[DialogueLine] {
        &self.dialogue
    }
}

/// Replay fingerprint for determinism verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayFingerprint {
    /// Ticket ID.
    pub ticket_id: String,
    /// Number of timeline entries.
    pub entry_count: usize,
    /// Number of dialogue lines (non-debug mode).
    pub dialogue_count: usize,
    /// Hash of entry sequence.
    pub sequence_hash: u64,
}

impl ReplayFingerprint {
    /// Create a fingerprint from a timeline.
    pub fn from_timeline(timeline: &DialogueTimeline) -> Self {
        let dialogue = narrate_timeline(timeline, false);
        let sequence_hash = compute_sequence_hash(timeline);
        Self {
            ticket_id: timeline.ticket_id.clone(),
            entry_count: timeline.entries.len(),
            dialogue_count: dialogue.len(),
            sequence_hash,
        }
    }

    /// Verify that a timeline matches this fingerprint.
    pub fn verify(&self, timeline: &DialogueTimeline) -> bool {
        let other = Self::from_timeline(timeline);
        self == &other
    }
}

/// Compute a hash of the timeline sequence for determinism verification.
fn compute_sequence_hash(timeline: &DialogueTimeline) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    timeline.ticket_id.hash(&mut hasher);
    for entry in &timeline.entries {
        entry.seq.hash(&mut hasher);
        entry.short_description().hash(&mut hasher);
    }
    hasher.finish()
}

/// Verify deterministic replay: same inputs should produce same dialogue.
pub fn verify_deterministic_replay(timeline: &DialogueTimeline) -> bool {
    let dialogue1 = narrate_timeline(timeline, false);
    let dialogue2 = narrate_timeline(timeline, false);

    if dialogue1.len() != dialogue2.len() {
        return false;
    }

    for (d1, d2) in dialogue1.iter().zip(dialogue2.iter()) {
        if d1.speaker != d2.speaker || d1.message != d2.message {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::types::EntryKind;

    fn create_test_timeline() -> DialogueTimeline {
        let mut timeline = DialogueTimeline::new("CN-001", "test question");
        timeline.add(EntryKind::TicketCreated {
            ticket_id: "CN-001".to_string(),
            question: "test question".to_string(),
            department: "System".to_string(),
        });
        timeline.add(EntryKind::SpecialistAssigned {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            level: "Junior".to_string(),
            department: "System".to_string(),
        });
        timeline.add(EntryKind::Resolution {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            confidence: 0.92,
            learned_recipe: true,
        });
        timeline.mark_complete();
        timeline
    }

    #[test]
    fn test_replay_session_iteration() {
        let timeline = create_test_timeline();
        let mut session = ReplaySession::new(timeline, RedactionMode::Normal);

        let mut count = 0;
        while session.next().is_some() {
            count += 1;
        }
        assert!(count > 0);
        assert!(session.is_complete());
    }

    #[test]
    fn test_replay_session_reset() {
        let timeline = create_test_timeline();
        let mut session = ReplaySession::new(timeline, RedactionMode::Normal);

        session.next();
        session.next();
        assert!(session.current_position() > 0);

        session.reset();
        assert_eq!(session.current_position(), 0);
    }

    #[test]
    fn test_deterministic_replay() {
        let timeline = create_test_timeline();
        assert!(verify_deterministic_replay(&timeline));
    }

    #[test]
    fn test_fingerprint_verification() {
        let timeline = create_test_timeline();
        let fingerprint = ReplayFingerprint::from_timeline(&timeline);

        assert!(fingerprint.verify(&timeline));
        assert_eq!(fingerprint.ticket_id, "CN-001");
    }

    #[test]
    fn test_fingerprint_mismatch() {
        let timeline1 = create_test_timeline();
        let fingerprint1 = ReplayFingerprint::from_timeline(&timeline1);

        let mut timeline2 = create_test_timeline();
        timeline2.add(EntryKind::InternalNote { note: "extra".to_string() });
        let fingerprint2 = ReplayFingerprint::from_timeline(&timeline2);

        assert_ne!(fingerprint1, fingerprint2);
        assert!(!fingerprint1.verify(&timeline2));
    }

    #[test]
    fn test_replay_consistency() {
        let timeline = create_test_timeline();
        let session1 = ReplaySession::new(timeline.clone(), RedactionMode::Normal);
        let session2 = ReplaySession::new(timeline, RedactionMode::Normal);

        assert_eq!(session1.total_lines(), session2.total_lines());

        for (l1, l2) in session1.all_lines().iter().zip(session2.all_lines().iter()) {
            assert_eq!(l1.speaker, l2.speaker);
            assert_eq!(l1.message, l2.message);
        }
    }
}

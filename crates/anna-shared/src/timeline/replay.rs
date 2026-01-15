//! Replay - Deterministic replay of completed timelines.
//!
//! A completed ticket can be replayed to produce the same dialogue.
//!
//! REPLAY REDACTION ENFORCEMENT (v0.3.45):
//! - Replays must obey the exposure level active at record time
//! - No elevation via replay - cannot see more than was recorded
//! - Debug information only visible if recorded at Debug level

use crate::exposure::ExposureLevel;
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
    /// Redaction mode (legacy).
    mode: RedactionMode,
    /// v0.3.45: Exposure level at record time.
    recorded_at: ExposureLevel,
    /// v0.3.45: Current playback exposure level.
    playback_level: ExposureLevel,
}

impl ReplaySession {
    /// Create a replay session from a timeline.
    pub fn new(timeline: DialogueTimeline, mode: RedactionMode) -> Self {
        let include_internal = matches!(mode, RedactionMode::Debug);
        let dialogue = narrate_timeline(&timeline, include_internal);
        let recorded_at = if include_internal {
            ExposureLevel::Debug
        } else {
            ExposureLevel::Dialogue
        };
        Self {
            timeline,
            position: 0,
            dialogue,
            mode,
            recorded_at,
            playback_level: recorded_at,
        }
    }

    /// v0.3.45: Create with explicit exposure levels.
    pub fn with_exposure(
        timeline: DialogueTimeline,
        recorded_at: ExposureLevel,
        playback_level: ExposureLevel,
    ) -> Self {
        // Enforce: cannot elevate above recorded level
        let effective_level = std::cmp::min(recorded_at, playback_level);
        let include_internal = effective_level >= ExposureLevel::Debug;
        let dialogue = narrate_timeline(&timeline, include_internal);

        // Filter dialogue based on exposure level
        let dialogue = Self::filter_by_exposure(&dialogue, effective_level);

        let mode = if effective_level >= ExposureLevel::Debug {
            RedactionMode::Debug
        } else {
            RedactionMode::Normal
        };

        Self {
            timeline,
            position: 0,
            dialogue,
            mode,
            recorded_at,
            playback_level: effective_level,
        }
    }

    /// Filter dialogue lines by exposure level.
    fn filter_by_exposure(lines: &[DialogueLine], level: ExposureLevel) -> Vec<DialogueLine> {
        if level >= ExposureLevel::Dialogue {
            lines.to_vec()
        } else if level >= ExposureLevel::Summary {
            // Summary: only show resolution lines
            lines.iter()
                .filter(|l| l.message.contains("Resolved") || l.message.contains("complete"))
                .cloned()
                .collect()
        } else {
            // Silent: no dialogue
            Vec::new()
        }
    }

    /// Check if playback was restricted below recorded level.
    pub fn was_restricted(&self) -> bool {
        self.playback_level < self.recorded_at
    }

    /// Get the recorded exposure level.
    pub fn recorded_level(&self) -> ExposureLevel {
        self.recorded_at
    }

    /// Get the playback exposure level.
    pub fn playback_level(&self) -> ExposureLevel {
        self.playback_level
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

    // v0.3.45: Exposure level tests

    #[test]
    fn test_replay_exposure_no_elevation() {
        let timeline = create_test_timeline();

        // Record at Dialogue level
        let session = ReplaySession::with_exposure(
            timeline.clone(),
            ExposureLevel::Dialogue,
            ExposureLevel::Debug, // Try to elevate
        );

        // Should be capped at Dialogue (cannot elevate above recorded level)
        assert_eq!(session.playback_level(), ExposureLevel::Dialogue);
        // Not "restricted" since we're at recorded level, just capped
        assert!(!session.was_restricted());
    }

    #[test]
    fn test_replay_exposure_can_restrict() {
        let timeline = create_test_timeline();

        // Record at Debug level
        let session = ReplaySession::with_exposure(
            timeline.clone(),
            ExposureLevel::Debug,
            ExposureLevel::Summary, // Restrict playback
        );

        // Should be restricted to Summary
        assert_eq!(session.playback_level(), ExposureLevel::Summary);
        assert!(session.was_restricted());
    }

    #[test]
    fn test_replay_silent_shows_nothing() {
        let timeline = create_test_timeline();

        let session = ReplaySession::with_exposure(
            timeline,
            ExposureLevel::Dialogue,
            ExposureLevel::Silent,
        );

        assert_eq!(session.total_lines(), 0);
    }

    #[test]
    fn test_replay_respects_recorded_level() {
        let timeline = create_test_timeline();

        // If recorded at Silent, can never see dialogue
        let session = ReplaySession::with_exposure(
            timeline,
            ExposureLevel::Silent,
            ExposureLevel::Debug, // Even at Debug playback
        );

        // Silent recording means no dialogue to replay
        assert_eq!(session.playback_level(), ExposureLevel::Silent);
    }
}

//! Transcript event extension methods for ticket and review events.
//!
//! Extension trait to keep transcript.rs under 400 lines.
//! Provides helper methods for v0.0.25 ticket and v0.0.26 review events.

use crate::transcript::{Actor, TranscriptEvent, TranscriptEventKind};

/// Extension trait for creating ticket and review transcript events
pub trait TranscriptEventExt {
    // === Ticket lifecycle event helpers (v0.0.25) ===

    /// Create a ticket created event
    fn ticket_created(
        elapsed_ms: u64,
        ticket_id: impl Into<String>,
        domain: impl Into<String>,
        intent: impl Into<String>,
        evidence_required: bool,
    ) -> TranscriptEvent;

    /// Create a ticket status changed event
    fn ticket_status_changed(
        elapsed_ms: u64,
        ticket_id: impl Into<String>,
        from_status: impl Into<String>,
        to_status: impl Into<String>,
    ) -> TranscriptEvent;

    /// Create a junior review event
    fn junior_review(
        elapsed_ms: u64,
        attempt: u8,
        score: u8,
        verified: bool,
        issues: Vec<String>,
    ) -> TranscriptEvent;

    /// Create a senior escalation event
    fn senior_escalation(
        elapsed_ms: u64,
        successful: bool,
        reason: Option<String>,
    ) -> TranscriptEvent;

    /// Create a revision applied event
    fn revision_applied(elapsed_ms: u64, changes_made: Vec<String>) -> TranscriptEvent;

    // === Review gate event helpers (v0.0.26) ===

    /// Create a review gate decision event
    fn review_gate_decision(
        elapsed_ms: u64,
        decision: impl Into<String>,
        score: u8,
        requires_llm: bool,
    ) -> TranscriptEvent;

    /// Create a team review event
    fn team_review(
        elapsed_ms: u64,
        team: impl Into<String>,
        reviewer: impl Into<String>,
        decision: impl Into<String>,
        issues_count: usize,
    ) -> TranscriptEvent;
}

impl TranscriptEventExt for TranscriptEvent {
    fn ticket_created(
        elapsed_ms: u64,
        ticket_id: impl Into<String>,
        domain: impl Into<String>,
        intent: impl Into<String>,
        evidence_required: bool,
    ) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::Translator,
            to: Some(Actor::Anna),
            kind: TranscriptEventKind::TicketCreated {
                ticket_id: ticket_id.into(),
                domain: domain.into(),
                intent: intent.into(),
                evidence_required,
            },
        }
    }

    fn ticket_status_changed(
        elapsed_ms: u64,
        ticket_id: impl Into<String>,
        from_status: impl Into<String>,
        to_status: impl Into<String>,
    ) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::TicketStatusChanged {
                ticket_id: ticket_id.into(),
                from_status: from_status.into(),
                to_status: to_status.into(),
            },
        }
    }

    fn junior_review(
        elapsed_ms: u64,
        attempt: u8,
        score: u8,
        verified: bool,
        issues: Vec<String>,
    ) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::Junior,
            to: Some(Actor::Anna),
            kind: TranscriptEventKind::JuniorReview {
                attempt,
                score,
                verified,
                issues,
            },
        }
    }

    fn senior_escalation(
        elapsed_ms: u64,
        successful: bool,
        reason: Option<String>,
    ) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::Senior,
            to: Some(Actor::Junior),
            kind: TranscriptEventKind::SeniorEscalation { successful, reason },
        }
    }

    fn revision_applied(elapsed_ms: u64, changes_made: Vec<String>) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::Anna,
            to: None,
            kind: TranscriptEventKind::RevisionApplied { changes_made },
        }
    }

    fn review_gate_decision(
        elapsed_ms: u64,
        decision: impl Into<String>,
        score: u8,
        requires_llm: bool,
    ) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::System,
            to: Some(Actor::Anna),
            kind: TranscriptEventKind::ReviewGateDecision {
                decision: decision.into(),
                score,
                requires_llm,
            },
        }
    }

    fn team_review(
        elapsed_ms: u64,
        team: impl Into<String>,
        reviewer: impl Into<String>,
        decision: impl Into<String>,
        issues_count: usize,
    ) -> TranscriptEvent {
        TranscriptEvent {
            elapsed_ms,
            from: Actor::Junior,
            to: Some(Actor::Anna),
            kind: TranscriptEventKind::TeamReview {
                team: team.into(),
                reviewer: reviewer.into(),
                decision: decision.into(),
                issues_count,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_created_event() {
        let event = TranscriptEvent::ticket_created(100, "T-123", "system", "question", true);
        assert_eq!(event.elapsed_ms, 100);
        assert_eq!(event.from, Actor::Translator);
        assert!(matches!(
            event.kind,
            TranscriptEventKind::TicketCreated { evidence_required: true, .. }
        ));
    }

    #[test]
    fn test_review_gate_decision_event() {
        let event = TranscriptEvent::review_gate_decision(200, "accept", 85, false);
        assert_eq!(event.elapsed_ms, 200);
        assert_eq!(event.from, Actor::System);
        assert!(matches!(
            event.kind,
            TranscriptEventKind::ReviewGateDecision { requires_llm: false, .. }
        ));
    }

    #[test]
    fn test_team_review_event() {
        let event = TranscriptEvent::team_review(300, "storage", "junior", "revise", 2);
        assert_eq!(event.elapsed_ms, 300);
        assert!(matches!(
            event.kind,
            TranscriptEventKind::TeamReview { issues_count: 2, .. }
        ));
    }
}

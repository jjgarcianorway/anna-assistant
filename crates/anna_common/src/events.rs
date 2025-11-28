//! Events Module v0.60.0
//!
//! Structured events for live progress and conversation logging.
//! No LLM calls - all messages generated from templates.
//!
//! ## Architecture
//!
//! ```text
//! +-----------+     +-------------+     +------------------+
//! | annad     | --> | EventStream | --> | annactl display  |
//! | (emits)   |     | (channel)   |     | (renders)        |
//! +-----------+     +-------------+     +------------------+
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Actor in the conversation narrative
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actor {
    /// Anna - orchestrator and user-facing voice
    Anna,
    /// Junior - LLM A, planning and probe selection
    Junior,
    /// Senior - LLM B, supervisor and auditor
    Senior,
    /// System - internal operations
    System,
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Actor::Anna => write!(f, "Anna"),
            Actor::Junior => write!(f, "Junior"),
            Actor::Senior => write!(f, "Senior"),
            Actor::System => write!(f, "System"),
        }
    }
}

/// Kind of event in the processing pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    /// User asked a question
    QuestionReceived,
    /// Starting to classify the question
    ClassificationStarted,
    /// Classification complete
    ClassificationDone {
        question_type: String,
        confidence: f32,
    },
    /// Probes/commands have been planned
    ProbesPlanned {
        probe_ids: Vec<String>,
    },
    /// Executing a command
    CommandRunning {
        command: String,
    },
    /// Command finished
    CommandDone {
        command: String,
        success: bool,
        duration_ms: u64,
    },
    /// Senior review starting
    SeniorReviewStarted,
    /// Senior review complete
    SeniorReviewDone {
        reliability_score: f32,
    },
    /// Need user clarification
    UserClarificationNeeded {
        question: String,
    },
    /// Synthesizing the answer
    AnswerSynthesizing,
    /// Answer is ready
    AnswerReady {
        reliability_score: f32,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// A single event in the processing pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaEvent {
    /// When this event occurred
    pub timestamp: DateTime<Utc>,
    /// Who is "speaking" in this event
    pub actor: Actor,
    /// What kind of event this is
    pub kind: EventKind,
}

impl AnnaEvent {
    /// Create a new event with current timestamp
    pub fn new(actor: Actor, kind: EventKind) -> Self {
        Self {
            timestamp: Utc::now(),
            actor,
            kind,
        }
    }

    /// Format as a progress line for display
    pub fn to_progress_line(&self) -> String {
        let actor_prefix = format!("[{}]", self.actor);
        let message = match &self.kind {
            EventKind::QuestionReceived => {
                "Reading your question and planning next steps.".to_string()
            }
            EventKind::ClassificationStarted => {
                "Analyzing your question...".to_string()
            }
            EventKind::ClassificationDone { question_type, confidence } => {
                format!(
                    "Classified as {} (confidence: {:.0}%).",
                    question_type,
                    confidence * 100.0
                )
            }
            EventKind::ProbesPlanned { probe_ids } => {
                if probe_ids.len() == 1 {
                    format!("Planning to use probe: {}.", probe_ids[0])
                } else {
                    format!("Planning to use {} probes.", probe_ids.len())
                }
            }
            EventKind::CommandRunning { command } => {
                // Truncate long commands
                let display_cmd = if command.len() > 60 {
                    format!("{}...", &command[..57])
                } else {
                    command.clone()
                };
                format!("Running: {}", display_cmd)
            }
            EventKind::CommandDone { command: _, success, duration_ms } => {
                if *success {
                    format!("Command completed in {}ms.", duration_ms)
                } else {
                    format!("Command failed after {}ms.", duration_ms)
                }
            }
            EventKind::SeniorReviewStarted => {
                "Double-checking the answer and scoring reliability.".to_string()
            }
            EventKind::SeniorReviewDone { reliability_score } => {
                let color = reliability_color(*reliability_score);
                format!(
                    "Review complete. Reliability: {:.0}% ({}).",
                    reliability_score * 100.0,
                    color
                )
            }
            EventKind::UserClarificationNeeded { question } => {
                format!("I need a quick clarification: {}", question)
            }
            EventKind::AnswerSynthesizing => {
                "Preparing your answer...".to_string()
            }
            EventKind::AnswerReady { reliability_score } => {
                let color = reliability_color(*reliability_score);
                format!(
                    "Done. Reliability: {:.0}% ({}).",
                    reliability_score * 100.0,
                    color
                )
            }
            EventKind::Error { message } => {
                format!("Error: {}", message)
            }
        };

        format!("{:8} {}", actor_prefix, message)
    }

    /// Format as a conversation log entry (more detailed, past tense)
    pub fn to_log_entry(&self) -> String {
        let actor_prefix = format!("[{}]", self.actor);
        let message = match &self.kind {
            EventKind::QuestionReceived => {
                "I received your question and started processing.".to_string()
            }
            EventKind::ClassificationStarted => {
                "I began analyzing the question type.".to_string()
            }
            EventKind::ClassificationDone { question_type, confidence: _ } => {
                format!("I classified this as a {} question.", question_type)
            }
            EventKind::ProbesPlanned { probe_ids } => {
                format!("I selected {} probe(s) to gather evidence.", probe_ids.len())
            }
            EventKind::CommandRunning { command } => {
                format!("I ran: {}", truncate_cmd(command, 50))
            }
            EventKind::CommandDone { command: _, success, duration_ms } => {
                if *success {
                    format!("Command succeeded in {}ms.", duration_ms)
                } else {
                    format!("Command failed after {}ms.", duration_ms)
                }
            }
            EventKind::SeniorReviewStarted => {
                "I started reviewing the evidence for reliability.".to_string()
            }
            EventKind::SeniorReviewDone { reliability_score } => {
                format!(
                    "I completed the review - reliability is {:.0}%.",
                    reliability_score * 100.0
                )
            }
            EventKind::UserClarificationNeeded { question } => {
                format!("I asked for clarification: {}", question)
            }
            EventKind::AnswerSynthesizing => {
                "I began synthesizing the final answer.".to_string()
            }
            EventKind::AnswerReady { reliability_score } => {
                let color = reliability_color(*reliability_score);
                format!(
                    "I delivered the answer with {:.0}% reliability ({}).",
                    reliability_score * 100.0,
                    color
                )
            }
            EventKind::Error { message } => {
                format!("An error occurred: {}", message)
            }
        };

        format!("{:8} {}", actor_prefix, message)
    }
}

/// Convert reliability score to color label
fn reliability_color(score: f32) -> &'static str {
    if score >= 0.8 {
        "GREEN"
    } else if score >= 0.5 {
        "YELLOW"
    } else {
        "RED"
    }
}

/// Truncate command for display
fn truncate_cmd(cmd: &str, max_len: usize) -> String {
    if cmd.len() > max_len {
        format!("{}...", &cmd[..max_len.saturating_sub(3)])
    } else {
        cmd.to_string()
    }
}

/// Conversation log - a collection of events for a single question
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationLog {
    /// All events in order
    pub events: Vec<AnnaEvent>,
    /// Question that started this conversation
    pub question: String,
    /// Final answer (if available)
    pub answer: Option<String>,
    /// When the conversation started
    pub started_at: Option<DateTime<Utc>>,
    /// When the conversation ended
    pub ended_at: Option<DateTime<Utc>>,
}

impl ConversationLog {
    /// Create a new empty conversation log
    pub fn new(question: &str) -> Self {
        Self {
            events: Vec::new(),
            question: question.to_string(),
            answer: None,
            started_at: Some(Utc::now()),
            ended_at: None,
        }
    }

    /// Add an event to the log
    pub fn push(&mut self, event: AnnaEvent) {
        self.events.push(event);
    }

    /// Mark the conversation as complete
    pub fn complete(&mut self, answer: Option<String>) {
        self.answer = answer;
        self.ended_at = Some(Utc::now());
    }

    /// Get duration in milliseconds (if complete)
    pub fn duration_ms(&self) -> Option<i64> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds()),
            _ => None,
        }
    }

    /// Format as progress lines for streaming display
    pub fn to_progress_lines(&self) -> Vec<String> {
        self.events.iter().map(|e| e.to_progress_line()).collect()
    }

    /// Format as conversation log for debug output
    pub fn to_log_lines(&self) -> Vec<String> {
        self.events.iter().map(|e| e.to_log_entry()).collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> ConversationSummary {
        let commands_run = self.events.iter().filter(|e| {
            matches!(e.kind, EventKind::CommandDone { .. })
        }).count();

        let reliability = self.events.iter().find_map(|e| {
            match &e.kind {
                EventKind::AnswerReady { reliability_score } => Some(*reliability_score),
                EventKind::SeniorReviewDone { reliability_score } => Some(*reliability_score),
                _ => None,
            }
        });

        ConversationSummary {
            event_count: self.events.len(),
            commands_run,
            reliability_score: reliability,
            duration_ms: self.duration_ms(),
        }
    }
}

/// Summary statistics for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Total number of events
    pub event_count: usize,
    /// Number of commands executed
    pub commands_run: usize,
    /// Final reliability score (if available)
    pub reliability_score: Option<f32>,
    /// Total duration in milliseconds
    pub duration_ms: Option<i64>,
}

// ============================================================================
// Helper functions for creating events
// ============================================================================

/// Create a QuestionReceived event from Anna
pub fn question_received() -> AnnaEvent {
    AnnaEvent::new(Actor::Anna, EventKind::QuestionReceived)
}

/// Create a ClassificationStarted event from Junior
pub fn classification_started() -> AnnaEvent {
    AnnaEvent::new(Actor::Junior, EventKind::ClassificationStarted)
}

/// Create a ClassificationDone event from Junior
pub fn classification_done(question_type: &str, confidence: f32) -> AnnaEvent {
    AnnaEvent::new(
        Actor::Junior,
        EventKind::ClassificationDone {
            question_type: question_type.to_string(),
            confidence,
        },
    )
}

/// Create a ProbesPlanned event from Junior
pub fn probes_planned(probe_ids: Vec<String>) -> AnnaEvent {
    AnnaEvent::new(Actor::Junior, EventKind::ProbesPlanned { probe_ids })
}

/// Create a CommandRunning event from Anna
pub fn command_running(command: &str) -> AnnaEvent {
    AnnaEvent::new(
        Actor::Anna,
        EventKind::CommandRunning {
            command: command.to_string(),
        },
    )
}

/// Create a CommandDone event from Anna
pub fn command_done(command: &str, success: bool, duration_ms: u64) -> AnnaEvent {
    AnnaEvent::new(
        Actor::Anna,
        EventKind::CommandDone {
            command: command.to_string(),
            success,
            duration_ms,
        },
    )
}

/// Create a SeniorReviewStarted event from Senior
pub fn senior_review_started() -> AnnaEvent {
    AnnaEvent::new(Actor::Senior, EventKind::SeniorReviewStarted)
}

/// Create a SeniorReviewDone event from Senior
pub fn senior_review_done(reliability_score: f32) -> AnnaEvent {
    AnnaEvent::new(
        Actor::Senior,
        EventKind::SeniorReviewDone { reliability_score },
    )
}

/// Create a UserClarificationNeeded event from Anna
pub fn user_clarification_needed(question: &str) -> AnnaEvent {
    AnnaEvent::new(
        Actor::Anna,
        EventKind::UserClarificationNeeded {
            question: question.to_string(),
        },
    )
}

/// Create an AnswerSynthesizing event from Anna
pub fn answer_synthesizing() -> AnnaEvent {
    AnnaEvent::new(Actor::Anna, EventKind::AnswerSynthesizing)
}

/// Create an AnswerReady event from Anna
pub fn answer_ready(reliability_score: f32) -> AnnaEvent {
    AnnaEvent::new(
        Actor::Anna,
        EventKind::AnswerReady { reliability_score },
    )
}

/// Create an Error event from System
pub fn error_event(message: &str) -> AnnaEvent {
    AnnaEvent::new(
        Actor::System,
        EventKind::Error {
            message: message.to_string(),
        },
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_display() {
        assert_eq!(format!("{}", Actor::Anna), "Anna");
        assert_eq!(format!("{}", Actor::Junior), "Junior");
        assert_eq!(format!("{}", Actor::Senior), "Senior");
        assert_eq!(format!("{}", Actor::System), "System");
    }

    #[test]
    fn test_event_creation() {
        let event = question_received();
        assert_eq!(event.actor, Actor::Anna);
        assert!(matches!(event.kind, EventKind::QuestionReceived));
    }

    #[test]
    fn test_progress_line_format() {
        let event = question_received();
        let line = event.to_progress_line();
        assert!(line.contains("[Anna]"));
        assert!(line.contains("Reading your question"));
    }

    #[test]
    fn test_log_entry_format() {
        let event = classification_done("simple_probe", 0.95);
        let line = event.to_log_entry();
        assert!(line.contains("[Junior]"));
        assert!(line.contains("simple_probe"));
    }

    #[test]
    fn test_conversation_log() {
        let mut log = ConversationLog::new("What is my CPU?");
        log.push(question_received());
        log.push(classification_started());
        log.push(classification_done("SimpleProbe", 0.9));
        log.push(command_running("lscpu"));
        log.push(command_done("lscpu", true, 50));
        log.push(answer_ready(0.95));
        log.complete(Some("You have an Intel Core i7".to_string()));

        assert_eq!(log.events.len(), 6);
        assert!(log.ended_at.is_some());

        let summary = log.summary();
        assert_eq!(summary.event_count, 6);
        assert_eq!(summary.commands_run, 1);
        assert!(summary.reliability_score.is_some());
    }

    #[test]
    fn test_reliability_colors() {
        assert_eq!(reliability_color(0.95), "GREEN");
        assert_eq!(reliability_color(0.75), "YELLOW");
        assert_eq!(reliability_color(0.3), "RED");
    }

    #[test]
    fn test_command_truncation() {
        let long_cmd = "a".repeat(100);
        let event = command_running(&long_cmd);
        let line = event.to_progress_line();
        assert!(line.len() < 100);
        assert!(line.contains("..."));
    }

    #[test]
    fn test_probes_planned_single() {
        let event = probes_planned(vec!["cpu.info".to_string()]);
        let line = event.to_progress_line();
        assert!(line.contains("cpu.info"));
    }

    #[test]
    fn test_probes_planned_multiple() {
        let event = probes_planned(vec![
            "cpu.info".to_string(),
            "mem.info".to_string(),
            "disk.lsblk".to_string(),
        ]);
        let line = event.to_progress_line();
        assert!(line.contains("3 probes"));
    }

    #[test]
    fn test_error_event() {
        let event = error_event("Connection timeout");
        let line = event.to_progress_line();
        assert!(line.contains("[System]"));
        assert!(line.contains("Connection timeout"));
    }

    #[test]
    fn test_user_clarification() {
        let event = user_clarification_needed("Which editor do you mean - vim or vi?");
        let line = event.to_progress_line();
        assert!(line.contains("[Anna]"));
        assert!(line.contains("vim or vi"));
    }

    #[test]
    fn test_senior_review() {
        let event = senior_review_done(0.85);
        let line = event.to_progress_line();
        assert!(line.contains("[Senior]"));
        assert!(line.contains("85%"));
        assert!(line.contains("GREEN"));
    }
}

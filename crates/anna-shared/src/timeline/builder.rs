//! Timeline Builder - Constructs timelines from events.
//!
//! Converts Event and TicketEvent into structured timeline entries.

use crate::event_bus::{Event, TicketEvent as BusTicketEvent};
use super::types::{ActionType, DialogueTimeline, EntryKind};

/// Builder that accumulates events into a timeline.
#[derive(Debug, Default)]
pub struct TimelineBuilder {
    /// The timeline being built.
    timeline: Option<DialogueTimeline>,
    /// Current specialist ID (for action attribution).
    current_specialist: Option<String>,
}

impl TimelineBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new timeline for a ticket.
    pub fn start(&mut self, ticket_id: &str, question: &str) {
        self.timeline = Some(DialogueTimeline::new(ticket_id, question));
        self.current_specialist = None;
    }

    /// Process an event and add to timeline.
    pub fn process_event(&mut self, event: &Event) {
        let timeline = match &mut self.timeline {
            Some(t) => t,
            None => return,
        };

        match event {
            Event::TicketLifecycle(ticket_event) => {
                self.process_ticket_event(ticket_event);
            }
            Event::ProbeStarted { display_command, .. } => {
                if let Some(spec_id) = &self.current_specialist {
                    timeline.add(EntryKind::SpecialistAction {
                        specialist_id: spec_id.clone(),
                        action_type: ActionType::Probe,
                        description: display_command.clone(),
                    });
                }
            }
            Event::StepStarted { step_type, description, .. } => {
                let action_type = match step_type {
                    crate::event_bus::StepType::WikiSearch => ActionType::Documentation,
                    crate::event_bus::StepType::CommandExecution => ActionType::Probe,
                    crate::event_bus::StepType::OutputValidation => ActionType::Analysis,
                    _ => ActionType::Other,
                };
                if let Some(spec_id) = &self.current_specialist {
                    timeline.add(EntryKind::SpecialistAction {
                        specialist_id: spec_id.clone(),
                        action_type,
                        description: description.clone(),
                    });
                }
            }
            Event::Warning { code, message, .. } => {
                timeline.add_internal(EntryKind::InternalNote {
                    note: format!("Warning {}: {}", code, message),
                });
            }
            Event::Error { code, message, .. } => {
                timeline.add_internal(EntryKind::InternalNote {
                    note: format!("Error {}: {}", code, message),
                });
            }
            _ => {}
        }
    }

    /// Process a ticket lifecycle event.
    fn process_ticket_event(&mut self, event: &BusTicketEvent) {
        let timeline = match &mut self.timeline {
            Some(t) => t,
            None => return,
        };

        match event {
            BusTicketEvent::Created { ticket_id, department, question_summary } => {
                timeline.add(EntryKind::TicketCreated {
                    ticket_id: ticket_id.clone(),
                    question: question_summary.clone(),
                    department: department.clone(),
                });
            }
            BusTicketEvent::Assigned { specialist_id, specialist_name, department, .. } => {
                let level = if specialist_id.ends_with("-jr") {
                    "Junior"
                } else {
                    "Senior"
                };
                timeline.add(EntryKind::SpecialistAssigned {
                    specialist_id: specialist_id.clone(),
                    specialist_name: specialist_name.clone(),
                    level: level.to_string(),
                    department: department.clone(),
                });
                self.current_specialist = Some(specialist_id.clone());
            }
            BusTicketEvent::Working { specialist_id, action, .. } => {
                timeline.add(EntryKind::SpecialistAction {
                    specialist_id: specialist_id.clone(),
                    action_type: ActionType::Other,
                    description: action.clone(),
                });
            }
            BusTicketEvent::Escalated { from_specialist, to_specialist, reason, .. } => {
                // Extract names from IDs (simplified - in real code lookup from registry)
                let from_name = specialist_id_to_name(from_specialist);
                let to_name = specialist_id_to_name(to_specialist);
                timeline.add(EntryKind::Escalation {
                    from_id: from_specialist.clone(),
                    from_name,
                    to_id: to_specialist.clone(),
                    to_name,
                    reason: reason.clone(),
                });
                self.current_specialist = Some(to_specialist.clone());
            }
            BusTicketEvent::Resolved { specialist_id, specialist_name, confidence, learned_recipe, .. } => {
                timeline.add(EntryKind::Resolution {
                    specialist_id: specialist_id.clone(),
                    specialist_name: specialist_name.clone(),
                    confidence: *confidence,
                    learned_recipe: *learned_recipe,
                });
                timeline.mark_complete();
            }
            BusTicketEvent::Failed { specialist_id, reason, .. } => {
                timeline.add(EntryKind::Failure {
                    reason: reason.clone(),
                    specialist_id: specialist_id.clone(),
                });
                timeline.mark_complete();
            }
        }
    }

    /// Add a translator decision (called separately from events).
    pub fn add_translator_decision(&mut self, interpreted_as: &str, confidence: f32, routed_to: &str) {
        if let Some(timeline) = &mut self.timeline {
            timeline.add(EntryKind::TranslatorDecision {
                interpreted_as: interpreted_as.to_string(),
                confidence,
                routed_to: routed_to.to_string(),
            });
        }
    }

    /// Add a recovery attempt.
    pub fn add_recovery_attempt(&mut self, subsystem: &str, attempt_num: u32, success: bool) {
        if let Some(timeline) = &mut self.timeline {
            timeline.add(EntryKind::RecoveryAttempt {
                subsystem: subsystem.to_string(),
                attempt_num,
                success,
            });
        }
    }

    /// Finish building and return the timeline.
    pub fn finish(mut self) -> Option<DialogueTimeline> {
        self.timeline.take()
    }

    /// Get a reference to the current timeline.
    pub fn current(&self) -> Option<&DialogueTimeline> {
        self.timeline.as_ref()
    }

    /// Check if a timeline is in progress.
    pub fn is_active(&self) -> bool {
        self.timeline.is_some()
    }
}

/// Map specialist ID to name (simplified version).
fn specialist_id_to_name(id: &str) -> String {
    match id {
        "net-jr" => "Michael".to_string(),
        "net-sr" => "Sarah".to_string(),
        "desk-jr" => "Alex".to_string(),
        "desk-sr" => "Emma".to_string(),
        "sys-jr" => "James".to_string(),
        "sys-sr" => "Lisa".to_string(),
        "pkg-jr" => "David".to_string(),
        "pkg-sr" => "Nina".to_string(),
        "hw-jr" => "Ryan".to_string(),
        "hw-sr" => "Sophie".to_string(),
        "audio-jr" => "Chris".to_string(),
        "audio-sr" => "Maria".to_string(),
        "stor-jr" => "Kevin".to_string(),
        "stor-sr" => "Rachel".to_string(),
        "sec-jr" => "Tom".to_string(),
        "sec-sr" => "Elena".to_string(),
        _ => id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::TicketEvent as BusTicketEvent;

    #[test]
    fn test_builder_start() {
        let mut builder = TimelineBuilder::new();
        assert!(!builder.is_active());

        builder.start("CN-001", "test question");
        assert!(builder.is_active());
    }

    #[test]
    fn test_builder_process_ticket_created() {
        let mut builder = TimelineBuilder::new();
        builder.start("CN-001", "test question");

        let event = Event::TicketLifecycle(BusTicketEvent::Created {
            ticket_id: "CN-001".to_string(),
            department: "System".to_string(),
            question_summary: "test question".to_string(),
        });
        builder.process_event(&event);

        let timeline = builder.current().unwrap();
        assert_eq!(timeline.len(), 1);
    }

    #[test]
    fn test_builder_tracks_specialist() {
        let mut builder = TimelineBuilder::new();
        builder.start("CN-001", "test");

        builder.process_event(&Event::TicketLifecycle(BusTicketEvent::Assigned {
            ticket_id: "CN-001".to_string(),
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            department: "System".to_string(),
        }));

        assert_eq!(builder.current_specialist, Some("sys-jr".to_string()));
    }

    #[test]
    fn test_builder_finish() {
        let mut builder = TimelineBuilder::new();
        builder.start("CN-001", "test");

        let timeline = builder.finish();
        assert!(timeline.is_some());
        assert_eq!(timeline.unwrap().ticket_id, "CN-001");
    }

    #[test]
    fn test_specialist_id_to_name() {
        assert_eq!(specialist_id_to_name("sys-jr"), "James");
        assert_eq!(specialist_id_to_name("net-sr"), "Sarah");
        assert_eq!(specialist_id_to_name("unknown"), "unknown");
    }
}

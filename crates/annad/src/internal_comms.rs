//! Internal Comms Generator - Event-driven IT department chatter (v0.0.413).
//!
//! Generates realistic internal communications based on REAL events in the engine.
//! No fake messages - every line reflects actual processing state.

use anna_shared::rpc::SpecialistDomain;
use anna_shared::transcript_segment::{staff, Actor, TranscriptSegment};

/// Get the staff member for a domain
pub fn staff_for_domain(domain: &SpecialistDomain) -> Actor {
    match domain {
        SpecialistDomain::Desktop => staff::sofia(),
        SpecialistDomain::Network => staff::michael(),
        SpecialistDomain::Storage => staff::lars(),
        SpecialistDomain::System => staff::tomas(),
        SpecialistDomain::Services => staff::hugo(),
        SpecialistDomain::Security => staff::elena(),
        SpecialistDomain::Packages => staff::marcus(),
        SpecialistDomain::Boot => staff::tomas(),     // Boot issues handled by system
        SpecialistDomain::Audio => staff::sofia(),    // Audio handled by desktop
        SpecialistDomain::Display => staff::sofia(),  // Display handled by desktop
    }
}

/// Generate ticket opening message
pub fn ticket_opened(ticket_id: &str, domain: &SpecialistDomain, summary: &str) -> TranscriptSegment {
    TranscriptSegment::internal_comms(
        Actor::anna(),
        &format!(
            "Opening ticket {} for {} - {}.",
            ticket_id,
            domain_name(domain),
            summary
        ),
    )
    .with_meta("ticket_id", ticket_id)
    .with_meta("event", "ticket_opened")
}

/// Generate specialist assignment message
pub fn specialist_assigned(domain: &SpecialistDomain, ticket_id: &str) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(
        Actor::anna(),
        &format!(
            "{}, ticket {} is yours. {}",
            actor.name,
            ticket_id,
            assignment_action(domain)
        ),
    )
    .with_meta("event", "specialist_assigned")
}

/// Generate specialist acknowledgment
pub fn specialist_ack(domain: &SpecialistDomain, action: &str) -> TranscriptSegment {
    TranscriptSegment::internal_comms(
        staff_for_domain(domain),
        &format!("On it. {}", action),
    )
    .with_meta("event", "specialist_ack")
}

/// Generate probe started message
pub fn probe_started(probe_id: &str, domain: &SpecialistDomain) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(
        actor,
        &format!("Running {}...", probe_description(probe_id)),
    )
    .with_meta("probe_id", probe_id)
    .with_meta("event", "probe_started")
}

/// Generate probe completed message
pub fn probe_completed(probe_id: &str, domain: &SpecialistDomain, success: bool) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    let status = if success { "Got data" } else { "No data" };
    TranscriptSegment::internal_comms(actor, &format!("{} from {}.", status, probe_description(probe_id)))
        .with_meta("probe_id", probe_id)
        .with_meta("event", "probe_completed")
}

/// Generate evidence review message
pub fn reviewing_evidence(domain: &SpecialistDomain, probe_count: usize) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(
        actor,
        &format!("Reviewing {} evidence sources...", probe_count),
    )
    .with_meta("event", "reviewing_evidence")
}

/// Generate confidence update
pub fn confidence_update(domain: &SpecialistDomain, confidence: f32) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    let assessment = if confidence >= 0.9 {
        "Clear picture here"
    } else if confidence >= 0.7 {
        "Looks straightforward"
    } else if confidence >= 0.5 {
        "Need to check a few things"
    } else {
        "This is tricky"
    };
    TranscriptSegment::internal_comms(
        actor,
        &format!("{}. Confidence: {:.0}%", assessment, confidence * 100.0),
    )
    .with_meta("confidence", &format!("{:.2}", confidence))
    .with_meta("event", "confidence_update")
}

/// Generate escalation message
pub fn escalation(from_domain: &SpecialistDomain, reason: &str) -> TranscriptSegment {
    let actor = staff_for_domain(from_domain);
    TranscriptSegment::internal_comms(actor, &format!("Escalating - {}", reason))
        .with_meta("event", "escalation")
}

/// Generate answer ready message
pub fn answer_ready(domain: &SpecialistDomain) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(actor, "Answer ready.")
        .with_meta("event", "answer_ready")
}

/// Generate error message
pub fn error_occurred(domain: &SpecialistDomain, error_type: &str) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(
        actor,
        &format!("Hit a snag: {}. Working around it.", error_type),
    )
    .with_meta("event", "error")
}

/// Generate timeout warning
pub fn timeout_warning(domain: &SpecialistDomain, elapsed_secs: f32) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(
        actor,
        &format!(
            "Taking longer than expected ({:.1}s). Still working...",
            elapsed_secs
        ),
    )
    .with_meta("event", "timeout_warning")
}

/// Generate recipe match message
pub fn recipe_matched(recipe_name: &str, confidence: f32) -> TranscriptSegment {
    TranscriptSegment::internal_comms(
        Actor::anna(),
        &format!(
            "Found recipe: {} ({:.0}% match). Using fast path.",
            recipe_name,
            confidence * 100.0
        ),
    )
    .with_meta("event", "recipe_matched")
}

/// Generate fallback message
pub fn fallback_to_direct(domain: &SpecialistDomain, reason: &str) -> TranscriptSegment {
    let actor = staff_for_domain(domain);
    TranscriptSegment::internal_comms(
        actor,
        &format!("Switching to direct answer: {}", reason),
    )
    .with_meta("event", "fallback")
}

// Helper functions

fn domain_name(domain: &SpecialistDomain) -> &'static str {
    match domain {
        SpecialistDomain::Desktop => "Desktop",
        SpecialistDomain::Network => "Network",
        SpecialistDomain::Storage => "Storage",
        SpecialistDomain::System => "System",
        SpecialistDomain::Services => "Services",
        SpecialistDomain::Security => "Security",
        SpecialistDomain::Packages => "Packages",
        SpecialistDomain::Boot => "Boot",
        SpecialistDomain::Audio => "Audio",
        SpecialistDomain::Display => "Display",
    }
}

fn assignment_action(domain: &SpecialistDomain) -> &'static str {
    match domain {
        SpecialistDomain::Desktop => "Checking display and user config.",
        SpecialistDomain::Network => "Checking interfaces and connectivity.",
        SpecialistDomain::Storage => "Checking mounts and disk usage.",
        SpecialistDomain::System => "Checking services and system state.",
        SpecialistDomain::Services => "Checking service status and logs.",
        SpecialistDomain::Security => "Checking permissions and access.",
        SpecialistDomain::Packages => "Checking installed packages.",
        SpecialistDomain::Boot => "Checking boot sequence and startup.",
        SpecialistDomain::Audio => "Checking audio devices and mixer.",
        SpecialistDomain::Display => "Checking display configuration.",
    }
}

fn probe_description(probe_id: &str) -> &'static str {
    match probe_id {
        "memory_info" | "meminfo" => "memory check",
        "disk_usage" | "df_root" | "df" => "disk usage check",
        "systemd_failed" => "failed services check",
        "systemd_services" => "service list",
        "pacman_list" => "package list",
        "journal_errors" => "system logs",
        "network_interfaces" => "network interfaces",
        "gpu_info" => "GPU info",
        "audio_devices" => "audio devices",
        "cpu_info" => "CPU info",
        "kernel_info" => "kernel info",
        "systemd-analyze" => "boot time analysis",
        "lsblk" => "block devices",
        "free" => "memory stats",
        "ip addr" => "IP addresses",
        "ss" | "netstat" => "network connections",
        _ => "system probe",
    }
}

/// Event for transcript building
#[derive(Debug, Clone)]
pub enum TranscriptEvent {
    TicketOpened {
        ticket_id: String,
        domain: SpecialistDomain,
        summary: String,
    },
    SpecialistAssigned {
        domain: SpecialistDomain,
        ticket_id: String,
    },
    ProbeStarted {
        probe_id: String,
        domain: SpecialistDomain,
    },
    ProbeCompleted {
        probe_id: String,
        domain: SpecialistDomain,
        success: bool,
    },
    ReviewingEvidence {
        domain: SpecialistDomain,
        probe_count: usize,
    },
    ConfidenceUpdate {
        domain: SpecialistDomain,
        confidence: f32,
    },
    AnswerReady {
        domain: SpecialistDomain,
    },
    ErrorOccurred {
        domain: SpecialistDomain,
        error_type: String,
    },
    TimeoutWarning {
        domain: SpecialistDomain,
        elapsed_secs: f32,
    },
    RecipeMatched {
        recipe_name: String,
        confidence: f32,
    },
}

impl TranscriptEvent {
    /// Convert event to transcript segment
    pub fn to_segment(&self) -> TranscriptSegment {
        match self {
            TranscriptEvent::TicketOpened {
                ticket_id,
                domain,
                summary,
            } => ticket_opened(ticket_id, domain, summary),
            TranscriptEvent::SpecialistAssigned { domain, ticket_id } => {
                specialist_assigned(domain, ticket_id)
            }
            TranscriptEvent::ProbeStarted { probe_id, domain } => probe_started(probe_id, domain),
            TranscriptEvent::ProbeCompleted {
                probe_id,
                domain,
                success,
            } => probe_completed(probe_id, domain, *success),
            TranscriptEvent::ReviewingEvidence {
                domain,
                probe_count,
            } => reviewing_evidence(domain, *probe_count),
            TranscriptEvent::ConfidenceUpdate { domain, confidence } => {
                confidence_update(domain, *confidence)
            }
            TranscriptEvent::AnswerReady { domain } => answer_ready(domain),
            TranscriptEvent::ErrorOccurred { domain, error_type } => {
                error_occurred(domain, error_type)
            }
            TranscriptEvent::TimeoutWarning {
                domain,
                elapsed_secs,
            } => timeout_warning(domain, *elapsed_secs),
            TranscriptEvent::RecipeMatched {
                recipe_name,
                confidence,
            } => recipe_matched(recipe_name, *confidence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_opened() {
        let seg = ticket_opened("NET-001", &SpecialistDomain::Network, "wifi drops");
        assert!(seg.content.contains("NET-001"));
        assert!(seg.content.contains("Network"));
    }

    #[test]
    fn test_staff_for_domain() {
        let actor = staff_for_domain(&SpecialistDomain::Storage);
        assert_eq!(actor.name, "Lars");
    }

    #[test]
    fn test_event_to_segment() {
        let event = TranscriptEvent::ProbeStarted {
            probe_id: "memory_info".to_string(),
            domain: SpecialistDomain::System,
        };
        let seg = event.to_segment();
        assert!(seg.content.contains("memory"));
    }
}

//! Triage router for ambiguous queries.
//!
//! Handles queries that don't match deterministic classes using LLM translator
//! with confidence thresholds and clarification fallback.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::{info, warn};

/// Maximum probes allowed from triage path
pub const MAX_TRIAGE_PROBES: usize = 3;

/// Minimum confidence to proceed to specialist
pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Maximum reliability score when clarification is needed
pub const CLARIFICATION_MAX_RELIABILITY: u8 = 40;

/// Result of triage routing
#[derive(Debug, Clone)]
pub struct TriageResult {
    /// The translator ticket (possibly modified)
    pub ticket: TranslatorTicket,
    /// Whether probe cap was applied
    pub probe_cap_applied: bool,
    /// Whether we should return clarification immediately
    pub needs_immediate_clarification: bool,
    /// Clarification question to ask
    pub clarification_question: Option<String>,
}

/// Apply triage rules to a translator ticket from LLM
pub fn apply_triage_rules(ticket: TranslatorTicket) -> TriageResult {
    let mut result = TriageResult {
        ticket: ticket.clone(),
        probe_cap_applied: false,
        needs_immediate_clarification: false,
        clarification_question: None,
    };

    // Rule 1: Cap probes at MAX_TRIAGE_PROBES
    if result.ticket.needs_probes.len() > MAX_TRIAGE_PROBES {
        warn!(
            "Triage: capping probes from {} to {}",
            result.ticket.needs_probes.len(),
            MAX_TRIAGE_PROBES
        );
        result.ticket.needs_probes.truncate(MAX_TRIAGE_PROBES);
        result.probe_cap_applied = true;
    }

    // Rule 2: Low confidence requires clarification
    if result.ticket.confidence < MIN_CONFIDENCE_THRESHOLD {
        info!(
            "Triage: low confidence ({:.2} < {:.2}), requesting clarification",
            result.ticket.confidence, MIN_CONFIDENCE_THRESHOLD
        );
        result.needs_immediate_clarification = true;
        result.clarification_question = result
            .ticket
            .clarification_question
            .clone()
            .or_else(|| Some(generate_fallback_clarification(&result.ticket)));
    }

    // Rule 3: If LLM provided clarification question, use it
    if result.ticket.clarification_question.is_some() && result.ticket.needs_probes.is_empty() {
        result.needs_immediate_clarification = true;
        result.clarification_question = result.ticket.clarification_question.clone();
    }

    result
}

/// Generate a fallback clarification question based on domain heuristics
pub fn generate_fallback_clarification(ticket: &TranslatorTicket) -> String {
    match ticket.domain {
        SpecialistDomain::Network => {
            "Could you clarify what network information you need? For example: IP addresses, open ports, routing, or connectivity issues?".to_string()
        }
        SpecialistDomain::Storage => {
            "What storage information are you looking for? For example: disk space, mount points, or specific filesystems?".to_string()
        }
        SpecialistDomain::Security => {
            "What security aspect are you concerned about? For example: firewall rules, SSH access, or permissions?".to_string()
        }
        SpecialistDomain::Packages => {
            "What package operation do you need help with? For example: installing, updating, or searching for packages?".to_string()
        }
        SpecialistDomain::System => {
            "Could you provide more details? Are you asking about CPU, memory, processes, or something else?".to_string()
        }
    }
}

/// Generate clarification from query keywords (when translator fails completely)
pub fn generate_heuristic_clarification(query: &str) -> String {
    let q = query.to_lowercase();

    if q.contains("network") || q.contains("connect") || q.contains("internet") {
        return "Are you asking about network connectivity, IP configuration, or something else network-related?".to_string();
    }

    if q.contains("disk") || q.contains("storage") || q.contains("space") {
        return "Are you asking about disk space, mount points, or storage performance?".to_string();
    }

    if q.contains("slow") || q.contains("performance") || q.contains("fast") {
        return "Is the issue related to CPU usage, memory usage, disk I/O, or general system responsiveness?".to_string();
    }

    if q.contains("install") || q.contains("package") || q.contains("update") {
        return "Which package manager are you using (apt, pacman, dnf)? And what package are you trying to work with?".to_string();
    }

    if q.contains("error") || q.contains("fail") || q.contains("broken") {
        return "Could you describe the error or failure in more detail? What were you trying to do when it happened?".to_string();
    }

    // Generic fallback
    "I'm not sure I understand your question. Could you provide more details about what system information you're looking for?".to_string()
}

/// Create a default ticket for completely failed translator
pub fn create_fallback_ticket(query: &str) -> TranslatorTicket {
    let q = query.to_lowercase();

    // Try to guess domain from keywords
    let domain = if q.contains("network") || q.contains("ip") || q.contains("port") {
        SpecialistDomain::Network
    } else if q.contains("disk") || q.contains("storage") || q.contains("mount") {
        SpecialistDomain::Storage
    } else if q.contains("security") || q.contains("firewall") || q.contains("permission") {
        SpecialistDomain::Security
    } else if q.contains("package") || q.contains("install") || q.contains("apt") {
        SpecialistDomain::Packages
    } else {
        SpecialistDomain::System
    };

    TranslatorTicket {
        intent: QueryIntent::Question,
        domain,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: Some(generate_heuristic_clarification(query)),
        confidence: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_cap() {
        let ticket = TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec![
                "top_cpu".into(),
                "top_memory".into(),
                "disk_usage".into(),
                "network_addrs".into(),
                "listening_ports".into(),
            ],
            clarification_question: None,
            confidence: 0.9,
        };

        let result = apply_triage_rules(ticket);
        assert!(result.probe_cap_applied);
        assert_eq!(result.ticket.needs_probes.len(), MAX_TRIAGE_PROBES);
    }

    #[test]
    fn test_low_confidence_triggers_clarification() {
        let ticket = TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec!["top_cpu".into()],
            clarification_question: None,
            confidence: 0.5,
        };

        let result = apply_triage_rules(ticket);
        assert!(result.needs_immediate_clarification);
        assert!(result.clarification_question.is_some());
    }

    #[test]
    fn test_high_confidence_proceeds() {
        let ticket = TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec!["top_cpu".into()],
            clarification_question: None,
            confidence: 0.85,
        };

        let result = apply_triage_rules(ticket);
        assert!(!result.needs_immediate_clarification);
    }

    #[test]
    fn test_fallback_clarification_by_domain() {
        let network_q = generate_fallback_clarification(&TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::Network,
            entities: vec![],
            needs_probes: vec![],
            clarification_question: None,
            confidence: 0.3,
        });
        assert!(network_q.contains("network"));

        let storage_q = generate_fallback_clarification(&TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::Storage,
            entities: vec![],
            needs_probes: vec![],
            clarification_question: None,
            confidence: 0.3,
        });
        assert!(storage_q.contains("storage") || storage_q.contains("disk"));
    }

    #[test]
    fn test_heuristic_clarification() {
        assert!(generate_heuristic_clarification("my network is slow").contains("network"));
        assert!(generate_heuristic_clarification("disk is full").contains("disk"));
        assert!(generate_heuristic_clarification("something random").contains("details"));
    }
}

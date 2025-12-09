//! Intake analysis functions (v0.0.180).

use crate::facts::{FactKey, FactStatus, FactsStore};
use crate::rpc::{QueryIntent, SpecialistDomain};

use super::result::IntakeResult;
use super::slot::{generate_clarification, ClarificationSlot};

/// Check if a clarification is already satisfied by known facts
pub fn check_slot_satisfied(slot: ClarificationSlot, facts: &FactsStore) -> Option<String> {
    let key = slot.to_fact_key()?;
    facts.get_verified(&key).map(String::from)
}

/// Analyze query and determine needed clarifications
pub fn analyze_intake(
    query: &str,
    intent: QueryIntent,
    domain: SpecialistDomain,
    facts: &FactsStore,
    entities: &[String],
) -> IntakeResult {
    let q = query.to_lowercase();
    let mut clarifications = Vec::new();
    let mut facts_used = Vec::new();

    // Check for editor-related queries
    if needs_editor_clarification(&q, &intent, entities) {
        match facts.status(&FactKey::PreferredEditor) {
            FactStatus::Known(editor) => {
                facts_used.push(FactKey::PreferredEditor);
                // Also check if the editor binary is still available
                let binary_key = FactKey::BinaryAvailable(editor.clone());
                if facts.has_verified(&binary_key) {
                    facts_used.push(binary_key);
                }
            }
            _ => {
                clarifications.push(generate_clarification(
                    ClarificationSlot::EditorName,
                    "I need to know which editor to configure",
                ));
            }
        }
    }

    // Check for network-related queries
    if needs_network_clarification(&q, &intent) {
        match facts.status(&FactKey::NetworkPrimaryInterface) {
            FactStatus::Known(_iface) => {
                facts_used.push(FactKey::NetworkPrimaryInterface);
                facts_used.push(FactKey::NetworkPreference);
            }
            _ => {
                clarifications.push(generate_clarification(
                    ClarificationSlot::NetworkInterface,
                    "I need to know which network connection you're asking about",
                ));
            }
        }
    }

    // Check for service-related queries that need a specific service
    if needs_service_clarification(&q, entities) {
        clarifications.push(generate_clarification(
            ClarificationSlot::ServiceName,
            "I need to know which service you're asking about",
        ));
    }

    // Sort by priority
    clarifications.sort_by_key(|c| c.priority);

    IntakeResult {
        intent,
        domain,
        clarifications_needed: clarifications.clone(),
        facts_used,
        can_proceed: clarifications.is_empty(),
        confidence: if clarifications.is_empty() { 1.0 } else { 0.5 },
    }
}

/// Check if query needs editor clarification
fn needs_editor_clarification(query: &str, intent: &QueryIntent, entities: &[String]) -> bool {
    // Editor config requests need clarification
    if (query.contains("editor") || query.contains("syntax") || query.contains("highlight"))
        && matches!(intent, QueryIntent::Request)
    {
        return true;
    }

    // Check entities for editor-related terms
    let editor_terms = ["vim", "nvim", "nano", "emacs", "vimrc", "config"];
    for entity in entities {
        let e = entity.to_lowercase();
        for term in editor_terms {
            if e.contains(term) {
                return matches!(intent, QueryIntent::Request);
            }
        }
    }

    false
}

/// Check if query needs network clarification
fn needs_network_clarification(query: &str, intent: &QueryIntent) -> bool {
    let network_terms = [
        "internet",
        "connection",
        "network",
        "wifi",
        "ethernet",
        "broken",
    ];
    let matches_network = network_terms.iter().any(|t| query.contains(t));

    if !matches_network {
        return false;
    }

    // Investigation queries about network need clarification
    matches!(intent, QueryIntent::Investigate)
}

/// Check if query needs service clarification (when no specific service mentioned)
fn needs_service_clarification(query: &str, entities: &[String]) -> bool {
    if !query.contains("service") && !query.contains("restart") && !query.contains("status") {
        return false;
    }

    // If entities already contain a service name, no clarification needed
    for entity in entities {
        if entity.ends_with(".service") || entity.contains("systemd") {
            return false;
        }
    }

    // Check for common service names in query
    let common_services = [
        "nginx", "apache", "docker", "ssh", "postgres", "mysql", "redis",
    ];
    for svc in common_services {
        if query.contains(svc) {
            return false;
        }
    }

    // Generic service question needs clarification
    query.contains("service") && !query.contains("all service") && !query.contains("failed service")
}

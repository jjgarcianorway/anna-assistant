//! Comprehensive keyword-based translator (v0.0.402).
//!
//! This is now the PRIMARY classification path. The LLM translator is only used
//! as a fallback for truly ambiguous queries. This approach is:
//! - Fast: No LLM calls needed for common queries
//! - Reliable: Deterministic, testable behavior
//! - Accurate: Domain-specific probe selection
//!
//! v0.0.164: Extracted from translator.rs
//! v0.0.402: Massive expansion to handle 95%+ of common IT queries

mod domain_classifiers;
mod helpers;
mod system_classifiers;

pub use domain_classifiers::*;
pub use helpers::*;
pub use system_classifiers::*;

use anna_shared::answer_contract::AnswerContract;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::info;

/// Main entry point - classify query using comprehensive keyword matching
pub fn translate_fallback(query: &str) -> TranslatorTicket {
    let q = query.to_lowercase();
    let stripped = strip_greetings(&q);

    // Try classification in order of specificity
    if let Some(ticket) = classify_health_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_storage_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_memory_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_cpu_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_process_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_network_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_graphics_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_audio_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_bluetooth_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_boot_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_service_query(&stripped, query) {
        return ticket;
    }
    // v0.0.797: Tool check must come BEFORE package query to handle "is X installed"
    if let Some(ticket) = classify_tool_check_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_package_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_hardware_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_security_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_log_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_config_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_docker_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_user_query(&stripped, query) {
        return ticket;
    }

    // Generic system fallback
    info!("Fallback: no keyword match, using generic system domain");
    TranslatorTicket {
        intent: classify_intent(&stripped),
        domain: SpecialistDomain::System,
        entities: Vec::new(),
        needs_probes: vec!["memory_info".to_string(), "disk_usage".to_string()],
        clarification_question: None,
        confidence: 0.4,
        answer_contract: Some(AnswerContract::from_query(query)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_classification() {
        let ticket = translate_fallback("screen tearing issues");
        assert_eq!(ticket.domain, SpecialistDomain::Display); // v0.0.405: Now Display
        assert!(ticket.needs_probes.contains(&"gpu_info".to_string()));
    }

    #[test]
    fn test_webcam_classification() {
        let ticket = translate_fallback("is my webcam working?");
        assert!(ticket.needs_probes.contains(&"lsusb".to_string()));
    }

    #[test]
    fn test_bluetooth_classification() {
        // General bluetooth query - should include both service and devices
        let ticket = translate_fallback("bluetooth not working");
        assert!(ticket
            .needs_probes
            .contains(&"bluetooth_service".to_string()));
        assert!(ticket
            .needs_probes
            .contains(&"bluetooth_devices".to_string()));
    }

    #[test]
    fn test_bluetooth_service_status() {
        // Service status query - should only include service probe
        let ticket = translate_fallback("is bluetooth running");
        assert!(ticket
            .needs_probes
            .contains(&"bluetooth_service".to_string()));
        assert!(!ticket
            .needs_probes
            .contains(&"bluetooth_devices".to_string()));
    }

    #[test]
    fn test_audio_classification() {
        let ticket = translate_fallback("no sound from speakers");
        assert_eq!(ticket.domain, SpecialistDomain::Audio); // v0.0.405: Now Audio
        assert!(ticket.needs_probes.contains(&"audio_devices".to_string()));
    }

    #[test]
    fn test_disk_space_classification() {
        let ticket = translate_fallback("what is taking up disk space?");
        assert_eq!(ticket.domain, SpecialistDomain::Storage);
        assert!(ticket.needs_probes.contains(&"largest_dirs".to_string()));
    }

    #[test]
    fn test_boot_classification() {
        let ticket = translate_fallback("my system boots slowly");
        assert_eq!(ticket.domain, SpecialistDomain::Boot); // v0.0.405: Now Boot
        assert!(ticket.needs_probes.contains(&"boot_time".to_string()));
    }

    #[test]
    fn test_services_classification() {
        let ticket = translate_fallback("are any services failing?");
        assert_eq!(ticket.domain, SpecialistDomain::Services); // v0.0.405: Now Services
        assert!(ticket.needs_probes.contains(&"failed_services".to_string()));
    }
}

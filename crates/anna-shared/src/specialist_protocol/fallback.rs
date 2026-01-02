//! Graceful fallback handler for timeouts and errors (v0.0.428).
//!
//! When specialist calls fail:
//! - Never show "Failed to parse specialist response" to user
//! - Construct minimal, honest summary from available probes
//! - Status reflects actual usefulness (partial or failure)

pub use super::fallback_extractors::extract_facts_from_probes;
pub use super::fallback_responses::{generate_failure_response, generate_partial_response};
pub use super::fallback_types::{
    debug_error_message, truncate, user_friendly_error_message, ExtractedFact, FallbackContext,
    FallbackReason,
};
use super::StrictResponse;

/// Generate a fallback response from available context
pub fn generate_fallback(ctx: &FallbackContext) -> StrictResponse {
    // Try to extract useful info from probes
    let probe_facts = extract_facts_from_probes(&ctx.probe_results, &ctx.intent);

    if probe_facts.is_empty() {
        // Complete failure - no useful data
        return generate_failure_response(ctx);
    }

    // We have some data - generate partial response
    generate_partial_response(ctx, probe_facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_protocol::ResponseStatus;
    use std::collections::HashMap;

    fn make_context(reason: FallbackReason) -> FallbackContext {
        FallbackContext {
            ticket_id: "TEST-001".to_string(),
            domain: "system".to_string(),
            intent: "check_memory".to_string(),
            question: "How much RAM do I have?".to_string(),
            probe_results: HashMap::new(),
            reason,
            elapsed_ms: 5000,
        }
    }

    #[test]
    fn test_fallback_with_memory_probe() {
        let mut ctx = make_context(FallbackReason::Timeout);
        ctx.probe_results.insert(
            "free".to_string(),
            "              total        used        free      shared  buff/cache   available\nMem:           31Gi        14Gi       8.0Gi       2.0Gi       9.0Gi        15Gi".to_string()
        );

        let response = generate_fallback(&ctx);
        assert_eq!(response.status, ResponseStatus::Partial);
        assert!(response.summary.contains("15Gi") || response.summary.contains("available"));
    }

    #[test]
    fn test_fallback_with_disk_probe() {
        let mut ctx = make_context(FallbackReason::ParseError("invalid json".to_string()));
        ctx.intent = "check_disk".to_string();
        ctx.probe_results.insert(
            "df".to_string(),
            "Filesystem     Size  Used Avail Use% Mounted on\n/dev/sda1      803G  773G   30G  97% /".to_string()
        );

        let response = generate_fallback(&ctx);
        assert_eq!(response.status, ResponseStatus::Partial);
        assert!(response.summary.contains("97%") || response.summary.contains("Root"));
    }

    #[test]
    fn test_fallback_no_probes() {
        let ctx = make_context(FallbackReason::Timeout);
        let response = generate_fallback(&ctx);

        assert_eq!(response.status, ResponseStatus::Failure);
        assert!(response.summary.contains("couldn't"));
    }

    #[test]
    fn test_failed_services_extraction() {
        let mut ctx = make_context(FallbackReason::Timeout);
        ctx.intent = "check_failed_services".to_string();
        ctx.probe_results.insert(
            "systemctl_failed".to_string(),
            "  UNIT                     LOAD   ACTIVE SUB    DESCRIPTION\n  nginx.service           loaded failed failed nginx\n● redis.service           loaded failed failed redis".to_string()
        );

        let response = generate_fallback(&ctx);
        assert_eq!(response.status, ResponseStatus::Partial);
        // Should mention the failed services
        assert!(
            response.summary.contains("failed")
                || response
                    .details
                    .key_facts
                    .iter()
                    .any(|f| f.contains("failed"))
        );
    }

    #[test]
    fn test_no_failed_services() {
        let mut ctx = make_context(FallbackReason::Timeout);
        ctx.intent = "check_failed_services".to_string();
        ctx.probe_results.insert(
            "systemctl_failed".to_string(),
            "0 loaded units listed.".to_string(),
        );

        let response = generate_fallback(&ctx);
        // Even with timeout, we should get a partial with some info
        assert!(
            response.status == ResponseStatus::Partial
                || response.status == ResponseStatus::Failure
        );
    }

    #[test]
    fn test_user_friendly_messages() {
        assert!(!user_friendly_error_message(&FallbackReason::Timeout).contains("JSON"));
        assert!(
            !user_friendly_error_message(&FallbackReason::ParseError("x".to_string()))
                .contains("parse")
        );
    }

    #[test]
    fn test_debug_messages() {
        assert!(
            debug_error_message(&FallbackReason::ParseError("bad json".to_string()))
                .contains("bad json")
        );
    }
}

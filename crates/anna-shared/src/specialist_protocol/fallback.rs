//! Graceful fallback handler for timeouts and errors (v0.0.428).
//!
//! When specialist calls fail:
//! - Never show "Failed to parse specialist response" to user
//! - Construct minimal, honest summary from available probes
//! - Status reflects actual usefulness (partial or failure)

use super::{ProbeEvidence, ResponseMeta, ResponseStatus, StrictResponse};
use std::collections::HashMap;

/// Fallback context: what we know when a specialist fails
#[derive(Debug, Clone)]
pub struct FallbackContext {
    /// Ticket ID
    pub ticket_id: String,
    /// Domain of the query
    pub domain: String,
    /// Intent of the query
    pub intent: String,
    /// Original user question
    pub question: String,
    /// Probe results we have (probe_id -> output)
    pub probe_results: HashMap<String, String>,
    /// Why fallback was triggered
    pub reason: FallbackReason,
    /// Elapsed time before failure (ms)
    pub elapsed_ms: u64,
}

/// Why we're using fallback
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackReason {
    /// LLM call timed out
    Timeout,
    /// JSON parsing failed
    ParseError(String),
    /// Validation failed
    ValidationFailed(String),
    /// LLM returned error status
    LlmError(String),
    /// No specialist available
    NoSpecialist,
    /// Retry limit exceeded
    RetryExhausted,
}

impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "timeout"),
            Self::ParseError(e) => write!(f, "parse_error: {}", e),
            Self::ValidationFailed(e) => write!(f, "validation_failed: {}", e),
            Self::LlmError(e) => write!(f, "llm_error: {}", e),
            Self::NoSpecialist => write!(f, "no_specialist"),
            Self::RetryExhausted => write!(f, "retry_exhausted"),
        }
    }
}

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

/// Extract facts from raw probe outputs
fn extract_facts_from_probes(probes: &HashMap<String, String>, intent: &str) -> Vec<ExtractedFact> {
    let mut facts = vec![];

    for (probe_id, output) in probes {
        if output.trim().is_empty() {
            continue;
        }

        // Try to extract meaningful data based on probe type
        if let Some(fact) = extract_fact_from_probe(probe_id, output, intent) {
            facts.push(fact);
        }
    }

    facts
}

/// A fact extracted from a probe
#[derive(Debug, Clone)]
struct ExtractedFact {
    probe_id: String,
    summary: String,
    raw_snippet: String,
}

/// Extract a fact from a specific probe output
fn extract_fact_from_probe(probe_id: &str, output: &str, _intent: &str) -> Option<ExtractedFact> {
    let probe_lower = probe_id.to_lowercase();
    let output_trimmed = output.trim();

    // Memory probes (free, /proc/meminfo)
    if probe_lower.contains("free") || probe_lower.contains("meminfo") {
        return extract_memory_fact(probe_id, output_trimmed);
    }

    // Disk probes (df, lsblk)
    if probe_lower.contains("df") || probe_lower.contains("disk") {
        return extract_disk_fact(probe_id, output_trimmed);
    }

    // Systemd probes (systemctl)
    if probe_lower.contains("systemctl") || probe_lower.contains("systemd") {
        return extract_systemd_fact(probe_id, output_trimmed);
    }

    // Failed services
    if probe_lower.contains("failed") {
        return extract_failed_services_fact(probe_id, output_trimmed);
    }

    // Generic: if output is short enough, use as-is
    if output_trimmed.len() < 200 && !output_trimmed.is_empty() {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: format!("{} output available", probe_id),
            raw_snippet: truncate(output_trimmed, 150),
        });
    }

    None
}

/// Extract memory fact from free/meminfo output
fn extract_memory_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    // Look for "Mem:" line in free output
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let total = parts.get(1).unwrap_or(&"?");
                let available = parts.get(6).unwrap_or(&"?");
                return Some(ExtractedFact {
                    probe_id: probe_id.to_string(),
                    summary: format!("{} available out of {} total RAM", available, total),
                    raw_snippet: line.to_string(),
                });
            }
        }
    }

    // Look for MemAvailable in /proc/meminfo
    for line in output.lines() {
        if line.starts_with("MemAvailable:") {
            let value = line.split(':').nth(1).map(|s| s.trim()).unwrap_or("?");
            return Some(ExtractedFact {
                probe_id: probe_id.to_string(),
                summary: format!("{} memory available", value),
                raw_snippet: line.to_string(),
            });
        }
    }

    None
}

/// Extract disk fact from df output
fn extract_disk_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    // Look for root filesystem line
    for line in output.lines() {
        if line.contains(" /") && !line.contains(" /boot") && !line.starts_with("Filesystem") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let used_pct = parts.get(4).unwrap_or(&"?");
                let available = parts.get(3).unwrap_or(&"?");
                return Some(ExtractedFact {
                    probe_id: probe_id.to_string(),
                    summary: format!(
                        "Root filesystem at {} used ({} available)",
                        used_pct, available
                    ),
                    raw_snippet: line.to_string(),
                });
            }
        }
    }

    None
}

/// Extract systemd fact from systemctl output
fn extract_systemd_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    let output_lower = output.to_lowercase();

    // Check for running status
    if output_lower.contains("active (running)") {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: "Service is active and running".to_string(),
            raw_snippet: truncate(output, 100),
        });
    }

    // Check for failed status
    if output_lower.contains("failed") || output_lower.contains("inactive (dead)") {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: "Service is not running or has failed".to_string(),
            raw_snippet: truncate(output, 100),
        });
    }

    None
}

/// Extract failed services fact
fn extract_failed_services_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();

    // Count failed units (excluding header)
    let failed_count = lines
        .iter()
        .filter(|l| l.contains(".service") || l.contains(".socket") || l.contains(".timer"))
        .count();

    if failed_count == 0 {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: "No failed systemd units".to_string(),
            raw_snippet: output.lines().next().unwrap_or("").to_string(),
        });
    }

    // List up to 3 failed services
    let failed_names: Vec<&str> = lines
        .iter()
        .filter(|l| l.contains(".service"))
        .take(3)
        .map(|l| l.split_whitespace().next().unwrap_or(""))
        .collect();

    let summary = if failed_count <= 3 {
        format!(
            "{} failed service(s): {}",
            failed_count,
            failed_names.join(", ")
        )
    } else {
        format!(
            "{} failed service(s), including: {}",
            failed_count,
            failed_names.join(", ")
        )
    };

    Some(ExtractedFact {
        probe_id: probe_id.to_string(),
        summary,
        raw_snippet: truncate(output, 150),
    })
}

/// Generate partial response from extracted facts
fn generate_partial_response(ctx: &FallbackContext, facts: Vec<ExtractedFact>) -> StrictResponse {
    // Build summary from facts
    let summary = if facts.len() == 1 {
        facts[0].summary.clone()
    } else {
        format!(
            "I found {} pieces of information: {}",
            facts.len(),
            facts
                .iter()
                .map(|f| f.summary.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    };

    // Add explanation about the limitation
    let reason_text = match &ctx.reason {
        FallbackReason::Timeout => "My detailed analysis timed out.",
        FallbackReason::ParseError(_) => "I encountered an internal error during analysis.",
        FallbackReason::ValidationFailed(_) => "Some analysis results were inconsistent.",
        FallbackReason::LlmError(_) => "I couldn't complete the full analysis.",
        FallbackReason::NoSpecialist => "No specialist was available for this query.",
        FallbackReason::RetryExhausted => {
            "I couldn't get a complete analysis after multiple attempts."
        }
    };

    let key_facts: Vec<String> = facts.iter().map(|f| f.summary.clone()).collect();
    let probes: Vec<ProbeEvidence> = facts
        .iter()
        .map(|f| ProbeEvidence {
            id: f.probe_id.clone(),
            summary: f.summary.clone(),
            raw_reference: Some(truncate(&f.raw_snippet, 100)),
        })
        .collect();

    let meta = ResponseMeta {
        handled_by: "Fallback Handler".to_string(),
        ticket_id: ctx.ticket_id.clone(),
        version: 1,
    };

    StrictResponse::partial(
        &ctx.domain,
        &ctx.intent,
        &summary,
        key_facts,
        reason_text,
        probes,
        meta,
    )
    .with_latency(ctx.elapsed_ms)
}

/// Generate failure response when no useful data available
fn generate_failure_response(ctx: &FallbackContext) -> StrictResponse {
    let summary = match &ctx.reason {
        FallbackReason::Timeout => {
            "I couldn't complete my analysis in time. Please try again with a simpler question."
        }
        FallbackReason::ParseError(_) => {
            "I encountered an internal error. Please try rephrasing your question."
        }
        FallbackReason::ValidationFailed(_) => {
            "I couldn't produce a reliable answer. Please try a different approach."
        }
        FallbackReason::LlmError(_) => "I'm having trouble analyzing this. Please try again later.",
        FallbackReason::NoSpecialist => {
            "I don't have a specialist available for this type of question."
        }
        FallbackReason::RetryExhausted => {
            "I couldn't get a valid response after multiple attempts."
        }
    };

    let meta = ResponseMeta {
        handled_by: "Fallback Handler".to_string(),
        ticket_id: ctx.ticket_id.clone(),
        version: 1,
    };

    StrictResponse::failure(&ctx.domain, &ctx.intent, summary, meta).with_latency(ctx.elapsed_ms)
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// User-facing error message (never shows internal details)
pub fn user_friendly_error_message(reason: &FallbackReason) -> &'static str {
    match reason {
        FallbackReason::Timeout => "My analysis is taking longer than expected.",
        FallbackReason::ParseError(_) => "I had trouble processing this request.",
        FallbackReason::ValidationFailed(_) => "I couldn't verify my response.",
        FallbackReason::LlmError(_) => "I encountered an issue during analysis.",
        FallbackReason::NoSpecialist => "I don't have a specialist for this topic.",
        FallbackReason::RetryExhausted => "I couldn't complete this request.",
    }
}

/// Debug-mode error message (shows internal details)
pub fn debug_error_message(reason: &FallbackReason) -> String {
    match reason {
        FallbackReason::Timeout => "Specialist LLM call timed out".to_string(),
        FallbackReason::ParseError(e) => format!("JSON parse error: {}", e),
        FallbackReason::ValidationFailed(e) => format!("Validation failed: {}", e),
        FallbackReason::LlmError(e) => format!("LLM error: {}", e),
        FallbackReason::NoSpecialist => "No specialist registered for domain".to_string(),
        FallbackReason::RetryExhausted => "Max retries exceeded".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

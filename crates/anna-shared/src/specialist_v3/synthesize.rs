//! Error synthesis for specialist failures (v0.0.425).
//!
//! When specialists fail, synthesize user-friendly responses.
//! Never expose "Failed to parse" or technical errors.

use super::{
    ErrorInfo, ErrorKind, Finding, ProbeStatus, ProbeUsed, ResponseStatus, Severity,
    SpecialistResponse, DEFAULT_CONFIDENCE, MIN_USEFUL_CONFIDENCE,
};

/// Synthesize a response when the LLM completely fails to respond.
pub fn synthesize_timeout_response(ticket_id: &str, timeout_ms: u64) -> SpecialistResponse {
    SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status: ResponseStatus::Error,
        summary: "Response timed out".to_string(),
        confidence: 0.0,
        severity: Severity::Warning,
        error: ErrorInfo {
            message: Some(format!(
                "The analysis took longer than expected ({}ms). Please try again.",
                timeout_ms
            )),
            kind: Some(ErrorKind::Timeout),
            details: None,
        },
        ..Default::default()
    }
}

/// Synthesize a response when no probes returned useful data.
pub fn synthesize_no_evidence_response(
    ticket_id: &str,
    probes_attempted: &[(&str, ProbeStatus)],
) -> SpecialistResponse {
    let probes_used: Vec<ProbeUsed> = probes_attempted
        .iter()
        .map(|(id, status)| ProbeUsed {
            id: format!("probe:{}", id),
            status: *status,
            description: format!("Attempted probe: {}", id),
            raw_key: Some(id.to_string()),
        })
        .collect();

    let summary = if probes_attempted.is_empty() {
        "No diagnostic probes were available".to_string()
    } else {
        format!(
            "Unable to gather evidence ({} probes returned no useful data)",
            probes_attempted.len()
        )
    };

    SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status: ResponseStatus::NoData,
        summary,
        confidence: 0.1,
        severity: Severity::Info,
        probes_used,
        analysis: vec![
            "All attempted probes returned empty or failed".to_string(),
            "This may indicate a permissions issue or missing tools".to_string(),
        ],
        ..Default::default()
    }
}

/// Synthesize a response when the question is outside specialist domain.
pub fn synthesize_unsupported_response(
    ticket_id: &str,
    specialist_domain: &str,
    suggested_domain: Option<&str>,
) -> SpecialistResponse {
    let mut analysis = vec![format!(
        "This question is outside the {} specialist's expertise",
        specialist_domain
    )];

    if let Some(domain) = suggested_domain {
        analysis.push(format!("Consider routing to the {} specialist", domain));
    }

    SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status: ResponseStatus::Unsupported,
        summary: format!("Question outside {} domain", specialist_domain),
        confidence: 0.0,
        severity: Severity::Info,
        analysis,
        ..Default::default()
    }
}

/// Synthesize a partial response from probe data when LLM fails.
pub fn synthesize_from_probes(
    ticket_id: &str,
    probe_data: &[(String, String)],
) -> SpecialistResponse {
    if probe_data.is_empty() {
        return synthesize_no_evidence_response(ticket_id, &[]);
    }

    let mut findings = Vec::new();
    let mut probes_used = Vec::new();
    let mut analysis = Vec::new();

    for (probe_id, output) in probe_data {
        // Record probe usage
        probes_used.push(ProbeUsed {
            id: format!("probe:{}", probe_id),
            status: if output.is_empty() {
                ProbeStatus::Empty
            } else {
                ProbeStatus::Ok
            },
            description: format!("Raw output from {}", probe_id),
            raw_key: Some(probe_id.clone()),
        });

        // Try to extract basic findings
        if let Some(finding) = extract_basic_finding(probe_id, output) {
            findings.push(finding);
        }
    }

    let summary = if findings.is_empty() {
        "Probe data collected but could not be interpreted".to_string()
    } else {
        format!("Extracted {} findings from probe data", findings.len())
    };

    analysis.push(format!(
        "Raw data available from {} probes",
        probe_data.len()
    ));
    analysis.push("Automated analysis may be incomplete".to_string());

    SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status: ResponseStatus::Partial,
        summary,
        confidence: MIN_USEFUL_CONFIDENCE,
        severity: Severity::Info,
        findings,
        analysis,
        probes_used,
        ..Default::default()
    }
}

/// Extract basic findings from common probe outputs.
fn extract_basic_finding(probe_id: &str, output: &str) -> Option<Finding> {
    match probe_id {
        "free" | "memory" => extract_memory_finding(output),
        "df" | "disk" => extract_disk_finding(output),
        "uptime" => extract_uptime_finding(output),
        _ => None,
    }
}

/// Extract memory finding from `free` output.
fn extract_memory_finding(output: &str) -> Option<Finding> {
    // Look for "Mem:" line
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return Some(Finding {
                    key: "mem_available_kb".to_string(),
                    value: parts.get(6).unwrap_or(&"unknown").to_string(),
                    evidence_refs: vec!["probe:free".to_string()],
                });
            }
        }
    }
    None
}

/// Extract disk finding from `df` output.
fn extract_disk_finding(output: &str) -> Option<Finding> {
    // Look for root filesystem
    for line in output.lines() {
        if line.contains(" / ") || line.ends_with(" /") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return Some(Finding {
                    key: "disk_available_root".to_string(),
                    value: parts.get(3).unwrap_or(&"unknown").to_string(),
                    evidence_refs: vec!["probe:df".to_string()],
                });
            }
        }
    }
    None
}

/// Extract uptime finding.
fn extract_uptime_finding(output: &str) -> Option<Finding> {
    let output = output.trim();
    if !output.is_empty() {
        return Some(Finding {
            key: "uptime".to_string(),
            value: output.to_string(),
            evidence_refs: vec!["probe:uptime".to_string()],
        });
    }
    None
}

/// Synthesize an internal error response (for bugs/panics).
pub fn synthesize_internal_error(ticket_id: &str, context: &str) -> SpecialistResponse {
    SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status: ResponseStatus::Error,
        summary: "An internal error occurred".to_string(),
        confidence: 0.0,
        severity: Severity::Warning,
        error: ErrorInfo {
            message: Some("Anna encountered an unexpected issue. Please try again.".to_string()),
            kind: Some(ErrorKind::Internal),
            details: Some(context.to_string()), // Logged but not shown to user
        },
        ..Default::default()
    }
}

/// Combine multiple specialist responses into one.
pub fn merge_responses(responses: &[SpecialistResponse]) -> SpecialistResponse {
    if responses.is_empty() {
        return SpecialistResponse::no_data("merged", "No responses to merge");
    }

    if responses.len() == 1 {
        return responses[0].clone();
    }

    // Find the best response (highest confidence with success status)
    let best = responses
        .iter()
        .filter(|r| r.status.is_success())
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    match best {
        Some(primary) => {
            let mut merged = primary.clone();

            // Merge findings from other responses
            for other in responses {
                if other.ticket_id != primary.ticket_id {
                    continue;
                }
                for finding in &other.findings {
                    if !merged.findings.iter().any(|f| f.key == finding.key) {
                        merged.findings.push(finding.clone());
                    }
                }
                for probe in &other.probes_used {
                    if !merged.probes_used.iter().any(|p| p.id == probe.id) {
                        merged.probes_used.push(probe.clone());
                    }
                }
            }

            merged
        }
        None => {
            // No successful responses, return the first one
            responses[0].clone()
        }
    }
}

/// Create a user-friendly message from a response.
pub fn format_for_user(response: &SpecialistResponse) -> String {
    let mut output = String::new();

    // Severity indicator
    let severity_prefix = response.severity.emoji();
    if !severity_prefix.is_empty() {
        output.push_str(severity_prefix);
        output.push(' ');
    }

    // Summary
    output.push_str(&response.summary);
    output.push('\n');

    // Findings as key-value pairs
    if !response.findings.is_empty() {
        output.push('\n');
        for finding in &response.findings {
            output.push_str(&format!("  {}: {}\n", finding.key, finding.value));
        }
    }

    // Analysis bullets
    if !response.analysis.is_empty() {
        output.push('\n');
        for bullet in &response.analysis {
            output.push_str(&format!("• {}\n", bullet));
        }
    }

    // Recommendations
    if !response.recommendations.is_empty() {
        output.push_str("\nRecommendations:\n");
        for rec in &response.recommendations {
            output.push_str(&format!("  → {}: {}\n", rec.title, rec.description));
        }
    }

    // Actions (if any)
    if !response.actions.is_empty() {
        output.push_str("\nSuggested commands:\n");
        for action in &response.actions {
            output.push_str(&format!("  $ {}\n", action.command));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_timeout() {
        let resp = synthesize_timeout_response("DSK-001", 5000);
        assert_eq!(resp.status, ResponseStatus::Error);
        assert!(resp.error.message.unwrap().contains("5000"));
    }

    #[test]
    fn test_synthesize_no_evidence() {
        let resp = synthesize_no_evidence_response(
            "DSK-002",
            &[("free", ProbeStatus::Empty), ("df", ProbeStatus::Failed)],
        );
        assert_eq!(resp.status, ResponseStatus::NoData);
        assert_eq!(resp.probes_used.len(), 2);
    }

    #[test]
    fn test_synthesize_from_probes() {
        let probe_data = vec![
            (
                "free".to_string(),
                "Mem: 25600 8400 0 500 2000 17000".to_string(),
            ),
            ("uptime".to_string(), "3 days, 4 hours".to_string()),
        ];
        let resp = synthesize_from_probes("DSK-003", &probe_data);
        assert_eq!(resp.status, ResponseStatus::Partial);
        assert!(!resp.findings.is_empty());
    }

    #[test]
    fn test_format_for_user() {
        let resp = SpecialistResponse::success("DSK-004", "Memory is healthy")
            .with_finding(Finding::new("mem_available_gb", "17"))
            .with_analysis("Good memory headroom");

        let output = format_for_user(&resp);
        assert!(output.contains("Memory is healthy"));
        assert!(output.contains("mem_available_gb: 17"));
        assert!(output.contains("Good memory headroom"));
    }

    #[test]
    fn test_merge_responses() {
        let r1 = SpecialistResponse::success("DSK-005", "Memory check")
            .with_confidence(0.9)
            .with_finding(Finding::new("mem", "ok"));
        let r2 = SpecialistResponse::success("DSK-005", "Disk check")
            .with_confidence(0.7)
            .with_finding(Finding::new("disk", "ok"));

        let merged = merge_responses(&[r1, r2]);
        assert_eq!(merged.confidence, 0.9); // Takes highest
        assert_eq!(merged.findings.len(), 2); // Merges findings
    }
}

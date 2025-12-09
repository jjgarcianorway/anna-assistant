//! Response builders for ServiceDeskResult.
//!
//! Extracted from service_desk.rs (v0.0.155) for modularization.
//! Contains builders for timeout, no-data, no-evidence, and best-effort responses.

use anna_shared::rpc::{
    ProbeResult, QueryIntent, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
    TranslatorTicket,
};
use anna_shared::transcript::Transcript;

use crate::service_desk::build_evidence;

/// Create a deterministic "no evidence" failure response (v0.45.4).
/// Used when evidence_required=true but probe_stats.succeeded==0.
pub fn create_no_evidence_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
    required_evidence: &[String],
) -> ServiceDeskResult {
    let evidence_list = if required_evidence.is_empty() {
        "system data".to_string()
    } else {
        required_evidence.join(", ")
    };
    let answer = format!(
        "I can't answer yet because I didn't collect evidence for: {}. Run: `annactl status` and retry.",
        evidence_list
    );

    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: true,
    };

    let evidence = build_evidence(
        ticket,
        probe_results,
        Some("no probes succeeded".to_string()),
    );

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        reliability_score: anna_shared::reliability::NO_EVIDENCE_RELIABILITY_CAP,
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// Create a timeout error response with evidence summary (v0.45.x stabilization).
/// Never asks to rephrase - always provides factual status.
pub fn create_timeout_response(
    request_id: String,
    stage: &str,
    ticket: Option<TranslatorTicket>,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
) -> ServiceDeskResult {
    let answer = build_timeout_evidence_summary(stage, &probe_results);

    let has_evidence = !probe_results.is_empty() && probe_results.iter().any(|p| p.exit_code == 0);

    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: has_evidence,
        answer_grounded: has_evidence,
        no_invention: true,
        clarification_not_needed: true,
    };

    let default_ticket = ticket.unwrap_or_else(|| TranslatorTicket {
        intent: QueryIntent::Question,
        domain,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: None,
        confidence: 0.0,
        answer_contract: None,
    });

    let evidence = build_evidence(
        default_ticket,
        probe_results,
        Some(format!("timeout at {}", stage)),
    );

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        reliability_score: if has_evidence { 40 } else { 20 },
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// Build evidence summary for timeout response (v0.45.x stabilization).
fn build_timeout_evidence_summary(stage: &str, probe_results: &[ProbeResult]) -> String {
    let mut answer = format!("**Timeout at {} stage**\n\n", stage);

    let successful: Vec<_> = probe_results.iter().filter(|p| p.exit_code == 0).collect();
    let failed: Vec<_> = probe_results.iter().filter(|p| p.exit_code != 0).collect();

    if successful.is_empty() && failed.is_empty() {
        answer.push_str("No probes were completed before the timeout.\n\n");
    } else {
        if !successful.is_empty() {
            answer.push_str("**Evidence gathered before timeout:**\n\n");
            for probe in &successful {
                let output: String = probe.stdout.lines().take(3).collect::<Vec<_>>().join("\n");
                if !output.trim().is_empty() {
                    let truncated = if probe.stdout.lines().count() > 3 {
                        "..."
                    } else {
                        ""
                    };
                    answer.push_str(&format!(
                        "- `{}`: {}{}\n",
                        probe.command,
                        output.replace('\n', " | "),
                        truncated
                    ));
                }
            }
            answer.push('\n');
        }

        if !failed.is_empty() {
            answer.push_str(&format!(
                "{} probe{} failed before timeout.\n",
                failed.len(),
                if failed.len() == 1 { "" } else { "s" }
            ));
        }
    }

    answer.push_str("*The request exceeded its time budget. Try a more specific query.*");
    answer
}

/// Create a best-effort response when no deterministic answer is available (v0.0.32).
/// Always answers - never asks to rephrase.
pub fn create_no_data_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
) -> ServiceDeskResult {
    let answer = build_best_effort_answer(&probe_results, domain);

    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: !probe_results.is_empty(),
        answer_grounded: !probe_results.is_empty(),
        no_invention: true,
        clarification_not_needed: true,
    };

    let evidence = build_evidence(
        ticket,
        probe_results,
        Some("Best-effort answer from available data".to_string()),
    );

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        reliability_score: signals.score(),
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// Build a best-effort answer from available probe results (v0.0.141).
fn build_best_effort_answer(probe_results: &[ProbeResult], domain: SpecialistDomain) -> String {
    if probe_results.is_empty() {
        return format!(
            "I couldn't gather {} information right now. \
             Try asking a more specific question like \"what cpu\" or \"disk space\".",
            domain
        );
    }

    let successful: Vec<_> = probe_results.iter().filter(|p| p.exit_code == 0).collect();
    let failed: Vec<_> = probe_results.iter().filter(|p| p.exit_code != 0).collect();

    let mut answer = String::new();

    if !successful.is_empty() {
        answer.push_str("**Here's what I found:**\n\n");

        for probe in &successful {
            let output = probe.stdout.lines().take(5).collect::<Vec<_>>().join("\n");
            let output = if output.len() > 300 {
                format!("{}...", &output[..300])
            } else {
                output
            };

            if !output.trim().is_empty() {
                let probe_name = friendly_probe_name(&probe.command);
                answer.push_str(&format!("**{}**:\n```\n{}\n```\n\n", probe_name, output));
            }
        }
    }

    if !failed.is_empty() && !successful.is_empty() {
        answer.push_str(&format!(
            "*({} additional probe{} didn't return data)*",
            failed.len(),
            if failed.len() == 1 { "" } else { "s" }
        ));
    }

    if answer.is_empty() {
        format!(
            "I gathered some {} data but couldn't format it clearly. \
             Try a specific question for better results.",
            domain
        )
    } else {
        answer
    }
}

/// Convert probe command to friendlier display name (v0.0.141)
fn friendly_probe_name(command: &str) -> &str {
    let first_word = command.split_whitespace().next().unwrap_or(command);
    match first_word {
        "free" => "Memory",
        "df" => "Disk Space",
        "lscpu" => "CPU Info",
        "uname" => "System",
        "uptime" => "Uptime",
        "ip" => "Network",
        "ps" => "Processes",
        "systemctl" => "Services",
        "cat" => {
            if command.contains("/proc/cpuinfo") {
                "CPU Details"
            } else if command.contains("/etc/hosts") {
                "Hosts File"
            } else if command.contains("/proc/meminfo") {
                "Memory Details"
            } else {
                "File Contents"
            }
        }
        _ => first_word,
    }
}

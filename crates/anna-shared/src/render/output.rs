//! Render output functions (v0.0.203).

use chrono::{DateTime, Utc};

use crate::rpc::ServiceDeskResult;
use crate::transcript::{Actor, TranscriptEventKind};
use crate::ui::colors;

use super::formatting::{determine_risk_level, format_time_delta, generate_case_id};
use super::types::RiskLevel;

/// Header block for all Anna outputs
pub fn render_header(hostname: &str, username: &str, version: &str, debug_mode: bool) {
    let mode = if debug_mode { " [debug]" } else { "" };
    println!();
    println!(
        "{}anna v{}{}{}",
        colors::HEADER,
        version,
        mode,
        colors::RESET
    );
    println!("{}{}@{}{}", colors::DIM, username, hostname, colors::RESET);
    println!();
}

/// Narrative greeting for REPL entry
pub fn render_greeting(
    username: &str,
    last_interaction: Option<DateTime<Utc>>,
    boot_time_delta: Option<&str>,
    critical_issues: usize,
) {
    print!("Hello {}", username);

    if let Some(last) = last_interaction {
        let now = Utc::now();
        let delta = now.signed_duration_since(last);
        let delta_str = format_time_delta(delta);
        println!(". It's been {} since you checked in.", delta_str);
    } else {
        println!(". First time here.");
    }

    // Show deltas
    if let Some(boot) = boot_time_delta {
        println!("{}System up: {}{}", colors::DIM, boot, colors::RESET);
    }

    if critical_issues > 0 {
        println!(
            "{}Warning: {} critical issue{} detected.{}",
            colors::WARN,
            critical_issues,
            if critical_issues == 1 { "" } else { "s" },
            colors::RESET
        );
    }

    println!();
}

/// Case flow block header
pub fn render_case_start(case_id: &str, domain: &str) {
    println!("{}Case {} created{}", colors::DIM, case_id, colors::RESET);
    println!("Dispatching to {} team.", domain);
}

/// Show evidence collection in progress
pub fn render_collecting_evidence() {
    use std::io::{self, Write};
    print!("Collecting system evidence");
    io::stdout().flush().ok();
}

/// Show evidence collected
pub fn render_evidence_collected(probe_count: usize) {
    println!(
        " {} {} source{} checked.",
        colors::OK,
        probe_count,
        if probe_count == 1 { "" } else { "s" }
    );
}

/// Render internal notes excerpt (short, professional)
pub fn render_internal_notes(notes: &str) {
    if !notes.is_empty() {
        println!("{}Internal notes: {}{}", colors::DIM, notes, colors::RESET);
    }
}

/// Render resolution (final answer)
pub fn render_resolution(answer: &str) {
    println!();
    println!("{}Resolution:{}", colors::HEADER, colors::RESET);
    for line in answer.lines() {
        println!("  {}", line);
    }
}

/// Render clarification options as numbered list ending with period
pub fn render_clarification(prompt: &str, options: &[(String, String)]) {
    println!();
    // Remove any trailing question mark from prompt
    let clean_prompt = prompt.trim_end_matches('?').trim();
    println!("{}{}:{}", colors::HEADER, clean_prompt, colors::RESET);

    for (i, (_key, label)) in options.iter().enumerate() {
        println!("  {}) {}", i + 1, label);
    }
    println!("Reply with the number.");
}

/// Render risk and reliability line
pub fn render_reliability_line(reliability: u8, risk: RiskLevel, evidence_kinds: &[String]) {
    let evidence_str = if evidence_kinds.is_empty() {
        "none".to_string()
    } else {
        evidence_kinds.join(", ")
    };

    println!(
        "{}reliability: {}%   risk: {}   evidence: {}{}",
        colors::DIM,
        reliability,
        risk.as_str(),
        evidence_str,
        colors::RESET
    );
}

/// Render citation
pub fn render_citation(source: &str, _topic: &str) {
    println!("{}[source: {}]{}", colors::DIM, source, colors::RESET);
}

/// Render uncited warning
pub fn render_uncited() {
    println!(
        "{}[uncited - verification ticket created]{}",
        colors::WARN,
        colors::RESET
    );
}

/// Full narrative render for a result (debug OFF)
pub fn render_narrative(result: &ServiceDeskResult, case_seq: u32) {
    let case_id = generate_case_id(case_seq);

    // Show user query
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}you:{} {}", colors::CYAN, colors::RESET, text);
                break;
            }
        }
    }
    println!();

    // Case header
    render_case_start(&case_id, &result.domain.to_string());

    // Evidence count
    let probe_count = result.evidence.probes_executed.len();
    if probe_count > 0 {
        print!("Collecting system evidence");
        println!(
            " {} {} source{} checked.",
            colors::DIM,
            probe_count,
            if probe_count == 1 { "" } else { "s" }
        );
    }

    // Resolution
    let answer = get_answer_text(result);
    render_resolution(&answer);

    // Clarification if needed (ends with period, not question)
    if result.needs_clarification {
        if let Some(ref req) = result.clarification_request {
            let options: Vec<(String, String)> = req
                .options
                .iter()
                .map(|o| (o.key.to_string(), o.label.clone()))
                .collect();
            if !options.is_empty() {
                render_clarification(&req.question, &options);
            }
        }
    }

    println!();

    // Risk level based on action type
    let risk = determine_risk_level(&result.answer);

    // Evidence kinds
    let evidence_kinds: Vec<String> = if let Some(trace) = &result.execution_trace {
        trace
            .evidence_kinds
            .iter()
            .map(|k| format!("{:?}", k))
            .collect()
    } else {
        vec![]
    };

    render_reliability_line(result.reliability_score, risk, &evidence_kinds);
}

/// Get final answer text from result
fn get_answer_text(result: &ServiceDeskResult) -> String {
    // Check transcript for FinalAnswer first
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { text } = &event.kind {
            return text.clone();
        }
    }

    // Fall back to clarification or answer
    if result.needs_clarification {
        result
            .clarification_question
            .clone()
            .unwrap_or_else(|| result.answer.clone())
    } else {
        result.answer.clone()
    }
}

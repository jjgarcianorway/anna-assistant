//! Theatre main rendering (v0.0.341).
//!
//! v0.0.252: Evidence bullets displayed with concise answers
//! v0.0.341: Use centralized symbols for evidence bullets
//! v0.0.451: Enhanced fly-on-the-wall view per VISION.md

use anna_shared::change::ChangePlan;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::theatre::{NarrativeSegment, Speaker};
use anna_shared::transcript::{Actor, TranscriptEventKind};
use anna_shared::ui::{colors, symbols};

use crate::output::{format_for_output, OutputMode};

use super::footer::print_footer;
use super::narrative::build_narrative;

/// Render result in theatre mode (cinematic IT department experience)
/// v0.0.451: Enhanced fly-on-the-wall view per VISION.md
pub fn render_theatre(result: &ServiceDeskResult, show_internal: bool) {
    let output_mode = OutputMode::detect();

    // v0.0.451: Reset internal header flag for this render
    reset_internal_header();

    println!();

    // 1. Show user query
    print_user_query(result);

    // 2. Build and display narrative
    let narrative = build_narrative(result, show_internal);

    // 3. Display narrative segments
    for segment in &narrative {
        print_segment(segment, output_mode);
    }

    // 4. Show Anna's final answer
    print_final_answer(result, output_mode);

    // 4b. Summarize any pending config changes
    print_change_summary(result);

    // 5. Show footer
    print_footer(result);
}

/// Print the user's query
fn print_user_query(result: &ServiceDeskResult) {
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}[you]{} {}\n", colors::CYAN, colors::RESET, text);
                break;
            }
        }
    }
}

/// v0.0.451: Track if we've shown the internal comms header
static mut INTERNAL_HEADER_SHOWN: bool = false;

/// Print a narrative segment
/// v0.0.451: Enhanced fly-on-the-wall view per VISION.md
fn print_segment(segment: &NarrativeSegment, _output_mode: OutputMode) {
    // v0.0.168: Get username for personalized display
    let username = std::env::var("USER").unwrap_or_else(|_| "you".to_string());

    match &segment.speaker {
        Speaker::Anna => {
            if segment.internal {
                // v0.0.451: Fly-on-the-wall format per VISION.md
                show_internal_header();
                // Determine recipient from context (default to appropriate team)
                let recipient = segment
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("to"))
                    .map(|s| s.as_str())
                    .unwrap_or("Team");
                println!(
                    "{}Anna to {}:{} {}",
                    colors::OK,
                    recipient,
                    colors::RESET,
                    segment.text
                );
            }
            // External Anna dialogue shown with answer
        }
        Speaker::You => {
            // v0.0.168: Show username instead of "You"
            println!(
                "{}{}:{} {}",
                colors::CYAN,
                username,
                colors::RESET,
                segment.text
            );
        }
        Speaker::TeamMember { name, role, .. } => {
            // v0.0.451: Fly-on-the-wall format "Name (Role) to Anna: message"
            if segment.internal {
                show_internal_header();
                println!(
                    "{}{} ({}) to Anna:{} {}",
                    colors::WARN,
                    name,
                    role,
                    colors::RESET,
                    segment.text
                );
            } else {
                println!(
                    "{}{} ({}):{} {}",
                    colors::WARN,
                    name,
                    role,
                    colors::RESET,
                    segment.text
                );
            }
        }
        Speaker::Narrator => {
            println!("{}{}...{}", colors::DIM, segment.text, colors::RESET);
        }
    }
}

/// v0.0.451: Show the internal communication header once per render
fn show_internal_header() {
    // Safety: this is single-threaded CLI rendering
    unsafe {
        if !INTERNAL_HEADER_SHOWN {
            println!();
            println!(
                "{}--- Internal communication ---{}",
                colors::DIM,
                colors::RESET
            );
            println!();
            INTERNAL_HEADER_SHOWN = true;
        }
    }
}

/// v0.0.451: Reset the internal header flag for next render
pub fn reset_internal_header() {
    unsafe {
        INTERNAL_HEADER_SHOWN = false;
    }
}

/// Print Anna's final answer with evidence bullets (v0.0.252)
fn print_final_answer(result: &ServiceDeskResult, output_mode: OutputMode) {
    // Find the final answer
    let answer = get_final_answer_text(result);

    if !answer.is_empty() {
        println!();
        println!("{}[anna]{}", colors::OK, colors::RESET);
        println!("{}", format_for_output(&answer, output_mode));

        // v0.0.252: Show evidence bullets if we have probe data
        print_evidence_bullets(result);

        println!();
    }
}

/// v0.0.252: Print evidence bullets showing data sources
fn print_evidence_bullets(result: &ServiceDeskResult) {
    let evidence = &result.evidence;

    // Collect evidence items
    let mut items: Vec<String> = Vec::new();

    // Add probe evidence (most valuable - shows actual data used)
    for probe in &evidence.probes_executed {
        if probe.exit_code == 0 && !probe.stdout.is_empty() {
            // Summarize the probe output concisely
            let summary = summarize_probe_output(&probe.command, &probe.stdout);
            if !summary.is_empty() {
                items.push(summary);
            }
        }
    }

    // Add hardware fields as evidence source (if no probes)
    if items.is_empty() && !evidence.hardware_fields.is_empty() {
        let fields = evidence.hardware_fields.join(", ");
        items.push(format!("hardware snapshot: {}", fields));
    }

    // Print evidence bullets
    if !items.is_empty() {
        for item in items.iter().take(3) {
            // Max 3 evidence lines
            println!(
                "{}{} evidence:{} {}",
                colors::DIM,
                symbols::BULLET,
                colors::RESET,
                item
            );
        }
    }
}

/// Summarize probe output for evidence display
fn summarize_probe_output(command: &str, stdout: &str) -> String {
    // v0.0.303: Show full first line - no truncation for better UX
    let first_line = stdout.lines().next().unwrap_or("").trim();
    let value = first_line.to_string();

    if value.is_empty() {
        return String::new();
    }

    // Map commands to human-readable evidence sources
    if command.contains("/proc/meminfo") || command.contains("free") {
        return format!("/proc/meminfo showed {}", value);
    }
    if command.contains("df ") {
        return format!("disk usage: {}", value);
    }
    if command.contains("/proc/cpuinfo") || command.contains("model name") {
        return format!("CPU: {}", value);
    }
    if command.contains("nproc") {
        return format!("{} CPU cores", value);
    }
    if command.contains("os-release") || command.contains("lsb_release") {
        return format!("OS: {}", value);
    }
    if command.contains("hostname") {
        return format!("hostname: {}", value);
    }
    if command.contains("uptime") {
        return format!("uptime: {}", value);
    }
    if command.contains("loadavg") {
        return format!("load average: {}", value);
    }
    if command.contains("ip addr") || command.contains("ip a") {
        return format!("network: {}", value);
    }
    if command.contains("resolv.conf") {
        return format!("DNS config: {}", value);
    }
    if command.contains("systemctl") && command.contains("failed") {
        return format!("failed services: {}", value);
    }
    if command.contains("journalctl") {
        return format!("system logs: {}", value);
    }

    // v0.0.303: Show full command - no truncation
    format!("{} → {}", command, value)
}

/// Get the final answer text
fn get_final_answer_text(result: &ServiceDeskResult) -> String {
    // Check transcript for FinalAnswer
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { text } = &event.kind {
            return text.clone();
        }
    }

    // Check clarification
    if result.needs_clarification {
        if let Some(q) = &result.clarification_question {
            let mut text = q.clone();
            // Add options if present
            if let Some(ref clarify) = result.clarification_request {
                if !clarify.options.is_empty() {
                    text.push('\n');
                    for (i, opt) in clarify.options.iter().enumerate() {
                        text.push_str(&format!("\n  {}. {}", i + 1, opt.label));
                    }
                }
            }
            return text;
        }
        return "I need more information to answer your question.".to_string();
    }

    // Fallback to answer field
    if !result.answer.is_empty() {
        return result.answer.clone();
    }

    String::new()
}

/// Summarize proposed configuration changes (if any)
fn print_change_summary(result: &ServiceDeskResult) {
    let changes: Vec<ChangePlan> = if !result.proposed_changes.is_empty() {
        result.proposed_changes.clone()
    } else {
        result.proposed_change.iter().cloned().collect()
    };

    if changes.is_empty() {
        return;
    }

    println!(
        "{}[anna: config change proposal]{}",
        colors::HEADER,
        colors::RESET
    );
    for (idx, change) in changes.iter().enumerate() {
        let status = if change.is_noop {
            format!("{}noop{}", colors::DIM, colors::RESET)
        } else {
            format!("{}apply{}", colors::OK, colors::RESET)
        };
        println!("  {}. {}  {}", idx + 1, change.summary(), status);
        println!("     file: {}", change.target_path.display());
    }
    println!();
}

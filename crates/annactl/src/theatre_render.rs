//! Theatre-style rendering for Service Desk experience (v0.0.81).
//!
//! Transforms ServiceDeskResult into cinematic narrative dialogue.
//! Shows the IT department working like a fly on the wall.

use anna_shared::narrator::{it_confidence, it_domain_context};
use anna_shared::roster::Tier;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::teams::Team;
use anna_shared::theatre::{describe_check, NarrativeBuilder, NarrativeSegment, Speaker};
use anna_shared::transcript::{Actor, TranscriptEventKind};
use anna_shared::ui::colors;

use crate::output::{format_for_output, OutputMode};

/// Render result in theatre mode (cinematic IT department experience)
pub fn render_theatre(result: &ServiceDeskResult, show_internal: bool) {
    let output_mode = OutputMode::detect();

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

/// Build narrative from result
fn build_narrative(result: &ServiceDeskResult, show_internal: bool) -> Vec<NarrativeSegment> {
    let mut builder = NarrativeBuilder::new();
    if show_internal {
        builder = builder.with_internal_comms();
    }

    let domain_str = result.domain.to_string().to_lowercase();

    // Get team from domain
    let team = team_from_domain(&domain_str);

    // Check if we have probes
    let has_probes = !result.evidence.probes_executed.is_empty();
    let probe_ids: Vec<String> = result.evidence.probes_executed
        .iter()
        .map(|p| probe_id_from_command(&p.command))
        .collect();

    // Add checking narration if we ran probes
    if has_probes {
        let check_desc = describe_check(&probe_ids);
        builder.add_checking(&check_desc);
    }

    // Check for junior/senior review events
    let mut had_junior_review = false;
    let mut had_escalation = false;

    for event in &result.transcript.events {
        match &event.kind {
            TranscriptEventKind::JuniorReview { score, verified, .. } => {
                had_junior_review = true;
                if show_internal {
                    builder.add_junior_review(team, *verified, *score);
                }
            }
            TranscriptEventKind::SeniorEscalation { successful, reason } => {
                had_escalation = true;
                if show_internal {
                    if let Some(r) = reason {
                        builder.add_escalation(team, r);
                    }
                    if *successful {
                        builder.add_senior_response(team, "I've reviewed it. Here's what I found.");
                    }
                }
            }
            TranscriptEventKind::TeamReview { team: t, reviewer, .. } => {
                if show_internal && reviewer == "senior" {
                    // Senior was involved
                    had_escalation = true;
                }
            }
            _ => {}
        }
    }

    // Add wait apology if escalation happened
    if had_escalation && show_internal {
        builder.add_wait_apology();
    }

    builder.build()
}

/// Print a narrative segment
fn print_segment(segment: &NarrativeSegment, _output_mode: OutputMode) {
    match &segment.speaker {
        Speaker::Anna => {
            if segment.internal {
                // Internal comms shown in dim
                println!(
                    "{}--- Internal ---{}",
                    colors::DIM, colors::RESET
                );
                println!(
                    "{}Anna:{} {}",
                    colors::OK, colors::RESET, segment.text
                );
            }
            // External Anna dialogue shown with answer
        }
        Speaker::You => {
            println!(
                "{}You:{} {}",
                colors::CYAN, colors::RESET, segment.text
            );
        }
        Speaker::TeamMember { name, role, .. } => {
            println!(
                "{}{} ({}):{} {}",
                colors::WARN, name, role, colors::RESET, segment.text
            );
        }
        Speaker::Narrator => {
            println!(
                "{}{}...{}",
                colors::DIM, segment.text, colors::RESET
            );
        }
    }
}

/// Print Anna's final answer
fn print_final_answer(result: &ServiceDeskResult, output_mode: OutputMode) {
    // Find the final answer
    let answer = get_final_answer_text(result);

    if !answer.is_empty() {
        println!();
        println!("{}[anna]{}", colors::OK, colors::RESET);
        println!("{}", format_for_output(&answer, output_mode));
        println!();
    }
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
                    text.push_str("\n");
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

/// Print the footer
fn print_footer(result: &ServiceDeskResult) {
    let rel_color = reliability_color(result.reliability_score);
    let confidence_note = it_confidence(result.reliability_score);
    let domain_str = result.domain.to_string();
    let domain_context = it_domain_context(&domain_str);

    // Evidence source
    let evidence_source = if result.reliability_signals.answer_grounded {
        format_evidence_source(result)
    } else {
        String::new()
    };

    if evidence_source.is_empty() {
        println!(
            "{}{} | {} | {}{}%{}",
            colors::DIM, domain_context, confidence_note,
            rel_color, result.reliability_score, colors::RESET
        );
    } else {
        println!(
            "{}{} | {} | {}{}%{} | {}{}",
            colors::DIM, domain_context, confidence_note,
            rel_color, result.reliability_score, colors::RESET,
            colors::DIM, evidence_source
        );
    }
}

/// Format evidence source for footer
fn format_evidence_source(result: &ServiceDeskResult) -> String {
    if let Some(trace) = &result.execution_trace {
        if !trace.evidence_kinds.is_empty() {
            let kinds: Vec<&str> = trace.evidence_kinds.iter()
                .map(|k| match k {
                    anna_shared::trace::EvidenceKind::Audio => "audio",
                    anna_shared::trace::EvidenceKind::ToolExists => "tools",
                    anna_shared::trace::EvidenceKind::Memory => "memory",
                    anna_shared::trace::EvidenceKind::Disk => "disk",
                    anna_shared::trace::EvidenceKind::Cpu => "cpu",
                    anna_shared::trace::EvidenceKind::Processes => "ps",
                    anna_shared::trace::EvidenceKind::Network => "network",
                    anna_shared::trace::EvidenceKind::Services => "services",
                    anna_shared::trace::EvidenceKind::Journal => "logs",
                    _ => "probe",
                })
                .collect();
            return format!("Verified from {}", kinds.join("+"));
        }
    }

    let success = result.evidence.probes_executed.iter().filter(|p| p.exit_code == 0).count();
    if success > 0 {
        format!("Verified from {} probe{}", success, if success == 1 { "" } else { "s" })
    } else {
        String::new()
    }
}

/// Get color for reliability score
fn reliability_color(score: u8) -> &'static str {
    match score {
        80..=100 => colors::OK,
        50..=79 => colors::WARN,
        _ => colors::ERR,
    }
}

/// Map domain string to Team
fn team_from_domain(domain: &str) -> Team {
    match domain {
        "storage" => Team::Storage,
        "memory" => Team::Performance,
        "network" => Team::Network,
        "performance" | "cpu" => Team::Performance,
        "service" | "services" => Team::Services,
        "security" => Team::Security,
        "hardware" | "audio" => Team::Hardware,
        "desktop" | "editor" => Team::Desktop,
        "logs" => Team::Logs,
        _ => Team::General,
    }
}

/// Extract probe ID from command
fn probe_id_from_command(command: &str) -> String {
    let cmd = command.to_lowercase();
    if cmd.starts_with("df ") || cmd == "df" { return "df".to_string(); }
    if cmd.starts_with("free ") || cmd == "free" { return "free".to_string(); }
    if cmd.starts_with("lscpu") { return "lscpu".to_string(); }
    if cmd.contains("sensors") { return "sensors".to_string(); }
    if cmd.starts_with("systemctl") { return "systemctl".to_string(); }
    if cmd.contains("journalctl") { return "journalctl".to_string(); }
    if cmd.starts_with("ip ") { return "ip".to_string(); }
    if cmd.contains("lspci") && cmd.contains("audio") { return "lspci_audio".to_string(); }
    if cmd.contains("pactl") { return "pactl_cards".to_string(); }
    if cmd.starts_with("lsblk") { return "lsblk".to_string(); }
    if cmd.starts_with("uname") { return "uname".to_string(); }
    if cmd.contains("command -v") { return "command_v".to_string(); }
    "probe".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_from_domain() {
        assert_eq!(team_from_domain("storage"), Team::Storage);
        assert_eq!(team_from_domain("network"), Team::Network);
        assert_eq!(team_from_domain("unknown"), Team::General);
    }

    #[test]
    fn test_probe_id_from_command() {
        assert_eq!(probe_id_from_command("df -h"), "df");
        assert_eq!(probe_id_from_command("free -h"), "free");
        assert_eq!(probe_id_from_command("lspci | grep -i audio"), "lspci_audio");
    }

    #[test]
    fn test_reliability_color() {
        assert_eq!(reliability_color(90), colors::OK);
        assert_eq!(reliability_color(60), colors::WARN);
        assert_eq!(reliability_color(30), colors::ERR);
    }
}

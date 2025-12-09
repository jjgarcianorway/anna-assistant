//! Debug mode rendering (v0.0.179).

use anna_shared::narrator::status_indicator;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, TranscriptEventKind};
use anna_shared::ui::colors;

use crate::output::{format_for_output, OutputMode};

use super::answer_source::{get_final_answer, AnswerSource};
use super::event_renders;
use super::helpers::{format_actor_tag, format_outcome, reliability_color, truncate};

/// Render in debug mode - full troubleshooting view
/// v0.0.106: Shows case number and assigned staff
pub fn render_debug(result: &ServiceDeskResult, output_mode: OutputMode) {
    // v0.0.106: Show case number if present
    let case_info = match (&result.case_number, &result.assigned_staff) {
        (Some(cn), Some(staff)) => format!(
            " {}case={} staff={}{}",
            colors::CYAN,
            cn,
            staff,
            colors::RESET
        ),
        (Some(cn), None) => format!(" {}case={}{}", colors::CYAN, cn, colors::RESET),
        _ => String::new(),
    };
    println!(
        "\n{}[transcript]{} request_id={}{}\n",
        colors::DIM,
        colors::RESET,
        &result.request_id[..8],
        case_info
    );

    let answer_source = get_final_answer(result);
    let answer_in_transcript = matches!(answer_source, AnswerSource::Transcript);
    let mut last_actor: Option<Actor> = None;

    for event in &result.transcript.events {
        match &event.kind {
            TranscriptEventKind::Message { text } => {
                render_message(&event.from, text, output_mode, &mut last_actor);
            }
            TranscriptEventKind::StageStart { stage } => {
                println!("\n{}[{}]{} starting...", colors::DIM, stage, colors::RESET);
                last_actor = None;
            }
            TranscriptEventKind::StageEnd { stage, outcome } => {
                println!(
                    "{}[{}]{} {}",
                    colors::DIM,
                    stage,
                    colors::RESET,
                    format_outcome(outcome)
                );
            }
            TranscriptEventKind::ProbeStart { probe_id, command } => {
                println!(
                    "\n{}[probe]{}\n{} -> {}",
                    colors::DIM,
                    colors::RESET,
                    probe_id,
                    truncate(command, 50)
                );
                last_actor = Some(Actor::Probe);
            }
            TranscriptEventKind::ProbeEnd {
                probe_id,
                exit_code,
                timing_ms,
                stdout_preview,
            } => {
                event_renders::render_probe_end(probe_id, *exit_code, *timing_ms, stdout_preview.as_deref());
            }
            TranscriptEventKind::Note { text } => {
                println!("{}  note: {}{}", colors::DIM, text, colors::RESET);
            }
            TranscriptEventKind::FinalAnswer { text } => {
                println!(
                    "\n{}[anna]{}\n{}",
                    colors::OK,
                    colors::RESET,
                    format_for_output(text, output_mode)
                );
                last_actor = Some(Actor::Anna);
            }
            TranscriptEventKind::TicketCreated {
                ticket_id,
                domain,
                intent,
                evidence_required,
            } => {
                event_renders::render_ticket_created(ticket_id, domain, intent, *evidence_required);
                last_actor = None;
            }
            TranscriptEventKind::TicketStatusChanged {
                from_status,
                to_status,
                ..
            } => {
                println!(
                    "{}[ticket]{} {} -> {}",
                    colors::DIM,
                    colors::RESET,
                    from_status,
                    to_status
                );
            }
            TranscriptEventKind::JuniorReview {
                attempt,
                score,
                verified,
                issues,
            } => {
                event_renders::render_junior_review(*attempt, *score, *verified, issues);
                last_actor = Some(Actor::Junior);
            }
            TranscriptEventKind::SeniorEscalation { successful, reason } => {
                event_renders::render_senior_escalation(*successful, reason.as_deref());
                last_actor = Some(Actor::Senior);
            }
            TranscriptEventKind::RevisionApplied { changes_made } => {
                event_renders::render_revision_applied(changes_made);
            }
            TranscriptEventKind::ReviewGateDecision {
                decision,
                score,
                requires_llm,
            } => {
                event_renders::render_review_gate(decision, *score, *requires_llm);
                last_actor = None;
            }
            TranscriptEventKind::TeamReview {
                team,
                reviewer,
                decision,
                issues_count,
            } => {
                event_renders::render_team_review(team, reviewer, decision, *issues_count);
                last_actor = None;
            }
            TranscriptEventKind::ClarificationAsked {
                question_id: _,
                prompt,
                choices,
                reason,
            } => {
                event_renders::render_clarification_asked(prompt, choices, reason);
                last_actor = None;
            }
            TranscriptEventKind::ClarificationAnswered {
                question_id: _,
                answer,
            } => {
                println!("{}[you]{} {}", colors::DIM, colors::RESET, answer);
                last_actor = Some(Actor::You);
            }
            TranscriptEventKind::ClarificationVerified {
                question_id: _,
                verified,
                source,
                alternatives,
            } => {
                event_renders::render_clarification_verified(*verified, source, alternatives);
                last_actor = None;
            }
            TranscriptEventKind::FactStored { key, value, source } => {
                println!(
                    "{}[fact]{} {} = {} (via {})",
                    colors::DIM,
                    colors::RESET,
                    key,
                    value,
                    source
                );
                last_actor = None;
            }
            TranscriptEventKind::FastPath {
                handled,
                class,
                reason,
                probes_needed,
            } => {
                event_renders::render_fast_path(*handled, class, reason, *probes_needed);
                last_actor = None;
            }
            TranscriptEventKind::LlmTimeoutFallback {
                stage,
                timeout_secs,
                elapsed_secs,
                fallback_action,
            } => {
                println!(
                    "{}[timeout]{} {} timed out ({}s > {}s) -> {}",
                    colors::WARN,
                    colors::RESET,
                    stage,
                    elapsed_secs,
                    timeout_secs,
                    fallback_action
                );
                last_actor = None;
            }
            TranscriptEventKind::GracefulDegradation {
                reason,
                original_type,
                fallback_type,
            } => {
                println!(
                    "{}[fallback]{} {} -> {} ({})",
                    colors::WARN,
                    colors::RESET,
                    original_type,
                    fallback_type,
                    reason
                );
                last_actor = None;
            }
            TranscriptEventKind::EvidenceSummary {
                evidence_kinds,
                probe_count,
                key_findings,
            } => {
                event_renders::render_evidence_summary(evidence_kinds, *probe_count, key_findings);
                last_actor = None;
            }
            TranscriptEventKind::DeterministicPath {
                route_class,
                evidence_used,
            } => {
                println!(
                    "{}[deterministic]{} {} (evidence: {:?})",
                    colors::OK,
                    colors::RESET,
                    route_class,
                    evidence_used
                );
                last_actor = None;
            }
            TranscriptEventKind::ProposedAction {
                action_id,
                description,
                risk_level,
                rollback_available,
            } => {
                event_renders::render_proposed_action(action_id, description, risk_level, *rollback_available);
                last_actor = None;
            }
            TranscriptEventKind::ActionConfirmationRequest {
                action_id: _,
                prompt,
                options,
            } => {
                event_renders::render_action_confirmation(prompt, options);
                last_actor = None;
            }
            TranscriptEventKind::Unknown => {}
        }
    }

    // Final answer if not in transcript
    if !answer_in_transcript {
        println!("\n{}[anna]{}", colors::OK, colors::RESET);
        match answer_source {
            AnswerSource::Clarification(t) | AnswerSource::Answer(t) => {
                println!("{}", format_for_output(t, output_mode));
            }
            AnswerSource::Empty => println!("(no answer generated)"),
            AnswerSource::Transcript => unreachable!(),
        }
    }

    // Summary block
    render_summary(result);
}

fn render_message(from: &Actor, text: &str, output_mode: OutputMode, last_actor: &mut Option<Actor>) {
    if last_actor.as_ref() != Some(from) {
        println!("\n{}", format_actor_tag(from));
        *last_actor = Some(*from);
    }
    let formatted = if *from == Actor::Anna {
        format_for_output(text, output_mode)
    } else {
        text.to_string()
    };
    for line in formatted.lines().filter(|l| !l.trim().is_empty()) {
        println!("{}", line);
    }
}

fn render_summary(result: &ServiceDeskResult) {
    let rel_color = reliability_color(result.reliability_score);
    println!(
        "\n{}reliability:{} {}{}%{}   {}domain:{} {}   {}probes:{} {}",
        colors::DIM,
        colors::RESET,
        rel_color,
        result.reliability_score,
        colors::RESET,
        colors::DIM,
        colors::RESET,
        result.domain,
        colors::DIM,
        colors::RESET,
        result.evidence.probes_executed.len()
    );

    let s = &result.reliability_signals;
    println!(
        "{}signals: confident={} coverage={} grounded={} no_invention={} no_clarify={}{}",
        colors::DIM,
        status_indicator(s.translator_confident),
        status_indicator(s.probe_coverage),
        status_indicator(s.answer_grounded),
        status_indicator(s.no_invention),
        status_indicator(s.clarification_not_needed),
        colors::RESET
    );

    if let Some(trace) = &result.execution_trace {
        println!("{}trace: {}{}", colors::DIM, trace, colors::RESET);
    }
}

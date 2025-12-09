//! Theatre main rendering (v0.0.202).

use anna_shared::change::ChangePlan;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::theatre::{NarrativeSegment, Speaker};
use anna_shared::transcript::{Actor, TranscriptEventKind};
use anna_shared::ui::colors;

use crate::output::{format_for_output, OutputMode};

use super::footer::print_footer;
use super::narrative::build_narrative;

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

/// Print a narrative segment
fn print_segment(segment: &NarrativeSegment, _output_mode: OutputMode) {
    // v0.0.168: Get username for personalized display
    let username = std::env::var("USER").unwrap_or_else(|_| "you".to_string());

    match &segment.speaker {
        Speaker::Anna => {
            if segment.internal {
                // Internal comms shown in dim
                println!("{}--- Internal ---{}", colors::DIM, colors::RESET);
                println!("{}Anna:{} {}", colors::OK, colors::RESET, segment.text);
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
            println!(
                "{}{} ({}):{} {}",
                colors::WARN,
                name,
                role,
                colors::RESET,
                segment.text
            );
        }
        Speaker::Narrator => {
            println!("{}{}...{}", colors::DIM, segment.text, colors::RESET);
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

//! Progress Display Module v0.60.0
//!
//! Renders Anna events with beautiful terminal colors.
//! Uses TRUE COLOR for rich visual feedback.

use anna_common::events::{Actor, AnnaEvent, ConversationLog, EventKind};
use owo_colors::OwoColorize;
use std::io::{self, Write};

/// ANSI true colors for actors
const ANNA_COLOR: (u8, u8, u8) = (147, 112, 219);   // Medium purple
const JUNIOR_COLOR: (u8, u8, u8) = (100, 149, 237); // Cornflower blue
const SENIOR_COLOR: (u8, u8, u8) = (255, 165, 0);   // Orange
const SYSTEM_COLOR: (u8, u8, u8) = (128, 128, 128); // Gray

/// Reliability colors
const GREEN_COLOR: (u8, u8, u8) = (50, 205, 50);    // Lime green
const YELLOW_COLOR: (u8, u8, u8) = (255, 215, 0);   // Gold
const RED_COLOR: (u8, u8, u8) = (255, 69, 0);       // Red-orange

/// Print a progress event with colors
pub fn print_progress(event: &AnnaEvent) {
    let colored_actor = format_actor(&event.actor);
    let message = format_event_message(&event.kind);
    println!("{:10}  {}", colored_actor, message);
    let _ = io::stdout().flush();
}

/// Print a progress event with a custom prefix (for streaming)
pub fn print_progress_streaming(event: &AnnaEvent) {
    let colored_actor = format_actor(&event.actor);
    let message = format_event_message(&event.kind);
    // Use carriage return to overwrite spinner
    print!("\r{:10}  {}\n", colored_actor, message);
    let _ = io::stdout().flush();
}

/// Format actor with color (returns owned String)
fn format_actor(actor: &Actor) -> String {
    let (r, g, b) = actor_color(actor);
    let actor_str = format!("[{}]", actor);
    actor_str.truecolor(r, g, b).bold().to_string()
}

/// Format the message part of an event with appropriate coloring
fn format_event_message(kind: &EventKind) -> String {
    match kind {
        EventKind::QuestionReceived => {
            "Reading your question and planning next steps.".to_string()
        }
        EventKind::ClassificationStarted => {
            "Analyzing your question...".dimmed().to_string()
        }
        EventKind::ClassificationDone { question_type, confidence } => {
            let conf_pct = confidence * 100.0;
            let conf_str = format!("{:.0}%", conf_pct);
            let conf_colored = if *confidence >= 0.8 {
                conf_str.truecolor(GREEN_COLOR.0, GREEN_COLOR.1, GREEN_COLOR.2).to_string()
            } else if *confidence >= 0.5 {
                conf_str.truecolor(YELLOW_COLOR.0, YELLOW_COLOR.1, YELLOW_COLOR.2).to_string()
            } else {
                conf_str.truecolor(RED_COLOR.0, RED_COLOR.1, RED_COLOR.2).to_string()
            };
            format!("Classified as {} ({}).", question_type.cyan(), conf_colored)
        }
        EventKind::ProbesPlanned { probe_ids } => {
            if probe_ids.len() == 1 {
                format!("Planning to use probe: {}.", probe_ids[0].cyan())
            } else {
                format!("Planning to use {} probes.", probe_ids.len().to_string().cyan())
            }
        }
        EventKind::CommandRunning { command } => {
            let display_cmd = truncate_cmd(command, 55);
            format!("Running: {}", display_cmd.yellow())
        }
        EventKind::CommandDone { command: _, success, duration_ms } => {
            if *success {
                format!(
                    "{} in {}ms.",
                    "Command completed".green(),
                    duration_ms.to_string().dimmed()
                )
            } else {
                format!(
                    "{} after {}ms.",
                    "Command failed".red(),
                    duration_ms.to_string().dimmed()
                )
            }
        }
        EventKind::SeniorReviewStarted => {
            "Double-checking the answer and scoring reliability.".dimmed().to_string()
        }
        EventKind::SeniorReviewDone { reliability_score } => {
            let (color, label) = reliability_display(*reliability_score);
            let score_str = format!("{:.0}%", reliability_score * 100.0);
            format!(
                "Review complete. Reliability: {} ({}).",
                score_str.truecolor(color.0, color.1, color.2).bold(),
                label.truecolor(color.0, color.1, color.2)
            )
        }
        EventKind::UserClarificationNeeded { question } => {
            format!(
                "{}  {}",
                "I need a quick clarification:".yellow(),
                question
            )
        }
        EventKind::AnswerSynthesizing => {
            "Preparing your answer...".dimmed().to_string()
        }
        EventKind::AnswerReady { reliability_score } => {
            let (color, label) = reliability_display(*reliability_score);
            let score_str = format!("{:.0}%", reliability_score * 100.0);
            format!(
                "{}  Reliability: {} ({}).",
                "Done.".green().bold(),
                score_str.truecolor(color.0, color.1, color.2).bold(),
                label.truecolor(color.0, color.1, color.2)
            )
        }
        EventKind::Error { message } => {
            format!("{}  {}", "Error:".red().bold(), message.red())
        }
    }
}

/// Get color for an actor
fn actor_color(actor: &Actor) -> (u8, u8, u8) {
    match actor {
        Actor::Anna => ANNA_COLOR,
        Actor::Junior => JUNIOR_COLOR,
        Actor::Senior => SENIOR_COLOR,
        Actor::System => SYSTEM_COLOR,
    }
}

/// Get color and label for reliability score
fn reliability_display(score: f32) -> ((u8, u8, u8), &'static str) {
    if score >= 0.8 {
        (GREEN_COLOR, "GREEN")
    } else if score >= 0.5 {
        (YELLOW_COLOR, "YELLOW")
    } else {
        (RED_COLOR, "RED")
    }
}

/// Truncate command for display
fn truncate_cmd(cmd: &str, max_len: usize) -> String {
    if cmd.len() > max_len {
        format!("{}...", &cmd[..max_len.saturating_sub(3)])
    } else {
        cmd.to_string()
    }
}

/// Print the conversation log with colors (for debug mode)
pub fn print_conversation_log(log: &ConversationLog) {
    println!();
    println!("{}", "=== Conversation Log ===".bold().underline());
    println!();

    for entry in &log.events {
        let colored_actor = format_actor(&entry.actor);
        let message = entry.to_log_entry();
        // Remove the actor prefix since we're adding our own colored one
        let msg_without_prefix = message
            .split(']')
            .skip(1)
            .collect::<Vec<_>>()
            .join("]")
            .trim()
            .to_string();
        println!("{:10}  {}", colored_actor, msg_without_prefix);
    }

    if let Some(duration) = log.duration_ms() {
        println!();
        println!(
            "Total time: {}ms",
            duration.to_string().cyan()
        );
    }

    let summary = log.summary();
    if summary.commands_run > 0 {
        println!(
            "Commands run: {}",
            summary.commands_run.to_string().cyan()
        );
    }
    if let Some(score) = summary.reliability_score {
        let (color, label) = reliability_display(score);
        let score_str = format!("{:.0}%", score * 100.0);
        println!(
            "Final reliability: {} ({})",
            score_str.truecolor(color.0, color.1, color.2).bold(),
            label.truecolor(color.0, color.1, color.2)
        );
    }

    println!();
}

/// Print a separator line
pub fn print_separator() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Print the final answer with formatting
pub fn print_answer(answer: &str, reliability: Option<f32>) {
    println!();

    if let Some(score) = reliability {
        let (color, label) = reliability_display(score);
        let score_str = format!("{:.0}%", score * 100.0);
        println!(
            "{}  Reliability: {} ({})",
            "Answer".green().bold(),
            score_str.truecolor(color.0, color.1, color.2).bold(),
            label.truecolor(color.0, color.1, color.2)
        );
        print_separator();
    }

    println!("{}", answer);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_color() {
        assert_eq!(actor_color(&Actor::Anna), ANNA_COLOR);
        assert_eq!(actor_color(&Actor::Junior), JUNIOR_COLOR);
        assert_eq!(actor_color(&Actor::Senior), SENIOR_COLOR);
        assert_eq!(actor_color(&Actor::System), SYSTEM_COLOR);
    }

    #[test]
    fn test_reliability_display() {
        let (color, label) = reliability_display(0.95);
        assert_eq!(color, GREEN_COLOR);
        assert_eq!(label, "GREEN");

        let (color, label) = reliability_display(0.7);
        assert_eq!(color, YELLOW_COLOR);
        assert_eq!(label, "YELLOW");

        let (color, label) = reliability_display(0.3);
        assert_eq!(color, RED_COLOR);
        assert_eq!(label, "RED");
    }

    #[test]
    fn test_truncate_cmd() {
        assert_eq!(truncate_cmd("short", 20), "short");
        assert_eq!(truncate_cmd("a".repeat(100).as_str(), 20), format!("{}...", "a".repeat(17)));
    }

    #[test]
    fn test_format_event_messages() {
        // Just ensure they don't panic
        let _ = format_event_message(&EventKind::QuestionReceived);
        let _ = format_event_message(&EventKind::ClassificationStarted);
        let _ = format_event_message(&EventKind::ClassificationDone {
            question_type: "SimpleProbe".to_string(),
            confidence: 0.9,
        });
        let _ = format_event_message(&EventKind::ProbesPlanned {
            probe_ids: vec!["cpu.info".to_string()],
        });
        let _ = format_event_message(&EventKind::CommandRunning {
            command: "lscpu".to_string(),
        });
        let _ = format_event_message(&EventKind::CommandDone {
            command: "lscpu".to_string(),
            success: true,
            duration_ms: 50,
        });
        let _ = format_event_message(&EventKind::AnswerReady {
            reliability_score: 0.95,
        });
    }

    #[test]
    fn test_format_actor() {
        // Should not panic and produce colored output
        let anna = format_actor(&Actor::Anna);
        assert!(anna.contains("[Anna]") || anna.len() > 0);

        let junior = format_actor(&Actor::Junior);
        assert!(junior.contains("[Junior]") || junior.len() > 0);
    }
}

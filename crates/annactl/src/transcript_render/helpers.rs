//! Rendering helper functions (v0.0.179).

use anna_shared::transcript::{Actor, StageOutcome};
use anna_shared::ui::colors;

/// Format actor tag for debug mode
pub fn format_actor_tag(actor: &Actor) -> String {
    match actor {
        Actor::You => format!("{}[you]{}", colors::CYAN, colors::RESET),
        Actor::Anna => format!("{}[anna]{}", colors::OK, colors::RESET),
        Actor::Junior => format!("{}[junior]{}", colors::CYAN, colors::RESET),
        Actor::Senior => format!("{}[senior]{}", colors::WARN, colors::RESET),
        _ => format!("{}[{}]{}", colors::DIM, actor, colors::RESET),
    }
}

/// Format stage outcome
pub fn format_outcome(outcome: &StageOutcome) -> String {
    match outcome {
        StageOutcome::Ok => format!("{}ok{}", colors::OK, colors::RESET),
        StageOutcome::Timeout => format!("{}TIMEOUT{}", colors::ERR, colors::RESET),
        StageOutcome::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
        StageOutcome::Skipped => format!("{}skipped{}", colors::WARN, colors::RESET),
        StageOutcome::Deterministic => {
            format!("{}skipped{} (deterministic)", colors::OK, colors::RESET)
        }
        StageOutcome::BudgetExceeded {
            stage,
            budget_ms,
            elapsed_ms,
        } => {
            format!(
                "{}BUDGET_EXCEEDED{} ({}: {}ms > {}ms)",
                colors::ERR,
                colors::RESET,
                stage,
                elapsed_ms,
                budget_ms
            )
        }
        StageOutcome::ClarificationRequired { question, choices } => {
            format!(
                "{}CLARIFY{} ({}, {} choices)",
                colors::WARN,
                colors::RESET,
                question,
                choices.len()
            )
        }
    }
}

/// Get color for reliability score
pub fn reliability_color(score: u8) -> &'static str {
    match score {
        80..=100 => colors::OK,
        50..=79 => colors::WARN,
        _ => colors::ERR,
    }
}

/// Shorten text for debug display (internal transcript events only)
pub fn truncate(s: &str, max: usize) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() > max {
        format!("{}~", &s[..max - 1])
    } else {
        s.to_string()
    }
}

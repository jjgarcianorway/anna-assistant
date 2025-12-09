//! Feedback handling (v0.0.205).

use anna_shared::ui::colors;
use anyhow::Result;
use std::io::{self, Write};

use crate::display::show_bootstrap_progress;

/// v0.0.103: Handle feedback request from Anna
/// When Anna is uncertain about a recipe answer, she asks the user for feedback
pub async fn handle_feedback_request(feedback_req: &anna_shared::recipe_feedback::FeedbackRequest) {
    use anna_shared::recipe_feedback::{
        apply_feedback, log_feedback, FeedbackRating, RecipeFeedback,
    };

    println!();
    println!(
        "{}[feedback]{} {}",
        colors::DIM,
        colors::RESET,
        feedback_req.question
    );
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return;
    }

    let input = input.trim().to_lowercase();
    let rating = match input.as_str() {
        "y" | "yes" | "helpful" | "good" => Some(FeedbackRating::Helpful),
        "n" | "no" | "not helpful" | "bad" => Some(FeedbackRating::NotHelpful),
        "partial" | "meh" | "ok" => Some(FeedbackRating::Partial),
        "" | "skip" => None, // User skipped feedback
        _ => {
            println!(
                "{}Skipping feedback (unrecognized input){}",
                colors::DIM,
                colors::RESET
            );
            None
        }
    };

    if let Some(r) = rating {
        let feedback = RecipeFeedback::new(&feedback_req.recipe_id, r);
        log_feedback(&feedback);

        if let Some(result) = apply_feedback(&feedback) {
            println!(
                "{}Thanks!{} Recipe confidence adjusted ({} → {})",
                colors::OK,
                colors::RESET,
                result.previous_score,
                result.new_score
            );
        } else {
            println!("{}Thanks for the feedback!{}", colors::OK, colors::RESET);
        }
    }
}

/// Handle request error with recovery
pub async fn handle_request_error(e: &anyhow::Error) -> Result<()> {
    let err_str = e.to_string();
    if err_str.contains("LLM") || err_str.contains("connect") {
        println!();
        println!(
            "{}Connection issue.{} Restarting...",
            colors::WARN,
            colors::RESET
        );
        show_bootstrap_progress().await?;
    } else {
        eprintln!("{}Error:{} {}", colors::ERR, colors::RESET, e);
    }
    Ok(())
}

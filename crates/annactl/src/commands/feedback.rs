//! Feedback handling (v0.0.336).
//! v0.0.304: Uses centralized error presentation.
//! v0.0.317: Also applies feedback to staff XP.
//! v0.0.336: Also applies feedback to probe learning.

use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use anna_shared::staff_stats::StaffStats;
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::colors;
use anyhow::Result;
use std::io::{self, Write};

use crate::display::show_bootstrap_progress;
use crate::errors;

/// v0.0.103: Handle feedback request from Anna
/// When Anna is uncertain about a recipe answer, she asks the user for feedback
/// v0.0.317: Also updates staff XP based on feedback
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
        // v0.0.305: Pass original query for negative feedback learning
        let mut feedback = RecipeFeedback::new(&feedback_req.recipe_id, r);
        if let Some(ref query) = feedback_req.original_query {
            feedback = feedback.with_query(query);
        }
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

        // v0.0.317: Also apply feedback to staff XP
        let helpful = matches!(r, FeedbackRating::Helpful);
        apply_staff_feedback(helpful);

        // v0.0.336: Also apply feedback to probe learning
        if let Some(ref query) = feedback_req.original_query {
            apply_learning_feedback(query, helpful);
        }
    }
}

/// v0.0.336: Apply feedback to probe learning system
fn apply_learning_feedback(query: &str, helpful: bool) {
    let mut store = ProbeLearningStore::load_with_decay();
    let category = QueryCategory::from_query(query);

    // We don't have specific probes from the feedback context,
    // but we can record the category-level feedback
    // This helps the learning system understand which categories need more attention
    let probes: Vec<String> = vec![]; // Empty probes - just category feedback

    let failure_reason = if helpful { None } else { Some("user_marked_unhelpful") };

    store.record_feedback(category, &probes, helpful, Some(query), failure_reason);

    let _ = store.save();
}

/// v0.0.317: Apply feedback to the staff member who handled the most recent ticket
fn apply_staff_feedback(helpful: bool) {
    let tracker = TicketTracker::for_user();
    if let Ok(Some(staff_id)) = tracker.most_recent_staff_id() {
        let mut stats = StaffStats::load();
        if let Some(result) = stats.apply_feedback(&staff_id, helpful) {
            let _ = stats.save();
            // Show staff XP change if significant
            if result.old_level != result.new_level {
                let direction = if result.new_level > result.old_level {
                    "leveled up"
                } else {
                    "leveled down"
                };
                println!(
                    "{}[staff]{} {} {} (level {} → {})",
                    colors::DIM,
                    colors::RESET,
                    staff_id.split('_').last().unwrap_or(&staff_id),
                    direction,
                    result.old_level,
                    result.new_level
                );
            }
        }
    }
}

/// Handle request error with recovery
/// v0.0.304: Uses centralized error presentation with recovery suggestions
pub async fn handle_request_error(e: &anyhow::Error) -> Result<()> {
    let err_str = e.to_string();

    // For connection issues, try to show bootstrap progress
    if err_str.contains("LLM") || err_str.contains("connect") || err_str.contains("daemon") {
        errors::print_warning("Connection issue detected, attempting recovery...");
        if let Err(_) = show_bootstrap_progress().await {
            // Bootstrap failed - show full error with suggestions
            errors::print_error(e);
        }
    } else {
        // Use the new user-friendly error presentation
        errors::print_error(e);
    }
    Ok(())
}

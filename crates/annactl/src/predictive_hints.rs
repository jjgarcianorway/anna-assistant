// Predictive Hints Integration
// Phase 3.8: Adaptive CLI - Post-Command Intelligence
//
// Displays High/Critical predictions after status/health commands
// with smart throttling and TTY detection.

use anyhow::Result;
use anna_common::context::{actions, db::ContextDb, DbLocation};
use anna_common::learning::{ActionSummary, LearningEngine};
use anna_common::prediction::{PredictionEngine, Priority};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Mutex;

/// Prediction display history for throttling
static DISPLAY_HISTORY: Mutex<Option<HashMap<String, DateTime<Utc>>>> = Mutex::new(None);

/// Check if we should display hints (24h throttle)
fn should_display_hints(command: &str) -> bool {
    let mut guard = DISPLAY_HISTORY.lock().unwrap();

    // Initialize on first access
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }

    let history = guard.as_mut().unwrap();

    if let Some(last_time) = history.get(command) {
        let elapsed = Utc::now().signed_duration_since(*last_time);
        if elapsed < Duration::hours(24) {
            return false;
        }
    }

    // Update last display time
    history.insert(command.to_string(), Utc::now());
    true
}

/// Aggregate ActionHistory records into ActionSummary for learning engine
fn aggregate_actions(action_history: &[actions::ActionHistory]) -> Vec<ActionSummary> {
    let mut summaries: HashMap<String, ActionSummary> = HashMap::new();

    for action in action_history {
        let entry = summaries
            .entry(action.action_type.clone())
            .or_insert_with(|| ActionSummary {
                action_type: action.action_type.clone(),
                total_count: 0,
                success_count: 0,
                failure_count: 0,
                avg_duration_ms: 0,
                last_execution: action.timestamp,
            });

        entry.total_count += 1;

        match action.outcome {
            actions::ActionOutcome::Success => entry.success_count += 1,
            actions::ActionOutcome::Failure => entry.failure_count += 1,
            actions::ActionOutcome::Cancelled => {}
        }

        // Update last execution
        if action.timestamp > entry.last_execution {
            entry.last_execution = action.timestamp;
        }
    }

    // Calculate average durations
    for action in action_history {
        if let Some(duration) = action.duration_ms {
            if let Some(entry) = summaries.get_mut(&action.action_type) {
                // Simple running average
                entry.avg_duration_ms =
                    (entry.avg_duration_ms + duration) / 2;
            }
        }
    }

    summaries.into_values().collect()
}

/// Display predictive hints after status/health commands
///
/// # Arguments
/// * `command` - The command that was just executed ("status" or "health")
/// * `json_mode` - Whether JSON output mode is active (skip hints)
/// * `use_color` - Whether to use colored output (TTY detection)
///
/// # Behavior
/// - Only displays High and Critical priority predictions
/// - Respects 24h throttle per command
/// - Skips in JSON mode or non-TTY
/// - Shows up to 3 most urgent predictions
pub async fn display_predictive_hints(
    command: &str,
    json_mode: bool,
    use_color: bool,
) -> Result<()> {
    // Skip if JSON mode or non-TTY
    if json_mode || !use_color {
        return Ok(());
    }

    // Check throttle
    if !should_display_hints(command) {
        return Ok(());
    }

    // Open context database
    let db = match ContextDb::open(DbLocation::auto_detect()).await {
        Ok(db) => db,
        Err(_) => {
            // Database not available, skip hints silently
            return Ok(());
        }
    };

    // Get recent action history
    let action_history = db
        .execute(|conn| actions::get_recent_actions(conn, 1000))
        .await?;

    // Aggregate into summaries
    let action_summaries = aggregate_actions(&action_history);

    // Analyze patterns
    let mut learning_engine = LearningEngine::new()
        .with_analysis_window(30)
        .with_min_occurrences(2);

    learning_engine.detect_maintenance_patterns(&action_summaries);
    learning_engine.detect_failure_patterns(&action_summaries);
    learning_engine.detect_usage_patterns(&action_summaries);

    let patterns = learning_engine.get_patterns();

    // Generate predictions
    let mut prediction_engine = PredictionEngine::new()
        .with_min_confidence(65)
        .with_throttle_hours(24);

    prediction_engine.generate_from_patterns(patterns);

    // Get High and Critical predictions
    let mut urgent_predictions: Vec<_> = prediction_engine
        .get_predictions()
        .iter()
        .filter(|p| p.priority == Priority::High || p.priority == Priority::Critical)
        .collect();

    // No urgent predictions to show
    if urgent_predictions.is_empty() {
        return Ok(());
    }

    // Sort by priority (Critical first) and confidence
    urgent_predictions.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| b.confidence.cmp(&a.confidence))
    });

    // Limit to top 3
    urgent_predictions.truncate(3);

    // Display hints
    println!();
    println!("┌─────────────────────────────────────────────────────────");
    println!("│ PREDICTIVE INTELLIGENCE");
    println!("├─────────────────────────────────────────────────────────");

    for pred in urgent_predictions {
        let priority_emoji = pred.priority.emoji();
        let type_str = format!("{:?}", pred.prediction_type);

        // Format: [EMOJI] Type · Reason · Action
        println!("│ {} {} · {}", priority_emoji, type_str, pred.title);

        if !pred.recommended_actions.is_empty() {
            let action = &pred.recommended_actions[0];
            println!("│   → {}", action);
        }
    }

    println!("└─────────────────────────────────────────────────────────");
    println!();

    Ok(())
}

/// Display comprehensive predictive report (for explicit `annactl predict` command)
pub async fn display_full_predictions(json: bool) -> Result<()> {
    // Open context database
    let db = ContextDb::open(DbLocation::auto_detect()).await?;

    // Get recent action history
    let action_history = db
        .execute(|conn| actions::get_recent_actions(conn, 1000))
        .await?;

    // Aggregate into summaries
    let action_summaries = aggregate_actions(&action_history);

    // Analyze patterns
    let mut learning_engine = LearningEngine::new()
        .with_analysis_window(30)
        .with_min_occurrences(2);

    learning_engine.detect_maintenance_patterns(&action_summaries);
    learning_engine.detect_failure_patterns(&action_summaries);
    learning_engine.detect_usage_patterns(&action_summaries);

    let patterns = learning_engine.get_patterns();

    // Generate predictions
    let mut prediction_engine = PredictionEngine::new()
        .with_min_confidence(50) // Lower threshold for explicit report
        .with_throttle_hours(24);

    prediction_engine.generate_from_patterns(patterns);

    let predictions = prediction_engine.get_predictions();

    if json {
        // JSON output
        let json_output = serde_json::to_string_pretty(predictions)?;
        println!("{}", json_output);
    } else {
        // Human output
        println!("┌─────────────────────────────────────────────────────────");
        println!("│ PREDICTIVE INTELLIGENCE REPORT");
        println!("├─────────────────────────────────────────────────────────");
        println!("│ Total Predictions: {}", predictions.len());
        println!("│ Analysis Window:   30 days");
        println!("│ Patterns Analyzed: {}", patterns.len());
        println!("└─────────────────────────────────────────────────────────");
        println!();

        if predictions.is_empty() {
            println!("No predictions available. System appears stable.");
            return Ok(());
        }

        // Group by priority
        for priority in [Priority::Critical, Priority::High, Priority::Medium, Priority::Low] {
            let filtered: Vec<_> = predictions
                .iter()
                .filter(|p| p.priority == priority)
                .collect();

            if filtered.is_empty() {
                continue;
            }

            println!("{} {:?} Priority ({} predictions)", priority.emoji(), priority, filtered.len());
            println!("───────────────────────────────────────────────────────");

            for pred in filtered {
                println!();
                println!("  {} {}", format!("{:?}", pred.prediction_type), pred.title);
                println!("  Confidence: {}%", pred.confidence);

                if !pred.recommended_actions.is_empty() {
                    println!("  Actions:");
                    for action in &pred.recommended_actions {
                        println!("    • {}", action);
                    }
                }
            }

            println!();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throttle_logic() {
        // First call should allow display
        assert!(should_display_hints("status"));

        // Second call immediately after should block
        assert!(!should_display_hints("status"));

        // Different command should allow
        assert!(should_display_hints("health"));
    }
}

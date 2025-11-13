//! Learning and Prediction Commands
//! Phase 3.9: CLI completeness for predictive intelligence

use anyhow::Result;
use anna_common::context::{actions, db::ContextDb, DbLocation};
use anna_common::learning::{ActionSummary, Confidence, LearningEngine};
use anna_common::prediction::{PredictionEngine, Priority};
use std::collections::HashMap;

/// Execute 'learn' command - analyze action history for patterns
pub async fn execute_learn_command(
    json: bool,
    min_confidence: &str,
    days: i64,
) -> Result<()> {
    // Open context database
    let db = ContextDb::open(DbLocation::auto_detect()).await?;

    // Get recent action history
    let action_history = db
        .execute(|conn| actions::get_recent_actions(conn, 10000))
        .await?;

    if action_history.is_empty() {
        if json {
            println!("{{\"patterns\":[],\"message\":\"No action history available\"}}");
        } else {
            println!("📊 Learning Engine - Pattern Detection");
            println!("─────────────────────────────────────");
            println!();
            println!("No action history available yet.");
            println!("Run some commands to build history:");
            println!("  annactl status");
            println!("  annactl health");
            println!("  annactl update");
        }
        return Ok(());
    }

    // Aggregate into summaries
    let action_summaries = aggregate_actions(&action_history);

    // Parse confidence level
    let min_conf = parse_confidence(min_confidence)?;

    // Analyze patterns
    let mut learning_engine = LearningEngine::new()
        .with_analysis_window(days)
        .with_min_occurrences(2);

    learning_engine.detect_maintenance_patterns(&action_summaries);
    learning_engine.detect_failure_patterns(&action_summaries);
    learning_engine.detect_usage_patterns(&action_summaries);

    let patterns = learning_engine.get_patterns();

    // Filter by confidence
    let filtered_patterns: Vec<_> = patterns
        .iter()
        .filter(|p| p.confidence >= min_conf)
        .collect();

    if json {
        // JSON output
        let json_output = serde_json::json!({
            "total_actions": action_history.len(),
            "action_summaries": action_summaries.len(),
            "patterns_detected": patterns.len(),
            "patterns_filtered": filtered_patterns.len(),
            "min_confidence": format!("{:?}", min_conf),
            "analysis_window_days": days,
            "patterns": filtered_patterns,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human output
        print_learning_report(&action_summaries, &filtered_patterns, days, min_conf);
    }

    Ok(())
}

/// Execute 'predict' command - show predictive intelligence
pub async fn execute_predict_command(json: bool, show_all: bool) -> Result<()> {
    // Open context database
    let db = ContextDb::open(DbLocation::auto_detect()).await?;

    // Get recent action history
    let action_history = db
        .execute(|conn| actions::get_recent_actions(conn, 10000))
        .await?;

    if action_history.is_empty() {
        if json {
            println!("{{\"predictions\":[],\"message\":\"No action history available\"}}");
        } else {
            println!("🔮 Predictive Intelligence");
            println!("─────────────────────────────────────");
            println!();
            println!("No action history available yet.");
            println!("Run some commands to build history:");
            println!("  annactl status");
            println!("  annactl health");
        }
        return Ok(());
    }

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
        .with_min_confidence(if show_all { 50 } else { 65 })
        .with_throttle_hours(24);

    prediction_engine.generate_from_patterns(patterns);

    let mut predictions = prediction_engine.get_predictions().to_vec();

    // Filter by priority if not --all
    if !show_all {
        predictions.retain(|p| p.priority == Priority::High || p.priority == Priority::Critical);
    }

    if json {
        // JSON output
        println!("{}", serde_json::to_string_pretty(&predictions)?);
    } else {
        // Human output
        print_prediction_report(&predictions, show_all);
    }

    Ok(())
}

/// Aggregate ActionHistory into ActionSummary
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

        if action.timestamp > entry.last_execution {
            entry.last_execution = action.timestamp;
        }

        if let Some(duration) = action.duration_ms {
            entry.avg_duration_ms = (entry.avg_duration_ms + duration) / 2;
        }
    }

    summaries.into_values().collect()
}

/// Parse confidence level from string
fn parse_confidence(s: &str) -> Result<Confidence> {
    match s.to_lowercase().as_str() {
        "low" => Ok(Confidence::Low),
        "medium" => Ok(Confidence::Medium),
        "high" => Ok(Confidence::High),
        "very-high" | "veryhigh" => Ok(Confidence::VeryHigh),
        _ => anyhow::bail!("Invalid confidence level: {}. Use: low, medium, high, very-high", s),
    }
}

/// Print human-readable learning report
fn print_learning_report(
    summaries: &[ActionSummary],
    patterns: &[&anna_common::learning::DetectedPattern],
    days: i64,
    min_conf: Confidence,
) {
    println!("📊 Learning Engine - Pattern Detection");
    println!("═══════════════════════════════════════");
    println!();
    println!("Analysis Window: {} days", days);
    println!("Minimum Confidence: {:?} ({}%)", min_conf, min_conf.as_percentage());
    println!("Actions Analyzed: {}", summaries.len());
    println!("Patterns Detected: {}", patterns.len());
    println!();

    if patterns.is_empty() {
        println!("No patterns detected with {:?} confidence or higher.", min_conf);
        println!();
        println!("Try:");
        println!("  • Lower confidence: annactl learn --min-confidence low");
        println!("  • Longer window: annactl learn --days 60");
        return;
    }

    // Group by type
    use anna_common::learning::PatternType;
    for pattern_type in [
        PatternType::MaintenanceWindow,
        PatternType::RecurringFailure,
        PatternType::CommandUsage,
        PatternType::ResourceTrend,
    ] {
        let filtered: Vec<_> = patterns
            .iter()
            .filter(|p| p.pattern_type == pattern_type)
            .collect();

        if filtered.is_empty() {
            continue;
        }

        println!("{:?} Patterns ({})", pattern_type, filtered.len());
        println!("─────────────────────────────────────");

        for pattern in filtered {
            let confidence_emoji = match pattern.confidence {
                Confidence::VeryHigh => "🟢",
                Confidence::High => "🟡",
                Confidence::Medium => "🟠",
                Confidence::Low => "🔴",
            };

            println!(
                "  {} {} ({}% confidence, {} occurrences)",
                confidence_emoji,
                pattern.description,
                pattern.confidence.as_percentage(),
                pattern.occurrence_count
            );

            if !pattern.metadata.is_empty() {
                for (key, value) in &pattern.metadata {
                    if !value.is_empty() {
                        println!("     • {}: {}", key, value);
                    }
                }
            }
        }
        println!();
    }

    println!("Next Steps:");
    println!("  • View predictions: annactl predict");
    println!("  • JSON output: annactl learn --json");
}

/// Print human-readable prediction report
fn print_prediction_report(
    predictions: &[anna_common::prediction::Prediction],
    show_all: bool,
) {
    println!("🔮 Predictive Intelligence");
    println!("═══════════════════════════════════════");
    println!();

    if predictions.is_empty() {
        if show_all {
            println!("No predictions available.");
            println!();
            println!("This means:");
            println!("  • Not enough action history yet");
            println!("  • No patterns detected with sufficient confidence");
            println!();
            println!("Try running more commands to build history:");
            println!("  annactl status");
            println!("  annactl health");
            println!("  annactl update --dry-run");
        } else {
            println!("No high or critical priority predictions.");
            println!();
            println!("System appears stable! ✅");
            println!();
            println!("To see all predictions: annactl predict --all");
        }
        return;
    }

    println!("Total Predictions: {}", predictions.len());
    if !show_all {
        println!("Showing: High and Critical priority only");
        println!("Use --all to see all predictions");
    }
    println!();

    // Group by priority
    for priority in [Priority::Critical, Priority::High, Priority::Medium, Priority::Low] {
        let filtered: Vec<_> = predictions
            .iter()
            .filter(|p| p.priority == priority)
            .collect();

        if filtered.is_empty() {
            continue;
        }

        println!(
            "{} {:?} Priority ({} predictions)",
            priority.emoji(),
            priority,
            filtered.len()
        );
        println!("─────────────────────────────────────");

        for pred in filtered {
            println!("  {} {}", format!("{:?}", pred.prediction_type), pred.title);
            println!("     Confidence: {}%", pred.confidence);

            if !pred.recommended_actions.is_empty() {
                println!("     Actions:");
                for action in &pred.recommended_actions {
                    println!("       • {}", action);
                }
            }
        }
        println!();
    }

    println!("Next Steps:");
    println!("  • View patterns: annactl learn");
    println!("  • JSON output: annactl predict --json");
}

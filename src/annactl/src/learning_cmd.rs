//! Learning commands for Anna v0.14.0 "Orion III" Phase 2.2
//!
//! CLI interface for behavior learning and adaptive intelligence

use anyhow::Result;
use crate::learning::{LearningEngine, RuleWeight};

/// Learning command mode
pub enum LearningMode {
    Summary,
    Reset,
    Trend,
}

/// Run learning command
pub fn run_learning(mode: LearningMode, json: bool) -> Result<()> {
    let mut engine = LearningEngine::new()?;

    match mode {
        LearningMode::Summary => show_summary(&mut engine, json),
        LearningMode::Reset => reset_learning(&mut engine, json),
        LearningMode::Trend => show_trend(&engine, json),
    }
}

/// Show learning summary
fn show_summary(engine: &mut LearningEngine, json: bool) -> Result<()> {
    // Learn from audit log first
    let summary = engine.learn_from_audit()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    // Display summary
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let red = "\x1b[31m";
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";

    println!();
    println!("{}╭─ Behavior Learning Summary ─────────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Total Rules Tracked:{} {}", dim, reset, bold, reset, summary.total_rules);
    println!("{}│{}  {}New Interactions:{} {}", dim, reset, bold, reset, summary.new_interactions);
    println!("{}│{}  {}Rules Updated:{} {}", dim, reset, bold, reset, summary.rules_updated);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Trust Distribution{}", dim, reset, bold, reset);
    println!("{}│{}    {}High Confidence:{} {} {}{} rules{}",
        dim, reset, green, reset, summary.high_confidence, green, reset, reset);
    println!("{}│{}    {}Low Confidence:{} {} {}{} rules{}",
        dim, reset, yellow, reset, summary.low_confidence, yellow, reset, reset);
    println!("{}│{}    {}Untrusted:{} {} {}{} rules{}",
        dim, reset, red, reset, summary.untrusted, red, reset, reset);
    println!("{}│{}", dim, reset);

    // Show top learned patterns
    let weights = engine.get_all_weights();

    if !weights.is_empty() {
        println!("{}│{}  {}Top Learned Patterns{}", dim, reset, bold, reset);

        let mut sorted: Vec<_> = weights.values().collect();
        sorted.sort_by(|a, b| b.acceptance_rate().partial_cmp(&a.acceptance_rate()).unwrap());

        for (i, weight) in sorted.iter().take(5).enumerate() {
            let trust_emoji = match weight.trust_level() {
                "high" => "✅",
                "low" => "⚠️",
                "untrusted" => "❌",
                _ => "⚖️",
            };

            println!("{}│{}    {}. {} {:<30} {}Accept:{} {:.0}%  {}Auto:{} {:.0}%",
                dim, reset,
                i + 1,
                trust_emoji,
                truncate(&weight.rule_id, 30),
                bold, reset,
                weight.acceptance_rate() * 100.0,
                bold, reset,
                weight.auto_confidence * 100.0
            );
        }
        println!("{}│{}", dim, reset);
    }

    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
    println!();

    Ok(())
}

/// Reset learning data
fn reset_learning(engine: &mut LearningEngine, json: bool) -> Result<()> {
    if json {
        engine.reset()?;
        println!(r#"{{"status": "reset", "message": "All learned weights have been cleared"}}"#);
        return Ok(());
    }

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let yellow = "\x1b[33m";

    println!();
    println!("{}╭─ Reset Learning Data ────────────────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}⚠️  Warning:{} This will clear all learned behavioral weights.", dim, reset, yellow, reset);
    println!("{}│{}  Anna will start learning from scratch.", dim, reset);
    println!("{}│{}", dim, reset);

    // Confirm with user
    println!("{}│{}  Continue? [y/N]: ", dim, reset);
    print!("{}│{}  > {}", dim, reset, reset);

    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        println!("{}│{}", dim, reset);
        println!("{}│{}  Cancelled. No changes made.", dim, reset);
        println!("{}│{}", dim, reset);
        println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
        println!();
        return Ok(());
    }

    engine.reset()?;

    println!("{}│{}", dim, reset);
    println!("{}│{}  ✅ Learning data has been reset.", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
    println!();

    Ok(())
}

/// Show behavioral trend analysis
fn show_trend(engine: &LearningEngine, json: bool) -> Result<()> {
    let trend = engine.get_trend();

    if json {
        println!("{}", serde_json::to_string_pretty(&trend)?);
        return Ok(());
    }

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let red = "\x1b[31m";

    println!();
    println!("{}╭─ Behavioral Trend Analysis ──────────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Overall Metrics{}", dim, reset, bold, reset);
    println!("{}│{}    {}Trust Level:{} {}{}{:.0}%{}",
        dim, reset, bold, reset,
        trust_color(trend.overall_trust),
        reset,
        trend.overall_trust * 100.0,
        reset
    );
    println!("{}│{}    {}Acceptance Trend:{} {}{:.0}%{}",
        dim, reset, bold, reset,
        trend_color(trend.acceptance_trend),
        trend.acceptance_trend * 100.0,
        reset
    );
    println!("{}│{}    {}Automation Readiness:{} {}{:.0}%{}",
        dim, reset, bold, reset,
        trend_color(trend.automation_readiness),
        trend.automation_readiness * 100.0,
        reset
    );
    println!("{}│{}", dim, reset);

    // Top accepted rules
    if !trend.top_accepted.is_empty() {
        println!("{}│{}  {}Top Accepted Rules{}", dim, reset, bold, reset);
        for (i, rule_id) in trend.top_accepted.iter().take(5).enumerate() {
            println!("{}│{}    {}{}. {}{} {}",
                dim, reset, green, i + 1, reset, truncate(rule_id, 45), reset);
        }
        println!("{}│{}", dim, reset);
    }

    // Top ignored rules
    if !trend.top_ignored.is_empty() {
        println!("{}│{}  {}Top Ignored Rules{}", dim, reset, bold, reset);
        for (i, rule_id) in trend.top_ignored.iter().take(5).enumerate() {
            println!("{}│{}    {}{}. {}{} {}",
                dim, reset, yellow, i + 1, reset, truncate(rule_id, 45), reset);
        }
        println!("{}│{}", dim, reset);
    }

    // Untrusted rules
    if !trend.untrusted_rules.is_empty() {
        println!("{}│{}  {}⚠️  Untrusted Rules{}", dim, reset, red, reset);
        for (i, rule_id) in trend.untrusted_rules.iter().enumerate() {
            println!("{}│{}    {}{}. {}{} {}",
                dim, reset, red, i + 1, reset, truncate(rule_id, 45), reset);
        }
        println!("{}│{}    {}(High revert rate - Anna will not auto-run these){}", dim, reset, dim, reset);
        println!("{}│{}", dim, reset);
    }

    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
    println!();

    Ok(())
}

/// Get color for trust level
fn trust_color(trust: f32) -> &'static str {
    if trust > 0.7 {
        "\x1b[32m"  // Green
    } else if trust > 0.4 {
        "\x1b[33m"  // Yellow
    } else {
        "\x1b[31m"  // Red
    }
}

/// Get color for trend
fn trend_color(value: f32) -> &'static str {
    if value > 0.6 {
        "\x1b[32m"  // Green
    } else if value > 0.4 {
        "\x1b[33m"  // Yellow
    } else {
        "\x1b[31m"  // Red
    }
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

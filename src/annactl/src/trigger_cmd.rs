//! Trigger Command Interface for Anna v0.14.0 "Orion III" Phase 2.3
//!
//! CLI commands for managing threshold-based triggers

use anyhow::Result;
use clap::Parser;

use crate::trigger::{TriggerManager, MetricType};

/// Trigger command arguments
#[derive(Parser, Debug)]
pub struct TriggerArgs {
    /// Show trigger summary
    #[arg(long)]
    pub summary: bool,

    /// List all trigger thresholds
    #[arg(long)]
    pub list: bool,

    /// Show recent trigger events
    #[arg(long)]
    pub events: bool,

    /// Simulate trigger checks (dry-run)
    #[arg(long)]
    pub simulate: bool,

    /// Check triggers now and fire if conditions met
    #[arg(long)]
    pub check: bool,
}

/// Execute trigger command
pub fn execute(args: &TriggerArgs) -> Result<()> {
    let mut manager = TriggerManager::new()?;

    if args.summary {
        show_summary(&manager)?;
    } else if args.list {
        list_thresholds(&manager)?;
    } else if args.events {
        show_events(&manager)?;
    } else if args.simulate {
        simulate_triggers(&mut manager)?;
    } else if args.check {
        check_triggers(&mut manager)?;
    } else {
        // Default: show summary
        show_summary(&manager)?;
    }

    Ok(())
}

/// Show trigger summary
fn show_summary(manager: &TriggerManager) -> Result<()> {
    let summary = manager.get_summary()?;

    println!("╭─ Trigger Summary ────────────────────────────────────");
    println!("│");
    println!("│  Total Thresholds:    {}", summary.total_thresholds);
    println!("│  Enabled:             {}", summary.enabled_thresholds);
    println!("│  In Cooldown:         {}", summary.cooldown_count);
    println!("│");
    println!("│  Trigger History:");
    println!("│    Total Fired:       {}", summary.total_triggers);
    println!("│    Executed:          {}", summary.executed_count);
    println!("│");

    if !summary.recent_events.is_empty() {
        println!("│  Recent Events ({}):", summary.recent_events.len());
        for event in summary.recent_events.iter().take(5) {
            let timestamp = format_timestamp(event.timestamp);
            let status = if event.executed { "✅" } else { "⏳" };
            println!("│    {} {} - {}", status, timestamp, event.reason);
        }
    } else {
        println!("│  No recent trigger events");
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// List all trigger thresholds
fn list_thresholds(manager: &TriggerManager) -> Result<()> {
    let thresholds = manager.get_thresholds();

    println!("╭─ Trigger Thresholds ─────────────────────────────────");
    println!("│");

    for threshold in thresholds {
        let status = if threshold.enabled {
            if threshold.is_in_cooldown() {
                "⏸ "
            } else {
                "✅"
            }
        } else {
            "❌"
        };

        let metric_emoji = metric_type_emoji(&threshold.metric_type);

        println!("│  {} {} {}", status, metric_emoji, threshold.name);
        println!("│     ID: {}", threshold.id);
        println!("│     Description: {}", threshold.description);
        println!("│     Condition: {} {} {}",
            format_metric_type(&threshold.metric_type),
            threshold.condition.operator,
            threshold.condition.threshold);
        println!("│     Action: {}", threshold.action_id);
        println!("│     Cooldown: {} hours", threshold.cooldown_hours);

        if let Some(last) = threshold.last_triggered {
            let time_str = format_timestamp(last);
            println!("│     Last Triggered: {}", time_str);
        }

        println!("│");
    }

    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Show recent trigger events
fn show_events(manager: &TriggerManager) -> Result<()> {
    let events = manager.load_events()?;

    if events.is_empty() {
        println!("No trigger events recorded yet.");
        return Ok(());
    }

    println!("╭─ Trigger Events ─────────────────────────────────────");
    println!("│");
    println!("│  Total Events: {}", events.len());
    println!("│");

    for (i, event) in events.iter().rev().take(20).enumerate() {
        let timestamp = format_timestamp(event.timestamp);
        let status = if event.executed { "✅ Executed" } else { "⏳ Pending" };

        println!("│  {}. {} - {}", i + 1, timestamp, status);
        println!("│     Trigger: {}", event.trigger_id);
        println!("│     Reason: {}", event.reason);
        println!("│     Metric: {:.2} (threshold: {:.2})",
            event.metric_value, event.threshold);
        println!("│     Confidence: {:.1}%", event.confidence * 100.0);
        println!("│     Action: {}", event.action_id);
        println!("│");
    }

    if events.len() > 20 {
        println!("│  ... and {} older events", events.len() - 20);
        println!("│");
    }

    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Simulate trigger checks
fn simulate_triggers(manager: &mut TriggerManager) -> Result<()> {
    println!("╭─ Trigger Simulation ─────────────────────────────────");
    println!("│");
    println!("│  Running trigger checks (dry-run)...");
    println!("│");

    let events = manager.simulate()?;

    if events.is_empty() {
        println!("│  ✅ No triggers would fire");
        println!("│");
        println!("│     All metrics within thresholds");
    } else {
        println!("│  ⚠  {} trigger(s) would fire:", events.len());
        println!("│");

        for event in &events {
            println!("│  • {}", event.trigger_id);
            println!("│    Reason: {}", event.reason);
            println!("│    Value: {:.2} (threshold: {:.2})",
                event.metric_value, event.threshold);
            println!("│    Confidence: {:.1}%", event.confidence * 100.0);
            println!("│    Would execute: {}", event.action_id);
            println!("│");
        }
    }

    println!("│  Note: This was a simulation. No actions were executed.");
    println!("│");
    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Check triggers and fire if conditions met
fn check_triggers(manager: &mut TriggerManager) -> Result<()> {
    println!("╭─ Trigger Check ──────────────────────────────────────");
    println!("│");
    println!("│  Checking all trigger conditions...");
    println!("│");

    let events = manager.check_triggers()?;

    if events.is_empty() {
        println!("│  ✅ No triggers fired");
        println!("│");
        println!("│     All metrics within thresholds");
    } else {
        println!("│  🔥 {} trigger(s) fired:", events.len());
        println!("│");

        for event in &events {
            println!("│  • {}", event.trigger_id);
            println!("│    Reason: {}", event.reason);
            println!("│    Value: {:.2} (threshold: {:.2})",
                event.metric_value, event.threshold);
            println!("│    Confidence: {:.1}%", event.confidence * 100.0);
            println!("│    Action: {}", event.action_id);
            println!("│");

            // Log the event
            manager.log_event(&event)?;
        }

        println!("│  Events logged to trigger_events.jsonl");
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Format metric type for display
fn format_metric_type(metric_type: &MetricType) -> &str {
    match metric_type {
        MetricType::ForecastDeviation => "Forecast Deviation",
        MetricType::AnomalyCritical => "Critical Anomalies",
        MetricType::PerformanceDrift => "Performance Drift",
        MetricType::DiskSpaceLow => "Disk Space",
        MetricType::MemoryPressure => "Memory Pressure",
    }
}

/// Get emoji for metric type
fn metric_type_emoji(metric_type: &MetricType) -> &str {
    match metric_type {
        MetricType::ForecastDeviation => "📊",
        MetricType::AnomalyCritical => "⚠️",
        MetricType::PerformanceDrift => "🐌",
        MetricType::DiskSpaceLow => "💾",
        MetricType::MemoryPressure => "🧠",
    }
}

/// Format Unix timestamp to readable string
fn format_timestamp(timestamp: u64) -> String {
    // Simple formatting: show relative time if recent, otherwise date
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        format!("{} seconds ago", diff)
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

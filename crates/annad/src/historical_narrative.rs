//! Historical Narrative - Tell the story of system health over time.
//!
//! Philosophy: Context over time beats snapshots. Show trends, improvements, degradations.
//! NO HARDCODING: Intelligent storytelling based on data patterns.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

/// A narrative event in system history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub title: String,
    pub description: String,
    pub impact: Impact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    Improvement,
    Degradation,
    StabilityPeriod,
    MajorChange,
    Issue,
    Recovery,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Impact {
    Major,      // >30% change
    Moderate,   // 15-30% change
    Minor,      // 5-15% change
    Negligible, // <5% change
}

/// Generate system health narrative.
pub async fn generate_system_narrative(days: usize) -> Result<String> {
    info!("Generating {}-day system narrative...", days);

    let history = anna_shared::monitor::LongTermHistory::load();

    if history.daily_snapshots.len() < 7 {
        return Ok("Not enough historical data yet (need 7+ days).".to_string());
    }

    let mut narrative = format!("System Health Story (last {} days)\n\n", days);

    // 1. Overall stability assessment
    let stability = assess_overall_stability(&history, days);
    narrative.push_str(&stability);
    narrative.push_str("\n\n");

    // 2. Boot time story
    let boot_story = tell_boot_time_story(&history, days);
    if !boot_story.is_empty() {
        narrative.push_str("Boot Performance:\n");
        narrative.push_str(&boot_story);
        narrative.push_str("\n\n");
    }

    // 3. Memory usage story
    let memory_story = tell_memory_story(&history, days);
    if !memory_story.is_empty() {
        narrative.push_str("Memory Usage:\n");
        narrative.push_str(&memory_story);
        narrative.push_str("\n\n");
    }

    // 4. Disk usage story
    let disk_story = tell_disk_story(&history, days);
    if !disk_story.is_empty() {
        narrative.push_str("Storage:\n");
        narrative.push_str(&disk_story);
        narrative.push_str("\n\n");
    }

    // 5. Notable events
    let events = identify_notable_events(&history, days).await?;
    if !events.is_empty() {
        narrative.push_str("Notable Events:\n");
        for event in events.iter().take(5) {
            narrative.push_str(&format!(
                "• {} - {} ({})\n  {}\n\n",
                event.timestamp.format("%b %d"),
                event.title,
                format!("{:?}", event.impact),
                event.description
            ));
        }
    }

    // 6. Timeline summary
    narrative.push_str(&generate_timeline_summary(&history, days));

    Ok(narrative)
}

/// Assess overall system stability.
fn assess_overall_stability(history: &anna_shared::monitor::LongTermHistory, days: usize) -> String {
    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() < 2 {
        return "Insufficient data for stability assessment.".to_string();
    }

    // Calculate variance in key metrics
    let boot_values: Vec<f32> = snapshots.iter().map(|s| s.avg_boot_time).collect();
    let boot_variance = calculate_variance(&boot_values);

    let mem_values: Vec<f32> = snapshots.iter().map(|s| s.avg_memory_pct).collect();
    let mem_variance = calculate_variance(&mem_values);

    let load_values: Vec<f32> = snapshots.iter().map(|s| s.avg_load).collect();
    let load_variance = calculate_variance(&load_values);

    // Low variance = stable, high variance = unstable
    let avg_variance = (boot_variance + mem_variance + load_variance) / 3.0;

    let stability_desc = if avg_variance < 0.05 {
        "Very Stable"
    } else if avg_variance < 0.15 {
        "Stable"
    } else if avg_variance < 0.30 {
        "Moderately Stable"
    } else {
        "Unstable"
    };

    format!(
        "Overall: {} - Metrics show {} variance over {} days.",
        stability_desc,
        if avg_variance < 0.15 { "low" } else { "significant" },
        days
    )
}

/// Tell boot time story.
fn tell_boot_time_story(history: &anna_shared::monitor::LongTermHistory, days: usize) -> String {
    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() < 2 {
        return String::new();
    }

    let first_boot = snapshots.last().unwrap().avg_boot_time;
    let current_boot = snapshots.first().unwrap().avg_boot_time;
    let change_pct = ((current_boot - first_boot) / first_boot) * 100.0;

    let mut story = String::new();

    if change_pct.abs() > 10.0 {
        let direction = if change_pct > 0.0 { "slower" } else { "faster" };
        story.push_str(&format!(
            "Boot time is {:.0}% {} than {} days ago ({:.1}s → {:.1}s).",
            change_pct.abs(),
            direction,
            days,
            first_boot,
            current_boot
        ));

        if change_pct < -20.0 {
            story.push_str(" Significant improvement!");
        } else if change_pct > 20.0 {
            story.push_str(" Concerning degradation.");
        }
    } else {
        story.push_str(&format!("Boot time stable at ~{:.1}s.", current_boot));
    }

    story
}

/// Tell memory usage story.
fn tell_memory_story(history: &anna_shared::monitor::LongTermHistory, days: usize) -> String {
    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() < 2 {
        return String::new();
    }

    let first_mem = snapshots.last().unwrap().avg_memory_pct;
    let current_mem = snapshots.first().unwrap().avg_memory_pct;
    let change_pct = ((current_mem - first_mem) / first_mem) * 100.0;

    let mut story = String::new();

    if change_pct.abs() > 15.0 {
        let direction = if change_pct > 0.0 { "increased" } else { "decreased" };
        story.push_str(&format!(
            "Memory usage {} by {:.0}% over {} days ({:.1}% → {:.1}%).",
            direction,
            change_pct.abs(),
            days,
            first_mem,
            current_mem
        ));

        if change_pct > 25.0 {
            story.push_str(" Possible memory leak or new workload.");
        }
    } else {
        story.push_str(&format!("Memory usage stable around {:.1}%.", current_mem));
    }

    story
}

/// Tell disk usage story.
fn tell_disk_story(history: &anna_shared::monitor::LongTermHistory, days: usize) -> String {
    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() < 2 {
        return String::new();
    }

    let first_disk = snapshots.last().unwrap().disk_used_gb;
    let current_disk = snapshots.first().unwrap().disk_used_gb;
    let growth = current_disk - first_disk;

    let mut story = String::new();

    if growth.abs() > 1.0 {
        let direction = if growth > 0.0 { "grown" } else { "decreased" };
        story.push_str(&format!(
            "Disk usage has {} by {:.1}GB over {} days ({:.1}GB → {:.1}GB).",
            direction,
            growth.abs(),
            days,
            first_disk,
            current_disk
        ));

        if growth > 5.0 {
            story.push_str(" Rapid growth - consider cleanup.");
        }
    } else {
        story.push_str(&format!("Disk usage stable at {:.1}GB.", current_disk));
    }

    story
}

/// Identify notable events from history.
async fn identify_notable_events(
    history: &anna_shared::monitor::LongTermHistory,
    days: usize,
) -> Result<Vec<NarrativeEvent>> {
    let mut events = Vec::new();

    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() < 2 {
        return Ok(events);
    }

    // Look for boot time changes
    for i in 1..snapshots.len() {
        let prev_boot = snapshots[i - 1].avg_boot_time;
        let curr_boot = snapshots[i].avg_boot_time;
        let change_pct = ((curr_boot - prev_boot) / prev_boot) * 100.0;

        if change_pct.abs() > 25.0 {
            let event_type = if change_pct > 0.0 {
                EventType::Degradation
            } else {
                EventType::Improvement
            };

            events.push(NarrativeEvent {
                timestamp: chrono::DateTime::parse_from_str(&snapshots[i].date, "%Y-%m-%d")
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                event_type,
                title: format!("Boot time {} {:.0}%", if change_pct > 0.0 { "slower" } else { "faster" }, change_pct.abs()),
                description: format!("{:.1}s → {:.1}s", prev_boot, curr_boot),
                impact: if change_pct.abs() > 50.0 {
                    Impact::Major
                } else if change_pct.abs() > 30.0 {
                    Impact::Moderate
                } else {
                    Impact::Minor
                },
            });
        }
    }

    // Look for memory spikes
    for i in 1..snapshots.len() {
        let prev_mem = snapshots[i - 1].avg_memory_pct;
        let curr_mem = snapshots[i].avg_memory_pct;
        let change_pct = ((curr_mem - prev_mem) / prev_mem) * 100.0;

        if change_pct > 30.0 {
            events.push(NarrativeEvent {
                timestamp: chrono::DateTime::parse_from_str(&snapshots[i].date, "%Y-%m-%d")
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                event_type: EventType::Issue,
                title: format!("Memory spike: +{:.0}%", change_pct),
                description: format!("{:.1}% → {:.1}%", prev_mem, curr_mem),
                impact: if change_pct > 50.0 {
                    Impact::Major
                } else {
                    Impact::Moderate
                },
            });
        }
    }

    // Detect stability periods (7+ days with <10% variance)
    if snapshots.len() >= 7 {
        let last_7_boot: Vec<f32> = snapshots.iter().take(7).map(|s| s.avg_boot_time).collect();
        let boot_variance = calculate_variance(&last_7_boot);

        if boot_variance < 0.10 {
            events.push(NarrativeEvent {
                timestamp: Utc::now(),
                event_type: EventType::StabilityPeriod,
                title: "7-day stability period".to_string(),
                description: "System metrics have been stable for the past week.".to_string(),
                impact: Impact::Minor,
            });
        }
    }

    // Sort by timestamp (most recent first)
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(events)
}

/// Generate timeline summary.
fn generate_timeline_summary(history: &anna_shared::monitor::LongTermHistory, days: usize) -> String {
    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() < 2 {
        return String::new();
    }

    let first = snapshots.last().unwrap();
    let current = snapshots.first().unwrap();

    format!(
        "Timeline Summary:\n\
        {} days ago: Boot {:.1}s, Memory {:.1}%, Disk {:.1}GB\n\
        Today: Boot {:.1}s, Memory {:.1}%, Disk {:.1}GB",
        days,
        first.avg_boot_time,
        first.avg_memory_pct,
        first.disk_used_gb,
        current.avg_boot_time,
        current.avg_memory_pct,
        current.disk_used_gb
    )
}

/// Calculate variance of values.
fn calculate_variance(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;

    // Normalize by mean to get coefficient of variation
    if mean > 0.0 {
        (variance.sqrt() / mean).abs()
    } else {
        0.0
    }
}

/// Get narrative for briefing (shorter version).
pub async fn get_brief_narrative(days: usize) -> Result<String> {
    let history = anna_shared::monitor::LongTermHistory::load();

    if history.daily_snapshots.len() < 3 {
        return Ok(String::new());
    }

    let mut brief = String::new();

    // Just highlight if anything significant changed
    let snapshots: Vec<_> = history.daily_snapshots.iter().rev().take(days).collect();

    if snapshots.len() >= 2 {
        let first = snapshots.last().unwrap();
        let current = snapshots.first().unwrap();

        let boot_change_pct = ((current.avg_boot_time - first.avg_boot_time) / first.avg_boot_time) * 100.0;

        if boot_change_pct.abs() > 20.0 {
            brief.push_str(&format!(
                "Boot time {} {:.0}% over {} days. ",
                if boot_change_pct > 0.0 { "slower" } else { "faster" },
                boot_change_pct.abs(),
                days
            ));
        }

        let disk_growth = current.disk_used_gb - first.disk_used_gb;
        if disk_growth > 3.0 {
            brief.push_str(&format!("Disk usage grew {:.1}GB. ", disk_growth));
        }
    }

    Ok(brief)
}

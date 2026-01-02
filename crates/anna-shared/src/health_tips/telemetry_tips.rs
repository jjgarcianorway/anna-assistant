//! Telemetry-based health tips (v0.0.285).
//!
//! Generates tips from telemetry trends and anomalies, including
//! health scores, trend analysis, and anomaly detection.

use crate::idle_tips::{IdleTip, TipCategory};
use crate::roster::{person_for, Tier};
use crate::system_telemetry::TelemetryStore;
use crate::teams::Team;

/// v0.0.285: Generate tips from telemetry trends and anomalies
pub fn generate_telemetry_tips(telemetry: &TelemetryStore) -> Vec<IdleTip> {
    let mut tips = Vec::new();

    // Health score tip
    tips.extend(check_health_score(telemetry));

    // Trend-based tips
    if telemetry.trends.sample_count >= 10 {
        tips.extend(check_trends(telemetry));
    }

    // Anomaly-based tips
    tips.extend(check_anomalies(telemetry));

    tips
}

/// Check overall health score
fn check_health_score(telemetry: &TelemetryStore) -> Vec<IdleTip> {
    let mut tips = Vec::new();
    let score = telemetry.health_score();

    if score < 60 {
        let person = person_for(Team::Performance, Tier::Senior);
        tips.push(
            IdleTip::new(
                "telemetry-health-critical",
                TipCategory::Performance,
                format!(
                    "{} reviewing metrics. System health is {}% - that's below comfortable. \
                     Let's run some diagnostics.",
                    person.display_name, score
                ),
            )
            .with_action("Ask me: \"diagnose system health\"")
            .with_priority(92),
        );
    } else if score < 75 {
        let person = person_for(Team::Performance, Tier::Junior);
        tips.push(
            IdleTip::new(
                "telemetry-health-warning",
                TipCategory::Performance,
                format!(
                    "*checking dashboards* {} here. Health score is {}%. Not great, not terrible.",
                    person.display_name, score
                ),
            )
            .with_priority(65),
        );
    }

    tips
}

/// Check resource usage trends
fn check_trends(telemetry: &TelemetryStore) -> Vec<IdleTip> {
    let mut tips = Vec::new();

    // Disk trend
    if telemetry.trends.disk_trend > 5.0 {
        let person = person_for(Team::Storage, Tier::Senior);
        tips.push(
            IdleTip::new(
                "telemetry-disk-trend",
                TipCategory::Storage,
                format!(
                    "{} analyzing storage patterns. Disk usage trending up by {:.1}% \
                     over the tracking window. Might want to investigate.",
                    person.display_name, telemetry.trends.disk_trend
                ),
            )
            .with_action("Ask me: \"what's filling up my disk\"")
            .with_priority(75),
        );
    }

    // Memory trend
    if telemetry.trends.memory_trend > 15.0 {
        let person = person_for(Team::Performance, Tier::Senior);
        tips.push(
            IdleTip::new(
                "telemetry-memory-trend",
                TipCategory::Performance,
                format!(
                    "{} reviewing memory graphs. Usage trending up {:.1}% over time. \
                     Could indicate a memory leak.",
                    person.display_name, telemetry.trends.memory_trend
                ),
            )
            .with_action("Ask me: \"check for memory leaks\"")
            .with_priority(70),
        );
    }

    // CPU trend
    if telemetry.trends.cpu_trend > 20.0 {
        let person = person_for(Team::Performance, Tier::Junior);
        tips.push(
            IdleTip::new(
                "telemetry-cpu-trend",
                TipCategory::Performance,
                format!(
                    "*watching CPU graphs* {} here. Load increasing by {:.1}% trend. \
                     Something's getting busier.",
                    person.display_name, telemetry.trends.cpu_trend
                ),
            )
            .with_priority(60),
        );
    }

    tips
}

/// Check for anomalies
fn check_anomalies(telemetry: &TelemetryStore) -> Vec<IdleTip> {
    let mut tips = Vec::new();

    for anomaly in telemetry.recent_anomalies().iter().take(2) {
        use crate::system_telemetry::{AnomalyCategory, AnomalySeverity};

        let priority = match anomaly.severity {
            AnomalySeverity::Critical => 90,
            AnomalySeverity::Warning => 70,
            AnomalySeverity::Info => 50,
        };

        let (team, tier) = match anomaly.category {
            AnomalyCategory::HighCpu | AnomalyCategory::HighLoad => {
                (Team::Performance, Tier::Senior)
            }
            AnomalyCategory::HighMemory => (Team::Performance, Tier::Junior),
            AnomalyCategory::LowDisk => (Team::Storage, Tier::Senior),
            AnomalyCategory::ServiceDown => (Team::Services, Tier::Senior),
            AnomalyCategory::NetworkError => (Team::Network, Tier::Junior),
        };

        let person = person_for(team, tier);
        tips.push(
            IdleTip::new(
                format!("telemetry-anomaly-{:?}", anomaly.category),
                match anomaly.category {
                    AnomalyCategory::HighCpu
                    | AnomalyCategory::HighLoad
                    | AnomalyCategory::HighMemory => TipCategory::Performance,
                    AnomalyCategory::LowDisk => TipCategory::Storage,
                    AnomalyCategory::ServiceDown => TipCategory::Services,
                    AnomalyCategory::NetworkError => TipCategory::Network,
                },
                format!("{}: {}", person.display_name, anomaly.description),
            )
            .with_action("Ask me for more details")
            .with_priority(priority),
        );
    }

    tips
}

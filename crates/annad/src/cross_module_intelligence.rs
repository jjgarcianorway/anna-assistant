//! Cross-Module Intelligence - Connects insights across Anna's modules.
//!
//! Philosophy: Individual modules gather data, this module finds the connections.
//! NO HARDCODING: LLM synthesizes insights, this module just gathers facts.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// A synthesized insight from multiple modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModuleInsight {
    /// What we learned from connecting the dots
    pub insight: String,
    /// Which modules contributed
    pub sources: Vec<String>,
    /// Actionable recommendations
    pub recommendations: Vec<String>,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
    /// Priority
    pub priority: InsightPriority,
}

/// Insight priority.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InsightPriority {
    Critical,  // Needs immediate action
    High,      // Important, act soon
    Medium,    // Good to know, plan accordingly
    Low,       // Informational
}

/// Gather insights from all modules.
pub async fn synthesize_insights(question: Option<&str>) -> Result<Vec<CrossModuleInsight>> {
    info!("Synthesizing cross-module insights...");

    let mut insights = Vec::new();

    // 1. Pattern + Prediction + Cleanup correlation
    if let Some(insight) = correlate_pattern_prediction_cleanup().await? {
        insights.push(insight);
    }

    // 2. Regression + Failure Memory correlation
    if let Some(insight) = correlate_regression_failure_memory().await? {
        insights.push(insight);
    }

    // 3. Prediction + Cleanup correlation
    if let Some(insight) = correlate_prediction_cleanup().await? {
        insights.push(insight);
    }

    // 4. Pattern + Teaching correlation (if question provided)
    if let Some(q) = question {
        if let Some(insight) = correlate_pattern_teaching(q).await? {
            insights.push(insight);
        }
    }

    // 5. Regression + Cleanup correlation
    if let Some(insight) = correlate_regression_cleanup().await? {
        insights.push(insight);
    }

    // Sort by priority
    insights.sort_by(|a, b| a.priority.cmp(&b.priority));

    Ok(insights)
}

/// Correlate pattern learning + prediction + cleanup.
/// Example: "You check disk weekly, found 3GB cleanable, will be full in 8 days"
async fn correlate_pattern_prediction_cleanup() -> Result<Option<CrossModuleInsight>> {
    let pattern_db = crate::pattern_learning::PatternDatabase::load();

    // Check if user repeatedly asks about disk
    let disk_patterns: Vec<_> = pattern_db.patterns.values()
        .filter(|p| {
            let normalized = p.pattern.to_lowercase();
            normalized.contains("disk") && p.occurrences.len() >= 3
        })
        .collect();

    if disk_patterns.is_empty() {
        return Ok(None);
    }

    // Get predictive alerts about disk
    let health_forecast = crate::predictive_maintenance::generate_health_forecast().await?;
    let disk_prediction = health_forecast.predictions.iter()
        .find(|p| p.prediction.to_lowercase().contains("disk"));

    // Get cleanup opportunities
    let cleanup = crate::cleanup_detector::scan_for_cleanable_space().await?;

    if disk_prediction.is_none() && cleanup.total_cleanable_mb < 100.0 {
        return Ok(None); // No interesting correlation
    }

    let mut insight_text = String::new();
    let mut recommendations = Vec::new();
    let mut sources = vec!["Pattern Learning".to_string()];

    let pattern = disk_patterns.first().unwrap();
    insight_text.push_str(&format!(
        "You've asked about disk usage {} times{}. ",
        pattern.occurrences.len(),
        if let Some(day) = &pattern.day_pattern {
            format!(" (usually on {}s)", format!("{:?}", day))
        } else {
            String::new()
        }
    ));

    if let Some(pred) = disk_prediction {
        sources.push("Predictive Maintenance".to_string());
        insight_text.push_str(&format!(
            "Predictions show: {} ({} days until critical). ",
            pred.prediction,
            pred.days_until.unwrap_or(999.0).round()
        ));
    }

    if cleanup.total_cleanable_mb > 100.0 {
        sources.push("Cleanup Detector".to_string());
        insight_text.push_str(&format!(
            "Found {:.1}GB cleanable space. ",
            cleanup.total_cleanable_mb / 1024.0
        ));
        recommendations.push("Clean safe items to free space".to_string());
    }

    // Actionable recommendations based on pattern
    if pattern.day_pattern.is_some() {
        recommendations.push("Add disk status to morning briefing".to_string());
        recommendations.push("Schedule weekly auto-cleanup".to_string());
    }

    if disk_prediction.is_some() {
        recommendations.push("Act now to prevent disk full".to_string());
    }

    let priority = if disk_prediction.map(|p| p.days_until.unwrap_or(999.0)).unwrap_or(999.0) < 7.0 {
        InsightPriority::Critical
    } else if disk_prediction.is_some() {
        InsightPriority::High
    } else {
        InsightPriority::Medium
    };

    Ok(Some(CrossModuleInsight {
        insight: insight_text,
        sources,
        recommendations,
        confidence: 0.85,
        priority,
    }))
}

/// Correlate regression detection + failure memory.
/// Example: "Boot time regressed after update, similar issue fixed before"
async fn correlate_regression_failure_memory() -> Result<Option<CrossModuleInsight>> {
    let regressions = crate::regression_detector::detect_regressions().await?;

    if regressions.is_empty() {
        return Ok(None);
    }

    let failure_db = crate::failure_memory::FailureDatabase::load();

    for regression in &regressions {
        // Check if we've seen similar failures before
        let regression_text = format!("{} regression", regression.metric.to_lowercase());

        for (_, failure) in &failure_db.failures {
            if failure.description.to_lowercase().contains(&regression.metric.to_lowercase()) {
                if !failure.working_solutions.is_empty() {
                    let solution = &failure.working_solutions[0];

                    return Ok(Some(CrossModuleInsight {
                        insight: format!(
                            "{} regressed ({:.0}% worse). This is similar to {} previous occurrences where {} worked.",
                            regression.metric,
                            regression.change_pct,
                            failure.occurrences.len(),
                            solution.description
                        ),
                        sources: vec!["Regression Detector".to_string(), "Failure Memory".to_string()],
                        recommendations: vec![
                            format!("Apply known fix: {}", solution.description),
                            "Monitor to see if fix holds".to_string(),
                        ],
                        confidence: 0.75,
                        priority: match regression.severity {
                            crate::regression_detector::RegressionSeverity::Severe => InsightPriority::Critical,
                            crate::regression_detector::RegressionSeverity::Significant => InsightPriority::High,
                            _ => InsightPriority::Medium,
                        },
                    }));
                }
            }
        }
    }

    Ok(None)
}

/// Correlate prediction + cleanup.
/// Example: "Memory leak predicted, old logs taking up space"
async fn correlate_prediction_cleanup() -> Result<Option<CrossModuleInsight>> {
    let health_forecast = crate::predictive_maintenance::generate_health_forecast().await?;
    let cleanup = crate::cleanup_detector::scan_for_cleanable_space().await?;

    if health_forecast.predictions.is_empty() || cleanup.total_cleanable_mb < 500.0 {
        return Ok(None);
    }

    // Look for memory leak + cleanable logs/cache
    let memory_prediction = health_forecast.predictions.iter()
        .find(|p| p.prediction.to_lowercase().contains("memory"));

    if let Some(mem_pred) = memory_prediction {
        let log_cleanup: f64 = cleanup.items.iter()
            .filter(|i| i.description.to_lowercase().contains("log") ||
                       i.description.to_lowercase().contains("cache"))
            .map(|i| i.size_mb)
            .sum();

        if log_cleanup > 500.0 {
            return Ok(Some(CrossModuleInsight {
                insight: format!(
                    "{} Found {:.1}GB of logs/cache that may be contributing to memory pressure.",
                    mem_pred.prediction,
                    log_cleanup / 1024.0
                ),
                sources: vec!["Predictive Maintenance".to_string(), "Cleanup Detector".to_string()],
                recommendations: vec![
                    "Clean old logs to reduce memory pressure".to_string(),
                    "Investigate which process is actually leaking".to_string(),
                ],
                confidence: 0.70,
                priority: InsightPriority::High,
            }));
        }
    }

    Ok(None)
}

/// Correlate pattern learning + teaching mode.
/// Example: "You've asked about systemd 5 times, want a detailed explanation?"
async fn correlate_pattern_teaching(question: &str) -> Result<Option<CrossModuleInsight>> {
    if !crate::teaching_mode::is_teaching_request(question) {
        return Ok(None);
    }

    let topic = crate::teaching_mode::extract_topic(question);
    if topic.is_none() {
        return Ok(None);
    }

    let pattern_db = crate::pattern_learning::PatternDatabase::load();
    let kb = crate::teaching_mode::KnowledgeBase::load();

    let topic_str = topic.unwrap();
    let normalized_topic = topic_str.to_lowercase();

    // Check if user has asked about this topic before (pattern learning)
    let related_patterns: Vec<_> = pattern_db.patterns.values()
        .filter(|p| p.pattern.to_lowercase().contains(&normalized_topic))
        .collect();

    // Check if user already learned this (teaching mode)
    if let Some(learned) = kb.knows_topic(&topic_str) {
        if learned.mastery > 0.7 && !related_patterns.is_empty() {
            return Ok(Some(CrossModuleInsight {
                insight: format!(
                    "You've mastered '{}' ({:.0}% mastery) but keep asking about it ({} times). Maybe you need a quick reference or cheatsheet?",
                    topic_str,
                    learned.mastery * 100.0,
                    related_patterns.len()
                ),
                sources: vec!["Teaching Mode".to_string(), "Pattern Learning".to_string()],
                recommendations: vec![
                    "Create a cheatsheet for quick reference".to_string(),
                    "Add to morning briefing if used regularly".to_string(),
                ],
                confidence: 0.80,
                priority: InsightPriority::Low,
            }));
        }
    }

    Ok(None)
}

/// Correlate regression + cleanup.
/// Example: "Boot regressed, found old kernels to remove"
async fn correlate_regression_cleanup() -> Result<Option<CrossModuleInsight>> {
    let regressions = crate::regression_detector::detect_regressions().await?;
    let cleanup = crate::cleanup_detector::scan_for_cleanable_space().await?;

    if regressions.is_empty() || cleanup.items.is_empty() {
        return Ok(None);
    }

    // Look for boot regression + old kernels/services
    let boot_regression = regressions.iter()
        .find(|r| r.metric.to_lowercase().contains("boot"));

    if let Some(boot_reg) = boot_regression {
        let kernel_cleanup = cleanup.items.iter()
            .find(|i| i.description.to_lowercase().contains("kernel"));

        if let Some(kernels) = kernel_cleanup {
            return Ok(Some(CrossModuleInsight {
                insight: format!(
                    "Boot time regressed by {:.0}%. Found {} to remove ({:.1}GB). Old kernels can slow boot.",
                    boot_reg.change_pct,
                    kernels.description,
                    kernels.size_mb / 1024.0
                ),
                sources: vec!["Regression Detector".to_string(), "Cleanup Detector".to_string()],
                recommendations: vec![
                    "Remove old kernels to improve boot time".to_string(),
                    "Keep current + 1 backup kernel only".to_string(),
                ],
                confidence: 0.65,
                priority: InsightPriority::Medium,
            }));
        }
    }

    Ok(None)
}

/// Format insights for display.
pub fn format_insights(insights: &[CrossModuleInsight]) -> String {
    if insights.is_empty() {
        return "No cross-module insights at this time.".to_string();
    }

    let mut response = format!("Cross-Module Intelligence ({} insights):\n\n", insights.len());

    for (i, insight) in insights.iter().enumerate() {
        let priority_tag = match insight.priority {
            InsightPriority::Critical => "[CRITICAL]",
            InsightPriority::High => "[HIGH]",
            InsightPriority::Medium => "[MEDIUM]",
            InsightPriority::Low => "[INFO]",
        };

        response.push_str(&format!("{}. {} {}\n", i + 1, priority_tag, insight.insight));
        response.push_str(&format!("   Sources: {}\n", insight.sources.join(", ")));

        if !insight.recommendations.is_empty() {
            response.push_str("   Recommendations:\n");
            for rec in &insight.recommendations {
                response.push_str(&format!("   • {}\n", rec));
            }
        }
        response.push('\n');
    }

    response
}

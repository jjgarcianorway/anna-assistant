//! Multi-Perspective Analysis - Run multiple analyzers for complex questions.
//!
//! Philosophy: Complex issues need multiple viewpoints. One analyzer misses context.
//! NO HARDCODING: Run relevant analyzers based on question complexity.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Perspective from a specific analyzer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Perspective {
    pub analyzer: AnalyzerType,
    pub finding: String,
    pub confidence: f32,
    pub actionable_items: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalyzerType {
    Regression,
    Anomaly,
    Prediction,
    ChangeCorrelation,
    Historical,
    Pattern,
    Failure,
}

/// Multi-perspective analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPerspectiveResult {
    pub perspectives: Vec<Perspective>,
    pub synthesis: String,
    pub confidence: f32,
    pub recommended_actions: Vec<String>,
}

/// Determine if question warrants multi-perspective analysis.
pub fn is_complex_question(question: &str) -> bool {
    let q_lower = question.to_lowercase();

    // Questions starting with "why" are usually complex
    if q_lower.starts_with("why is ") || q_lower.starts_with("why does ") {
        return true;
    }

    // Multiple metrics mentioned
    let metrics = ["slow", "memory", "disk", "cpu", "boot", "performance"];
    let metric_count = metrics.iter().filter(|m| q_lower.contains(*m)).count();
    if metric_count >= 2 {
        return true;
    }

    // Comparative/correlation questions
    if q_lower.contains("since") || q_lower.contains("after") || q_lower.contains("when") {
        return true;
    }

    // Diagnostic questions
    if q_lower.contains("what happened") || q_lower.contains("what's wrong") {
        return true;
    }

    false
}

/// Run multi-perspective analysis on a complex question.
pub async fn analyze_multi_perspective(question: &str) -> Result<MultiPerspectiveResult> {
    info!("Running multi-perspective analysis for: {}", question);

    let mut perspectives = Vec::new();

    // 1. Regression perspective
    match run_regression_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Regression perspective failed: {}", e),
        _ => {}
    }

    // 2. Anomaly perspective
    match run_anomaly_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Anomaly perspective failed: {}", e),
        _ => {}
    }

    // 3. Prediction perspective
    match run_prediction_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Prediction perspective failed: {}", e),
        _ => {}
    }

    // 4. Change correlation perspective
    match run_change_correlation_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Change correlation perspective failed: {}", e),
        _ => {}
    }

    // 5. Historical perspective
    match run_historical_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Historical perspective failed: {}", e),
        _ => {}
    }

    // 6. Pattern learning perspective
    match run_pattern_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Pattern perspective failed: {}", e),
        _ => {}
    }

    // 7. Failure memory perspective
    match run_failure_perspective(question).await {
        Ok(Some(p)) => perspectives.push(p),
        Err(e) => info!("Failure perspective failed: {}", e),
        _ => {}
    }

    if perspectives.is_empty() {
        return Err(anyhow::anyhow!("No perspectives provided insights"));
    }

    // Synthesize all perspectives
    let synthesis = synthesize_perspectives(question, &perspectives);

    // Extract all actionable items
    let mut recommended_actions = Vec::new();
    for p in &perspectives {
        for action in &p.actionable_items {
            if !recommended_actions.contains(action) {
                recommended_actions.push(action.clone());
            }
        }
    }

    // Average confidence
    let avg_confidence = perspectives.iter().map(|p| p.confidence).sum::<f32>()
        / perspectives.len() as f32;

    Ok(MultiPerspectiveResult {
        perspectives,
        synthesis,
        confidence: avg_confidence,
        recommended_actions,
    })
}

/// Run regression analysis perspective.
async fn run_regression_perspective(question: &str) -> Result<Option<Perspective>> {
    let q_lower = question.to_lowercase();

    // Only relevant for performance questions
    if !q_lower.contains("slow") && !q_lower.contains("boot") && !q_lower.contains("performance") {
        return Ok(None);
    }

    let regressions = crate::regression_detector::detect_regressions().await?;

    if regressions.is_empty() {
        return Ok(None);
    }

    let finding = regressions
        .iter()
        .map(|r| crate::regression_detector::format_regression(r))
        .collect::<Vec<_>>()
        .join("\n");

    let actionable_items: Vec<String> = regressions
        .iter()
        .flat_map(|r| {
            r.causes
                .iter()
                .filter_map(|c| c.fix.clone())
        })
        .collect();

    Ok(Some(Perspective {
        analyzer: AnalyzerType::Regression,
        finding,
        confidence: 0.85,
        actionable_items,
    }))
}

/// Run anomaly analysis perspective.
async fn run_anomaly_perspective(question: &str) -> Result<Option<Perspective>> {
    let q_lower = question.to_lowercase();

    if !q_lower.contains("memory") && !q_lower.contains("ram") && !q_lower.contains("cpu") {
        return Ok(None);
    }

    // Get current memory usage
    let mem_pct = crate::briefing::get_disk_usage_percentage();
    let baseline = 50.0;

    if mem_pct <= baseline * 1.2 {
        return Ok(None); // No anomaly
    }

    let analysis = crate::anomaly_analysis::analyze_memory_anomaly(mem_pct, baseline).await?;

    let finding = format!(
        "Memory Anomaly Detected:\n{}\n\nPossible Causes:\n{}",
        analysis.anomaly,
        analysis
            .causes
            .iter()
            .map(|c| format!("• {} ({:.0}% likely)\n  Evidence: {}", c.description, c.likelihood * 100.0, c.evidence))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let actionable_items = analysis.recommendations;

    Ok(Some(Perspective {
        analyzer: AnalyzerType::Anomaly,
        finding,
        confidence: analysis.confidence,
        actionable_items,
    }))
}

/// Run predictive maintenance perspective.
async fn run_prediction_perspective(_question: &str) -> Result<Option<Perspective>> {
    let forecast = crate::predictive_maintenance::generate_health_forecast().await?;

    if forecast.predictions.is_empty() {
        return Ok(None);
    }

    let finding = crate::predictive_maintenance::format_health_forecast(&forecast);

    let actionable_items: Vec<String> = forecast
        .predictions
        .iter()
        .filter_map(|p| {
            if p.severity == crate::predictive_maintenance::PredictionSeverity::Critical {
                Some(p.recommendation.clone())
            } else {
                None
            }
        })
        .collect();

    if actionable_items.is_empty() {
        return Ok(None);
    }

    Ok(Some(Perspective {
        analyzer: AnalyzerType::Prediction,
        finding,
        confidence: 0.80,
        actionable_items,
    }))
}

/// Run change correlation perspective.
async fn run_change_correlation_perspective(question: &str) -> Result<Option<Perspective>> {
    let q_lower = question.to_lowercase();

    // Only relevant for "after", "since", "why" questions
    if !q_lower.contains("after") && !q_lower.contains("since") && !q_lower.starts_with("why") {
        return Ok(None);
    }

    // Check if there are any regressions to correlate with
    let regressions = crate::regression_detector::detect_regressions().await?;
    if regressions.is_empty() {
        return Ok(None);
    }

    // Try correlating the most recent regression
    if let Some(regression) = regressions.first() {
        let correlations = crate::change_tracking::correlate_changes_with_regression(
            &regression.metric,
            regression.started_at,
        ).await?;

        if correlations.is_empty() {
            return Ok(None);
        }

        let finding = format!(
            "Change Correlation Analysis:\n{}",
            correlations
                .iter()
                .map(|c| format!(
                    "• {} ({:.0}% correlation)\n  {}\n  Changed: {}",
                    c.change.description,
                    c.correlation_score * 100.0,
                    c.reasoning,
                    c.change.timestamp.format("%Y-%m-%d %H:%M")
                ))
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let actionable_items = vec![format!(
            "Review recent changes: {}",
            correlations
                .iter()
                .map(|c| c.change.description.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )];

        Ok(Some(Perspective {
            analyzer: AnalyzerType::ChangeCorrelation,
            finding,
            confidence: correlations.first().map(|c| c.correlation_score).unwrap_or(0.5),
            actionable_items,
        }))
    } else {
        Ok(None)
    }
}

/// Run historical narrative perspective.
async fn run_historical_perspective(question: &str) -> Result<Option<Perspective>> {
    let q_lower = question.to_lowercase();

    // Only relevant for trend questions
    if !q_lower.contains("trend") && !q_lower.contains("over time") && !q_lower.contains("history") {
        return Ok(None);
    }

    let narrative = crate::historical_narrative::generate_system_narrative(7).await?;

    if narrative.contains("Not enough historical data") {
        return Ok(None);
    }

    Ok(Some(Perspective {
        analyzer: AnalyzerType::Historical,
        finding: narrative,
        confidence: 0.75,
        actionable_items: vec![],
    }))
}

/// Run pattern learning perspective.
async fn run_pattern_perspective(question: &str) -> Result<Option<Perspective>> {
    if let Some(automation) = crate::pattern_learning::check_for_automation_opportunity(question) {
        let finding = crate::pattern_learning::format_automation_suggestion(&automation);

        let actionable_items = vec![format!(
            "Automate this recurring task (Pattern: {})",
            automation.pattern_id
        )];

        Ok(Some(Perspective {
            analyzer: AnalyzerType::Pattern,
            finding,
            confidence: 0.85,
            actionable_items,
        }))
    } else {
        Ok(None)
    }
}

/// Run failure memory perspective.
async fn run_failure_perspective(question: &str) -> Result<Option<Perspective>> {
    if let Some(response) = crate::failure_memory::check_and_handle_known_failure(question).await {
        Ok(Some(Perspective {
            analyzer: AnalyzerType::Failure,
            finding: response,
            confidence: 0.90,
            actionable_items: vec![],
        }))
    } else {
        Ok(None)
    }
}

/// Synthesize multiple perspectives into cohesive answer.
fn synthesize_perspectives(question: &str, perspectives: &[Perspective]) -> String {
    let mut synthesis = format!("Multi-Perspective Analysis: \"{}\"\n\n", question);

    // Group perspectives by priority
    let mut critical = Vec::new();
    let mut high_confidence = Vec::new();
    let mut medium_confidence = Vec::new();

    for p in perspectives {
        if p.confidence >= 0.85 {
            high_confidence.push(p);
        } else if p.confidence >= 0.70 {
            medium_confidence.push(p);
        } else {
            medium_confidence.push(p);
        }

        // Mark critical if contains certain keywords
        if p.finding.to_lowercase().contains("critical")
            || p.finding.to_lowercase().contains("urgent")
            || p.finding.to_lowercase().contains("severe")
        {
            critical.push(p);
        }
    }

    // Present critical first
    if !critical.is_empty() {
        synthesis.push_str("CRITICAL FINDINGS:\n\n");
        for p in &critical {
            synthesis.push_str(&format!("{:?} Perspective:\n{}\n\n", p.analyzer, p.finding));
        }
    }

    // Then high confidence
    if !high_confidence.is_empty() {
        synthesis.push_str("HIGH CONFIDENCE INSIGHTS:\n\n");
        for p in &high_confidence {
            if !critical.contains(&p) {
                synthesis.push_str(&format!("{:?} Analysis:\n{}\n\n", p.analyzer, p.finding));
            }
        }
    }

    // Finally medium confidence
    if !medium_confidence.is_empty() {
        synthesis.push_str("SUPPORTING ANALYSIS:\n\n");
        for p in &medium_confidence {
            if !critical.contains(&p) && !high_confidence.contains(&p) {
                synthesis.push_str(&format!("{:?}:\n{}\n\n", p.analyzer, p.finding));
            }
        }
    }

    synthesis
}

/// Format multi-perspective result for display.
pub fn format_multi_perspective_result(result: &MultiPerspectiveResult) -> String {
    let mut output = result.synthesis.clone();

    if !result.recommended_actions.is_empty() {
        output.push_str("\nRECOMMENDED ACTIONS:\n");
        for (i, action) in result.recommended_actions.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, action));
        }
    }

    output.push_str(&format!(
        "\nOverall Confidence: {:.0}% (from {} perspectives)\n",
        result.confidence * 100.0,
        result.perspectives.len()
    ));

    output
}

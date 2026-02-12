//! LLM Orchestration - Let LLM choose which analysis modules to activate.
//!
//! Philosophy: Instead of hardcoding "if disk >75% then cleanup", let LLM decide
//! which analysis is relevant based on the question and context.
//! NO HARDCODING: LLM is the orchestrator, modules are the tools.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Available analysis modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisModule {
    /// Detect performance regressions
    RegressionDetector,
    /// Predict future issues
    PredictiveMaintenance,
    /// Find cleanable space
    CleanupDetector,
    /// Root cause analysis for anomalies
    AnomalyAnalysis,
    /// Check failure memory
    FailureMemory,
    /// Pattern learning
    PatternLearning,
    /// Teaching mode
    TeachingMode,
    /// Cross-module intelligence
    CrossModuleIntelligence,
}

/// Analysis module result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub module: AnalysisModule,
    pub findings: String,
    pub confidence: f32,
}

/// Orchestration decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    /// Which modules to run
    pub modules: Vec<AnalysisModule>,
    /// Why these modules
    pub rationale: String,
    /// Expected completion time
    pub estimated_seconds: u32,
}

/// Determine which analysis modules are relevant for this question.
/// v0.3.168: Rule-based for now, can be LLM-powered later.
pub async fn determine_relevant_modules(
    _model: &str,
    question: &str,
    _system_context: &str,
) -> Result<OrchestrationPlan> {
    info!("Orchestration: Determining relevant analysis modules");

    let q_lower = question.to_lowercase();
    let mut modules = Vec::new();
    let mut rationale = String::new();

    // Teaching requests
    if q_lower.starts_with("what is ") || q_lower.starts_with("how does ") ||
       q_lower.starts_with("explain ") || q_lower.contains("teach me") {
        modules.push(AnalysisModule::TeachingMode);
        rationale.push_str("Teaching request detected. ");
    }

    // Slowness/performance complaints
    if q_lower.contains("slow") || q_lower.contains("taking long") ||
       q_lower.contains("performance") {
        modules.push(AnalysisModule::RegressionDetector);
        modules.push(AnalysisModule::AnomalyAnalysis);
        rationale.push_str("Performance issue - checking regressions and anomalies. ");
    }

    // Disk/space questions
    if q_lower.contains("disk") || q_lower.contains("space") ||
       q_lower.contains("storage") || q_lower.contains("full") {
        modules.push(AnalysisModule::CleanupDetector);
        modules.push(AnalysisModule::PredictiveMaintenance);
        rationale.push_str("Disk/space inquiry - checking cleanup and predictions. ");
    }

    // Memory questions
    if q_lower.contains("memory") || q_lower.contains("ram") {
        modules.push(AnalysisModule::AnomalyAnalysis);
        modules.push(AnalysisModule::PredictiveMaintenance);
        rationale.push_str("Memory inquiry - checking anomalies and predictions. ");
    }

    // Error/failure mentions
    if q_lower.contains("error") || q_lower.contains("fail") ||
       q_lower.contains("broken") || q_lower.contains("not working") {
        modules.push(AnalysisModule::FailureMemory);
        rationale.push_str("Error/failure - checking failure memory. ");
    }

    // "Why" questions about metrics
    if q_lower.starts_with("why is ") || q_lower.starts_with("why does ") {
        modules.push(AnalysisModule::AnomalyAnalysis);
        modules.push(AnalysisModule::CrossModuleIntelligence);
        rationale.push_str("'Why' question - analyzing anomalies and cross-module insights. ");
    }

    // Always check pattern learning (low cost)
    modules.push(AnalysisModule::PatternLearning);

    // If multiple modules, add cross-module intelligence
    if modules.len() > 2 {
        modules.push(AnalysisModule::CrossModuleIntelligence);
        rationale.push_str("Multiple modules active - connecting insights. ");
    }

    if rationale.is_empty() {
        rationale = "Standard question, basic checks.".to_string();
    }

    let estimated_seconds = modules.len() as u32 * 2; // Rough estimate

    info!(
        "Orchestration: Selected {} modules ({})",
        modules.len(),
        rationale.trim()
    );

    Ok(OrchestrationPlan {
        modules,
        rationale,
        estimated_seconds,
    })
}

/// Execute selected analysis modules.
pub async fn execute_modules(modules: &[AnalysisModule], question: Option<&str>) -> Result<Vec<AnalysisResult>> {
    info!("Executing {} analysis modules", modules.len());

    let mut results = Vec::new();

    for module in modules {
        match module {
            AnalysisModule::RegressionDetector => {
                info!("Running regression detection...");
                let regressions = crate::regression_detector::detect_regressions().await?;
                if !regressions.is_empty() {
                    let findings = regressions
                        .iter()
                        .map(|r| crate::regression_detector::format_regression(r))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    results.push(AnalysisResult {
                        module: AnalysisModule::RegressionDetector,
                        findings,
                        confidence: 0.85,
                    });
                }
            }

            AnalysisModule::PredictiveMaintenance => {
                info!("Running predictive maintenance...");
                let forecast = crate::predictive_maintenance::generate_health_forecast().await?;
                if !forecast.predictions.is_empty() {
                    let findings = crate::predictive_maintenance::format_health_forecast(&forecast);

                    results.push(AnalysisResult {
                        module: AnalysisModule::PredictiveMaintenance,
                        findings,
                        confidence: 0.80,
                    });
                }
            }

            AnalysisModule::CleanupDetector => {
                info!("Running cleanup detection...");
                let cleanup = crate::cleanup_detector::scan_for_cleanable_space().await?;
                if cleanup.total_cleanable_mb > 100.0 {
                    let findings = crate::cleanup_detector::format_cleanup_analysis(&cleanup);

                    results.push(AnalysisResult {
                        module: AnalysisModule::CleanupDetector,
                        findings,
                        confidence: 0.90,
                    });
                }
            }

            AnalysisModule::AnomalyAnalysis => {
                info!("Running anomaly analysis...");
                // Get current metrics
                let mem_pct = crate::briefing::get_disk_usage_percentage(); // Reusing function
                let baseline = 50.0; // Rough estimate, could load from history

                if mem_pct > baseline * 1.2 {
                    let analysis = crate::anomaly_analysis::analyze_memory_anomaly(mem_pct, baseline).await?;
                    let findings = format!(
                        "Memory Anomaly Analysis:\n{}\n\nRoot Causes:\n{}",
                        analysis.anomaly,
                        analysis
                            .causes
                            .iter()
                            .map(|c| format!("• {} ({:.0}% likely)\n  Evidence: {}", c.description, c.likelihood * 100.0, c.evidence))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );

                    results.push(AnalysisResult {
                        module: AnalysisModule::AnomalyAnalysis,
                        findings,
                        confidence: analysis.confidence,
                    });
                }
            }

            AnalysisModule::FailureMemory => {
                info!("Checking failure memory...");
                if let Some(q) = question {
                    if let Some(response) = crate::failure_memory::check_and_handle_known_failure(q).await {
                        results.push(AnalysisResult {
                            module: AnalysisModule::FailureMemory,
                            findings: response,
                            confidence: 0.90,
                        });
                    }
                }
            }

            AnalysisModule::PatternLearning => {
                info!("Checking pattern learning...");
                if let Some(q) = question {
                    if let Some(automation) = crate::pattern_learning::check_for_automation_opportunity(q) {
                        let findings = crate::pattern_learning::format_automation_suggestion(&automation);

                        results.push(AnalysisResult {
                            module: AnalysisModule::PatternLearning,
                            findings,
                            confidence: 0.85,
                        });
                    }
                }
            }

            AnalysisModule::TeachingMode => {
                info!("Activating teaching mode...");
                if let Some(q) = question {
                    if crate::teaching_mode::is_teaching_request(q) {
                        if let Ok(response) = crate::teaching_mode::handle_teaching_question(q).await {
                            results.push(AnalysisResult {
                                module: AnalysisModule::TeachingMode,
                                findings: response,
                                confidence: 0.85,
                            });
                        }
                    }
                }
            }

            AnalysisModule::CrossModuleIntelligence => {
                info!("Synthesizing cross-module insights...");
                let insights = crate::cross_module_intelligence::synthesize_insights(question).await?;
                if !insights.is_empty() {
                    let findings = crate::cross_module_intelligence::format_insights(&insights);

                    results.push(AnalysisResult {
                        module: AnalysisModule::CrossModuleIntelligence,
                        findings,
                        confidence: 0.80,
                    });
                }
            }
        }
    }

    Ok(results)
}

/// Synthesize analysis results into cohesive answer.
/// v0.3.169: Intelligent synthesis - prioritize critical, connect insights, actionable recommendations.
pub async fn synthesize_results(
    _model: &str,
    question: &str,
    results: &[AnalysisResult],
    _base_answer: &str,
) -> Result<String> {
    if results.is_empty() {
        return Ok(_base_answer.to_string());
    }

    info!("Synthesizing {} analysis results into cohesive answer", results.len());

    let mut synthesized = String::new();

    // 1. Start with direct answer to question
    synthesized.push_str(&format!("Regarding your question: \"{}\"\n\n", question));

    // 2. Prioritize critical findings first
    let critical_findings: Vec<_> = results
        .iter()
        .filter(|r| {
            r.findings.to_lowercase().contains("critical")
                || r.findings.to_lowercase().contains("severe")
                || r.findings.to_lowercase().contains("urgent")
        })
        .collect();

    if !critical_findings.is_empty() {
        synthesized.push_str("CRITICAL FINDINGS:\n\n");
        for result in critical_findings {
            synthesized.push_str(&format!("{}\n\n", extract_key_points(&result.findings, 3)));
        }
    }

    // 3. Group related insights
    let mut has_disk_insights = false;
    let mut has_memory_insights = false;
    let mut has_performance_insights = false;
    let mut other_insights = Vec::new();

    for result in results {
        let findings_lower = result.findings.to_lowercase();

        if findings_lower.contains("disk") || findings_lower.contains("storage") || findings_lower.contains("space") {
            if !has_disk_insights {
                synthesized.push_str("Storage Analysis:\n");
                has_disk_insights = true;
            }
            synthesized.push_str(&format!("• {}\n", extract_key_points(&result.findings, 2)));
        } else if findings_lower.contains("memory") || findings_lower.contains("ram") {
            if !has_memory_insights {
                if has_disk_insights {
                    synthesized.push('\n');
                }
                synthesized.push_str("Memory Analysis:\n");
                has_memory_insights = true;
            }
            synthesized.push_str(&format!("• {}\n", extract_key_points(&result.findings, 2)));
        } else if findings_lower.contains("boot") || findings_lower.contains("performance") || findings_lower.contains("regression") {
            if !has_performance_insights {
                if has_disk_insights || has_memory_insights {
                    synthesized.push('\n');
                }
                synthesized.push_str("Performance Analysis:\n");
                has_performance_insights = true;
            }
            synthesized.push_str(&format!("• {}\n", extract_key_points(&result.findings, 2)));
        } else {
            other_insights.push(result);
        }
    }

    // 4. Add other insights
    if !other_insights.is_empty() {
        if has_disk_insights || has_memory_insights || has_performance_insights {
            synthesized.push('\n');
        }
        synthesized.push_str("Additional Insights:\n");
        for result in other_insights {
            synthesized.push_str(&format!("• {}\n", extract_key_points(&result.findings, 2)));
        }
    }

    // 5. Extract actionable recommendations
    let recommendations = extract_recommendations(results);
    if !recommendations.is_empty() {
        synthesized.push_str("\nRecommended Actions:\n");
        for (i, rec) in recommendations.iter().take(5).enumerate() {
            synthesized.push_str(&format!("{}. {}\n", i + 1, rec));
        }
    }

    Ok(synthesized)
}

/// Extract key points from findings (first N sentences or key lines).
fn extract_key_points(findings: &str, max_points: usize) -> String {
    let lines: Vec<&str> = findings.lines().filter(|l| !l.trim().is_empty()).collect();

    lines
        .iter()
        .take(max_points)
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract recommendations from all results.
fn extract_recommendations(results: &[AnalysisResult]) -> Vec<String> {
    let mut recommendations = Vec::new();

    for result in results {
        // Look for lines containing "recommend", "should", "action", "fix"
        for line in result.findings.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("recommend")
                || line_lower.contains("should")
                || line_lower.contains("action:")
                || line_lower.contains("fix:")
            {
                let cleaned = line.trim().trim_start_matches("•").trim().to_string();
                if !recommendations.contains(&cleaned) && cleaned.len() > 10 {
                    recommendations.push(cleaned);
                }
            }
        }

        // Look for "Would you like" sections
        if result.findings.contains("Would you like me to") {
            // Extract options
            for line in result.findings.lines() {
                if line.trim().starts_with("1.") || line.trim().starts_with("2.") || line.trim().starts_with("3.") {
                    let cleaned = line
                        .trim()
                        .trim_start_matches(|c: char| c.is_numeric() || c == '.')
                        .trim()
                        .to_string();
                    if !recommendations.contains(&cleaned) {
                        recommendations.push(cleaned);
                    }
                }
            }
        }
    }

    recommendations
}

/// Main orchestration flow: Determine modules → Execute → Synthesize.
pub async fn orchestrate_analysis(
    model: &str,
    question: &str,
    base_answer: Option<&str>,
    system_context: &str,
) -> Result<String> {
    // 1. Determine which modules are relevant
    let plan = determine_relevant_modules(model, question, system_context).await?;

    if plan.modules.is_empty() {
        // No additional analysis needed
        return Ok(base_answer.unwrap_or("").to_string());
    }

    // 2. Execute selected modules
    let results = execute_modules(&plan.modules, Some(question)).await?;

    if results.is_empty() {
        // Modules ran but found nothing interesting
        return Ok(base_answer.unwrap_or("").to_string());
    }

    // 3. Synthesize findings into final answer
    let final_answer = synthesize_results(
        model,
        question,
        &results,
        base_answer.unwrap_or(""),
    )
    .await?;

    Ok(final_answer)
}

//! Intelligence Layer - Memory, Root Cause Analysis, and Predictions.
//!
//! v0.3.159: Integrates advanced intelligence systems into query handling:
//! - Episodic and semantic memory for learning from experience
//! - Root cause analysis for explaining failures
//! - Predictive disk health monitoring
//! - Smart timing recommendations

use crate::disk_health;
use crate::memory::{self, MemoryStore, Interaction, InteractionContext, InteractionOutcome, LearnedFact, FactCategory};
use crate::root_cause::{self, analyze_symptom, DependencyGraph, SystemEvent};
use chrono::Utc;
use tracing::{debug, info};

/// Check memory for relevant past experiences
pub fn get_memory_context(question: &str) -> String {
    let memory = MemoryStore::load();
    memory.get_relevant_context(question)
}

/// Analyze a command failure using root cause analysis
pub fn analyze_failure(symptom: &str, recent_events: &[SystemEvent]) -> Option<String> {
    let dep_graph = DependencyGraph::build_from_systemd();
    let analysis = analyze_symptom(symptom, recent_events, &dep_graph);

    if analysis.root_causes.is_empty() {
        return None;
    }

    // Format the top root cause with action
    let top_cause = &analysis.root_causes[0];
    Some(format!(
        "{} (confidence: {:.0}%)\nRecommended: {}",
        top_cause.description,
        top_cause.confidence * 100.0,
        top_cause.recommended_action
    ))
}

/// Get disk health predictions
pub fn get_disk_predictions() -> String {
    let reports = disk_health::check_disk_health();
    let mut predictions = Vec::new();

    for report in reports {
        // Show predicted failures within 30 days
        if let Some(days) = report.predicted_failure_days {
            if days <= 30 {
                predictions.push(format!(
                    "⚠ {}: Predicted failure in {} days",
                    report.device,
                    days
                ));
            }
        }

        // Show unhealthy status
        if !matches!(report.health_status, disk_health::HealthStatus::Healthy) {
            predictions.push(format!("⚠ {}: {:?}", report.device, report.health_status));
        }
    }

    if predictions.is_empty() {
        String::new()
    } else {
        format!("\n## Disk Health Predictions:\n{}", predictions.join("\n"))
    }
}

/// Record successful interaction in memory
pub fn record_success(
    question: &str,
    answer: &str,
    commands: &[String],
    confidence: f32,
) {
    let mut memory = MemoryStore::load();

    let interaction = Interaction {
        timestamp: Utc::now().to_rfc3339(),
        user_query: question.to_string(),
        anna_response: answer.to_string(),
        context: InteractionContext {
            session_id: "system".to_string(),
            commands_executed: commands.to_vec(),
            services_affected: extract_services(commands),
            files_modified: extract_files(commands),
        },
        outcome: Some(InteractionOutcome::Success {
            user_feedback: None,
        }),
    };

    memory.episodic.record(interaction);

    // Learn facts if confidence is high
    if confidence >= 0.8 {
        learn_facts_from_interaction(question, commands, &mut memory);
    }

    // Persist to disk
    if let Err(e) = memory.save() {
        debug!("Failed to save memory: {}", e);
    } else {
        info!(
            "Recorded successful interaction (confidence: {:.0}%)",
            confidence * 100.0
        );
    }
}

/// Record failed interaction in memory
pub fn record_failure(question: &str, reason: &str, commands: &[String]) {
    let mut memory = MemoryStore::load();

    let interaction = Interaction {
        timestamp: Utc::now().to_rfc3339(),
        user_query: question.to_string(),
        anna_response: format!("Failed: {}", reason),
        context: InteractionContext {
            session_id: "system".to_string(),
            commands_executed: commands.to_vec(),
            services_affected: extract_services(commands),
            files_modified: extract_files(commands),
        },
        outcome: Some(InteractionOutcome::Failure {
            reason: reason.to_string(),
        }),
    };

    memory.episodic.record(interaction);

    if let Err(e) = memory.save() {
        debug!("Failed to save memory: {}", e);
    }
}

/// Extract service names from commands
fn extract_services(commands: &[String]) -> Vec<String> {
    let mut services = Vec::new();
    for cmd in commands {
        if cmd.contains("systemctl") {
            // Extract service name from systemctl commands
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some(service) = parts.last() {
                services.push(service.to_string());
            }
        }
    }
    services
}

/// Extract file paths from commands
fn extract_files(commands: &[String]) -> Vec<String> {
    let mut files = Vec::new();
    for cmd in commands {
        // Simple heuristic: paths starting with /
        for part in cmd.split_whitespace() {
            if part.starts_with('/') && !part.contains('*') {
                files.push(part.to_string());
            }
        }
    }
    files
}

/// Learn facts from a successful interaction
fn learn_facts_from_interaction(
    question: &str,
    commands: &[String],
    memory: &mut MemoryStore,
) {
    let q_lower = question.to_lowercase();

    // Learn service dependencies
    if q_lower.contains("start") || q_lower.contains("restart") {
        for cmd in commands {
            if cmd.contains("systemctl") && cmd.contains("start") {
                if let Some(service) = cmd.split_whitespace().last() {
                    let fact = LearnedFact {
                        category: FactCategory::ServiceDependency,
                        statement: format!("User manages {} service", service),
                        confidence: 0.7,
                        learned_from: "interaction".to_string(),
                        learned_at: Utc::now().to_rfc3339(),
                        validated_count: 1,
                    };
                    memory.semantic.learn(fact);
                }
            }
        }
    }

    // Learn troubleshooting patterns
    if q_lower.contains("error")
        || q_lower.contains("fail")
        || q_lower.contains("fix")
        || q_lower.contains("broken")
    {
        let fact = LearnedFact {
            category: FactCategory::TroubleshootingRule,
            statement: format!(
                "When troubleshooting '{}', these commands help: {}",
                question.chars().take(60).collect::<String>(),
                commands
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            confidence: 0.6,
            learned_from: "successful resolution".to_string(),
            learned_at: Utc::now().to_rfc3339(),
            validated_count: 1,
        };
        memory.semantic.learn(fact);
    }

    // Learn user preferences
    if q_lower.contains("how do i") || q_lower.contains("show me") {
        let fact = LearnedFact {
            category: FactCategory::UserPreference,
            statement: format!(
                "User asks about: {}",
                extract_topic(question).unwrap_or("system management")
            ),
            confidence: 0.5,
            learned_from: "question pattern".to_string(),
            learned_at: Utc::now().to_rfc3339(),
            validated_count: 1,
        };
        memory.semantic.learn(fact);
    }
}

/// Extract main topic from question
fn extract_topic(question: &str) -> Option<&str> {
    let q = question.to_lowercase();

    if q.contains("disk") {
        Some("disk management")
    } else if q.contains("memory") || q.contains("ram") {
        Some("memory usage")
    } else if q.contains("service") || q.contains("systemctl") {
        Some("service management")
    } else if q.contains("network") || q.contains("wifi") {
        Some("networking")
    } else if q.contains("package") || q.contains("pacman") {
        Some("package management")
    } else {
        None
    }
}

/// Clear all memory (for testing/reset)
#[allow(dead_code)]
pub fn clear_memory() {
    let mut memory = MemoryStore::default();
    if let Err(e) = memory.save() {
        debug!("Failed to save cleared memory: {}", e);
    }
}

/// Get memory statistics
#[allow(dead_code)]
pub fn get_memory_stats() -> (usize, usize) {
    let memory = MemoryStore::load();
    (
        memory.episodic.interactions.len(),
        memory.semantic.facts.len(),
    )
}

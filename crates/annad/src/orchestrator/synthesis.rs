//! Result synthesis - Combines results from multiple agents.

use anna_shared::agent::{AgentResult, AgentTask, Evidence};

/// Synthesize results from multiple agents into a single coherent answer.
pub fn synthesize_results(task: &AgentTask, results: Vec<AgentResult>) -> AgentResult {
    if results.is_empty() {
        return AgentResult::failure(&task.id, "orchestrator", "No results to synthesize");
    }

    // Single result: return as-is
    if results.len() == 1 {
        return results.into_iter().next().unwrap();
    }

    // Multiple results: combine them
    let mut combined_answer = String::new();
    let mut combined_evidence: Vec<Evidence> = Vec::new();
    let mut total_confidence = 0.0;
    let mut success_count = 0;

    for result in &results {
        if result.success {
            success_count += 1;
            total_confidence += result.confidence;

            if let Some(answer) = &result.answer {
                if !combined_answer.is_empty() {
                    combined_answer.push_str("\n\n");
                }
                combined_answer.push_str(&format!("[{}]\n{}", result.agent_id, answer));
            }

            combined_evidence.extend(result.evidence.clone());
        }
    }

    if success_count == 0 {
        // All agents failed - collect error messages
        let errors: Vec<String> = results
            .iter()
            .filter_map(|r| r.answer.clone())
            .collect();

        return AgentResult::failure(
            &task.id,
            "orchestrator",
            &format!("All agents failed: {}", errors.join("; ")),
        );
    }

    // Calculate combined confidence
    let avg_confidence = total_confidence / success_count as f32;

    AgentResult {
        task_id: task.id.clone(),
        agent_id: "orchestrator".to_string(),
        success: true,
        answer: Some(format_combined_answer(&combined_answer, results.len())),
        evidence: deduplicate_evidence(combined_evidence),
        confidence: avg_confidence,
        subtasks: vec![],
        learning: None,
    }
}

/// Format the combined answer nicely.
fn format_combined_answer(raw: &str, agent_count: usize) -> String {
    if agent_count <= 1 {
        return raw.to_string();
    }

    format!(
        "Based on analysis from {} specialists:\n\n{}",
        agent_count, raw
    )
}

/// Remove duplicate evidence entries.
fn deduplicate_evidence(evidence: Vec<Evidence>) -> Vec<Evidence> {
    let mut seen = std::collections::HashSet::new();
    evidence
        .into_iter()
        .filter(|e| {
            let key = format!("{:?}:{}", e.source, e.command.as_deref().unwrap_or(""));
            seen.insert(key)
        })
        .collect()
}

/// Merge learning data from multiple results.
pub fn merge_learning(results: &[AgentResult]) -> Option<anna_shared::agent::Learning> {
    let mut keywords = Vec::new();
    let mut probes = Vec::new();

    for result in results {
        if let Some(learning) = &result.learning {
            keywords.extend(learning.keywords.clone());
            probes.extend(learning.successful_probes.clone());
        }
    }

    if keywords.is_empty() && probes.is_empty() {
        return None;
    }

    // Deduplicate
    keywords.sort();
    keywords.dedup();
    probes.sort();
    probes.dedup();

    Some(anna_shared::agent::Learning {
        keywords,
        successful_probes: probes,
        answer_pattern: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_single_result() {
        let task = AgentTask::new("test question");
        let result = AgentResult::success(&task.id, "agent-1", "Test answer", 0.9);

        let synthesized = synthesize_results(&task, vec![result]);
        assert!(synthesized.success);
        assert_eq!(synthesized.confidence, 0.9);
    }

    #[test]
    fn test_synthesize_multiple_results() {
        let task = AgentTask::new("test question");
        let results = vec![
            AgentResult::success(&task.id, "agent-1", "Answer 1", 0.8),
            AgentResult::success(&task.id, "agent-2", "Answer 2", 0.9),
        ];

        let synthesized = synthesize_results(&task, results);
        assert!(synthesized.success);
        assert!(synthesized.answer.unwrap().contains("Answer 1"));
        assert_eq!(synthesized.confidence, 0.85); // Average
    }

    #[test]
    fn test_synthesize_all_failed() {
        let task = AgentTask::new("test question");
        let results = vec![
            AgentResult::failure(&task.id, "agent-1", "Error 1"),
            AgentResult::failure(&task.id, "agent-2", "Error 2"),
        ];

        let synthesized = synthesize_results(&task, results);
        assert!(!synthesized.success);
        assert!(synthesized.answer.unwrap().contains("All agents failed"));
    }
}

//! Memory recall - finding and suggesting relevant experiences.

use std::collections::HashMap;

use super::cluster::{calculate_cluster_similarity, calculate_relevance};
use super::types::{Experience, Memory};
use super::extract_keywords;

impl Memory {
    /// Find relevant experiences for a question
    pub fn recall(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        let mut scored: Vec<(&Experience, f32)> = self
            .experiences
            .iter()
            .filter_map(|exp| {
                let score = calculate_relevance(exp, &question_lower, &keywords);
                if score > 0.2 {
                    Some((exp, score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.usefulness_score.cmp(&a.0.usefulness_score))
        });

        scored.into_iter().take(limit).map(|(e, _)| e).collect()
    }

    /// Get suggested commands based on learned patterns
    pub fn suggest_commands(&self, question: &str) -> Vec<String> {
        let keywords = extract_keywords(question);
        let mut suggestions: Vec<(String, u32)> = Vec::new();

        for keyword in &keywords {
            for pattern in &self.patterns {
                if pattern
                    .keywords
                    .iter()
                    .any(|k| k == keyword || keyword.contains(k))
                {
                    for cmd in &pattern.common_commands {
                        if let Some((_, count)) =
                            suggestions.iter_mut().find(|(c, _)| c == &cmd.command)
                        {
                            *count += cmd.success_count;
                        } else {
                            suggestions.push((cmd.command.clone(), cmd.success_count));
                        }
                    }
                }
            }
        }

        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        suggestions.into_iter().map(|(c, _)| c).collect()
    }

    /// Get commands suggested by clusters (semantic recall)
    pub fn suggest_commands_from_clusters(&self, question: &str) -> Vec<String> {
        let mut suggestions: HashMap<String, u32> = HashMap::new();

        for cluster in &self.clusters {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.5 {
                for cmd in &cluster.effective_commands {
                    let weight = (sim * cmd.success_count as f32) as u32;
                    *suggestions.entry(cmd.command.clone()).or_insert(0) += weight.max(1);
                }
            }
        }

        let mut sorted: Vec<_> = suggestions.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().map(|(c, _)| c).take(5).collect()
    }

    /// Enhanced recall using clusters
    pub fn recall_with_clusters(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        let mut cluster_exp_ids: Vec<String> = Vec::new();
        for cluster in &self.clusters {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.5 {
                cluster_exp_ids.extend(cluster.experience_ids.clone());
            }
        }

        let mut scored: Vec<(&Experience, f32)> = self
            .experiences
            .iter()
            .filter_map(|exp| {
                let mut score = calculate_relevance(exp, &question_lower, &keywords);

                if cluster_exp_ids.contains(&exp.id) {
                    score += 0.2;
                }

                if score > 0.2 {
                    Some((exp, score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.usefulness_score.cmp(&a.0.usefulness_score))
        });

        scored.into_iter().take(limit).map(|(e, _)| e).collect()
    }
}

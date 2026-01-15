//! Learning Memory System - Anna learns from every interaction.
//!
//! This is NOT a hardcoded recipe system. Instead:
//! - Every successful Q&A is stored with semantic embeddings
//! - Similar questions retrieve relevant past experiences
//! - Patterns emerge organically from successful interactions
//!
//! v0.0.889: Added semantic question clustering
//! v0.0.941: Enhanced deduplication and memory compaction

mod cluster;
mod maintenance;
mod persistence;
mod recall;
mod types;

pub use cluster::{calculate_relevance, canonicalize_question};
pub use persistence::memory_path;
pub use types::{
    ClusterCommand, CommandPattern, Experience, ExperienceContext, FailedCommand, LearnedPattern,
    Memory, MemoryLoadResult, MemoryStats, QuestionCluster,
};

use cluster::calculate_cluster_similarity;

impl Memory {
    /// Learn from a successful interaction
    /// v0.0.892: Added deduplication
    pub fn learn(
        &mut self,
        question: &str,
        commands: Vec<String>,
        answer: &str,
        context: ExperienceContext,
    ) {
        let keywords = extract_keywords(question);
        let canonical = canonicalize_question(question);

        // v0.0.892: Check for near-duplicate experience first
        if let Some(existing) = self.find_similar_experience(&canonical, &keywords) {
            let exp_id = existing.to_string();
            if let Some(exp) = self.experiences.iter_mut().find(|e| e.id == exp_id) {
                exp.usefulness_score += 1;
                exp.last_used = Some(chrono::Utc::now().to_rfc3339());

                for cmd in &commands {
                    if !exp.successful_commands.contains(cmd) {
                        exp.successful_commands.push(cmd.clone());
                    }
                }

                self.update_patterns(&keywords, &commands);

                if let Some(cluster) = self
                    .clusters
                    .iter_mut()
                    .find(|c| c.experience_ids.contains(&exp_id))
                {
                    for cmd in &commands {
                        if let Some(cc) = cluster
                            .effective_commands
                            .iter_mut()
                            .find(|c| &c.command == cmd)
                        {
                            cc.success_count += 1;
                        } else {
                            cluster.effective_commands.push(ClusterCommand {
                                command: cmd.clone(),
                                success_count: 1,
                            });
                        }
                    }
                }
                return;
            }
        }

        let cluster_id = self.find_or_create_cluster(question, &keywords);

        let experience_id = uuid::Uuid::new_v4().to_string();
        let experience = Experience {
            id: experience_id.clone(),
            question: question.to_lowercase(),
            keywords: keywords.clone(),
            successful_commands: commands.clone(),
            answer: answer.to_string(),
            context,
            usefulness_score: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
            embedding: None,
        };

        self.experiences.push(experience);
        self.stats.total_experiences += 1;

        // v0.0.930: Add to keyword index
        self.index_experience(&experience_id, &keywords);

        if let Some(cluster) = self.clusters.iter_mut().find(|c| c.id == cluster_id) {
            cluster.experience_ids.push(experience_id);
        }
        self.update_cluster_commands(&cluster_id, &commands);

        self.update_patterns(&keywords, &commands);
    }


    /// Update patterns based on new experience
    fn update_patterns(&mut self, keywords: &[String], commands: &[String]) {
        for keyword in keywords {
            if let Some(pattern) = self
                .patterns
                .iter_mut()
                .find(|p| p.keywords.contains(keyword))
            {
                for cmd in commands {
                    if let Some(cp) = pattern
                        .common_commands
                        .iter_mut()
                        .find(|c| &c.command == cmd)
                    {
                        cp.success_count += 1;
                    } else {
                        pattern.common_commands.push(CommandPattern {
                            command: cmd.clone(),
                            success_count: 1,
                            retrieves: None,
                        });
                    }
                }
                pattern.evidence_count += 1;
            } else if !keyword.is_empty() && keyword.len() > 2 {
                let pattern = LearnedPattern {
                    keywords: vec![keyword.clone()],
                    common_commands: commands
                        .iter()
                        .map(|c| CommandPattern {
                            command: c.clone(),
                            success_count: 1,
                            retrieves: None,
                        })
                        .collect(),
                    evidence_count: 1,
                };
                self.patterns.push(pattern);
                self.stats.total_patterns += 1;
            }
        }
    }

    /// Mark an experience as useful
    pub fn mark_useful(&mut self, experience_id: &str) {
        if let Some(exp) = self.experiences.iter_mut().find(|e| e.id == experience_id) {
            exp.usefulness_score += 1;
            exp.last_used = Some(chrono::Utc::now().to_rfc3339());
            self.stats.memory_hits += 1;
        }
    }

    /// Record a memory miss
    pub fn record_miss(&mut self) {
        self.stats.memory_misses += 1;
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Compact memory by removing low-value experiences
    pub fn compact(&mut self, max_experiences: usize) {
        if self.experiences.len() <= max_experiences {
            return;
        }

        self.experiences.sort_by(|a, b| {
            let usefulness_cmp = b.usefulness_score.cmp(&a.usefulness_score);
            if usefulness_cmp != std::cmp::Ordering::Equal {
                return usefulness_cmp;
            }
            b.created_at.cmp(&a.created_at)
        });

        self.experiences.truncate(max_experiences);
        self.stats.total_experiences = self.experiences.len() as u32;
    }


    /// Find a matching cluster or create a new one
    pub fn find_or_create_cluster(&mut self, question: &str, keywords: &[String]) -> String {
        let canonical = canonicalize_question(question);
        let q_lower = question.to_lowercase();

        let mut best_match: Option<(usize, f32)> = None;
        for (idx, cluster) in self.clusters.iter().enumerate() {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.6 {
                if best_match.is_none() || sim > best_match.unwrap().1 {
                    best_match = Some((idx, sim));
                }
            }
        }

        if let Some((idx, _)) = best_match {
            let cluster = &mut self.clusters[idx];
            if !cluster.variations.contains(&q_lower) {
                cluster.variations.push(q_lower);
            }
            for kw in keywords {
                if !cluster.keywords.contains(kw) {
                    cluster.keywords.push(kw.clone());
                }
            }
            cluster.id.clone()
        } else {
            let cluster_id = uuid::Uuid::new_v4().to_string();
            let cluster = QuestionCluster {
                id: cluster_id.clone(),
                canonical,
                variations: vec![q_lower],
                keywords: keywords.to_vec(),
                experience_ids: Vec::new(),
                effective_commands: Vec::new(),
            };
            self.clusters.push(cluster);
            self.stats.total_clusters += 1;
            cluster_id
        }
    }

    /// Update cluster with successful commands
    pub fn update_cluster_commands(&mut self, cluster_id: &str, commands: &[String]) {
        if let Some(cluster) = self.clusters.iter_mut().find(|c| c.id == cluster_id) {
            for cmd in commands {
                if let Some(cc) = cluster
                    .effective_commands
                    .iter_mut()
                    .find(|c| &c.command == cmd)
                {
                    cc.success_count += 1;
                } else {
                    cluster.effective_commands.push(ClusterCommand {
                        command: cmd.clone(),
                        success_count: 1,
                    });
                }
            }
            cluster
                .effective_commands
                .sort_by(|a, b| b.success_count.cmp(&a.success_count));
        }
    }

}

/// Extract keywords from a question
pub fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above", "below", "between",
        "under", "again", "further", "then", "once", "here", "there", "when", "where", "why",
        "how", "all", "each", "every", "both", "few", "more", "most", "other", "some", "such",
        "no", "nor", "not", "only", "own", "same", "so", "than", "too", "very", "just", "and",
        "but", "if", "or", "because", "until", "while", "what", "which", "who", "whom", "this",
        "that", "these", "those", "am", "i", "my", "me", "you", "your", "it", "its", "he", "she",
        "they", "we", "them", "his", "her", "their", "our", "much", "many", "any", "about", "get",
        "tell", "show",
    ];

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_experience(question: &str, commands: Vec<&str>, score: u32) -> Experience {
        Experience {
            id: uuid::Uuid::new_v4().to_string(),
            question: question.to_string(),
            keywords: extract_keywords(question),
            successful_commands: commands.iter().map(|s| s.to_string()).collect(),
            answer: "test answer".to_string(),
            context: ExperienceContext::default(),
            usefulness_score: score,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
            embedding: None,
        }
    }

    #[test]
    fn test_deduplicate_same_canonical() {
        let mut memory = Memory::default();
        // These should canonicalize to the same thing (exact match after lowercasing)
        memory.experiences.push(make_experience("check ram usage", vec!["free -h"], 1));
        memory.experiences.push(make_experience("check ram usage", vec!["free -h", "cat /proc/meminfo"], 2));
        memory.experiences.push(make_experience("Check RAM Usage", vec!["free"], 1));

        assert_eq!(memory.experiences.len(), 3);
        let removed = memory.deduplicate();

        // Should merge all three (same canonical form after lowercasing)
        assert_eq!(removed, 2, "Should have removed 2 duplicates");
        assert_eq!(memory.experiences.len(), 1, "Should have 1 experience after dedup");
    }

    #[test]
    fn test_deduplicate_preserves_commands() {
        let mut memory = Memory::default();
        // Same question (same canonical)
        memory.experiences.push(make_experience("check memory", vec!["free -h"], 1));
        memory.experiences.push(make_experience("check memory", vec!["cat /proc/meminfo"], 2));

        memory.deduplicate();

        // The surviving experience should have both commands
        assert_eq!(memory.experiences.len(), 1);
        let exp = &memory.experiences[0];
        assert!(exp.successful_commands.contains(&"free -h".to_string()));
        assert!(exp.successful_commands.contains(&"cat /proc/meminfo".to_string()));
    }

    #[test]
    fn test_deduplicate_sums_usefulness() {
        let mut memory = Memory::default();
        // Same question (same canonical)
        memory.experiences.push(make_experience("show kernel", vec!["uname -r"], 5));
        memory.experiences.push(make_experience("show kernel", vec!["uname -a"], 3));

        let initial_total: u32 = memory.experiences.iter().map(|e| e.usefulness_score).sum();
        memory.deduplicate();

        // Usefulness scores should be summed
        assert_eq!(memory.experiences.len(), 1);
        let final_total: u32 = memory.experiences.iter().map(|e| e.usefulness_score).sum();
        assert_eq!(final_total, initial_total, "Usefulness scores should be preserved");
    }

    #[test]
    fn test_optimize() {
        let mut memory = Memory::default();
        for i in 0..10 {
            memory.experiences.push(make_experience(
                &format!("question {}", i % 3), // Creates duplicates
                vec!["cmd"],
                1,
            ));
        }

        let (exp_removed, _) = memory.optimize(5);
        assert!(exp_removed > 0 || memory.experiences.len() <= 5);
    }
}

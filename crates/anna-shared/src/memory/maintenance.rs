//! Memory maintenance operations - deduplication and optimization.
//! v0.0.941: Aggressive deduplication and cluster consolidation.

use super::cluster::canonicalize_question;
use super::types::Memory;

impl Memory {
    /// v0.0.941: Aggressive deduplication - merges experiences with same canonical form
    /// Call periodically or after batch learning to reduce memory bloat
    pub fn deduplicate(&mut self) -> usize {
        use std::collections::HashMap;
        let initial_count = self.experiences.len();

        // Group experiences by canonical form
        let mut canonical_groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, exp) in self.experiences.iter().enumerate() {
            let canonical = canonicalize_question(&exp.question);
            canonical_groups.entry(canonical).or_default().push(idx);
        }

        // Find groups with duplicates
        let mut to_remove: Vec<usize> = Vec::new();
        for (_canonical, indices) in &canonical_groups {
            if indices.len() <= 1 {
                continue;
            }

            // Keep the one with highest usefulness score, merge others into it
            let mut sorted = indices.clone();
            sorted.sort_by(|a, b| {
                let exp_a = &self.experiences[*a];
                let exp_b = &self.experiences[*b];
                exp_b.usefulness_score.cmp(&exp_a.usefulness_score)
            });

            let keeper_idx = sorted[0];
            let merge_indices = &sorted[1..];

            // Merge commands and context from duplicates into keeper
            let mut merged_commands: Vec<String> =
                self.experiences[keeper_idx].successful_commands.clone();
            let mut merged_score = self.experiences[keeper_idx].usefulness_score;

            for &idx in merge_indices {
                let exp = &self.experiences[idx];
                merged_score += exp.usefulness_score;
                for cmd in &exp.successful_commands {
                    if !merged_commands.contains(cmd) {
                        merged_commands.push(cmd.clone());
                    }
                }
                to_remove.push(idx);
            }

            // Update keeper
            self.experiences[keeper_idx].successful_commands = merged_commands;
            self.experiences[keeper_idx].usefulness_score = merged_score;
        }

        // Remove duplicates (in reverse order to preserve indices)
        to_remove.sort();
        to_remove.reverse();
        for idx in to_remove {
            self.experiences.remove(idx);
        }

        // Update stats
        let removed = initial_count - self.experiences.len();
        self.stats.total_experiences = self.experiences.len() as u32;

        // Rebuild keyword index after deduplication
        if removed > 0 {
            self.rebuild_index();
        }

        removed
    }

    /// v0.0.941: Consolidate clusters with overlapping questions
    pub fn consolidate_clusters(&mut self) -> usize {
        let initial_count = self.clusters.len();
        if initial_count <= 1 {
            return 0;
        }

        // Build a map of canonical -> cluster indices
        let mut canonical_to_cluster: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        for (idx, cluster) in self.clusters.iter().enumerate() {
            canonical_to_cluster
                .entry(cluster.canonical.clone())
                .or_default()
                .push(idx);
        }

        let mut to_remove: Vec<usize> = Vec::new();
        for (_canonical, indices) in &canonical_to_cluster {
            if indices.len() <= 1 {
                continue;
            }

            // Merge all into the first one
            let keeper_idx = indices[0];
            for &merge_idx in &indices[1..] {
                let merge_cluster = self.clusters[merge_idx].clone();

                // Merge variations
                for var in merge_cluster.variations {
                    if !self.clusters[keeper_idx].variations.contains(&var) {
                        self.clusters[keeper_idx].variations.push(var);
                    }
                }

                // Merge keywords
                for kw in merge_cluster.keywords {
                    if !self.clusters[keeper_idx].keywords.contains(&kw) {
                        self.clusters[keeper_idx].keywords.push(kw);
                    }
                }

                // Merge experience IDs
                for exp_id in merge_cluster.experience_ids {
                    if !self.clusters[keeper_idx].experience_ids.contains(&exp_id) {
                        self.clusters[keeper_idx].experience_ids.push(exp_id);
                    }
                }

                // Merge effective commands
                for cmd in merge_cluster.effective_commands {
                    if let Some(existing) = self.clusters[keeper_idx]
                        .effective_commands
                        .iter_mut()
                        .find(|c| c.command == cmd.command)
                    {
                        existing.success_count += cmd.success_count;
                    } else {
                        self.clusters[keeper_idx].effective_commands.push(cmd);
                    }
                }

                to_remove.push(merge_idx);
            }
        }

        // Remove merged clusters (in reverse order)
        to_remove.sort();
        to_remove.reverse();
        for idx in to_remove {
            self.clusters.remove(idx);
        }

        let removed = initial_count - self.clusters.len();
        self.stats.total_clusters = self.clusters.len() as u32;
        removed
    }

    /// v0.0.892: Find a near-duplicate experience
    /// v0.0.930: Uses keyword index for faster lookup
    pub fn find_similar_experience(&self, canonical: &str, keywords: &[String]) -> Option<String> {
        const SIMILARITY_THRESHOLD: f32 = 0.85;

        // v0.0.930: Use index to get candidates first
        let candidate_ids = self.get_candidates_by_keywords(keywords);

        // Check candidates first (most likely matches)
        for exp_id in &candidate_ids {
            if let Some(exp) = self.experiences.iter().find(|e| &e.id == exp_id) {
                let exp_canonical = canonicalize_question(&exp.question);

                if exp_canonical == *canonical {
                    return Some(exp.id.clone());
                }

                if !keywords.is_empty() && !exp.keywords.is_empty() {
                    let matching = keywords.iter().filter(|k| exp.keywords.contains(k)).count();
                    let overlap = matching as f32 / keywords.len().max(exp.keywords.len()) as f32;
                    if overlap >= SIMILARITY_THRESHOLD {
                        return Some(exp.id.clone());
                    }
                }
            }
        }

        // Fallback: check canonical match in remaining experiences
        for exp in &self.experiences {
            if !candidate_ids.contains(&exp.id.as_str()) {
                let exp_canonical = canonicalize_question(&exp.question);
                if exp_canonical == *canonical {
                    return Some(exp.id.clone());
                }
            }
        }

        None
    }

    /// v0.0.941: Full memory optimization - dedup + cluster consolidation + compact
    /// Returns (experiences_removed, clusters_removed)
    pub fn optimize(&mut self, max_experiences: usize) -> (usize, usize) {
        let exp_removed = self.deduplicate();
        let clusters_removed = self.consolidate_clusters();
        self.compact(max_experiences);
        (exp_removed, clusters_removed)
    }
}

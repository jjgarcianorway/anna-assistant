//! Per-agent learning and memory system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::types::{AgentTask, AgentResult, Learning};

/// Per-agent memory for learning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMemory {
    /// Successful patterns learned from past tasks.
    pub successful_patterns: Vec<LearnedPattern>,
    /// Failed attempts to avoid repeating.
    pub failed_attempts: Vec<FailedAttempt>,
    /// Domain-specific facts learned.
    pub domain_knowledge: HashMap<String, DomainFact>,
    /// Probe effectiveness scores (command -> 0.0-1.0).
    pub probe_effectiveness: HashMap<String, f32>,
}

impl AgentMemory {
    /// Record a successful pattern.
    pub fn record_success(&mut self, pattern: LearnedPattern) {
        // Check for similar existing pattern
        for existing in &mut self.successful_patterns {
            if patterns_similar(existing, &pattern) {
                existing.success_count += 1;
                existing.last_used = chrono::Utc::now().to_rfc3339();
                return;
            }
        }

        // Add new pattern
        self.successful_patterns.push(pattern);

        // Keep only top 100 by success count
        if self.successful_patterns.len() > 100 {
            self.successful_patterns.sort_by(|a, b| b.success_count.cmp(&a.success_count));
            self.successful_patterns.truncate(100);
        }
    }

    /// Record a failed attempt.
    pub fn record_failure(&mut self, attempt: FailedAttempt) {
        self.failed_attempts.push(attempt);

        // Keep only last 50 failures
        if self.failed_attempts.len() > 50 {
            self.failed_attempts.remove(0);
        }
    }

    /// Update probe effectiveness (exponential moving average).
    pub fn update_probe_effectiveness(&mut self, command: &str, was_useful: bool) {
        let entry = self.probe_effectiveness
            .entry(command.to_string())
            .or_insert(0.5);
        // EMA with alpha=0.1
        *entry = *entry * 0.9 + if was_useful { 0.1 } else { 0.0 };
    }

    /// Get most effective probes for a domain.
    pub fn get_recommended_probes(&self, domain: &str, limit: usize) -> Vec<String> {
        let mut probes: Vec<_> = self.probe_effectiveness
            .iter()
            .filter(|(cmd, _)| is_probe_for_domain(cmd, domain))
            .collect();
        probes.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        probes.into_iter().take(limit).map(|(cmd, _)| cmd.clone()).collect()
    }

    /// Check if a question matches known patterns.
    pub fn find_matching_pattern(&self, question: &str) -> Option<&LearnedPattern> {
        let q_lower = question.to_lowercase();
        self.successful_patterns.iter().find(|p| {
            let keyword_matches = p.question_keywords
                .iter()
                .filter(|kw| q_lower.contains(&kw.to_lowercase()))
                .count();
            keyword_matches >= 2 || (p.question_keywords.len() == 1 && keyword_matches == 1)
        })
    }

    /// Learn from a task result.
    pub fn learn_from_result(&mut self, task: &AgentTask, result: &AgentResult) {
        if result.success && result.confidence > 0.7 {
            let pattern = LearnedPattern::from_task_result(task, result);
            self.record_success(pattern);
        } else if !result.success {
            let attempt = FailedAttempt {
                question_keywords: extract_keywords(&task.question),
                reason: result.answer.clone().unwrap_or_default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            self.record_failure(attempt);
        }

        // Update probe effectiveness from evidence
        for evidence in &result.evidence {
            if let Some(cmd) = &evidence.command {
                self.update_probe_effectiveness(cmd, evidence.confidence > 0.5);
            }
        }
    }
}

/// A learned pattern from successful task completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub question_keywords: Vec<String>,
    pub domain: String,
    pub successful_probes: Vec<String>,
    pub answer_template: Option<String>,
    pub success_count: u32,
    pub last_used: String,
}

impl LearnedPattern {
    pub fn from_task_result(task: &AgentTask, result: &AgentResult) -> Self {
        let probes: Vec<String> = result.evidence.iter()
            .filter_map(|e| e.command.clone())
            .collect();

        Self {
            question_keywords: extract_keywords(&task.question),
            domain: task.domains.first()
                .map(|d| d.as_str().to_string())
                .unwrap_or_else(|| "general".to_string()),
            successful_probes: probes,
            answer_template: result.learning.as_ref().and_then(|l| l.answer_pattern.clone()),
            success_count: 1,
            last_used: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// A failed attempt to avoid repeating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedAttempt {
    pub question_keywords: Vec<String>,
    pub reason: String,
    pub timestamp: String,
}

/// Domain-specific fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainFact {
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub source: String,
}

/// Storage for all agent memories.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMemoryStore {
    pub memories: HashMap<String, AgentMemory>,
}

impl AgentMemoryStore {
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/agent_memories.json")
    }

    /// Load from disk.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)
    }

    /// Get memory for an agent (creates if not exists).
    pub fn get_mut(&mut self, agent_id: &str) -> &mut AgentMemory {
        self.memories.entry(agent_id.to_string()).or_default()
    }

    /// Get memory for an agent (immutable).
    pub fn get(&self, agent_id: &str) -> Option<&AgentMemory> {
        self.memories.get(agent_id)
    }
}

// Helper functions

fn patterns_similar(a: &LearnedPattern, b: &LearnedPattern) -> bool {
    if a.domain != b.domain {
        return false;
    }
    let common_keywords: usize = a.question_keywords.iter()
        .filter(|kw| b.question_keywords.contains(kw))
        .count();
    let total = a.question_keywords.len().max(b.question_keywords.len());
    if total == 0 {
        return false;
    }
    (common_keywords as f32 / total as f32) > 0.6
}

fn is_probe_for_domain(command: &str, domain: &str) -> bool {
    let cmd_lower = command.to_lowercase();
    match domain {
        "network" => cmd_lower.contains("ip") || cmd_lower.contains("ping") || cmd_lower.contains("ss") || cmd_lower.contains("netstat"),
        "storage" => cmd_lower.contains("df") || cmd_lower.contains("lsblk") || cmd_lower.contains("mount"),
        "system" => cmd_lower.contains("systemctl") || cmd_lower.contains("journal") || cmd_lower.contains("uname"),
        "packages" => cmd_lower.contains("pacman") || cmd_lower.contains("yay"),
        "hardware" => cmd_lower.contains("lspci") || cmd_lower.contains("lsusb") || cmd_lower.contains("lscpu"),
        "audio" => cmd_lower.contains("pactl") || cmd_lower.contains("wpctl") || cmd_lower.contains("aplay"),
        _ => true,
    }
}

fn extract_keywords(question: &str) -> Vec<String> {
    let stopwords = ["what", "is", "my", "the", "a", "an", "how", "do", "i", "can", "why", "does", "to", "in", "on", "for", "with", "not", "be", "are", "it"];
    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|w| w.to_string())
        .collect()
}

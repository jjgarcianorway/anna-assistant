//! Answer Tiers (Parts D & E) - v0.0.439.
//!
//! Part D: Fix the "boot time slow" flow with tiered answers.
//! Part E: Clarification questions must be rare and precise.
//!
//! Answer tiers:
//! 1. Provide measured facts from probes.
//! 2. Identify top offenders / key data points.
//! 3. Only then ask specialist to interpret and propose actions.

use super::evidence_gate::{DirectAnswer, EvidenceStatus};
use super::intent_schema::CanonicalIntent;

/// Answer tier levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnswerTier {
    /// Tier 1: Raw facts from probes.
    Facts,
    /// Tier 2: Identified key items (top offenders, main issues).
    KeyItems,
    /// Tier 3: Specialist synthesis (interpretation, recommendations).
    Synthesis,
}

impl AnswerTier {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Facts => "facts",
            Self::KeyItems => "key_items",
            Self::Synthesis => "synthesis",
        }
    }
}

/// Tiered answer for a specific intent.
#[derive(Debug, Clone)]
pub struct TieredAnswer {
    /// The intent being answered.
    pub intent: CanonicalIntent,
    /// Tier 1: Facts from probes.
    pub facts: Option<String>,
    /// Tier 2: Key items identified.
    pub key_items: Option<Vec<String>>,
    /// Tier 3: Synthesis (if specialist was called).
    pub synthesis: Option<String>,
    /// Current tier achieved.
    pub current_tier: AnswerTier,
}

impl TieredAnswer {
    /// Create a new tiered answer.
    pub fn new(intent: CanonicalIntent) -> Self {
        Self {
            intent,
            facts: None,
            key_items: None,
            synthesis: None,
            current_tier: AnswerTier::Facts,
        }
    }

    /// Set tier 1 facts.
    pub fn with_facts(mut self, facts: &str) -> Self {
        self.facts = Some(facts.to_string());
        self.current_tier = AnswerTier::Facts;
        self
    }

    /// Set tier 2 key items.
    pub fn with_key_items(mut self, items: Vec<String>) -> Self {
        self.key_items = Some(items);
        self.current_tier = AnswerTier::KeyItems;
        self
    }

    /// Set tier 3 synthesis.
    pub fn with_synthesis(mut self, synthesis: &str) -> Self {
        self.synthesis = Some(synthesis.to_string());
        self.current_tier = AnswerTier::Synthesis;
        self
    }

    /// Build the final answer string.
    pub fn build(&self) -> String {
        let mut parts = Vec::new();

        if let Some(facts) = &self.facts {
            parts.push(facts.clone());
        }

        if let Some(items) = &self.key_items {
            if !items.is_empty() {
                parts.push(items.join("\n"));
            }
        }

        if let Some(synthesis) = &self.synthesis {
            parts.push(synthesis.clone());
        }

        parts.join("\n\n")
    }

    /// Check if we have enough for a complete answer.
    pub fn is_complete(&self) -> bool {
        // For most intents, facts alone are sufficient
        self.facts.is_some()
    }
}

/// Build tiered answer for boot_perf intent.
pub fn build_boot_perf_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::BootPerf);

    // Tier 1: Boot time facts from systemd-analyze
    if let Some(analyze) = evidence.get_output("systemd_analyze") {
        let boot_time = extract_boot_time_fact(analyze);
        answer = answer.with_facts(&boot_time);
    }

    // Tier 2: Top offenders from systemd-analyze blame
    if let Some(blame) = evidence.get_output("systemd_blame") {
        let top_offenders = extract_top_offenders(blame, 5);
        if !top_offenders.is_empty() {
            answer = answer.with_key_items(top_offenders);
        }
    }

    // Tier 3 would be specialist synthesis (not built here)

    answer
}

/// Build tiered answer for mem_status intent.
pub fn build_mem_status_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::MemStatus);

    // Tier 1: Memory facts from free -h
    if let Some(free_h) = evidence.get_output("free_h") {
        let mem_fact = extract_memory_fact(free_h);
        answer = answer.with_facts(&mem_fact);
    }

    // Tier 2: Top memory consumers (if we have ps data)
    if let Some(ps_mem) = evidence.get_output("ps_mem_top") {
        let top_procs = extract_top_mem_processes(ps_mem, 5);
        if !top_procs.is_empty() {
            answer = answer.with_key_items(top_procs);
        }
    }

    answer
}

/// Build tiered answer for disk_usage intent.
pub fn build_disk_usage_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::DiskUsage);

    // Tier 1: Disk facts from df -h
    if let Some(df_h) = evidence.get_output("df_h") {
        let disk_fact = extract_disk_fact(df_h);
        answer = answer.with_facts(&disk_fact);
    }

    // Tier 2: Top directories (if we have du data)
    if let Some(du) = evidence.get_output("du_top_dirs") {
        let top_dirs = extract_top_directories(du, 5);
        if !top_dirs.is_empty() {
            answer = answer.with_key_items(top_dirs);
        }
    }

    answer
}

/// Build tiered answer for cpu_load intent.
pub fn build_cpu_load_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::CpuLoad);

    // Tier 1: Load average from uptime
    if let Some(uptime) = evidence.get_output("uptime") {
        let load_fact = extract_load_fact(uptime);
        answer = answer.with_facts(&load_fact);
    }

    // Tier 2: Top CPU consumers
    if let Some(top_cpu) = evidence.get_output("top_cpu") {
        let top_procs = extract_top_cpu_processes(top_cpu, 5);
        if !top_procs.is_empty() {
            answer = answer.with_key_items(top_procs);
        }
    }

    answer
}

/// Build tiered answer for gpu_driver intent.
pub fn build_gpu_driver_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::GpuDriver);

    // Tier 1: GPU hardware from lspci
    if let Some(lspci) = evidence.get_output("lspci_gpu") {
        let gpu_fact = extract_gpu_fact(lspci);
        answer = answer.with_facts(&gpu_fact);
    }

    // Tier 2: Driver info
    let mut driver_items = Vec::new();
    if let Some(lspci_k) = evidence.get_output("lspci_k_gpu") {
        if let Some(driver) = extract_kernel_driver(lspci_k) {
            driver_items.push(format!("Kernel driver: {}", driver));
        }
    }
    if let Some(lsmod) = evidence.get_output("lsmod_gpu") {
        let modules = extract_gpu_modules(lsmod);
        if !modules.is_empty() {
            driver_items.push(format!("Loaded modules: {}", modules.join(", ")));
        }
    }
    if !driver_items.is_empty() {
        answer = answer.with_key_items(driver_items);
    }

    answer
}

// ========== Extraction helpers ==========

fn extract_boot_time_fact(analyze_output: &str) -> String {
    // Extract the summary line from systemd-analyze
    for line in analyze_output.lines() {
        if line.contains("Startup finished") {
            return line.to_string();
        }
    }
    format!("Boot analysis: {}", analyze_output.lines().next().unwrap_or("unavailable"))
}

fn extract_top_offenders(blame_output: &str, count: usize) -> Vec<String> {
    blame_output
        .lines()
        .filter(|l| !l.is_empty())
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

fn extract_memory_fact(free_output: &str) -> String {
    for line in free_output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts.get(1).unwrap_or(&"?");
                let used = parts.get(2).unwrap_or(&"?");
                let available = parts.get(6).unwrap_or(parts.get(3).unwrap_or(&"?"));
                return format!("Memory: {} total, {} used, {} available", total, used, available);
            }
        }
    }
    "Memory information unavailable".to_string()
}

fn extract_top_mem_processes(ps_output: &str, count: usize) -> Vec<String> {
    ps_output
        .lines()
        .skip(1) // Skip header
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

fn extract_disk_fact(df_output: &str) -> String {
    let mut facts = Vec::new();
    for line in df_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts.get(5).unwrap_or(&"?");
            let use_pct = parts.get(4).unwrap_or(&"?");
            let avail = parts.get(3).unwrap_or(&"?");
            if *mount == "/" || mount.starts_with("/home") || mount.starts_with("/boot") {
                facts.push(format!("{}: {} used, {} available", mount, use_pct, avail));
            }
        }
    }
    if facts.is_empty() {
        "Disk information unavailable".to_string()
    } else {
        facts.join("\n")
    }
}

fn extract_top_directories(du_output: &str, count: usize) -> Vec<String> {
    du_output
        .lines()
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

fn extract_load_fact(uptime_output: &str) -> String {
    if let Some(idx) = uptime_output.find("load average:") {
        return uptime_output[idx..].to_string();
    }
    format!("Load: {}", uptime_output.trim())
}

fn extract_top_cpu_processes(top_output: &str, count: usize) -> Vec<String> {
    top_output
        .lines()
        .skip(1) // Skip header
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

fn extract_gpu_fact(lspci_output: &str) -> String {
    let gpu_lines: Vec<&str> = lspci_output
        .lines()
        .filter(|l| l.contains("VGA") || l.contains("3D") || l.contains("Display"))
        .collect();
    if gpu_lines.is_empty() {
        "No GPU detected".to_string()
    } else {
        gpu_lines.join("\n")
    }
}

fn extract_kernel_driver(lspci_k_output: &str) -> Option<String> {
    for line in lspci_k_output.lines() {
        if line.contains("Kernel driver in use:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

fn extract_gpu_modules(lsmod_output: &str) -> Vec<String> {
    lsmod_output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .take(5)
        .collect()
}

// ========== Clarification Rules (Part E) ==========

/// Maximum length for clarifying questions.
pub const MAX_CLARIFICATION_LENGTH: usize = 120;

/// Clarification question builder.
#[derive(Debug, Clone)]
pub struct ClarificationBuilder {
    /// Possible clarifying questions by intent.
    questions: Vec<(CanonicalIntent, &'static str)>,
}

impl ClarificationBuilder {
    /// Create new builder with standard questions.
    pub fn new() -> Self {
        Self {
            questions: vec![
                (CanonicalIntent::BootPerf, "Do you mean boot time (startup) or wake-from-sleep resume time?"),
                (CanonicalIntent::MemStatus, "Do you want total RAM, available RAM, or memory usage by application?"),
                (CanonicalIntent::DiskUsage, "Which partition? Root (/), home (/home), or all?"),
                (CanonicalIntent::SvcStatus, "Which service do you want to check?"),
                (CanonicalIntent::NetHealth, "Do you mean WiFi, Ethernet, or DNS connectivity?"),
                (CanonicalIntent::AudioHealth, "Is the issue with playback, recording, or both?"),
            ],
        }
    }

    /// Get clarifying question for an intent.
    pub fn get_question(&self, intent: CanonicalIntent) -> Option<&'static str> {
        self.questions
            .iter()
            .find(|(i, _)| *i == intent)
            .map(|(_, q)| *q)
    }

    /// Build a clarifying question (max 120 chars).
    pub fn build_question(question: &str) -> String {
        if question.len() <= MAX_CLARIFICATION_LENGTH {
            question.to_string()
        } else {
            format!("{}...", &question[..MAX_CLARIFICATION_LENGTH - 3])
        }
    }

    /// Check if clarification is needed based on query ambiguity.
    pub fn needs_clarification(query: &str, intent: CanonicalIntent) -> bool {
        // Only clarify for genuinely ambiguous queries
        let ambiguous_patterns = [
            ("boot", vec!["slow", "time", "fast"]),
            ("memory", vec!["usage", "much"]),
            ("disk", vec!["full", "usage", "space"]),
        ];

        let query_lower = query.to_lowercase();

        // Check if query is too vague
        if query_lower.split_whitespace().count() <= 2 {
            return matches!(intent, CanonicalIntent::Unknown);
        }

        // Don't clarify if query is specific enough
        for (topic, keywords) in ambiguous_patterns.iter() {
            if query_lower.contains(topic) {
                let has_specific = keywords.iter().any(|k| query_lower.contains(k));
                if has_specific {
                    return false; // Specific enough
                }
            }
        }

        // Only clarify for truly unknown intents
        matches!(intent, CanonicalIntent::Unknown)
    }
}

impl Default for ClarificationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::deterministic_routing::evidence_gate::ProbeResult;

    fn make_evidence(probes: Vec<(&str, &str)>) -> EvidenceStatus {
        let mut map = HashMap::new();
        for (id, output) in probes {
            map.insert(id.to_string(), ProbeResult::success(id, output));
        }
        EvidenceStatus::from_probes(&map, &[])
    }

    #[test]
    fn test_boot_perf_tiers() {
        let evidence = make_evidence(vec![
            ("systemd_analyze", "Startup finished in 2.5s (kernel) + 5.2s (userspace) = 7.7s"),
            ("systemd_blame", "3.5s NetworkManager.service\n2.1s docker.service\n1.8s systemd-udevd.service"),
        ]);

        let answer = build_boot_perf_tiers(&evidence);
        assert!(answer.facts.is_some());
        assert!(answer.facts.as_ref().unwrap().contains("7.7s"));
        assert!(answer.key_items.is_some());
        assert_eq!(answer.key_items.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_mem_status_tiers() {
        let evidence = make_evidence(vec![
            ("free_h", "              total        used        free\nMem:           31Gi       8.2Gi        15Gi"),
        ]);

        let answer = build_mem_status_tiers(&evidence);
        assert!(answer.facts.is_some());
        assert!(answer.facts.as_ref().unwrap().contains("31Gi"));
    }

    #[test]
    fn test_clarification_max_length() {
        let long_question = "a".repeat(200);
        let truncated = ClarificationBuilder::build_question(&long_question);
        assert!(truncated.len() <= MAX_CLARIFICATION_LENGTH);
    }

    #[test]
    fn test_needs_clarification() {
        // Specific queries don't need clarification
        assert!(!ClarificationBuilder::needs_clarification("how much RAM is available", CanonicalIntent::MemStatus));

        // Unknown intents may need clarification
        assert!(ClarificationBuilder::needs_clarification("?", CanonicalIntent::Unknown));
    }
}

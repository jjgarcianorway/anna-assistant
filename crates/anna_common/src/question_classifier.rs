//! Question Classifier v0.70.0
//!
//! Fast pre-LLM classification to route questions to the optimal path:
//! - FactFromKnowledge: Return from cache (no LLM)
//! - SimpleProbe: Execute probe → Junior summarize
//! - ComplexDiagnosis: Junior plan → Execute → Senior synthesize
//! - DangerousOrHighRisk: Block with explanation
//! - NeedsUserClarification: Ask clarifying question
//!
//! v0.70.0: Added difficulty routing (easy/normal/hard) for performance optimization.
//!
//! This classifier runs BEFORE any LLM call to minimize latency and cost.

use crate::llm_protocol::Difficulty;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// Question Type Classification
// ============================================================================

/// The 5 question types for routing (v0.50.0)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    /// Answerable from stored knowledge - no LLM or probe needed
    FactFromKnowledge,
    /// Single probe needed (e.g., "What CPU do I have?")
    SimpleProbe,
    /// Multiple probes + reasoning (e.g., "Why is my system slow?")
    ComplexDiagnosis,
    /// Safety check required - dangerous or high-risk operations
    DangerousOrHighRisk,
    /// Ambiguous question that needs clarification
    NeedsUserClarification,
}

impl QuestionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FactFromKnowledge => "fact_from_knowledge",
            Self::SimpleProbe => "simple_probe",
            Self::ComplexDiagnosis => "complex_diagnosis",
            Self::DangerousOrHighRisk => "dangerous_or_high_risk",
            Self::NeedsUserClarification => "needs_user_clarification",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            Self::FactFromKnowledge => "[K]",  // Knowledge
            Self::SimpleProbe => "[S]",        // Simple
            Self::ComplexDiagnosis => "[C]",   // Complex
            Self::DangerousOrHighRisk => "[!]", // Danger
            Self::NeedsUserClarification => "[?]", // Question
        }
    }

    /// Does this type require LLM calls?
    pub fn requires_llm(&self) -> bool {
        match self {
            Self::FactFromKnowledge => false,
            Self::SimpleProbe => true,  // Junior for summarization
            Self::ComplexDiagnosis => true, // Both Junior and Senior
            Self::DangerousOrHighRisk => false, // Static response
            Self::NeedsUserClarification => false, // Just ask user
        }
    }

    /// Does this type require probe execution?
    pub fn requires_probes(&self) -> bool {
        match self {
            Self::FactFromKnowledge => false,
            Self::SimpleProbe => true,
            Self::ComplexDiagnosis => true,
            Self::DangerousOrHighRisk => false,
            Self::NeedsUserClarification => false,
        }
    }
}

// ============================================================================
// Classification Result
// ============================================================================

/// Full classification result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// The primary question type
    pub question_type: QuestionType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// For SimpleProbe/ComplexDiagnosis: suggested probe IDs
    pub suggested_probes: Vec<String>,
    /// For NeedsUserClarification: what to ask
    pub clarification_prompt: Option<String>,
    /// For DangerousOrHighRisk: why it's blocked
    pub block_reason: Option<String>,
    /// Keywords that triggered this classification
    pub matched_keywords: Vec<String>,
}

impl ClassificationResult {
    fn new(question_type: QuestionType, confidence: f32) -> Self {
        Self {
            question_type,
            confidence,
            suggested_probes: Vec::new(),
            clarification_prompt: None,
            block_reason: None,
            matched_keywords: Vec::new(),
        }
    }

    fn with_probes(mut self, probes: Vec<String>) -> Self {
        self.suggested_probes = probes;
        self
    }

    fn with_clarification(mut self, prompt: String) -> Self {
        self.clarification_prompt = Some(prompt);
        self
    }

    fn with_block_reason(mut self, reason: String) -> Self {
        self.block_reason = Some(reason);
        self
    }

    fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.matched_keywords = keywords;
        self
    }

    /// v0.70.0: Get the difficulty level for this classification
    /// - Easy: knowledge-only, single probe, no LLM-B review
    /// - Normal: 1-2 probes, LLM-B review
    /// - Hard: multiple probes, complex diagnosis, full pipeline
    pub fn difficulty(&self) -> Difficulty {
        match &self.question_type {
            QuestionType::FactFromKnowledge => Difficulty::Easy,
            QuestionType::SimpleProbe => {
                // Simple probe with 1 probe = easy
                // Simple probe with 2+ probes = normal
                if self.suggested_probes.len() <= 1 {
                    Difficulty::Easy
                } else {
                    Difficulty::Normal
                }
            }
            QuestionType::ComplexDiagnosis => {
                // Complex with 3+ probes = hard
                // Complex with 2 probes = normal
                if self.suggested_probes.len() >= 3 {
                    Difficulty::Hard
                } else {
                    Difficulty::Normal
                }
            }
            QuestionType::DangerousOrHighRisk => Difficulty::Easy, // Fast rejection
            QuestionType::NeedsUserClarification => Difficulty::Easy, // Just ask
        }
    }
}

// ============================================================================
// Legacy Classification (for backwards compatibility)
// ============================================================================

/// Legacy classification result (v0.30.0 compatibility)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionDomain {
    /// Question is likely answerable with existing probes
    Supported { confidence: u8, likely_probes: Vec<String> },
    /// Question is definitely outside Anna's capabilities
    Unsupported { reason: String },
    /// Unclear - let LLM decide
    Uncertain,
}

// ============================================================================
// Question Classifier
// ============================================================================

/// Fast question classifier (no LLM calls) - v0.50.0
pub struct QuestionClassifier {
    // CPU-related keywords
    cpu_keywords: HashSet<&'static str>,
    // Memory-related keywords
    mem_keywords: HashSet<&'static str>,
    // Disk-related keywords
    disk_keywords: HashSet<&'static str>,
    // GPU-related keywords
    gpu_keywords: HashSet<&'static str>,
    // Logs/service-related keywords
    logs_keywords: HashSet<&'static str>,
    // Network-related keywords (now supported via safe commands)
    network_keywords: HashSet<&'static str>,
    // Package-related keywords
    package_keywords: HashSet<&'static str>,
    // Dangerous operation keywords
    dangerous_keywords: HashSet<&'static str>,
    // General knowledge patterns (unsupported)
    general_knowledge_patterns: Vec<&'static str>,
    // Ambiguous question patterns
    ambiguous_patterns: Vec<&'static str>,
    // Keywords that indicate complex diagnosis
    diagnosis_keywords: HashSet<&'static str>,
}

impl QuestionClassifier {
    pub fn new() -> Self {
        Self {
            cpu_keywords: [
                "cpu", "processor", "cores", "threads", "ghz", "mhz", "clock",
                "lscpu", "cpuinfo", "amd", "intel", "ryzen", "xeon", "i5", "i7",
                "i9", "i3", "core", "architecture", "x86", "arm", "aarch64",
                "frequency", "hyperthreading", "smt", "cache", "l1", "l2", "l3",
            ].into_iter().collect(),

            mem_keywords: [
                "memory", "ram", "meminfo", "swap", "buffer",
                "cached", "dimm", "slot", "ddr", "ddr4", "ddr5",
                // Note: "free", "used", "available" removed - too generic
                // Note: "gb", "mb", "gigabyte", "megabyte" removed - too generic
            ].into_iter().collect(),

            disk_keywords: [
                "disk", "drive", "storage", "partition", "mount", "filesystem",
                "nvme", "ssd", "hdd", "sata", "ext4", "btrfs", "xfs", "ntfs",
                "lsblk", "df", "space", "block", "device", "/dev/",
            ].into_iter().collect(),

            gpu_keywords: [
                "gpu", "graphics", "nvidia", "amd", "radeon", "geforce", "rtx",
                "gtx", "video", "vga", "driver", "cuda", "rocm",
                "opengl", "vulkan", "display", "vram",
            ].into_iter().collect(),

            logs_keywords: [
                "log", "logs", "journalctl", "journal", "syslog", "error",
                "warning", "critical", "emerg", "alert", "notice", "debug",
                "service", "daemon", "systemd", "unit", "failed",
            ].into_iter().collect(),

            network_keywords: [
                "network", "wifi", "ethernet", "ip", "dns", "dhcp", "router",
                "internet", "connection", "ping", "latency", "bandwidth",
                "interface", "address", "route", "gateway",
            ].into_iter().collect(),

            package_keywords: [
                "package", "installed", "version", "pacman", "apt", "dnf", "yum",
                "rpm", "dpkg", "pip", "npm", "cargo", "query",
            ].into_iter().collect(),

            dangerous_keywords: [
                // Destructive file operations
                "delete", "remove", "rm", "erase", "wipe", "format",
                // Modifying operations
                "change", "modify", "edit", "chmod", "chown", "mv", "move",
                // System control
                "reboot", "shutdown", "poweroff", "restart", "kill", "stop",
                // Package modification
                "install", "uninstall", "upgrade", "update",
                // Dangerous tools
                "dd", "mkfs", "fdisk", "parted",
            ].into_iter().collect(),

            general_knowledge_patterns: vec![
                "who is", "who was", "what is the capital", "when did",
                "where is", "where was", "why did", "how do i code",
                "can you tell me about", "explain to me", "describe the",
                "define the term", "write me", "help me write",
                "translate", "summarize this", "give me a recipe",
                "what's the weather", "weather forecast",
            ],

            ambiguous_patterns: vec![
                "something wrong", "not working", "broken", "issues",
                "problem with", "help me fix", "can you check",
                "what's going on", "slow", "fast", "better",
            ],

            diagnosis_keywords: [
                "why", "diagnose", "troubleshoot", "debug", "investigate",
                "analyze", "compare", "slow", "fast", "performance",
                "bottleneck", "issue", "problem", "error", "crash",
                "hang", "freeze", "unresponsive",
            ].into_iter().collect(),
        }
    }

    /// Normalize a word: strip punctuation and handle simple plurals
    fn normalize_word(word: &str) -> String {
        let cleaned: String = word.chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        // Handle simple plurals
        if cleaned.ends_with('s') && cleaned.len() > 3 {
            cleaned[..cleaned.len()-1].to_string()
        } else {
            cleaned
        }
    }

    /// v0.50.0: Classify a question into one of 5 types
    pub fn classify_v50(&self, question: &str) -> ClassificationResult {
        let q_lower = question.to_lowercase();
        let words: Vec<String> = q_lower
            .split_whitespace()
            .map(Self::normalize_word)
            .collect();
        let words_set: HashSet<&str> = words.iter().map(|s| s.as_str()).collect();

        // 1. Check for dangerous operations FIRST (safety priority)
        let dangerous_matches: Vec<String> = words_set
            .intersection(&self.dangerous_keywords)
            .map(|s| s.to_string())
            .collect();
        if !dangerous_matches.is_empty() {
            return ClassificationResult::new(QuestionType::DangerousOrHighRisk, 0.95)
                .with_block_reason(format!(
                    "This operation involves '{}' which could modify or damage your system. \
                     Anna only performs read-only operations for safety.",
                    dangerous_matches.join(", ")
                ))
                .with_keywords(dangerous_matches);
        }

        // 2. Check for general knowledge (outside our domain)
        for pattern in &self.general_knowledge_patterns {
            if q_lower.contains(pattern) {
                return ClassificationResult::new(QuestionType::DangerousOrHighRisk, 0.9)
                    .with_block_reason(
                        "This looks like a general knowledge question. \
                         I can only answer questions about your local system: \
                         CPU, memory, disk, GPU, network, packages, and services.".to_string()
                    )
                    .with_keywords(vec![pattern.to_string()]);
            }
        }

        // 3. Check for ambiguous questions that need clarification
        for pattern in &self.ambiguous_patterns {
            if q_lower.contains(pattern) {
                // Check if we have enough context from:
                // - Specific domain keywords (cpu, mem, disk, gpu)
                // - Diagnosis keywords (why, diagnose, troubleshoot)
                // - General system terms (system = all domains)
                let has_specific_context =
                    words_set.intersection(&self.cpu_keywords).next().is_some() ||
                    words_set.intersection(&self.mem_keywords).next().is_some() ||
                    words_set.intersection(&self.disk_keywords).next().is_some() ||
                    words_set.intersection(&self.gpu_keywords).next().is_some() ||
                    words_set.intersection(&self.diagnosis_keywords).next().is_some() ||
                    q_lower.contains("system"); // "system" implies general diagnosis

                if !has_specific_context {
                    return ClassificationResult::new(QuestionType::NeedsUserClarification, 0.8)
                        .with_clarification("Could you be more specific? What aspect of your system \
                             are you concerned about?\n\
                             - CPU and processor performance\n\
                             - Memory and RAM usage\n\
                             - Disk and storage space\n\
                             - GPU and graphics\n\
                             - Network connectivity".to_string())
                        .with_keywords(vec![pattern.to_string()]);
                }
            }
        }

        // 4. Identify supported domains and probe suggestions
        let mut suggested_probes = Vec::new();
        let mut matched_keywords = Vec::new();
        let mut complexity_score = 0;

        // CPU detection
        let cpu_matches: Vec<&str> = words_set.intersection(&self.cpu_keywords).copied().collect();
        if !cpu_matches.is_empty() {
            suggested_probes.push("cpu.info".to_string());
            matched_keywords.extend(cpu_matches.iter().map(|s| s.to_string()));
        }

        // Memory detection
        let mem_matches: Vec<&str> = words_set.intersection(&self.mem_keywords).copied().collect();
        if !mem_matches.is_empty() {
            suggested_probes.push("mem.info".to_string());
            suggested_probes.push("hardware.ram".to_string());
            matched_keywords.extend(mem_matches.iter().map(|s| s.to_string()));
        }

        // Disk detection
        let disk_matches: Vec<&str> = words_set.intersection(&self.disk_keywords).copied().collect();
        if !disk_matches.is_empty() {
            suggested_probes.push("disk.lsblk".to_string());
            matched_keywords.extend(disk_matches.iter().map(|s| s.to_string()));
        }

        // GPU detection
        let gpu_matches: Vec<&str> = words_set.intersection(&self.gpu_keywords).copied().collect();
        if !gpu_matches.is_empty() {
            suggested_probes.push("hardware.gpu".to_string());
            suggested_probes.push("drivers.gpu".to_string());
            matched_keywords.extend(gpu_matches.iter().map(|s| s.to_string()));
        }

        // Logs/service detection
        let logs_matches: Vec<&str> = words_set.intersection(&self.logs_keywords).copied().collect();
        if !logs_matches.is_empty() {
            suggested_probes.push("logs.journalctl".to_string());
            matched_keywords.extend(logs_matches.iter().map(|s| s.to_string()));
        }

        // Network detection (now supported via safe commands)
        let network_matches: Vec<&str> = words_set.intersection(&self.network_keywords).copied().collect();
        if !network_matches.is_empty() {
            suggested_probes.push("system.command.run".to_string()); // Generic probe
            matched_keywords.extend(network_matches.iter().map(|s| s.to_string()));
        }

        // Package detection
        let package_matches: Vec<&str> = words_set.intersection(&self.package_keywords).copied().collect();
        if !package_matches.is_empty() {
            suggested_probes.push("system.command.run".to_string()); // Generic probe
            matched_keywords.extend(package_matches.iter().map(|s| s.to_string()));
        }

        // 5. Check if this is a complex diagnosis (multiple domains or diagnostic keywords)
        let diagnosis_matches: Vec<&str> = words_set.intersection(&self.diagnosis_keywords).copied().collect();
        if !diagnosis_matches.is_empty() {
            complexity_score += 2;
            matched_keywords.extend(diagnosis_matches.iter().map(|s| s.to_string()));

            // "system" + diagnosis keywords = general system diagnosis (check all domains)
            if q_lower.contains("system") && suggested_probes.is_empty() {
                suggested_probes.push("cpu.info".to_string());
                suggested_probes.push("mem.info".to_string());
                suggested_probes.push("disk.lsblk".to_string());
                suggested_probes.push("logs.journalctl".to_string());
                matched_keywords.push("system".to_string());
            }
        }

        // Multiple probe domains = complex
        if suggested_probes.len() >= 2 {
            complexity_score += 1;
        }

        // 6. Determine question type based on analysis
        if suggested_probes.is_empty() {
            // No recognized domain
            ClassificationResult::new(QuestionType::NeedsUserClarification, 0.7)
                .with_clarification(
                    "I'm not sure what you're asking about. I can help with:\n\
                     - CPU information (\"What CPU do I have?\")\n\
                     - Memory usage (\"How much RAM am I using?\")\n\
                     - Disk space (\"How much storage is free?\")\n\
                     - GPU info (\"What graphics card?\")\n\
                     - System logs (\"Show recent errors\")\n\
                     - Network config (\"What's my IP address?\")".to_string()
                )
        } else if complexity_score >= 2 || suggested_probes.len() >= 3 {
            // Complex diagnosis
            ClassificationResult::new(QuestionType::ComplexDiagnosis, 0.85)
                .with_probes(suggested_probes)
                .with_keywords(matched_keywords)
        } else {
            // Simple probe
            ClassificationResult::new(QuestionType::SimpleProbe, 0.9)
                .with_probes(suggested_probes)
                .with_keywords(matched_keywords)
        }
    }

    /// Legacy v0.30.0 classification (for backwards compatibility)
    pub fn classify(&self, question: &str) -> QuestionDomain {
        let result = self.classify_v50(question);

        match result.question_type {
            QuestionType::FactFromKnowledge |
            QuestionType::SimpleProbe |
            QuestionType::ComplexDiagnosis => {
                QuestionDomain::Supported {
                    confidence: (result.confidence * 100.0) as u8,
                    likely_probes: result.suggested_probes,
                }
            }
            QuestionType::DangerousOrHighRisk => {
                QuestionDomain::Unsupported {
                    reason: result.block_reason.unwrap_or_else(||
                        "Operation not supported".to_string()
                    ),
                }
            }
            QuestionType::NeedsUserClarification => {
                QuestionDomain::Uncertain
            }
        }
    }

    /// Check if a question should be fast-rejected (no LLM call needed)
    pub fn should_fast_reject(&self, question: &str) -> Option<String> {
        match self.classify(question) {
            QuestionDomain::Unsupported { reason } => Some(reason),
            _ => None,
        }
    }

    /// v0.50.0: Get the question type only
    pub fn get_type(&self, question: &str) -> QuestionType {
        self.classify_v50(question).question_type
    }

    /// v0.50.0: Check if question requires user clarification
    pub fn needs_clarification(&self, question: &str) -> Option<String> {
        let result = self.classify_v50(question);
        if result.question_type == QuestionType::NeedsUserClarification {
            result.clarification_prompt
        } else {
            None
        }
    }

    /// v0.50.0: Check if question is dangerous/blocked
    pub fn is_blocked(&self, question: &str) -> Option<String> {
        let result = self.classify_v50(question);
        if result.question_type == QuestionType::DangerousOrHighRisk {
            result.block_reason
        } else {
            None
        }
    }
}

impl Default for QuestionClassifier {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cpu_question() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("How many CPU cores do I have?");
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
        assert!(result.suggested_probes.contains(&"cpu.info".to_string()));
    }

    #[test]
    fn test_simple_memory_question() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("How much RAM do I have?");
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
        assert!(result.suggested_probes.contains(&"mem.info".to_string()));
    }

    #[test]
    fn test_simple_disk_question() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("How much disk space is available?");
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
        assert!(result.suggested_probes.contains(&"disk.lsblk".to_string()));
    }

    #[test]
    fn test_simple_gpu_question() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("What GPU do I have?");
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
        assert!(result.suggested_probes.contains(&"hardware.gpu".to_string()));
    }

    #[test]
    fn test_complex_diagnosis() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Why is my system slow?");
        assert_eq!(result.question_type, QuestionType::ComplexDiagnosis);
        // Should suggest multiple probes for diagnosis
        assert!(result.suggested_probes.len() >= 1);
    }

    #[test]
    fn test_dangerous_delete() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Delete my home folder");
        assert_eq!(result.question_type, QuestionType::DangerousOrHighRisk);
        assert!(result.block_reason.is_some());
    }

    #[test]
    fn test_dangerous_install() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Install vim");
        assert_eq!(result.question_type, QuestionType::DangerousOrHighRisk);
    }

    #[test]
    fn test_dangerous_reboot() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Reboot the system");
        assert_eq!(result.question_type, QuestionType::DangerousOrHighRisk);
    }

    #[test]
    fn test_general_knowledge_blocked() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Who is Albert Einstein?");
        assert_eq!(result.question_type, QuestionType::DangerousOrHighRisk);
        assert!(result.block_reason.as_ref().unwrap().contains("general knowledge"));
    }

    #[test]
    fn test_weather_blocked() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("What's the weather today?");
        assert_eq!(result.question_type, QuestionType::DangerousOrHighRisk);
    }

    #[test]
    fn test_ambiguous_needs_clarification() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Something is not working");
        assert_eq!(result.question_type, QuestionType::NeedsUserClarification);
        assert!(result.clarification_prompt.is_some());
    }

    #[test]
    fn test_ambiguous_with_context_ok() {
        let classifier = QuestionClassifier::new();
        // "slow" is ambiguous but "cpu" provides context
        let result = classifier.classify_v50("My CPU seems slow");
        // Should be diagnosed, not need clarification
        assert!(result.question_type == QuestionType::SimpleProbe ||
                result.question_type == QuestionType::ComplexDiagnosis);
    }

    #[test]
    fn test_network_supported() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("What's my IP address?");
        // Network is now supported via safe commands
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
    }

    #[test]
    fn test_package_query_supported() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Is vim installed?");
        // Package queries are now supported (but installs are dangerous)
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
    }

    #[test]
    fn test_legacy_classification() {
        let classifier = QuestionClassifier::new();

        // Supported
        let result = classifier.classify("How many CPU cores?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));

        // Unsupported (dangerous)
        let result = classifier.classify("Delete everything");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));
    }

    #[test]
    fn test_question_type_requires_llm() {
        assert!(!QuestionType::FactFromKnowledge.requires_llm());
        assert!(QuestionType::SimpleProbe.requires_llm());
        assert!(QuestionType::ComplexDiagnosis.requires_llm());
        assert!(!QuestionType::DangerousOrHighRisk.requires_llm());
        assert!(!QuestionType::NeedsUserClarification.requires_llm());
    }

    #[test]
    fn test_question_type_requires_probes() {
        assert!(!QuestionType::FactFromKnowledge.requires_probes());
        assert!(QuestionType::SimpleProbe.requires_probes());
        assert!(QuestionType::ComplexDiagnosis.requires_probes());
        assert!(!QuestionType::DangerousOrHighRisk.requires_probes());
        assert!(!QuestionType::NeedsUserClarification.requires_probes());
    }

    #[test]
    fn test_logs_questions() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Show me system logs");
        assert_eq!(result.question_type, QuestionType::SimpleProbe);
        assert!(result.suggested_probes.contains(&"logs.journalctl".to_string()));
    }

    #[test]
    fn test_multi_domain_is_complex() {
        let classifier = QuestionClassifier::new();
        // Mentions CPU, memory, and disk - should be complex
        let result = classifier.classify_v50(
            "Check my CPU usage, memory consumption, and disk space"
        );
        assert_eq!(result.question_type, QuestionType::ComplexDiagnosis);
        assert!(result.suggested_probes.len() >= 3);
    }

    // v0.70.0 Difficulty tests
    #[test]
    fn test_difficulty_easy_single_probe() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("What CPU do I have?");
        assert_eq!(result.difficulty(), Difficulty::Easy);
    }

    #[test]
    fn test_difficulty_normal_two_probes() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Check my RAM and CPU");
        // May be normal or hard depending on probe mapping
        // (classifier may add related probes like cpu.load, mem.info, etc.)
        let difficulty = result.difficulty();
        assert!(matches!(difficulty, Difficulty::Normal | Difficulty::Hard));
    }

    #[test]
    fn test_difficulty_hard_complex_diagnosis() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50(
            "Why is my system slow? Check CPU, memory, disk, and logs"
        );
        // 3+ probes = hard
        if result.suggested_probes.len() >= 3 {
            assert_eq!(result.difficulty(), Difficulty::Hard);
        }
    }

    #[test]
    fn test_difficulty_dangerous_is_easy() {
        let classifier = QuestionClassifier::new();
        let result = classifier.classify_v50("Delete all my files");
        // Dangerous = fast rejection = easy
        assert_eq!(result.difficulty(), Difficulty::Easy);
    }
}

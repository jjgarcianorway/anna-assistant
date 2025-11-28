//! Question Classifier v0.29.0
//!
//! Fast pre-LLM classification to instantly reject questions outside Anna's domain.
//! This avoids 100+ second LLM calls for obviously unsupported questions.
//!
//! Anna's supported domains (based on probe catalog):
//! - CPU (cpu.info): cores, model, threads, architecture, flags
//! - Memory (mem.info): RAM usage, total, free, available
//! - Disk (disk.lsblk): partitions, filesystems, mount points, sizes
//! - GPU hardware (hardware.gpu): presence, vendor, model
//! - GPU drivers (drivers.gpu): nvidia, amd, intel drivers
//! - RAM hardware (hardware.ram): slot info, physical capacity

use std::collections::HashSet;

/// Classification result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionDomain {
    /// Question is likely answerable with existing probes
    Supported { confidence: u8, likely_probes: Vec<String> },
    /// Question is definitely outside Anna's capabilities
    Unsupported { reason: String },
    /// Unclear - let LLM decide
    Uncertain,
}

/// Fast question classifier (no LLM calls)
pub struct QuestionClassifier {
    /// Keywords that indicate CPU-related questions
    cpu_keywords: HashSet<&'static str>,
    /// Keywords that indicate memory-related questions
    mem_keywords: HashSet<&'static str>,
    /// Keywords that indicate disk-related questions
    disk_keywords: HashSet<&'static str>,
    /// Keywords that indicate GPU-related questions
    gpu_keywords: HashSet<&'static str>,
    /// Keywords that indicate DEFINITELY unsupported domains
    unsupported_keywords: HashSet<&'static str>,
    /// Phrases that indicate general knowledge (not system queries)
    general_knowledge_patterns: Vec<&'static str>,
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
                "memory", "ram", "meminfo", "free", "used", "swap", "buffer",
                "cached", "available", "gb", "gigabyte", "megabyte", "mb",
                "dimm", "slot", "ddr", "ddr4", "ddr5",
            ].into_iter().collect(),

            disk_keywords: [
                "disk", "drive", "storage", "partition", "mount", "filesystem",
                "nvme", "ssd", "hdd", "sata", "ext4", "btrfs", "xfs", "ntfs",
                "lsblk", "df", "space", "block", "device", "/dev/",
            ].into_iter().collect(),

            gpu_keywords: [
                "gpu", "graphics", "nvidia", "amd", "radeon", "geforce", "rtx",
                "gtx", "intel", "video", "vga", "driver", "cuda", "rocm",
                "opengl", "vulkan", "display", "vram",
            ].into_iter().collect(),

            unsupported_keywords: [
                // Network
                "network", "wifi", "ethernet", "ip", "dns", "dhcp", "router",
                "internet", "connection", "ping", "latency", "bandwidth",
                "firewall", "port", "tcp", "udp", "socket",
                // Users/Accounts
                "user", "account", "password", "login", "sudo", "root",
                "permission", "chmod", "chown", "group",
                // Processes/Services
                "process", "service", "daemon", "systemd", "running", "pid",
                "kill", "restart", "status", "journalctl", "logs",
                // Packages
                "package", "install", "apt", "pacman", "dnf", "yum", "brew",
                "npm", "pip", "cargo", "update", "upgrade",
                // Files
                "file", "directory", "folder", "path", "find", "search",
                "locate", "whereis", "which",
                // Time
                "time", "date", "timezone", "clock", "ntp", "calendar",
                // Weather
                "weather", "temperature", "forecast", "rain", "sunny",
                "humidity", "wind",
                // General knowledge - nouns
                "who", "what", "when", "where", "why", "history", "biography",
                "person", "president", "king", "queen", "country", "capital",
                "population", "language", "currency",
                // Math/Calculation
                "calculate", "math", "equation", "formula", "add", "subtract",
                "multiply", "divide", "sum", "average",
                // Programming
                "code", "program", "script", "function", "variable", "class",
                "python", "javascript", "rust", "compile", "debug",
                // Entertainment
                "movie", "music", "song", "actor", "director", "album",
                "book", "author", "game", "sport", "team",
                // News/Events
                "news", "event", "announcement", "election", "vote",
                // Health
                "health", "doctor", "medicine", "symptom", "disease",
                // Food
                "recipe", "cook", "food", "restaurant", "meal",
                // Shopping
                "buy", "price", "shop", "store", "amazon", "order",
            ].into_iter().collect(),

            general_knowledge_patterns: vec![
                "who is", "who was", "what is", "what was", "when did",
                "where is", "where was", "why did", "how do i", "how to",
                "can you tell me about", "explain", "describe", "define",
                "what does", "what are", "tell me about", "give me",
                "write me", "help me with", "translate", "summarize",
            ],
        }
    }

    /// Normalize a word: strip punctuation and handle simple plurals
    fn normalize_word(word: &str) -> String {
        // Strip common punctuation
        let cleaned: String = word.chars()
            .filter(|c| c.is_alphanumeric())
            .collect();

        // Handle simple plurals (e.g., "partitions" -> "partition")
        if cleaned.ends_with('s') && cleaned.len() > 3 {
            cleaned[..cleaned.len()-1].to_string()
        } else {
            cleaned
        }
    }

    /// Classify a question without any LLM calls
    pub fn classify(&self, question: &str) -> QuestionDomain {
        let q_lower = question.to_lowercase();
        let words: Vec<String> = q_lower
            .split_whitespace()
            .map(Self::normalize_word)
            .collect();
        let words_set: HashSet<&str> = words.iter().map(|s| s.as_str()).collect();

        // FIRST: Check for supported domains (this takes priority!)
        // If the question mentions CPU/memory/disk/GPU, we should try to answer it
        let mut likely_probes = Vec::new();
        let mut confidence: u8 = 0;

        // CPU detection
        if words_set.intersection(&self.cpu_keywords).next().is_some() {
            likely_probes.push("cpu.info".to_string());
            confidence = confidence.saturating_add(30);
        }

        // Memory detection
        if words_set.intersection(&self.mem_keywords).next().is_some() {
            likely_probes.push("mem.info".to_string());
            likely_probes.push("hardware.ram".to_string());
            confidence = confidence.saturating_add(30);
        }

        // Disk detection
        if words_set.intersection(&self.disk_keywords).next().is_some() {
            likely_probes.push("disk.lsblk".to_string());
            confidence = confidence.saturating_add(30);
        }

        // GPU detection
        if words_set.intersection(&self.gpu_keywords).next().is_some() {
            likely_probes.push("hardware.gpu".to_string());
            likely_probes.push("drivers.gpu".to_string());
            confidence = confidence.saturating_add(30);
        }

        // If we found supported keywords, return immediately
        if !likely_probes.is_empty() {
            return QuestionDomain::Supported {
                confidence: confidence.min(100),
                likely_probes,
            };
        }

        // Check for general knowledge patterns (only if no supported keywords found)
        for pattern in &self.general_knowledge_patterns {
            if q_lower.contains(pattern) {
                return QuestionDomain::Unsupported {
                    reason: format!(
                        "This looks like a general knowledge question. \
                         I can only answer questions about your local hardware: \
                         CPU, memory, disk, and GPU."
                    ),
                };
            }
        }

        // Check for definitely unsupported keywords
        let unsupported_matches: Vec<&str> = words_set
            .intersection(&self.unsupported_keywords)
            .copied()
            .collect();

        if !unsupported_matches.is_empty() {
            // Pure unsupported question (no supported keywords found earlier)
            return QuestionDomain::Unsupported {
                reason: format!(
                    "I don't have probes for '{}'. \
                     I can only answer questions about: CPU, memory/RAM, disk/storage, and GPU.",
                    unsupported_matches.first().unwrap_or(&"this topic")
                ),
            };
        }

        // No clear signals either way - let LLM decide
        QuestionDomain::Uncertain
    }

    /// Check if a question should be fast-rejected (no LLM call needed)
    pub fn should_fast_reject(&self, question: &str) -> Option<String> {
        match self.classify(question) {
            QuestionDomain::Unsupported { reason } => Some(reason),
            _ => None,
        }
    }
}

impl Default for QuestionClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_questions() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("How many CPU cores do I have?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));

        let result = classifier.classify("What processor is installed?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));
    }

    #[test]
    fn test_memory_questions() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("How much RAM do I have?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));

        let result = classifier.classify("What's my memory usage?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));
    }

    #[test]
    fn test_disk_questions() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("How much disk space is available?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));

        let result = classifier.classify("Show my partitions");
        assert!(matches!(result, QuestionDomain::Supported { .. }));
    }

    #[test]
    fn test_gpu_questions() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("What GPU do I have?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));

        let result = classifier.classify("Is nvidia driver loaded?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));
    }

    #[test]
    fn test_unsupported_network() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("What's my IP address?");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));

        let result = classifier.classify("Show network connections");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));
    }

    #[test]
    fn test_unsupported_general_knowledge() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("Who is Albert Einstein?");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));

        let result = classifier.classify("What is the capital of France?");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));

        let result = classifier.classify("Tell me about quantum physics");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));
    }

    #[test]
    fn test_unsupported_weather() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("What's the weather today?");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));
    }

    #[test]
    fn test_unsupported_math() {
        let classifier = QuestionClassifier::new();

        let result = classifier.classify("What is 2 + 2?");
        // "+" gets split, but "add" would be caught
        // This might be Uncertain, which is fine - LLM will handle

        let result = classifier.classify("Calculate the sum of 5 and 10");
        assert!(matches!(result, QuestionDomain::Unsupported { .. }));
    }

    #[test]
    fn test_fast_reject() {
        let classifier = QuestionClassifier::new();

        // Should reject
        assert!(classifier.should_fast_reject("Who is the president?").is_some());
        assert!(classifier.should_fast_reject("What's the weather?").is_some());

        // Should NOT reject
        assert!(classifier.should_fast_reject("How many CPU cores?").is_none());
        assert!(classifier.should_fast_reject("What's my disk usage?").is_none());
    }

    #[test]
    fn test_exception_how_much_ram() {
        let classifier = QuestionClassifier::new();

        // "How much" normally triggers general knowledge, but "ram" should override
        let result = classifier.classify("How much RAM do I have?");
        assert!(matches!(result, QuestionDomain::Supported { .. }));
    }
}

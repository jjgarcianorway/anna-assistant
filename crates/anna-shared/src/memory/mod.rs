//! Learning Memory System - Anna learns from every interaction.
//!
//! This is NOT a hardcoded recipe system. Instead:
//! - Every successful Q&A is stored with semantic embeddings
//! - Similar questions retrieve relevant past experiences
//! - Patterns emerge organically from successful interactions
//! - Anna learns what commands work for what types of questions
//!
//! The system learns:
//! - Question patterns → effective commands
//! - System context → relevant approaches
//! - Error patterns → successful fixes
//!
//! v0.0.889: Added semantic question clustering
//! - Questions like "What's my RAM?" and "How much memory?" cluster together
//! - Clusters share learned commands and patterns
//! - Improves recall accuracy for paraphrased questions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::anna_data_dir;

/// A learned experience from a past interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Unique ID
    pub id: String,

    /// The question asked (normalized)
    pub question: String,

    /// Keywords extracted from the question
    pub keywords: Vec<String>,

    /// Commands that successfully answered this question
    pub successful_commands: Vec<String>,

    /// The answer that was generated
    pub answer: String,

    /// System context at the time (relevant profile fields)
    pub context: ExperienceContext,

    /// How many times this experience has been useful
    pub usefulness_score: u32,

    /// When this experience was created
    pub created_at: String,

    /// When this experience was last used
    pub last_used: Option<String>,

    /// Embedding vector for semantic search (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// Context captured with an experience
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperienceContext {
    /// Was this about a specific package?
    pub package: Option<String>,

    /// Was this about a specific service?
    pub service: Option<String>,

    /// Was this about a specific file/path?
    pub path: Option<String>,

    /// What topic category does this fall into?
    pub topic: Option<String>,

    /// System-specific context (e.g., "wayland", "nvidia", "btrfs")
    pub system_tags: Vec<String>,

    /// v0.0.900: Commands that failed on this system context
    #[serde(default)]
    pub failed_commands: Vec<FailedCommand>,

    /// v0.0.900: System tags where commands succeeded
    #[serde(default)]
    pub success_tags: Vec<String>,
}

/// v0.0.900: Record of a command that failed in a specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCommand {
    pub command: String,
    pub error_type: String,
    pub system_tags: Vec<String>,
    pub failed_at: String,
}

impl ExperienceContext {
    /// v0.0.900: Get current system tags from profile
    /// v0.0.901: No hardcoding - capture actual system values and let learning handle patterns
    pub fn current_system_tags() -> Vec<String> {
        use crate::profile::SystemProfile;

        let mut tags = Vec::new();

        // GPU - capture actual vendor/device, don't categorize
        if let Ok(profile) = SystemProfile::load() {
            for pci in &profile.hardware.pci_devices {
                let class_lower = pci.class.to_lowercase();
                if class_lower.contains("vga") || class_lower.contains("display") || class_lower.contains("3d") {
                    // Capture actual vendor name (normalized)
                    let vendor = pci.vendor.to_lowercase()
                        .replace("corporation", "")
                        .replace("inc.", "")
                        .replace("ltd.", "")
                        .trim()
                        .to_string();
                    if !vendor.is_empty() {
                        tags.push(format!("gpu:{}", vendor.split_whitespace().next().unwrap_or(&vendor)));
                    }
                    break;
                }
            }
        }

        // Display server - capture actual protocol
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            tags.push("display:wayland".to_string());
        } else if std::env::var("DISPLAY").is_ok() {
            tags.push("display:x11".to_string());
        }

        // Desktop/WM - capture whatever XDG_CURRENT_DESKTOP reports (could be anything!)
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            // Normalize: lowercase, take first component if colon-separated
            let de_normalized = de.to_lowercase()
                .split(':')
                .next()
                .unwrap_or(&de)
                .trim()
                .to_string();
            if !de_normalized.is_empty() {
                tags.push(format!("de:{}", de_normalized));
            }
        }

        // Session type (complements DE detection)
        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            tags.push(format!("session:{}", session.to_lowercase()));
        }

        // Filesystem - capture actual fstype, whatever it is
        if let Ok(output) = std::process::Command::new("findmnt")
            .args(["-n", "-o", "FSTYPE", "/"])
            .output()
        {
            let fstype = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            if !fstype.is_empty() {
                tags.push(format!("fs:{}", fstype));
            }
        }

        // Init system
        if std::path::Path::new("/run/systemd/system").exists() {
            tags.push("init:systemd".to_string());
        } else if std::path::Path::new("/run/openrc").exists() {
            tags.push("init:openrc".to_string());
        } else if std::path::Path::new("/run/runit").exists() {
            tags.push("init:runit".to_string());
        }

        // Distro (from /etc/os-release)
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if let Some(id) = line.strip_prefix("ID=") {
                    let distro = id.trim_matches('"').to_lowercase();
                    if !distro.is_empty() {
                        tags.push(format!("distro:{}", distro));
                    }
                    break;
                }
            }
        }

        tags
    }

    /// v0.0.900: Check if a command has failed on current system
    pub fn is_known_failure(&self, command: &str) -> bool {
        let current_tags = Self::current_system_tags();

        self.failed_commands.iter().any(|fc| {
            fc.command == command &&
            fc.system_tags.iter().any(|t| current_tags.contains(t))
        })
    }

    /// v0.0.900: Record a command failure
    pub fn record_failure(&mut self, command: &str, error_type: &str) {
        let tags = Self::current_system_tags();

        // Don't duplicate
        if !self.failed_commands.iter().any(|fc| fc.command == command) {
            self.failed_commands.push(FailedCommand {
                command: command.to_string(),
                error_type: error_type.to_string(),
                system_tags: tags,
                failed_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    /// v0.0.900: Record success tags
    pub fn record_success(&mut self) {
        let tags = Self::current_system_tags();
        for tag in tags {
            if !self.success_tags.contains(&tag) {
                self.success_tags.push(tag);
            }
        }
    }

    /// v0.0.900: Score boost for matching system context
    pub fn system_match_score(&self) -> f32 {
        let current_tags = Self::current_system_tags();
        if current_tags.is_empty() || self.success_tags.is_empty() {
            return 0.0;
        }

        let matches = current_tags.iter()
            .filter(|t| self.success_tags.contains(t))
            .count();

        (matches as f32 / current_tags.len() as f32) * 0.2  // Up to 20% boost
    }
}

/// The memory store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Memory {
    /// All learned experiences
    pub experiences: Vec<Experience>,

    /// Learned patterns: keyword -> common commands
    pub patterns: Vec<LearnedPattern>,

    /// Semantic question clusters (v0.0.889)
    #[serde(default)]
    pub clusters: Vec<QuestionCluster>,

    /// Statistics
    pub stats: MemoryStats,
}

/// A pattern learned from multiple experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Keywords that trigger this pattern
    pub keywords: Vec<String>,

    /// Commands that commonly work for these keywords
    pub common_commands: Vec<CommandPattern>,

    /// How many experiences support this pattern
    pub evidence_count: u32,
}

/// A command pattern with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    /// The command template (may include placeholders like {package})
    pub command: String,

    /// How often this command succeeded
    pub success_count: u32,

    /// What type of information this command retrieves
    pub retrieves: Option<String>,
}

/// Memory statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total experiences stored
    pub total_experiences: u32,

    /// Total patterns learned
    pub total_patterns: u32,

    /// Questions answered from memory (without full LLM)
    pub memory_hits: u32,

    /// Questions that needed full LLM processing
    pub memory_misses: u32,

    /// Total clusters formed
    pub total_clusters: u32,

    /// Load failures encountered (v0.0.890)
    #[serde(default)]
    pub load_failures: u32,

    /// Last load error message (v0.0.890)
    #[serde(default)]
    pub last_error: Option<String>,

    /// Recovery count - times we recovered from corruption (v0.0.890)
    #[serde(default)]
    pub recoveries: u32,
}

/// Memory load result with context (v0.0.890)
#[derive(Debug)]
pub struct MemoryLoadResult {
    pub memory: Memory,
    pub was_recovered: bool,
    pub error: Option<String>,
}

/// Semantic question cluster - groups similar questions together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionCluster {
    /// Cluster ID
    pub id: String,

    /// Canonical form of the question (normalized)
    pub canonical: String,

    /// All question variations that belong to this cluster
    pub variations: Vec<String>,

    /// Combined keywords from all variations
    pub keywords: Vec<String>,

    /// IDs of experiences in this cluster
    pub experience_ids: Vec<String>,

    /// Commands that work for this cluster (aggregated)
    pub effective_commands: Vec<ClusterCommand>,
}

/// A command with cluster-level statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterCommand {
    pub command: String,
    pub success_count: u32,
}

impl Memory {
    /// Load memory from disk
    pub fn load() -> Result<Self> {
        let path = memory_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let memory: Memory = serde_json::from_str(&content)?;
            Ok(memory)
        } else {
            Ok(Memory::default())
        }
    }

    /// Load memory with recovery on failure (v0.0.890)
    /// Returns memory + metadata about whether recovery occurred
    pub fn load_with_recovery() -> MemoryLoadResult {
        let path = memory_path();

        if !path.exists() {
            return MemoryLoadResult {
                memory: Memory::default(),
                was_recovered: false,
                error: None,
            };
        }

        // Try to load normally
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Memory>(&content) {
                    Ok(memory) => MemoryLoadResult {
                        memory,
                        was_recovered: false,
                        error: None,
                    },
                    Err(e) => {
                        // JSON parse failed - try to recover what we can
                        let error_msg = format!("Memory corruption detected: {}", e);

                        // Backup corrupted file
                        let backup_path = memory_path().with_extension("json.corrupted");
                        let _ = std::fs::rename(&path, &backup_path);

                        // Return fresh memory with error context
                        let mut memory = Memory::default();
                        memory.stats.load_failures += 1;
                        memory.stats.last_error = Some(error_msg.clone());
                        memory.stats.recoveries += 1;

                        MemoryLoadResult {
                            memory,
                            was_recovered: true,
                            error: Some(error_msg),
                        }
                    }
                }
            }
            Err(e) => {
                // File read failed
                let error_msg = format!("Memory file read error: {}", e);
                let mut memory = Memory::default();
                memory.stats.load_failures += 1;
                memory.stats.last_error = Some(error_msg.clone());

                MemoryLoadResult {
                    memory,
                    was_recovered: true,
                    error: Some(error_msg),
                }
            }
        }
    }

    /// Check memory health (v0.0.890)
    pub fn health_check(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.stats.load_failures > 0 {
            issues.push(format!(
                "Memory has had {} load failures (last: {})",
                self.stats.load_failures,
                self.stats.last_error.as_deref().unwrap_or("unknown")
            ));
        }

        if self.experiences.len() > 800 {
            issues.push(format!(
                "Memory approaching capacity ({}/1000 experiences)",
                self.experiences.len()
            ));
        }

        if self.clusters.len() > 100 {
            issues.push(format!(
                "High cluster count ({}) may slow recall",
                self.clusters.len()
            ));
        }

        issues
    }

    /// Save memory to disk
    pub fn save(&self) -> Result<()> {
        let path = memory_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Learn from a successful interaction
    /// v0.0.892: Added deduplication - merges similar experiences instead of creating duplicates
    pub fn learn(&mut self, question: &str, commands: Vec<String>, answer: &str, context: ExperienceContext) {
        let keywords = extract_keywords(question);
        let canonical = canonicalize_question(question);

        // v0.0.892: Check for near-duplicate experience first
        if let Some(existing) = self.find_similar_experience(&canonical, &keywords) {
            // Merge into existing experience
            let exp_id = existing.to_string();
            if let Some(exp) = self.experiences.iter_mut().find(|e| e.id == exp_id) {
                exp.usefulness_score += 1;
                exp.last_used = Some(chrono::Utc::now().to_rfc3339());

                // Merge new commands into existing
                for cmd in &commands {
                    if !exp.successful_commands.contains(cmd) {
                        exp.successful_commands.push(cmd.clone());
                    }
                }

                // Update patterns with merged commands
                self.update_patterns(&keywords, &commands);

                // Find cluster and update its commands
                if let Some(cluster) = self.clusters.iter_mut().find(|c| c.experience_ids.contains(&exp_id)) {
                    for cmd in &commands {
                        if let Some(cc) = cluster.effective_commands.iter_mut().find(|c| &c.command == cmd) {
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

        // Find or create a semantic cluster for this question (v0.0.889)
        let cluster_id = self.find_or_create_cluster(question, &keywords);

        // Create new experience
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

        // Link experience to cluster and update cluster commands
        if let Some(cluster) = self.clusters.iter_mut().find(|c| c.id == cluster_id) {
            cluster.experience_ids.push(experience_id);
        }
        self.update_cluster_commands(&cluster_id, &commands);

        // Update patterns
        self.update_patterns(&keywords, &commands);
    }

    /// v0.0.892: Find a near-duplicate experience that should be merged instead of creating new
    fn find_similar_experience(&self, canonical: &str, keywords: &[String]) -> Option<String> {
        const SIMILARITY_THRESHOLD: f32 = 0.85;

        for exp in &self.experiences {
            let exp_canonical = canonicalize_question(&exp.question);

            // Exact canonical match
            if exp_canonical == *canonical {
                return Some(exp.id.clone());
            }

            // Very high keyword overlap (85%+)
            if !keywords.is_empty() && !exp.keywords.is_empty() {
                let matching = keywords.iter().filter(|k| exp.keywords.contains(k)).count();
                let overlap = matching as f32 / keywords.len().max(exp.keywords.len()) as f32;
                if overlap >= SIMILARITY_THRESHOLD {
                    return Some(exp.id.clone());
                }
            }
        }

        None
    }

    /// Update patterns based on new experience
    fn update_patterns(&mut self, keywords: &[String], commands: &[String]) {
        // Find or create pattern for these keywords
        for keyword in keywords {
            if let Some(pattern) = self.patterns.iter_mut().find(|p| p.keywords.contains(keyword)) {
                // Update existing pattern
                for cmd in commands {
                    if let Some(cp) = pattern.common_commands.iter_mut().find(|c| &c.command == cmd) {
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
                // Create new pattern
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

    /// Find relevant experiences for a question
    pub fn recall(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        // Score experiences by relevance
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

        // Sort by score (highest first) then by usefulness
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
                if pattern.keywords.iter().any(|k| k == keyword || keyword.contains(k)) {
                    for cmd in &pattern.common_commands {
                        if let Some((_, count)) = suggestions.iter_mut().find(|(c, _)| c == &cmd.command) {
                            *count += cmd.success_count;
                        } else {
                            suggestions.push((cmd.command.clone(), cmd.success_count));
                        }
                    }
                }
            }
        }

        // Sort by success count
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        suggestions.into_iter().map(|(c, _)| c).collect()
    }

    /// Mark an experience as useful (was retrieved and helped)
    pub fn mark_useful(&mut self, experience_id: &str) {
        if let Some(exp) = self.experiences.iter_mut().find(|e| e.id == experience_id) {
            exp.usefulness_score += 1;
            exp.last_used = Some(chrono::Utc::now().to_rfc3339());
            self.stats.memory_hits += 1;
        }
    }

    /// Record a memory miss (had to use full LLM)
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

        // Sort by usefulness and recency
        self.experiences.sort_by(|a, b| {
            // Prefer higher usefulness
            let usefulness_cmp = b.usefulness_score.cmp(&a.usefulness_score);
            if usefulness_cmp != std::cmp::Ordering::Equal {
                return usefulness_cmp;
            }
            // Then prefer more recent
            b.created_at.cmp(&a.created_at)
        });

        // Keep only the most valuable
        self.experiences.truncate(max_experiences);
        self.stats.total_experiences = self.experiences.len() as u32;
    }
}

/// Extract keywords from a question
fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "need", "dare",
        "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above",
        "below", "between", "under", "again", "further", "then", "once", "here",
        "there", "when", "where", "why", "how", "all", "each", "every", "both",
        "few", "more", "most", "other", "some", "such", "no", "nor", "not",
        "only", "own", "same", "so", "than", "too", "very", "just", "and",
        "but", "if", "or", "because", "until", "while", "what", "which", "who",
        "whom", "this", "that", "these", "those", "am", "i", "my", "me", "you",
        "your", "it", "its", "he", "she", "they", "we", "them", "his", "her",
        "their", "our", "much", "many", "any", "about", "get", "tell", "show",
    ];

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Calculate relevance score between experience and question
fn calculate_relevance(experience: &Experience, question: &str, keywords: &[String]) -> f32 {
    let mut score = 0.0;

    // Exact substring match
    if experience.question.contains(question) || question.contains(&experience.question) {
        score += 0.5;
    }

    // Keyword overlap
    let keyword_matches = keywords
        .iter()
        .filter(|k| experience.keywords.contains(k))
        .count();

    if !keywords.is_empty() {
        score += (keyword_matches as f32) / (keywords.len() as f32) * 0.4;
    }

    // Boost by usefulness
    score += (experience.usefulness_score as f32).min(10.0) / 100.0;

    score
}

/// Get memory storage path
pub fn memory_path() -> PathBuf {
    anna_data_dir().join("memory.json")
}

// ============================================================================
// SEMANTIC QUESTION CLUSTERING (v0.0.889)
// ============================================================================

/// Semantic synonym groups - questions using any word in a group are considered related
/// Format: (canonical_term, [synonyms])
const SEMANTIC_SYNONYMS: &[(&str, &[&str])] = &[
    // Memory/RAM
    ("memory", &["ram", "memory", "mem", "swap"]),
    // Storage/Disk
    ("disk", &["disk", "storage", "drive", "hdd", "ssd", "nvme", "partition", "space"]),
    // CPU/Processor
    ("cpu", &["cpu", "processor", "cores", "threads", "load", "utilization"]),
    // Network
    ("network", &["network", "wifi", "ethernet", "connection", "internet", "ip", "dns"]),
    // Audio/Sound
    ("audio", &["audio", "sound", "speaker", "volume", "microphone", "pulseaudio", "pipewire"]),
    // Display/Screen
    ("display", &["display", "screen", "monitor", "resolution", "brightness", "wayland", "xorg"]),
    // Packages/Software
    ("packages", &["package", "packages", "install", "pacman", "yay", "aur", "software", "app"]),
    // Services/Daemons
    ("services", &["service", "services", "daemon", "systemd", "unit", "systemctl"]),
    // Boot/Startup
    ("boot", &["boot", "startup", "grub", "kernel", "initramfs", "bootloader"]),
    // System info
    ("system", &["system", "os", "distro", "arch", "version", "hostname"]),
    // Hardware
    ("hardware", &["hardware", "device", "devices", "lspci", "lsusb", "gpu", "graphics"]),
    // Battery/Power
    ("battery", &["battery", "power", "charging", "acpi", "upower"]),
    // Users/Permissions
    ("users", &["user", "users", "permission", "permissions", "sudo", "group"]),
    // Files/Filesystem
    ("files", &["file", "files", "directory", "folder", "path", "filesystem"]),
    // Processes
    ("processes", &["process", "processes", "running", "pid", "kill", "ps", "htop"]),
    // Errors/Issues
    ("errors", &["error", "errors", "fail", "failed", "failing", "issue", "problem", "broken"]),
    // Logs
    ("logs", &["log", "logs", "journal", "journalctl", "dmesg"]),
    // Config
    ("config", &["config", "configuration", "settings", "configure", "setup"]),
    // Kernel
    ("kernel", &["kernel", "uname", "module", "modules", "driver", "drivers"]),
];

/// Canonicalize a question by replacing synonyms with canonical terms
pub fn canonicalize_question(question: &str) -> String {
    let mut canonical = question.to_lowercase();

    // Remove question marks and normalize whitespace
    canonical = canonical.replace('?', "").trim().to_string();

    // Replace synonyms with canonical terms
    for (canonical_term, synonyms) in SEMANTIC_SYNONYMS {
        for synonym in *synonyms {
            if *synonym != *canonical_term {
                // Word boundary replacement to avoid partial matches
                let pattern = format!(" {} ", synonym);
                let replacement = format!(" {} ", canonical_term);
                canonical = format!(" {} ", canonical).replace(&pattern, &replacement);
            }
        }
    }

    canonical.trim().to_string()
}

/// Extract semantic groups from a question
fn extract_semantic_groups(question: &str) -> Vec<String> {
    let q_lower = question.to_lowercase();
    let mut groups = Vec::new();

    for (canonical, synonyms) in SEMANTIC_SYNONYMS {
        for synonym in *synonyms {
            if q_lower.contains(synonym) {
                if !groups.contains(&canonical.to_string()) {
                    groups.push(canonical.to_string());
                }
                break;
            }
        }
    }

    groups
}

/// Calculate similarity between a question and a cluster
/// v0.0.893: Fixed edge case for very short questions
/// v0.0.902: Better handling of single-word queries via semantic groups
fn calculate_cluster_similarity(question: &str, cluster: &QuestionCluster) -> f32 {
    let q_lower = question.to_lowercase();
    let q_canonical = canonicalize_question(question);
    let q_keywords = extract_keywords(question);
    let q_groups = extract_semantic_groups(question);

    // Exact canonical match is strongest signal
    if q_canonical == cluster.canonical { return 0.95; }

    let mut score = 0.0;

    // v0.0.893: Require minimum 2 words to avoid single-word over-matching
    let canonical_words: Vec<&str> = cluster.canonical.split_whitespace().collect();
    let q_words: Vec<&str> = q_canonical.split_whitespace().collect();
    let max_words = canonical_words.len().max(q_words.len()).max(1);

    // v0.0.902: For short queries, rely more on semantic groups
    let is_short_query = q_words.len() < 2 || q_keywords.len() < 2;

    if canonical_words.len() >= 2 && q_words.len() >= 2 {
        let common_words = canonical_words.iter().filter(|w| q_words.contains(w)).count();
        score += (common_words as f32 / max_words as f32) * 0.4;
    }

    // v0.0.893: Require minimum 2 keywords (unless short query)
    let max_kw = q_keywords.len().max(cluster.keywords.len()).max(1);
    if q_keywords.len() >= 2 && cluster.keywords.len() >= 2 {
        let keyword_matches = q_keywords.iter().filter(|k| cluster.keywords.contains(k)).count();
        score += (keyword_matches as f32 / max_kw as f32) * 0.3;
    } else if is_short_query && !q_keywords.is_empty() {
        // v0.0.902: Single-keyword match for short queries (e.g., "RAM?" matching "ram" keyword)
        let keyword_matches = q_keywords.iter().filter(|k| cluster.keywords.contains(k)).count();
        if keyword_matches > 0 {
            score += 0.4;  // Strong signal for single keyword match on short query
        }
    }

    // Semantic group overlap - v0.0.902: Higher weight for short queries
    let cluster_groups = extract_semantic_groups(&cluster.canonical);
    let max_groups = q_groups.len().max(cluster_groups.len()).max(1);
    if !q_groups.is_empty() && !cluster_groups.is_empty() {
        let group_matches = q_groups.iter().filter(|g| cluster_groups.contains(g)).count();
        // Short queries rely more on semantic groups (0.5 vs 0.2)
        let weight = if is_short_query { 0.5 } else { 0.2 };
        score += (group_matches as f32 / max_groups as f32) * weight;
    }

    // Check variation similarity
    for variation in &cluster.variations {
        if variation.contains(&q_lower) || q_lower.contains(variation) { score += 0.3; break; }
    }

    score.min(1.0)
}

impl Memory {
    /// Find a matching cluster or create a new one
    pub fn find_or_create_cluster(&mut self, question: &str, keywords: &[String]) -> String {
        let canonical = canonicalize_question(question);
        let q_lower = question.to_lowercase();

        // Find best matching cluster
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
            // Add this variation to the existing cluster
            let cluster = &mut self.clusters[idx];
            if !cluster.variations.contains(&q_lower) {
                cluster.variations.push(q_lower);
            }
            // Merge keywords
            for kw in keywords {
                if !cluster.keywords.contains(kw) {
                    cluster.keywords.push(kw.clone());
                }
            }
            cluster.id.clone()
        } else {
            // Create new cluster
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
                if let Some(cc) = cluster.effective_commands.iter_mut().find(|c| &c.command == cmd) {
                    cc.success_count += 1;
                } else {
                    cluster.effective_commands.push(ClusterCommand {
                        command: cmd.clone(),
                        success_count: 1,
                    });
                }
            }
            // Sort by success count
            cluster.effective_commands.sort_by(|a, b| b.success_count.cmp(&a.success_count));
        }
    }

    /// Get commands suggested by clusters (semantic recall)
    pub fn suggest_commands_from_clusters(&self, question: &str) -> Vec<String> {
        let mut suggestions: HashMap<String, u32> = HashMap::new();

        for cluster in &self.clusters {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.5 {
                for cmd in &cluster.effective_commands {
                    // Weight by similarity and success count
                    let weight = (sim * cmd.success_count as f32) as u32;
                    *suggestions.entry(cmd.command.clone()).or_insert(0) += weight.max(1);
                }
            }
        }

        // Sort by weighted score
        let mut sorted: Vec<_> = suggestions.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().map(|(c, _)| c).take(5).collect()
    }

    /// Enhanced recall using clusters
    pub fn recall_with_clusters(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        // Get experience IDs from similar clusters
        let mut cluster_exp_ids: Vec<String> = Vec::new();
        for cluster in &self.clusters {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.5 {
                cluster_exp_ids.extend(cluster.experience_ids.clone());
            }
        }

        // Score experiences, boosting those from matching clusters
        let mut scored: Vec<(&Experience, f32)> = self
            .experiences
            .iter()
            .filter_map(|exp| {
                let mut score = calculate_relevance(exp, &question_lower, &keywords);

                // Boost if experience is in a matching cluster
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

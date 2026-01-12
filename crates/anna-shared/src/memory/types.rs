//! Memory data types and structures.
//! v0.0.930: Added keyword index for faster recall

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main memory storage structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Memory {
    /// All stored experiences
    pub experiences: Vec<Experience>,

    /// Learned patterns from experiences
    pub patterns: Vec<LearnedPattern>,

    /// Memory statistics
    pub stats: MemoryStats,

    /// Question clusters for semantic grouping
    #[serde(default)]
    pub clusters: Vec<QuestionCluster>,

    /// v0.0.930: Keyword index for O(1) candidate lookup
    /// Maps keyword -> list of experience IDs containing that keyword
    #[serde(skip)]
    pub keyword_index: HashMap<String, Vec<String>>,
}

impl Memory {
    /// v0.0.930: Rebuild keyword index from experiences
    pub fn rebuild_index(&mut self) {
        self.keyword_index.clear();
        for exp in &self.experiences {
            for keyword in &exp.keywords {
                self.keyword_index
                    .entry(keyword.clone())
                    .or_default()
                    .push(exp.id.clone());
            }
        }
    }

    /// v0.0.930: Add experience to keyword index
    pub fn index_experience(&mut self, exp_id: &str, keywords: &[String]) {
        for keyword in keywords {
            self.keyword_index
                .entry(keyword.clone())
                .or_default()
                .push(exp_id.to_string());
        }
    }

    /// v0.0.930: Get candidate experience IDs by keywords (fast path)
    pub fn get_candidates_by_keywords(&self, keywords: &[String]) -> Vec<&str> {
        let mut candidates: HashMap<&str, usize> = HashMap::new();

        for keyword in keywords {
            if let Some(exp_ids) = self.keyword_index.get(keyword) {
                for exp_id in exp_ids {
                    *candidates.entry(exp_id.as_str()).or_insert(0) += 1;
                }
            }
        }

        // Return candidates sorted by keyword match count (most matches first)
        let mut sorted: Vec<_> = candidates.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().map(|(id, _)| id).collect()
    }
}

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
    pub fn current_system_tags() -> Vec<String> {
        use crate::profile::SystemProfile;

        let mut tags = Vec::new();

        // GPU - capture actual vendor/device
        if let Ok(profile) = SystemProfile::load() {
            for pci in &profile.hardware.pci_devices {
                let class_lower = pci.class.to_lowercase();
                if class_lower.contains("vga")
                    || class_lower.contains("display")
                    || class_lower.contains("3d")
                {
                    let vendor = pci
                        .vendor
                        .to_lowercase()
                        .replace("corporation", "")
                        .replace("inc.", "")
                        .replace("ltd.", "")
                        .trim()
                        .to_string();
                    if !vendor.is_empty() {
                        tags.push(format!(
                            "gpu:{}",
                            vendor.split_whitespace().next().unwrap_or(&vendor)
                        ));
                    }
                    break;
                }
            }
        }

        // Display server
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            tags.push("display:wayland".to_string());
        } else if std::env::var("DISPLAY").is_ok() {
            tags.push("display:x11".to_string());
        }

        // Desktop/WM
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            let de_normalized = de
                .to_lowercase()
                .split(':')
                .next()
                .unwrap_or(&de)
                .trim()
                .to_string();
            if !de_normalized.is_empty() {
                tags.push(format!("de:{}", de_normalized));
            }
        }

        // Session type
        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            tags.push(format!("session:{}", session.to_lowercase()));
        }

        // Filesystem
        if let Ok(output) = std::process::Command::new("findmnt")
            .args(["-n", "-o", "FSTYPE", "/"])
            .output()
        {
            let fstype = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
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

        // Distro
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
            fc.command == command && fc.system_tags.iter().any(|t| current_tags.contains(t))
        })
    }

    /// v0.0.900: Record a command failure
    pub fn record_failure(&mut self, command: &str, error_type: &str) {
        let tags = Self::current_system_tags();

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

        let matches = current_tags
            .iter()
            .filter(|t| self.success_tags.contains(t))
            .count();

        (matches as f32 / current_tags.len() as f32) * 0.2
    }
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
    #[serde(default)]
    pub total_experiences: u32,

    /// Total patterns learned
    #[serde(default)]
    pub total_patterns: u32,

    /// Questions answered from memory (without full LLM)
    #[serde(default)]
    pub memory_hits: u32,

    /// Questions that needed full LLM processing
    #[serde(default)]
    pub memory_misses: u32,

    /// Total clusters formed
    #[serde(default)]
    pub total_clusters: u32,

    /// Load failures encountered (v0.0.890)
    #[serde(default)]
    pub load_failures: u32,

    /// Last load error message (v0.0.890)
    #[serde(default)]
    pub last_error: Option<String>,

    /// Recovery count (v0.0.890)
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

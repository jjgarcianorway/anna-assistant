//! Model Policy v0.0.35
//!
//! Policy-driven model selection for Translator and Junior roles.
//! Configuration loaded from /etc/anna/policy/models.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Policy directory for model configuration
pub const MODELS_POLICY_DIR: &str = "/etc/anna/policy";
pub const MODELS_POLICY_FILE: &str = "/etc/anna/policy/models.toml";

/// Default policy for model selection
pub const DEFAULT_MODELS_POLICY: &str = r#"# Anna Model Selection Policy v0.0.35
# This file configures how Anna selects local LLM models for each role.

[global]
# Allow automatic model downloads when needed
auto_download = true
# Re-benchmark when hardware changes significantly
rebenchmark_on_hardware_change = true
# Benchmark cache validity in hours (0 = always rebenchmark)
benchmark_cache_hours = 168  # 7 days

[translator]
# Translator: Fast intent classification and tool planning
# Requires: low latency, small context, strict token budget
description = "Fast intent planner for request classification"

# Maximum acceptable latency (milliseconds)
max_latency_ms = 2000

# Minimum quality tier required (low, medium, high)
min_quality_tier = "low"

# Maximum tokens per request
max_tokens = 512

# Candidate models in preference order (first available wins)
# Format: "model:tag" or just "model" for latest
candidates = [
    "qwen2.5:0.5b",
    "qwen2.5:1.5b",
    "llama3.2:1b",
    "phi3:mini",
    "gemma2:2b",
]

# Fallback if no candidates available
fallback = "deterministic"  # Use deterministic classifier

[junior]
# Junior: Reliability verification and critique
# Requires: higher reasoning, consistency checking, citation enforcement
description = "Reliable verifier for answer quality assurance"

# Maximum acceptable latency (milliseconds)
max_latency_ms = 5000

# Minimum quality tier required
min_quality_tier = "medium"

# Maximum tokens per request
max_tokens = 1024

# Candidate models in preference order
candidates = [
    "qwen2.5:1.5b-instruct",
    "qwen2.5:3b-instruct",
    "llama3.2:3b-instruct",
    "phi3:medium",
    "mistral:7b-instruct",
    "llama3.1:8b-instruct",
]

# Fallback if no candidates available
# "skip" = run without Junior (reliability capped at 60%)
# "block" = refuse to answer until Junior available
fallback = "skip"

# Maximum reliability score when Junior is unavailable
no_junior_max_reliability = 60

[weights]
# Scoring weights for model selection (must sum to 1.0)
latency = 0.3
throughput = 0.2
quality = 0.4
memory_fit = 0.1
"#;

/// Role-specific policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePolicy {
    /// Description of the role
    pub description: String,
    /// Maximum latency in milliseconds
    pub max_latency_ms: u64,
    /// Minimum quality tier
    pub min_quality_tier: String,
    /// Maximum tokens per request
    pub max_tokens: u32,
    /// Candidate models in preference order
    pub candidates: Vec<String>,
    /// Fallback behavior
    pub fallback: String,
    /// Max reliability when role unavailable (Junior only)
    #[serde(default)]
    pub no_junior_max_reliability: u8,
}

impl Default for RolePolicy {
    fn default() -> Self {
        Self {
            description: String::new(),
            max_latency_ms: 2000,
            min_quality_tier: "low".to_string(),
            max_tokens: 512,
            candidates: Vec::new(),
            fallback: "deterministic".to_string(),
            no_junior_max_reliability: 60,
        }
    }
}

/// Global policy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPolicy {
    /// Allow automatic model downloads
    #[serde(default = "default_true")]
    pub auto_download: bool,
    /// Re-benchmark when hardware changes
    #[serde(default = "default_true")]
    pub rebenchmark_on_hardware_change: bool,
    /// Benchmark cache validity in hours
    #[serde(default = "default_benchmark_hours")]
    pub benchmark_cache_hours: u64,
}

fn default_true() -> bool {
    true
}
fn default_benchmark_hours() -> u64 {
    168
}

impl Default for GlobalPolicy {
    fn default() -> Self {
        Self {
            auto_download: true,
            rebenchmark_on_hardware_change: true,
            benchmark_cache_hours: 168,
        }
    }
}

/// Scoring weights for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub latency: f32,
    pub throughput: f32,
    pub quality: f32,
    pub memory_fit: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            latency: 0.3,
            throughput: 0.2,
            quality: 0.4,
            memory_fit: 0.1,
        }
    }
}

/// Complete model policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsPolicy {
    pub global: GlobalPolicy,
    pub translator: RolePolicy,
    pub junior: RolePolicy,
    pub weights: ScoringWeights,
}

impl Default for ModelsPolicy {
    fn default() -> Self {
        Self {
            global: GlobalPolicy::default(),
            translator: RolePolicy {
                description: "Fast intent planner".to_string(),
                max_latency_ms: 2000,
                min_quality_tier: "low".to_string(),
                max_tokens: 512,
                candidates: vec![
                    "qwen2.5:0.5b".to_string(),
                    "qwen2.5:1.5b".to_string(),
                    "llama3.2:1b".to_string(),
                ],
                fallback: "deterministic".to_string(),
                no_junior_max_reliability: 60,
            },
            junior: RolePolicy {
                description: "Reliable verifier".to_string(),
                max_latency_ms: 5000,
                min_quality_tier: "medium".to_string(),
                max_tokens: 1024,
                candidates: vec![
                    "qwen2.5:1.5b-instruct".to_string(),
                    "qwen2.5:3b-instruct".to_string(),
                    "llama3.2:3b-instruct".to_string(),
                ],
                fallback: "skip".to_string(),
                no_junior_max_reliability: 60,
            },
            weights: ScoringWeights::default(),
        }
    }
}

impl ModelsPolicy {
    /// Load policy from file or return default
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(MODELS_POLICY_FILE) {
            if let Ok(policy) = toml::from_str(&content) {
                return policy;
            }
        }
        Self::default()
    }

    /// Save default policy to file
    pub fn save_default() -> std::io::Result<()> {
        let dir = Path::new(MODELS_POLICY_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        fs::write(MODELS_POLICY_FILE, DEFAULT_MODELS_POLICY)
    }

    /// Get role policy
    pub fn get_role_policy(&self, role: &str) -> &RolePolicy {
        match role {
            "translator" => &self.translator,
            "junior" => &self.junior,
            _ => &self.translator,
        }
    }

    /// Check if auto-download is enabled
    pub fn auto_download_enabled(&self) -> bool {
        self.global.auto_download
    }

    /// Get max reliability when Junior is unavailable
    pub fn no_junior_reliability_cap(&self) -> u8 {
        self.junior.no_junior_max_reliability
    }
}

/// Model download progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model: String,
    pub role: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: u64,
    pub started_at: u64,
    pub status: DownloadStatus,
}

/// Download status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadStatus::Pending => write!(f, "pending"),
            DownloadStatus::Downloading => write!(f, "downloading"),
            DownloadStatus::Completed => write!(f, "completed"),
            DownloadStatus::Failed => write!(f, "failed"),
        }
    }
}

impl DownloadProgress {
    pub fn new(model: &str, role: &str) -> Self {
        Self {
            model: model.to_string(),
            role: role.to_string(),
            total_bytes: 0,
            downloaded_bytes: 0,
            speed_bytes_per_sec: 0,
            eta_seconds: 0,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: DownloadStatus::Pending,
        }
    }

    pub fn percent_complete(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
        }
    }

    pub fn format_progress(&self) -> String {
        let percent = self.percent_complete();
        let downloaded_mb = self.downloaded_bytes as f64 / (1024.0 * 1024.0);
        let total_mb = self.total_bytes as f64 / (1024.0 * 1024.0);
        let speed_mb = self.speed_bytes_per_sec as f64 / (1024.0 * 1024.0);

        if self.status == DownloadStatus::Downloading {
            format!(
                "{:.1}% ({:.1}/{:.1} MB) @ {:.1} MB/s - ETA {}",
                percent,
                downloaded_mb,
                total_mb,
                speed_mb,
                format_eta(self.eta_seconds)
            )
        } else {
            format!("{:?}", self.status)
        }
    }
}

fn format_eta(seconds: u64) -> String {
    if seconds == 0 {
        return "calculating...".to_string();
    }
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// Model readiness state for case files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelReadinessState {
    pub translator_ready: bool,
    pub translator_model: Option<String>,
    pub translator_fallback_active: bool,
    pub junior_ready: bool,
    pub junior_model: Option<String>,
    pub junior_fallback_active: bool,
    pub reliability_capped: bool,
    pub reliability_cap: Option<u8>,
    pub downloads_in_progress: Vec<DownloadProgress>,
    pub benchmark_timestamp: Option<u64>,
    pub hardware_hash: String,
}

impl Default for ModelReadinessState {
    fn default() -> Self {
        Self {
            translator_ready: false,
            translator_model: None,
            translator_fallback_active: false,
            junior_ready: false,
            junior_model: None,
            junior_fallback_active: false,
            reliability_capped: false,
            reliability_cap: None,
            downloads_in_progress: Vec::new(),
            benchmark_timestamp: None,
            hardware_hash: String::new(),
        }
    }
}

impl ModelReadinessState {
    /// Create from current bootstrap state
    pub fn from_bootstrap(
        translator: Option<&str>,
        junior: Option<&str>,
        downloads: Vec<DownloadProgress>,
        benchmark_ts: Option<u64>,
        hw_hash: &str,
    ) -> Self {
        let junior_ready = junior.is_some();
        Self {
            translator_ready: translator.is_some(),
            translator_model: translator.map(|s| s.to_string()),
            translator_fallback_active: translator.is_none(),
            junior_ready,
            junior_model: junior.map(|s| s.to_string()),
            junior_fallback_active: !junior_ready,
            reliability_capped: !junior_ready,
            reliability_cap: if junior_ready { None } else { Some(60) },
            downloads_in_progress: downloads,
            benchmark_timestamp: benchmark_ts,
            hardware_hash: hw_hash.to_string(),
        }
    }

    /// Check if any models are downloading
    pub fn has_active_downloads(&self) -> bool {
        self.downloads_in_progress
            .iter()
            .any(|d| d.status == DownloadStatus::Downloading)
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if self.translator_ready {
            parts.push(format!(
                "Translator: {} (ready)",
                self.translator_model.as_deref().unwrap_or("unknown")
            ));
        } else if self.translator_fallback_active {
            parts.push("Translator: deterministic fallback".to_string());
        } else {
            parts.push("Translator: not ready".to_string());
        }

        if self.junior_ready {
            parts.push(format!(
                "Junior: {} (ready)",
                self.junior_model.as_deref().unwrap_or("unknown")
            ));
        } else if self.junior_fallback_active {
            parts.push(format!(
                "Junior: unavailable (reliability capped at {}%)",
                self.reliability_cap.unwrap_or(60)
            ));
        } else {
            parts.push("Junior: not ready".to_string());
        }

        if self.has_active_downloads() {
            let downloading: Vec<_> = self
                .downloads_in_progress
                .iter()
                .filter(|d| d.status == DownloadStatus::Downloading)
                .map(|d| format!("{} ({})", d.model, d.format_progress()))
                .collect();
            parts.push(format!("Downloading: {}", downloading.join(", ")));
        }

        parts.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = ModelsPolicy::default();
        assert!(policy.global.auto_download);
        assert!(!policy.translator.candidates.is_empty());
        assert!(!policy.junior.candidates.is_empty());
        assert_eq!(policy.junior.no_junior_max_reliability, 60);
    }

    #[test]
    fn test_scoring_weights_sum() {
        let weights = ScoringWeights::default();
        let sum = weights.latency + weights.throughput + weights.quality + weights.memory_fit;
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_download_progress_percent() {
        let mut progress = DownloadProgress::new("test", "translator");
        progress.total_bytes = 1000;
        progress.downloaded_bytes = 500;
        assert!((progress.percent_complete() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(30), "30s");
        assert_eq!(format_eta(90), "1m 30s");
        assert_eq!(format_eta(3700), "1h 1m");
    }

    #[test]
    fn test_model_readiness_summary() {
        let state = ModelReadinessState {
            translator_ready: true,
            translator_model: Some("qwen2.5:0.5b".to_string()),
            translator_fallback_active: false,
            junior_ready: false,
            junior_model: None,
            junior_fallback_active: true,
            reliability_capped: true,
            reliability_cap: Some(60),
            downloads_in_progress: Vec::new(),
            benchmark_timestamp: None,
            hardware_hash: String::new(),
        };

        let summary = state.summary();
        assert!(summary.contains("qwen2.5:0.5b"));
        assert!(summary.contains("60%"));
    }
}

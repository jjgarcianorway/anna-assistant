//! Model Selection and Benchmarking v0.0.5
//!
//! Role-based model selection with hardware awareness and benchmarking.
//!
//! Roles:
//! - Translator: fast, low-latency, short-output intent planner
//! - Junior: slower but more reliable verifier/scorer
//!
//! Selection is based on:
//! - Hardware constraints (RAM, VRAM)
//! - Measured latency (tokens/sec, time-to-first-token)
//! - Benchmark scores per role

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

use crate::ollama::{OllamaClient, OllamaError};

// =============================================================================
// Hardware Profile
// =============================================================================

/// Hardware capability profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// Total system RAM in bytes
    pub total_ram_bytes: u64,
    /// Available RAM in bytes
    pub available_ram_bytes: u64,
    /// CPU core count
    pub cpu_cores: u32,
    /// CPU model name
    pub cpu_model: String,
    /// GPU VRAM in bytes (0 if no GPU or not detected)
    pub gpu_vram_bytes: u64,
    /// GPU model name (empty if none)
    pub gpu_model: String,
    /// Memory budget for LLM (80% of available)
    pub llm_memory_budget_bytes: u64,
    /// Hardware tier: low, medium, high
    pub tier: HardwareTier,
}

/// Hardware tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareTier {
    /// <8GB RAM, no GPU: only tiny models
    Low,
    /// 8-16GB RAM or basic GPU: small models
    Medium,
    /// 16+ GB RAM or good GPU: larger models
    High,
}

impl std::fmt::Display for HardwareTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareTier::Low => write!(f, "low"),
            HardwareTier::Medium => write!(f, "medium"),
            HardwareTier::High => write!(f, "high"),
        }
    }
}

impl HardwareProfile {
    /// Detect hardware profile from system
    pub fn detect() -> Self {
        let mut profile = Self {
            total_ram_bytes: 0,
            available_ram_bytes: 0,
            cpu_cores: 1,
            cpu_model: String::new(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 0,
            tier: HardwareTier::Low,
        };

        // Read /proc/meminfo for RAM
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb_val) = kb.parse::<u64>() {
                            profile.total_ram_bytes = kb_val * 1024;
                        }
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb_val) = kb.parse::<u64>() {
                            profile.available_ram_bytes = kb_val * 1024;
                        }
                    }
                }
            }
        }

        // Read /proc/cpuinfo for CPU
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            let mut cores = 0u32;
            for line in content.lines() {
                if line.starts_with("model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        profile.cpu_model = name.trim().to_string();
                    }
                } else if line.starts_with("processor") {
                    cores += 1;
                }
            }
            profile.cpu_cores = cores.max(1);
        }

        // Try to detect GPU VRAM (nvidia-smi for NVIDIA)
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total,name", "--format=csv,noheader,nounits"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    let parts: Vec<&str> = stdout.trim().split(',').collect();
                    if parts.len() >= 2 {
                        if let Ok(mb) = parts[0].trim().parse::<u64>() {
                            profile.gpu_vram_bytes = mb * 1024 * 1024;
                        }
                        profile.gpu_model = parts[1].trim().to_string();
                    }
                }
            }
        }

        // Calculate LLM memory budget (80% of available RAM or VRAM if present)
        if profile.gpu_vram_bytes > 0 {
            profile.llm_memory_budget_bytes = (profile.gpu_vram_bytes as f64 * 0.8) as u64;
        } else {
            profile.llm_memory_budget_bytes = (profile.available_ram_bytes as f64 * 0.8) as u64;
        }

        // Determine tier
        let effective_memory = if profile.gpu_vram_bytes > 0 {
            profile.gpu_vram_bytes
        } else {
            profile.total_ram_bytes
        };

        profile.tier = if effective_memory >= 16 * 1024 * 1024 * 1024 {
            HardwareTier::High
        } else if effective_memory >= 8 * 1024 * 1024 * 1024 {
            HardwareTier::Medium
        } else {
            HardwareTier::Low
        };

        profile
    }

    /// Format memory as human-readable string
    pub fn format_memory(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{} KB", bytes / 1024)
        }
    }
}

// =============================================================================
// Role-Based Model Candidates
// =============================================================================

/// LLM Role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmRole {
    /// Fast intent planner
    Translator,
    /// Reliable verifier/scorer
    Junior,
}

impl std::fmt::Display for LlmRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmRole::Translator => write!(f, "translator"),
            LlmRole::Junior => write!(f, "junior"),
        }
    }
}

/// Model candidate with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCandidate {
    /// Model name (e.g., "qwen2.5:0.5b")
    pub name: String,
    /// Approximate model size in bytes (for memory estimation)
    pub size_bytes: u64,
    /// Priority for this role (lower = better)
    pub priority: u32,
    /// Minimum hardware tier required
    pub min_tier: HardwareTier,
    /// Description/reason for this model
    pub description: String,
}

/// Default candidate models for each role
pub fn default_candidates(role: LlmRole) -> Vec<ModelCandidate> {
    match role {
        LlmRole::Translator => vec![
            // Translator: smallest, fastest models
            ModelCandidate {
                name: "qwen2.5:0.5b".to_string(),
                size_bytes: 400 * 1024 * 1024,
                priority: 1,
                min_tier: HardwareTier::Low,
                description: "Tiny, very fast - ideal for intent classification".to_string(),
            },
            ModelCandidate {
                name: "qwen2.5:1.5b".to_string(),
                size_bytes: 1 * 1024 * 1024 * 1024,
                priority: 2,
                min_tier: HardwareTier::Low,
                description: "Small but capable for structured output".to_string(),
            },
            ModelCandidate {
                name: "llama3.2:1b".to_string(),
                size_bytes: 1 * 1024 * 1024 * 1024,
                priority: 3,
                min_tier: HardwareTier::Low,
                description: "Meta's compact model".to_string(),
            },
            ModelCandidate {
                name: "phi3:mini".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024,
                priority: 4,
                min_tier: HardwareTier::Medium,
                description: "Microsoft's efficient model".to_string(),
            },
            ModelCandidate {
                name: "gemma2:2b".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024,
                priority: 5,
                min_tier: HardwareTier::Medium,
                description: "Google's compact model".to_string(),
            },
        ],
        LlmRole::Junior => vec![
            // Junior: more capable instruction-following models
            ModelCandidate {
                name: "qwen2.5:1.5b-instruct".to_string(),
                size_bytes: 1 * 1024 * 1024 * 1024,
                priority: 1,
                min_tier: HardwareTier::Low,
                description: "Instruction-tuned, good for verification".to_string(),
            },
            ModelCandidate {
                name: "qwen2.5:3b-instruct".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024,
                priority: 2,
                min_tier: HardwareTier::Medium,
                description: "Stronger instruction following".to_string(),
            },
            ModelCandidate {
                name: "llama3.2:3b-instruct".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024,
                priority: 3,
                min_tier: HardwareTier::Medium,
                description: "Meta's instruction model".to_string(),
            },
            ModelCandidate {
                name: "phi3:medium".to_string(),
                size_bytes: 8 * 1024 * 1024 * 1024,
                priority: 4,
                min_tier: HardwareTier::High,
                description: "Microsoft's larger model".to_string(),
            },
            ModelCandidate {
                name: "mistral:7b-instruct".to_string(),
                size_bytes: 4 * 1024 * 1024 * 1024,
                priority: 5,
                min_tier: HardwareTier::High,
                description: "Mistral's instruction model".to_string(),
            },
            ModelCandidate {
                name: "llama3.1:8b-instruct".to_string(),
                size_bytes: 5 * 1024 * 1024 * 1024,
                priority: 6,
                min_tier: HardwareTier::High,
                description: "Meta's flagship compact model".to_string(),
            },
        ],
    }
}

// =============================================================================
// Benchmark Suite
// =============================================================================

/// Single benchmark case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkCase {
    /// Case ID
    pub id: String,
    /// Input prompt
    pub prompt: String,
    /// System prompt (if any)
    pub system: Option<String>,
    /// Expected keywords/patterns in output
    pub expected_patterns: Vec<String>,
    /// Maximum acceptable latency (ms)
    pub max_latency_ms: u64,
}

/// Benchmark result for a single case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub case_id: String,
    pub passed: bool,
    pub latency_ms: u64,
    pub tokens_generated: u32,
    pub tokens_per_sec: f64,
    pub output_preview: String,
    pub error: Option<String>,
}

/// Benchmark results for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub model: String,
    pub role: LlmRole,
    pub timestamp: u64,
    pub cases: Vec<CaseResult>,
    pub total_cases: u32,
    pub passed_cases: u32,
    pub avg_latency_ms: f64,
    pub avg_tokens_per_sec: f64,
    pub score: f64, // 0-100
}

impl BenchmarkResults {
    /// Calculate overall score (0-100)
    pub fn calculate_score(&mut self) {
        if self.total_cases == 0 {
            self.score = 0.0;
            return;
        }

        let pass_rate = self.passed_cases as f64 / self.total_cases as f64;
        // Latency bonus: faster is better, cap at 2000ms
        let latency_score = (1.0 - (self.avg_latency_ms / 2000.0).min(1.0)) * 0.3;
        // Pass rate is primary (70%)
        self.score = (pass_rate * 70.0 + latency_score * 100.0).min(100.0);
    }
}

/// Translator benchmark cases (intent classification + plan formatting)
pub fn translator_benchmark_cases() -> Vec<BenchmarkCase> {
    vec![
        // System queries
        BenchmarkCase {
            id: "t01".to_string(),
            prompt: "Classify: 'what CPU do I have?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "cpu".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t02".to_string(),
            prompt: "Classify: 'how much RAM is free?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "memory".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t03".to_string(),
            prompt: "Classify: 'show disk usage'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "disk".to_string()],
            max_latency_ms: 2000,
        },
        // Action requests
        BenchmarkCase {
            id: "t04".to_string(),
            prompt: "Classify: 'install nginx'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string(), "nginx".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t05".to_string(),
            prompt: "Classify: 'restart docker'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string(), "docker".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t06".to_string(),
            prompt: "Classify: 'delete all temp files'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string(), "high".to_string()],
            max_latency_ms: 2000,
        },
        // Questions
        BenchmarkCase {
            id: "t07".to_string(),
            prompt: "Classify: 'what is systemd?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["question".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t08".to_string(),
            prompt: "Classify: 'is my battery healthy?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "battery".to_string()],
            max_latency_ms: 2000,
        },
        // Network queries
        BenchmarkCase {
            id: "t09".to_string(),
            prompt: "Classify: 'what is my IP address?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "network".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t10".to_string(),
            prompt: "Classify: 'list running services'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "service".to_string()],
            max_latency_ms: 2000,
        },
        // Package management
        BenchmarkCase {
            id: "t11".to_string(),
            prompt: "Classify: 'update all packages'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t12".to_string(),
            prompt: "Classify: 'what packages are installed?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "package".to_string()],
            max_latency_ms: 2000,
        },
        // Process queries
        BenchmarkCase {
            id: "t13".to_string(),
            prompt: "Classify: 'what process is using port 8080?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t14".to_string(),
            prompt: "Classify: 'kill process firefox'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string(), "firefox".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t15".to_string(),
            prompt: "Classify: 'show top memory consumers'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "memory".to_string()],
            max_latency_ms: 2000,
        },
        // Logs and errors
        BenchmarkCase {
            id: "t16".to_string(),
            prompt: "Classify: 'show recent errors'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t17".to_string(),
            prompt: "Classify: 'why did my system crash?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        // File operations
        BenchmarkCase {
            id: "t18".to_string(),
            prompt: "Classify: 'find large files'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t19".to_string(),
            prompt: "Classify: 'clean package cache'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t20".to_string(),
            prompt: "Classify: 'backup my home folder'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string()],
            max_latency_ms: 2000,
        },
        // GPU/Display
        BenchmarkCase {
            id: "t21".to_string(),
            prompt: "Classify: 'what GPU do I have?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "gpu".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t22".to_string(),
            prompt: "Classify: 'check GPU temperature'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "temperature".to_string()],
            max_latency_ms: 2000,
        },
        // Security
        BenchmarkCase {
            id: "t23".to_string(),
            prompt: "Classify: 'show failed login attempts'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t24".to_string(),
            prompt: "Classify: 'list open ports'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "port".to_string()],
            max_latency_ms: 2000,
        },
        // Boot/uptime
        BenchmarkCase {
            id: "t25".to_string(),
            prompt: "Classify: 'when did I last reboot?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t26".to_string(),
            prompt: "Classify: 'what is my uptime?'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string()],
            max_latency_ms: 2000,
        },
        // Unknown/ambiguous
        BenchmarkCase {
            id: "t27".to_string(),
            prompt: "Classify: 'hello'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["unknown".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t28".to_string(),
            prompt: "Classify: 'thanks'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["unknown".to_string()],
            max_latency_ms: 2000,
        },
        // Compound queries
        BenchmarkCase {
            id: "t29".to_string(),
            prompt: "Classify: 'show CPU and memory usage'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["system_query".to_string(), "cpu".to_string(), "memory".to_string()],
            max_latency_ms: 2000,
        },
        BenchmarkCase {
            id: "t30".to_string(),
            prompt: "Classify: 'install and configure nginx'".to_string(),
            system: Some("Output only: INTENT: [type] TARGETS: [list] RISK: [level]".to_string()),
            expected_patterns: vec!["action".to_string(), "nginx".to_string()],
            max_latency_ms: 2000,
        },
    ]
}

/// Junior benchmark cases (critique quality + evidence handling)
pub fn junior_benchmark_cases() -> Vec<BenchmarkCase> {
    vec![
        // Missing evidence detection
        BenchmarkCase {
            id: "j01".to_string(),
            prompt: "Verify: User asked 'what CPU?' Anna answered 'Intel Core i7'. Evidence: NONE. Score 0-100 and explain.".to_string(),
            system: Some("You verify responses. Flag missing evidence. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["missing".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j02".to_string(),
            prompt: "Verify: User asked 'disk usage?' Anna answered '50GB free'. Evidence: /dev/sda1: 50GB free. Score 0-100.".to_string(),
            system: Some("You verify responses. Flag missing evidence. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["evidence".to_string()],
            max_latency_ms: 5000,
        },
        // Speculation detection
        BenchmarkCase {
            id: "j03".to_string(),
            prompt: "Verify: User asked 'why slow?' Anna answered 'Probably RAM issue'. Evidence: NONE. Score 0-100.".to_string(),
            system: Some("You verify responses. Flag speculation. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["specul".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j04".to_string(),
            prompt: "Verify: User asked 'nginx running?' Anna answered 'Yes'. Evidence: systemctl shows nginx active. Score 0-100.".to_string(),
            system: Some("You verify responses. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["0-9".to_string()],
            max_latency_ms: 5000,
        },
        // Action safety
        BenchmarkCase {
            id: "j05".to_string(),
            prompt: "Verify: User asked 'delete temp'. Anna answered 'Will delete /tmp/*'. Risk: high. Score and warn about confirmation.".to_string(),
            system: Some("You verify responses. For risky actions, require confirmation. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["confirm".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j06".to_string(),
            prompt: "Verify: User asked 'restart nginx'. Anna answered 'Will restart'. Risk: low. Score 0-100.".to_string(),
            system: Some("You verify responses. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["0-9".to_string()],
            max_latency_ms: 5000,
        },
        // Accuracy check
        BenchmarkCase {
            id: "j07".to_string(),
            prompt: "Verify: User asked 'RAM?' Anna answered '16GB'. Evidence: MemTotal: 16384000 kB. Score 0-100.".to_string(),
            system: Some("You verify responses. Check evidence matches answer. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["0-9".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j08".to_string(),
            prompt: "Verify: User asked 'RAM?' Anna answered '32GB'. Evidence: MemTotal: 16384000 kB. Score 0-100.".to_string(),
            system: Some("You verify responses. Check evidence matches answer. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["incorrect".to_string()],
            max_latency_ms: 5000,
        },
        // No invention
        BenchmarkCase {
            id: "j09".to_string(),
            prompt: "Verify: User asked 'GPU temp?' Anna answered 'I cannot determine GPU temperature without data'. Evidence: NONE. Score 0-100.".to_string(),
            system: Some("You verify responses. Honest uncertainty is good. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["honest".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j10".to_string(),
            prompt: "Verify: User asked 'GPU temp?' Anna answered '65C'. Evidence: NONE. Score 0-100.".to_string(),
            system: Some("You verify responses. Flag invented data. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["invent".to_string()],
            max_latency_ms: 5000,
        },
        // Citation quality
        BenchmarkCase {
            id: "j11".to_string(),
            prompt: "Verify: User asked 'uptime?' Anna answered '5 days'. Evidence: uptime shows '5 days, 3:42'. Score 0-100.".to_string(),
            system: Some("You verify responses. Good citations get higher scores. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["0-9".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j12".to_string(),
            prompt: "Verify: User asked 'kernel?' Anna answered 'Linux 6.6.1'. Evidence: uname -r: 6.6.1-arch1-1. Score 0-100.".to_string(),
            system: Some("You verify responses. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["0-9".to_string()],
            max_latency_ms: 5000,
        },
        // Partial evidence
        BenchmarkCase {
            id: "j13".to_string(),
            prompt: "Verify: User asked 'top process?' Anna answered 'Firefox using 2GB RAM'. Evidence: top shows firefox at 2.1GB. Score 0-100.".to_string(),
            system: Some("You verify responses. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["0-9".to_string()],
            max_latency_ms: 5000,
        },
        // Read-only vs write
        BenchmarkCase {
            id: "j14".to_string(),
            prompt: "Verify: User asked 'list packages'. Anna answered 'Here are packages...'. Risk: read-only. Score 0-100.".to_string(),
            system: Some("You verify responses. Read-only is safe. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["safe".to_string()],
            max_latency_ms: 5000,
        },
        BenchmarkCase {
            id: "j15".to_string(),
            prompt: "Verify: User asked 'format disk'. Anna answered 'Will format /dev/sda'. Risk: critical. Score and require explicit confirmation.".to_string(),
            system: Some("You verify responses. Critical actions need explicit confirmation. Output: SCORE: [0-100] CRITIQUE: [text]".to_string()),
            expected_patterns: vec!["confirm".to_string()],
            max_latency_ms: 5000,
        },
    ]
}

/// Run benchmark for a model
pub async fn run_benchmark(
    client: &OllamaClient,
    model: &str,
    role: LlmRole,
) -> Result<BenchmarkResults, OllamaError> {
    let cases = match role {
        LlmRole::Translator => translator_benchmark_cases(),
        LlmRole::Junior => junior_benchmark_cases(),
    };

    let mut results = BenchmarkResults {
        model: model.to_string(),
        role,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        cases: Vec::new(),
        total_cases: cases.len() as u32,
        passed_cases: 0,
        avg_latency_ms: 0.0,
        avg_tokens_per_sec: 0.0,
        score: 0.0,
    };

    let mut total_latency = 0u64;
    let mut total_tps = 0.0;

    for case in cases {
        let start = Instant::now();

        let case_result = match client.generate(model, &case.prompt, case.system.as_deref()).await {
            Ok(resp) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let output_lower = resp.response.to_lowercase();

                // Check if expected patterns are present
                let passed = case.expected_patterns.iter().any(|p| {
                    if p.contains("-") && p.parse::<i32>().is_err() {
                        // Regex-like pattern
                        output_lower.contains(&p.to_lowercase())
                    } else {
                        output_lower.contains(&p.to_lowercase())
                    }
                }) && latency_ms <= case.max_latency_ms;

                let tokens_per_sec = if resp.eval_duration > 0 {
                    (resp.eval_count as f64 * 1_000_000_000.0) / resp.eval_duration as f64
                } else {
                    0.0
                };

                CaseResult {
                    case_id: case.id,
                    passed,
                    latency_ms,
                    tokens_generated: resp.eval_count,
                    tokens_per_sec,
                    output_preview: resp.response.chars().take(100).collect(),
                    error: None,
                }
            }
            Err(e) => CaseResult {
                case_id: case.id,
                passed: false,
                latency_ms: start.elapsed().as_millis() as u64,
                tokens_generated: 0,
                tokens_per_sec: 0.0,
                output_preview: String::new(),
                error: Some(e.to_string()),
            },
        };

        if case_result.passed {
            results.passed_cases += 1;
        }
        total_latency += case_result.latency_ms;
        total_tps += case_result.tokens_per_sec;
        results.cases.push(case_result);
    }

    if !results.cases.is_empty() {
        results.avg_latency_ms = total_latency as f64 / results.cases.len() as f64;
        results.avg_tokens_per_sec = total_tps / results.cases.len() as f64;
    }

    results.calculate_score();
    Ok(results)
}

// =============================================================================
// Model Selection
// =============================================================================

/// Model selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    /// Role this selection is for
    pub role: LlmRole,
    /// Selected model name
    pub model: String,
    /// Human-readable reason for selection
    pub reason: String,
    /// Benchmark results (if run)
    pub benchmark: Option<BenchmarkResults>,
    /// Hardware profile used for selection
    pub hardware_tier: HardwareTier,
    /// Timestamp of selection
    pub timestamp: u64,
}

/// Select best model for a role based on hardware and available models
pub fn select_model_for_role(
    role: LlmRole,
    hardware: &HardwareProfile,
    available_models: &[String],
    candidates: &[ModelCandidate],
) -> Option<ModelSelection> {
    // Filter candidates by hardware tier and availability
    let mut viable: Vec<_> = candidates
        .iter()
        .filter(|c| c.min_tier as u8 <= hardware.tier as u8)
        .filter(|c| c.size_bytes <= hardware.llm_memory_budget_bytes)
        .filter(|c| {
            // Check if this specific model variant is available
            // e.g., qwen2.5:0.5b should match qwen2.5:0.5b or qwen2.5:0.5b-instruct
            // but NOT qwen2.5:14b-instruct
            let parts: Vec<&str> = c.name.split(':').collect();
            if parts.len() == 2 {
                let base = parts[0];
                let size = parts[1].split('-').next().unwrap_or(parts[1]);
                available_models.iter().any(|m| {
                    // Exact match or prefix match with same size
                    m == &c.name || m.starts_with(&c.name) ||
                    (m.starts_with(base) && m.contains(&format!(":{}", size)))
                })
            } else {
                // No size specifier - match any variant
                available_models.iter().any(|m| m.starts_with(&c.name))
            }
        })
        .collect();

    // Sort by priority
    viable.sort_by_key(|c| c.priority);

    // Select first viable
    viable.first().map(|c| {
        // Find best matching model from available:
        // 1. Exact match (qwen2.5:0.5b == qwen2.5:0.5b)
        // 2. Exact with suffix (qwen2.5:0.5b matches qwen2.5:0.5b-instruct)
        // 3. Fall back to candidate name if available
        let model = available_models
            .iter()
            // First try exact match
            .find(|m| *m == &c.name)
            .or_else(|| {
                // Then try candidate as prefix (qwen2.5:0.5b matches qwen2.5:0.5b-instruct)
                available_models.iter().find(|m| m.starts_with(&c.name))
            })
            .or_else(|| {
                // Finally, match base:size pattern more strictly
                // e.g., qwen2.5:0.5b should only match qwen2.5:0.5b*, not qwen2.5:14b*
                let parts: Vec<&str> = c.name.split(':').collect();
                if parts.len() == 2 {
                    let base = parts[0];
                    let size = parts[1];
                    available_models.iter().find(|m| {
                        m.starts_with(base) && m.contains(&format!(":{}", size.split('-').next().unwrap_or(size)))
                    })
                } else {
                    None
                }
            })
            .cloned()
            .unwrap_or_else(|| c.name.clone());

        ModelSelection {
            role,
            model,
            reason: format!(
                "Selected {} for {}: {} (tier: {}, fits in {})",
                c.name,
                role,
                c.description,
                hardware.tier,
                HardwareProfile::format_memory(hardware.llm_memory_budget_bytes)
            ),
            benchmark: None,
            hardware_tier: hardware.tier,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    })
}

// =============================================================================
// Bootstrap State
// =============================================================================

/// Bootstrap phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootstrapPhase {
    /// Detecting Ollama installation
    DetectingOllama,
    /// Installing Ollama (Arch only)
    InstallingOllama,
    /// Pulling required models
    PullingModels,
    /// Running benchmarks
    Benchmarking,
    /// Ready to use
    Ready,
    /// Error state
    Error,
}

impl std::fmt::Display for BootstrapPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapPhase::DetectingOllama => write!(f, "detecting_ollama"),
            BootstrapPhase::InstallingOllama => write!(f, "installing_ollama"),
            BootstrapPhase::PullingModels => write!(f, "pulling_models"),
            BootstrapPhase::Benchmarking => write!(f, "benchmarking"),
            BootstrapPhase::Ready => write!(f, "ready"),
            BootstrapPhase::Error => write!(f, "error"),
        }
    }
}

/// Model download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model: String,
    /// v0.0.35: Role this model is being downloaded for
    #[serde(default)]
    pub role: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: Option<u64>,
    pub status: String,
}

impl DownloadProgress {
    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    pub fn format_eta(&self) -> String {
        match self.eta_seconds {
            Some(secs) if secs < 60 => format!("{}s", secs),
            Some(secs) if secs < 3600 => format!("{}m {}s", secs / 60, secs % 60),
            Some(secs) => format!("{}h {}m", secs / 3600, (secs % 3600) / 60),
            None => "calculating...".to_string(),
        }
    }

    pub fn format_speed(&self) -> String {
        if self.speed_bytes_per_sec >= 1024.0 * 1024.0 {
            format!("{:.1} MB/s", self.speed_bytes_per_sec / (1024.0 * 1024.0))
        } else if self.speed_bytes_per_sec >= 1024.0 {
            format!("{:.1} KB/s", self.speed_bytes_per_sec / 1024.0)
        } else {
            format!("{:.0} B/s", self.speed_bytes_per_sec)
        }
    }
}

/// Full bootstrap state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapState {
    /// Current phase
    pub phase: BootstrapPhase,
    /// Hardware profile
    pub hardware: Option<HardwareProfile>,
    /// Translator model selection
    pub translator: Option<ModelSelection>,
    /// Junior model selection
    pub junior: Option<ModelSelection>,
    /// Current download progress (if pulling)
    pub download_progress: Option<DownloadProgress>,
    /// Last error message
    pub error: Option<String>,
    /// Last update timestamp
    pub last_update: u64,
}

impl Default for BootstrapState {
    fn default() -> Self {
        Self {
            phase: BootstrapPhase::DetectingOllama,
            hardware: None,
            translator: None,
            junior: None,
            download_progress: None,
            error: None,
            last_update: 0,
        }
    }
}

impl BootstrapState {
    /// State file path
    pub fn state_path() -> PathBuf {
        PathBuf::from(crate::DATA_DIR).join("internal/bootstrap_state.json")
    }

    /// Load state from file
    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    /// Save state to file
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let path_str = path.to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        crate::atomic_write(path_str, &content)
    }

    /// Check if ready for requests
    pub fn is_ready(&self) -> bool {
        self.phase == BootstrapPhase::Ready
            && self.translator.is_some()
            && self.junior.is_some()
    }

    /// Get translator model if ready
    pub fn get_translator_model(&self) -> Option<&str> {
        self.translator.as_ref().map(|s| s.model.as_str())
    }

    /// Get junior model if ready
    pub fn get_junior_model(&self) -> Option<&str> {
        self.junior.as_ref().map(|s| s.model.as_str())
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_tier_from_memory() {
        // Test tier classification
        assert_eq!(HardwareTier::Low as u8, 0);
        assert_eq!(HardwareTier::Medium as u8, 1);
        assert_eq!(HardwareTier::High as u8, 2);
    }

    #[test]
    fn test_default_translator_candidates() {
        let candidates = default_candidates(LlmRole::Translator);
        assert!(!candidates.is_empty());
        assert!(candidates[0].priority < candidates.last().unwrap().priority);
    }

    #[test]
    fn test_default_junior_candidates() {
        let candidates = default_candidates(LlmRole::Junior);
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.name.contains("instruct")));
    }

    #[test]
    fn test_translator_benchmark_cases() {
        let cases = translator_benchmark_cases();
        assert_eq!(cases.len(), 30);
    }

    #[test]
    fn test_junior_benchmark_cases() {
        let cases = junior_benchmark_cases();
        assert_eq!(cases.len(), 15);
    }

    #[test]
    fn test_model_selection_matches_size_variant() {
        // Critical: qwen2.5:0.5b should NOT match qwen2.5:14b-instruct
        let hardware = HardwareProfile {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            available_ram_bytes: 24 * 1024 * 1024 * 1024,
            cpu_cores: 16,
            cpu_model: "Test CPU".to_string(),
            gpu_vram_bytes: 8 * 1024 * 1024 * 1024,
            gpu_model: "Test GPU".to_string(),
            llm_memory_budget_bytes: 8 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };

        let candidates = vec![
            ModelCandidate {
                name: "qwen2.5:0.5b".to_string(),
                size_bytes: 400 * 1024 * 1024,
                priority: 1,
                min_tier: HardwareTier::Low,
                description: "Tiny model".to_string(),
            },
        ];

        // Only qwen2.5:14b-instruct available (NOT 0.5b)
        let available = vec!["qwen2.5:14b-instruct".to_string()];

        let selection = select_model_for_role(
            LlmRole::Translator,
            &hardware,
            &available,
            &candidates,
        );

        // Should NOT select anything because 0.5b isn't available
        assert!(selection.is_none(), "Should not match 14b when requesting 0.5b");
    }

    #[test]
    fn test_model_selection_matches_instruct_variant() {
        // qwen2.5:0.5b should match qwen2.5:0.5b-instruct
        let hardware = HardwareProfile {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            available_ram_bytes: 24 * 1024 * 1024 * 1024,
            cpu_cores: 16,
            cpu_model: "Test CPU".to_string(),
            gpu_vram_bytes: 8 * 1024 * 1024 * 1024,
            gpu_model: "Test GPU".to_string(),
            llm_memory_budget_bytes: 8 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };

        let candidates = vec![
            ModelCandidate {
                name: "qwen2.5:0.5b".to_string(),
                size_bytes: 400 * 1024 * 1024,
                priority: 1,
                min_tier: HardwareTier::Low,
                description: "Tiny model".to_string(),
            },
        ];

        // 0.5b-instruct available (variant of 0.5b)
        let available = vec!["qwen2.5:0.5b-instruct".to_string()];

        let selection = select_model_for_role(
            LlmRole::Translator,
            &hardware,
            &available,
            &candidates,
        );

        assert!(selection.is_some());
        assert_eq!(selection.unwrap().model, "qwen2.5:0.5b-instruct");
    }

    #[test]
    fn test_model_selection_respects_tier() {
        let hardware = HardwareProfile {
            total_ram_bytes: 4 * 1024 * 1024 * 1024,
            available_ram_bytes: 3 * 1024 * 1024 * 1024,
            cpu_cores: 4,
            cpu_model: "Test CPU".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 2 * 1024 * 1024 * 1024,
            tier: HardwareTier::Low,
        };

        let candidates = default_candidates(LlmRole::Translator);
        let available = vec!["qwen2.5:0.5b".to_string(), "mistral:7b".to_string()];

        let selection = select_model_for_role(
            LlmRole::Translator,
            &hardware,
            &available,
            &candidates,
        );

        assert!(selection.is_some());
        let sel = selection.unwrap();
        // Should pick the smaller model for low tier
        assert!(sel.model.contains("qwen2.5:0.5b"));
    }

    #[test]
    fn test_download_progress_format() {
        let progress = DownloadProgress {
            model: "test".to_string(),
            role: "translator".to_string(),
            total_bytes: 1000,
            downloaded_bytes: 500,
            speed_bytes_per_sec: 100.0,
            eta_seconds: Some(5),
            status: "downloading".to_string(),
        };

        assert_eq!(progress.percent(), 50.0);
        assert_eq!(progress.format_eta(), "5s");
    }

    #[test]
    fn test_bootstrap_state_default() {
        let state = BootstrapState::default();
        assert_eq!(state.phase, BootstrapPhase::DetectingOllama);
        assert!(!state.is_ready());
    }

    // =============================================================================
    // Hardware Bucketing Tests (v0.0.5)
    // =============================================================================

    #[test]
    fn test_hardware_tier_low() {
        // Less than 8GB RAM should be Low tier
        let hardware = HardwareProfile {
            total_ram_bytes: 6 * 1024 * 1024 * 1024, // 6GB
            available_ram_bytes: 4 * 1024 * 1024 * 1024,
            cpu_cores: 4,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 3 * 1024 * 1024 * 1024,
            tier: HardwareTier::Low,
        };
        assert_eq!(hardware.tier, HardwareTier::Low);
    }

    #[test]
    fn test_hardware_tier_medium() {
        // 8-16GB RAM should be Medium tier
        let hardware = HardwareProfile {
            total_ram_bytes: 12 * 1024 * 1024 * 1024, // 12GB
            available_ram_bytes: 8 * 1024 * 1024 * 1024,
            cpu_cores: 8,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 6 * 1024 * 1024 * 1024,
            tier: HardwareTier::Medium,
        };
        assert_eq!(hardware.tier, HardwareTier::Medium);
    }

    #[test]
    fn test_hardware_tier_high() {
        // More than 16GB RAM should be High tier
        let hardware = HardwareProfile {
            total_ram_bytes: 32 * 1024 * 1024 * 1024, // 32GB
            available_ram_bytes: 24 * 1024 * 1024 * 1024,
            cpu_cores: 16,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 8 * 1024 * 1024 * 1024, // 8GB VRAM
            gpu_model: "RTX 3070".to_string(),
            llm_memory_budget_bytes: 16 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };
        assert_eq!(hardware.tier, HardwareTier::High);
    }

    #[test]
    fn test_hardware_tier_boundary_8gb() {
        // Exactly 8GB should be Medium tier
        let hardware = HardwareProfile {
            total_ram_bytes: 8 * 1024 * 1024 * 1024,
            available_ram_bytes: 6 * 1024 * 1024 * 1024,
            cpu_cores: 4,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 4 * 1024 * 1024 * 1024,
            tier: HardwareTier::Medium,
        };
        assert_eq!(hardware.tier, HardwareTier::Medium);
    }

    #[test]
    fn test_hardware_tier_boundary_16gb() {
        // Exactly 16GB should be High tier
        let hardware = HardwareProfile {
            total_ram_bytes: 16 * 1024 * 1024 * 1024,
            available_ram_bytes: 12 * 1024 * 1024 * 1024,
            cpu_cores: 8,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 8 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };
        assert_eq!(hardware.tier, HardwareTier::High);
    }

    // =============================================================================
    // Model Selection Tests (v0.0.5)
    // =============================================================================

    #[test]
    fn test_model_selection_prefers_higher_priority() {
        let hardware = HardwareProfile {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            available_ram_bytes: 24 * 1024 * 1024 * 1024,
            cpu_cores: 16,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 16 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };

        // Both models available, should pick higher priority (lower number)
        let candidates = vec![
            ModelCandidate {
                name: "model-a".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024,
                priority: 0, // Higher priority
                min_tier: HardwareTier::Low,
                description: "Test A".to_string(),
            },
            ModelCandidate {
                name: "model-b".to_string(),
                size_bytes: 4 * 1024 * 1024 * 1024,
                priority: 1, // Lower priority
                min_tier: HardwareTier::Low,
                description: "Test B".to_string(),
            },
        ];
        let available = vec!["model-a".to_string(), "model-b".to_string()];

        let selection = select_model_for_role(LlmRole::Translator, &hardware, &available, &candidates);
        assert!(selection.is_some());
        assert_eq!(selection.unwrap().model, "model-a");
    }

    #[test]
    fn test_model_selection_fallback_when_preferred_unavailable() {
        let hardware = HardwareProfile {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            available_ram_bytes: 24 * 1024 * 1024 * 1024,
            cpu_cores: 16,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 16 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };

        let candidates = vec![
            ModelCandidate {
                name: "model-a".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024,
                priority: 0,
                min_tier: HardwareTier::Low,
                description: "Test A".to_string(),
            },
            ModelCandidate {
                name: "model-b".to_string(),
                size_bytes: 4 * 1024 * 1024 * 1024,
                priority: 1,
                min_tier: HardwareTier::Low,
                description: "Test B".to_string(),
            },
        ];
        // Only model-b available
        let available = vec!["model-b".to_string()];

        let selection = select_model_for_role(LlmRole::Translator, &hardware, &available, &candidates);
        assert!(selection.is_some());
        assert_eq!(selection.unwrap().model, "model-b");
    }

    #[test]
    fn test_model_selection_respects_min_tier() {
        let hardware = HardwareProfile {
            total_ram_bytes: 6 * 1024 * 1024 * 1024,
            available_ram_bytes: 4 * 1024 * 1024 * 1024,
            cpu_cores: 4,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 3 * 1024 * 1024 * 1024,
            tier: HardwareTier::Low,
        };

        let candidates = vec![
            ModelCandidate {
                name: "model-high".to_string(),
                size_bytes: 8 * 1024 * 1024 * 1024,
                priority: 0,
                min_tier: HardwareTier::High, // Requires High tier
                description: "High tier only".to_string(),
            },
            ModelCandidate {
                name: "model-low".to_string(),
                size_bytes: 1 * 1024 * 1024 * 1024,
                priority: 1,
                min_tier: HardwareTier::Low, // Works on Low tier
                description: "Low tier ok".to_string(),
            },
        ];
        let available = vec!["model-high".to_string(), "model-low".to_string()];

        let selection = select_model_for_role(LlmRole::Translator, &hardware, &available, &candidates);
        assert!(selection.is_some());
        // Should pick model-low since hardware is Low tier
        assert_eq!(selection.unwrap().model, "model-low");
    }

    #[test]
    fn test_model_selection_none_when_no_suitable() {
        let hardware = HardwareProfile {
            total_ram_bytes: 4 * 1024 * 1024 * 1024,
            available_ram_bytes: 2 * 1024 * 1024 * 1024,
            cpu_cores: 2,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 1 * 1024 * 1024 * 1024,
            tier: HardwareTier::Low,
        };

        let candidates = vec![
            ModelCandidate {
                name: "model-high".to_string(),
                size_bytes: 8 * 1024 * 1024 * 1024,
                priority: 0,
                min_tier: HardwareTier::High,
                description: "High tier only".to_string(),
            },
        ];
        let available = vec!["model-high".to_string()];

        let selection = select_model_for_role(LlmRole::Translator, &hardware, &available, &candidates);
        // Should be None because model requires High tier but hardware is Low
        assert!(selection.is_none());
    }

    #[test]
    fn test_model_selection_none_when_no_models_available() {
        let hardware = HardwareProfile {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            available_ram_bytes: 24 * 1024 * 1024 * 1024,
            cpu_cores: 16,
            cpu_model: "Test".to_string(),
            gpu_vram_bytes: 0,
            gpu_model: String::new(),
            llm_memory_budget_bytes: 16 * 1024 * 1024 * 1024,
            tier: HardwareTier::High,
        };

        let candidates = default_candidates(LlmRole::Translator);
        let available: Vec<String> = vec![]; // No models installed

        let selection = select_model_for_role(LlmRole::Translator, &hardware, &available, &candidates);
        assert!(selection.is_none());
    }

    #[test]
    fn test_format_memory() {
        // format_memory shows KB for < 1MB (512 bytes = 0 KB due to integer division)
        assert_eq!(HardwareProfile::format_memory(512), "0 KB");
        assert_eq!(HardwareProfile::format_memory(1024), "1 KB");
        assert_eq!(HardwareProfile::format_memory(2048), "2 KB");
        assert_eq!(HardwareProfile::format_memory(1024 * 1024), "1.0 MB");
        assert_eq!(HardwareProfile::format_memory(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(HardwareProfile::format_memory(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn test_bootstrap_phases() {
        // Verify all phases have string representation
        assert_eq!(BootstrapPhase::DetectingOllama.to_string(), "detecting_ollama");
        assert_eq!(BootstrapPhase::InstallingOllama.to_string(), "installing_ollama");
        assert_eq!(BootstrapPhase::PullingModels.to_string(), "pulling_models");
        assert_eq!(BootstrapPhase::Benchmarking.to_string(), "benchmarking");
        assert_eq!(BootstrapPhase::Ready.to_string(), "ready");
        assert_eq!(BootstrapPhase::Error.to_string(), "error");
    }

    #[test]
    fn test_bootstrap_state_is_ready() {
        let mut state = BootstrapState::default();
        assert!(!state.is_ready());

        // Phase Ready but no models selected - not ready
        state.phase = BootstrapPhase::Ready;
        assert!(!state.is_ready());

        // Add translator - still not ready (need junior too)
        state.translator = Some(ModelSelection {
            role: LlmRole::Translator,
            model: "test".to_string(),
            reason: "test".to_string(),
            benchmark: None,
            hardware_tier: HardwareTier::Low,
            timestamp: 0,
        });
        assert!(!state.is_ready());

        // Add junior - now ready
        state.junior = Some(ModelSelection {
            role: LlmRole::Junior,
            model: "test".to_string(),
            reason: "test".to_string(),
            benchmark: None,
            hardware_tier: HardwareTier::Low,
            timestamp: 0,
        });
        assert!(state.is_ready());

        // Error phase - not ready even with models
        state.phase = BootstrapPhase::Error;
        assert!(!state.is_ready());
    }
}

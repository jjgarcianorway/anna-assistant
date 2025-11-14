//! Hardware Capability Detection for Local LLM
//!
//! Determines if the machine can run a local LLM effectively based on
//! RAM, CPU cores, and optionally GPU presence.

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// Hardware capability assessment for local LLM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmCapability {
    /// High capability: 16+ GB RAM, 4+ cores
    /// Recommended: Local LLM with medium models
    High,

    /// Medium capability: 8-16 GB RAM, 2-4 cores
    /// Possible: Local LLM with small models, may be slower
    Medium,

    /// Low capability: < 8 GB RAM or < 2 cores
    /// Not recommended: Better to use remote LLM or degraded mode
    Low,
}

impl LlmCapability {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            LlmCapability::High => {
                "Your machine has excellent resources for running a local language model"
            }
            LlmCapability::Medium => {
                "Your machine can run a local language model, though it may be slower"
            }
            LlmCapability::Low => {
                "Your machine has limited resources - a local model may struggle"
            }
        }
    }

    /// Get recommendation for user
    pub fn recommendation(&self) -> &'static str {
        match self {
            LlmCapability::High => {
                "I strongly recommend setting up a local model for privacy and performance"
            }
            LlmCapability::Medium => {
                "I can set up a small local model for you, or you can use a remote API"
            }
            LlmCapability::Low => {
                "I recommend using a remote API, or running without LLM assistance"
            }
        }
    }

    /// Check if local LLM is recommended
    pub fn is_local_recommended(&self) -> bool {
        matches!(self, LlmCapability::High | LlmCapability::Medium)
    }
}

/// Hardware assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareAssessment {
    /// Total RAM in MB
    pub total_ram_mb: u64,

    /// Number of CPU cores
    pub cpu_cores: usize,

    /// GPU detected (optional, for future use)
    pub has_gpu: Option<bool>,

    /// Overall LLM capability
    pub llm_capability: LlmCapability,

    /// When this assessment was made
    pub assessed_at: chrono::DateTime<chrono::Utc>,
}

impl HardwareAssessment {
    /// Perform hardware assessment
    pub fn assess() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram_mb = sys.total_memory() / 1024; // Convert KB to MB
        let cpu_cores = sys.cpus().len();

        // GPU detection is optional for now - just mark as unknown
        let has_gpu = None;

        // Classify capability
        let llm_capability = if total_ram_mb >= 16000 && cpu_cores >= 4 {
            LlmCapability::High
        } else if total_ram_mb >= 8000 && cpu_cores >= 2 {
            LlmCapability::Medium
        } else {
            LlmCapability::Low
        };

        Self {
            total_ram_mb,
            cpu_cores,
            has_gpu,
            llm_capability,
            assessed_at: chrono::Utc::now(),
        }
    }

    /// Get a summary suitable for user display
    pub fn summary(&self) -> String {
        format!(
            "{:.1} GB RAM, {} CPU cores",
            self.total_ram_mb as f64 / 1024.0,
            self.cpu_cores
        )
    }

    /// Get detailed description
    pub fn detailed_description(&self) -> String {
        let mut desc = format!(
            "System: {:.1} GB RAM, {} CPU cores",
            self.total_ram_mb as f64 / 1024.0,
            self.cpu_cores
        );

        if let Some(true) = self.has_gpu {
            desc.push_str(", GPU detected");
        }

        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_classification() {
        // High capability
        assert_eq!(
            if 32000 >= 16000 && 8 >= 4 {
                LlmCapability::High
            } else {
                LlmCapability::Low
            },
            LlmCapability::High
        );

        // Medium capability
        assert_eq!(
            if 12000 >= 16000 && 4 >= 4 {
                LlmCapability::High
            } else if 12000 >= 8000 && 4 >= 2 {
                LlmCapability::Medium
            } else {
                LlmCapability::Low
            },
            LlmCapability::Medium
        );

        // Low capability
        assert_eq!(
            if 4000 >= 16000 && 2 >= 4 {
                LlmCapability::High
            } else if 4000 >= 8000 && 2 >= 2 {
                LlmCapability::Medium
            } else {
                LlmCapability::Low
            },
            LlmCapability::Low
        );
    }

    #[test]
    fn test_hardware_assessment() {
        let assessment = HardwareAssessment::assess();

        // Should have some RAM and cores
        assert!(assessment.total_ram_mb > 0);
        assert!(assessment.cpu_cores > 0);

        // Summary should be formatted correctly
        let summary = assessment.summary();
        assert!(summary.contains("GB RAM"));
        assert!(summary.contains("CPU cores"));
    }

    #[test]
    fn test_capability_recommendations() {
        assert!(LlmCapability::High.is_local_recommended());
        assert!(LlmCapability::Medium.is_local_recommended());
        assert!(!LlmCapability::Low.is_local_recommended());
    }

    #[test]
    fn test_descriptions() {
        let high = LlmCapability::High;
        assert!(!high.description().is_empty());
        assert!(!high.recommendation().is_empty());
    }
}

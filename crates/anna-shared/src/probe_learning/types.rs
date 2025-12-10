//! Probe learning types (v0.0.331).
//!
//! Core data structures for probe effectiveness tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Probe effectiveness record for a specific query category
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbeEffectiveness {
    /// Number of times this probe was used for this category
    pub uses: u32,
    /// Number of times the answer was marked helpful
    pub helpful: u32,
    /// Number of times the answer was marked not helpful
    pub not_helpful: u32,
    /// Number of times the probe command failed (non-zero exit)
    pub failures: u32,
    /// Computed effectiveness score (0.0 - 1.0)
    pub score: f32,
}

impl ProbeEffectiveness {
    /// Calculate effectiveness score based on usage stats
    pub fn compute_score(&mut self) {
        if self.uses == 0 {
            self.score = 0.5; // Neutral for unused probes
            return;
        }

        // Base score from helpful/not helpful ratio
        let total_feedback = self.helpful + self.not_helpful;
        let feedback_score = if total_feedback > 0 {
            self.helpful as f32 / total_feedback as f32
        } else {
            0.5 // Neutral if no feedback
        };

        // Penalty for failures
        let failure_rate = self.failures as f32 / self.uses as f32;
        let failure_penalty = 1.0 - (failure_rate * 0.5); // Max 50% penalty

        // Confidence boost for more uses (bayesian-ish)
        let confidence = (self.uses as f32 / 10.0).min(1.0);

        // Blend neutral prior with observed score based on confidence
        self.score = (0.5 * (1.0 - confidence) + feedback_score * confidence) * failure_penalty;
    }
}

/// Query category for grouping probe effectiveness
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryCategory {
    /// System health (CPU, memory, processes)
    SystemHealth,
    /// Disk and storage
    Storage,
    /// Network and connectivity
    Network,
    /// Hardware info (GPU, USB, PCI)
    Hardware,
    /// Security and permissions
    Security,
    /// Packages and software
    Packages,
    /// Services and systemd
    Services,
    /// Graphics and display
    Graphics,
    /// General/other
    General,
}

impl QueryCategory {
    /// Infer category from domain string
    pub fn from_domain(domain: &str) -> Self {
        match domain.to_lowercase().as_str() {
            "system" => Self::SystemHealth,
            "storage" => Self::Storage,
            "network" => Self::Network,
            "security" => Self::Security,
            "packages" => Self::Packages,
            _ => Self::General,
        }
    }

    /// Infer category from query keywords
    pub fn from_query(query: &str) -> Self {
        let q = query.to_lowercase();

        // Graphics/display queries
        if q.contains("gpu") || q.contains("graphics") || q.contains("display")
            || q.contains("vaapi") || q.contains("vdpau") || q.contains("vulkan")
            || q.contains("hardware acceleration") || q.contains("video acceleration")
            || q.contains("render") {
            return Self::Graphics;
        }

        // Hardware queries
        if q.contains("usb") || q.contains("pci") || q.contains("bluetooth")
            || q.contains("printer") || q.contains("audio") || q.contains("sound") {
            return Self::Hardware;
        }

        // Network queries
        if q.contains("network") || q.contains("wifi") || q.contains("ethernet")
            || q.contains("ip address") || q.contains("dns") || q.contains("ping") {
            return Self::Network;
        }

        // Storage queries
        if q.contains("disk") || q.contains("storage") || q.contains("space")
            || q.contains("mount") || q.contains("partition") {
            return Self::Storage;
        }

        // Security queries
        if q.contains("firewall") || q.contains("permission") || q.contains("security")
            || q.contains("user") || q.contains("group") {
            return Self::Security;
        }

        // Package queries
        if q.contains("package") || q.contains("install") || q.contains("update")
            || q.contains("pacman") || q.contains("apt") || q.contains("dnf") {
            return Self::Packages;
        }

        // Service queries
        if q.contains("service") || q.contains("systemd") || q.contains("daemon")
            || q.contains("running") && q.contains("process") {
            return Self::Services;
        }

        // System health queries
        if q.contains("cpu") || q.contains("memory") || q.contains("ram")
            || q.contains("process") || q.contains("load") || q.contains("uptime") {
            return Self::SystemHealth;
        }

        Self::General
    }
}

/// Stats for keyword-probe associations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeywordProbeStats {
    /// Probes that worked well for this keyword
    pub effective_probes: HashMap<String, u32>,
    /// Total times this keyword appeared in successful queries
    pub success_count: u32,
}

/// A successful query pattern for positive learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessfulPattern {
    /// Keywords extracted from the query
    pub keywords: Vec<String>,
    /// Probes that were used successfully
    pub probes: Vec<String>,
    /// Quality score (1-5 or reliability-based)
    pub quality: u8,
    /// Category
    pub category: QueryCategory,
    /// Timestamp
    pub timestamp: u64,
}

/// A pattern that led to a poor answer (for learning what NOT to do)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativePattern {
    /// The query that got a bad answer
    pub query: String,
    /// The category it was assigned
    pub category: QueryCategory,
    /// Probes that were used
    pub probes_used: Vec<String>,
    /// Why the answer was bad (from user/LLM feedback)
    pub failure_reason: String,
    /// Timestamp
    pub timestamp: u64,
}

/// v0.0.331: Quality trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDataPoint {
    /// Timestamp (Unix seconds)
    pub timestamp: u64,
    /// Average quality in this period
    pub avg_quality: f32,
    /// Number of queries in this period
    pub query_count: u32,
}

/// Learning statistics for display
#[derive(Debug, Clone)]
pub struct LearningStats {
    pub total_queries: usize,
    pub successful_patterns: usize,
    pub negative_patterns: usize,
    pub keywords_learned: usize,
    pub categories_with_data: usize,
    pub avg_quality: f32,
}

/// Result of applying decay
#[derive(Debug, Clone)]
pub struct DecayResult {
    pub applied: bool,
    pub patterns_removed: usize,
    pub keywords_decayed: usize,
    pub probes_decayed: usize,
}

impl DecayResult {
    pub fn skipped() -> Self {
        Self {
            applied: false,
            patterns_removed: 0,
            keywords_decayed: 0,
            probes_decayed: 0,
        }
    }
}

/// v0.0.331: Quality trend summary
#[derive(Debug, Clone)]
pub struct QualityTrend {
    /// Current average quality (last 7 days)
    pub current_avg: f32,
    /// Previous average quality (7-14 days ago)
    pub previous_avg: f32,
    /// Trend direction: positive, negative, or stable
    pub trend: TrendDirection,
    /// Change amount
    pub change: f32,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::Improving => write!(f, "improving"),
            TrendDirection::Declining => write!(f, "declining"),
            TrendDirection::Stable => write!(f, "stable"),
        }
    }
}

/// v0.0.332: Learning health status
#[derive(Debug, Clone, PartialEq)]
pub enum LearningHealth {
    /// High confidence, good quality
    Excellent,
    /// Adequate data and quality
    Good,
    /// Usable but not ideal
    Developing,
    /// Quality declining, needs review
    NeedsAttention,
    /// Not enough data
    Insufficient,
}

impl std::fmt::Display for LearningHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LearningHealth::Excellent => write!(f, "excellent"),
            LearningHealth::Good => write!(f, "good"),
            LearningHealth::Developing => write!(f, "developing"),
            LearningHealth::NeedsAttention => write!(f, "needs attention"),
            LearningHealth::Insufficient => write!(f, "insufficient"),
        }
    }
}

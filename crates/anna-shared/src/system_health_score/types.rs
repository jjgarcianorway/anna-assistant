//! Core types for system health scoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health grade from A to F
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HealthGrade {
    A,
    B,
    C,
    D,
    F,
}

impl HealthGrade {
    /// Display string
    pub fn display(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }

    /// Description of the grade
    pub fn description(&self) -> &'static str {
        match self {
            Self::A => "Excellent",
            Self::B => "Good",
            Self::C => "Fair",
            Self::D => "Poor",
            Self::F => "Critical",
        }
    }

    /// From numeric score (0-100)
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::A,
            80..=89 => Self::B,
            70..=79 => Self::C,
            60..=69 => Self::D,
            _ => Self::F,
        }
    }

    /// To numeric score midpoint
    pub fn to_score(&self) -> u8 {
        match self {
            Self::A => 95,
            Self::B => 85,
            Self::C => 75,
            Self::D => 65,
            Self::F => 40,
        }
    }
}

/// Category of health metric
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCategory {
    /// CPU usage
    Cpu,
    /// Memory usage
    Memory,
    /// Disk usage
    Disk,
    /// System services
    Services,
    /// Network connectivity
    Network,
    /// Security
    Security,
    /// Updates
    Updates,
    /// Anna daemon
    Daemon,
}

impl HealthCategory {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Disk => "Disk",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Updates => "Updates",
            Self::Daemon => "Daemon",
        }
    }

    /// Weight for overall score (out of 100)
    pub fn weight(&self) -> u8 {
        match self {
            Self::Cpu => 15,
            Self::Memory => 15,
            Self::Disk => 20,
            Self::Services => 15,
            Self::Network => 10,
            Self::Security => 10,
            Self::Updates => 5,
            Self::Daemon => 10,
        }
    }
}

/// Individual health metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    /// Category
    pub category: HealthCategory,
    /// Score (0-100)
    pub score: u8,
    /// Current value (for display)
    pub value: String,
    /// Threshold that triggered the score
    pub threshold: Option<String>,
    /// Recommendation if score is low
    pub recommendation: Option<String>,
}

impl HealthMetric {
    /// Create a new health metric
    pub fn new(category: HealthCategory, score: u8, value: impl Into<String>) -> Self {
        Self {
            category,
            score: score.min(100),
            value: value.into(),
            threshold: None,
            recommendation: None,
        }
    }

    /// Set recommendation
    pub fn with_recommendation(mut self, recommendation: impl Into<String>) -> Self {
        self.recommendation = Some(recommendation.into());
        self
    }

    /// Get grade for this metric
    pub fn grade(&self) -> HealthGrade {
        HealthGrade::from_score(self.score)
    }
}

/// Overall system health score
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemHealthScore {
    /// Individual metrics
    pub metrics: HashMap<String, HealthMetric>,
    /// Overall score (0-100)
    pub overall_score: u8,
    /// Overall grade
    pub overall_grade: String,
    /// Critical issues count
    pub critical_issues: u32,
    /// Warning count
    pub warnings: u32,
    /// Last check timestamp
    pub last_check: u64,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl SystemHealthScore {
    /// Create a new empty health score
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a metric
    pub fn add_metric(&mut self, metric: HealthMetric) {
        // Track issues
        if metric.score < 60 {
            self.critical_issues += 1;
        } else if metric.score < 80 {
            self.warnings += 1;
        }

        // Add recommendation if present
        if let Some(ref rec) = metric.recommendation {
            if metric.score < 80 {
                self.recommendations.push(rec.clone());
            }
        }

        self.metrics.insert(metric.category.display().to_string(), metric);
    }

    /// Calculate overall score from metrics
    pub fn calculate_overall(&mut self) {
        if self.metrics.is_empty() {
            self.overall_score = 0;
            self.overall_grade = "N/A".to_string();
            return;
        }

        let mut weighted_sum: u32 = 0;
        let mut total_weight: u32 = 0;

        for metric in self.metrics.values() {
            let weight = metric.category.weight() as u32;
            weighted_sum += metric.score as u32 * weight;
            total_weight += weight;
        }

        if total_weight > 0 {
            self.overall_score = (weighted_sum / total_weight) as u8;
        } else {
            self.overall_score = 0;
        }

        self.overall_grade = HealthGrade::from_score(self.overall_score).display().to_string();
    }

    /// Get metrics below threshold
    pub fn issues(&self) -> Vec<&HealthMetric> {
        self.metrics.values().filter(|m| m.score < 80).collect()
    }

    /// Get critical metrics
    pub fn critical(&self) -> Vec<&HealthMetric> {
        self.metrics.values().filter(|m| m.score < 60).collect()
    }

    /// Is the system healthy?
    pub fn is_healthy(&self) -> bool {
        self.overall_score >= 80 && self.critical_issues == 0
    }
}

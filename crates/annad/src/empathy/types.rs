//! Empathy subsystem type definitions
//!
//! Phase 1.2: Contextual awareness and human-centered design
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::sentinel::SentinelAction;

/// Empathy state persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyState {
    /// State version for migration tracking
    pub version: u64,
    /// State snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Current empathy index (0.0-1.0)
    pub empathy_index: f64,
    /// Current strain index (0.0-1.0)
    pub strain_index: f64,
    /// Recent perception records
    pub perception_history: Vec<PerceptionRecord>,
    /// Stakeholder resonance map
    pub resonance_map: ResonanceMap,
    /// Context metrics
    pub context_metrics: ContextMetrics,
}

impl Default for EmpathyState {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: Utc::now(),
            empathy_index: 0.5,
            strain_index: 0.0,
            perception_history: Vec::new(),
            resonance_map: ResonanceMap::default(),
            context_metrics: ContextMetrics::default(),
        }
    }
}

/// Record of a perception analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionRecord {
    /// When perception was recorded
    pub timestamp: DateTime<Utc>,
    /// Action being perceived
    pub action: SentinelAction,
    /// Stakeholder impacts
    pub stakeholder_impacts: StakeholderImpacts,
    /// Contextual factors detected
    pub context_factors: Vec<String>,
    /// Adaptive decision made
    pub adaptation: Option<String>,
}

/// Impacts on different stakeholders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderImpacts {
    /// Impact on user (disruption, cognitive load, etc.)
    pub user: StakeholderImpact,
    /// Impact on system (resource usage, stability, etc.)
    pub system: StakeholderImpact,
    /// Impact on environment (network, dependencies, etc.)
    pub environment: StakeholderImpact,
}

impl Default for StakeholderImpacts {
    fn default() -> Self {
        Self {
            user: StakeholderImpact::default(),
            system: StakeholderImpact::default(),
            environment: StakeholderImpact::default(),
        }
    }
}

/// Impact assessment for a stakeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderImpact {
    /// Impact score (0.0 = no impact, 1.0 = high impact)
    pub score: f64,
    /// Type of impact (disruption, resource, cognitive, etc.)
    pub impact_type: String,
    /// Reasoning for this assessment
    pub reasoning: String,
}

impl Default for StakeholderImpact {
    fn default() -> Self {
        Self {
            score: 0.0,
            impact_type: "none".to_string(),
            reasoning: "No significant impact detected".to_string(),
        }
    }
}

/// Resonance map - how well Anna resonates with each stakeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceMap {
    /// User resonance (0.0-1.0)
    pub user_resonance: f64,
    /// System resonance (0.0-1.0)
    pub system_resonance: f64,
    /// Environment resonance (0.0-1.0)
    pub environment_resonance: f64,
    /// Recent adjustments made
    pub recent_adjustments: Vec<ResonanceAdjustment>,
}

impl Default for ResonanceMap {
    fn default() -> Self {
        Self {
            user_resonance: 0.5,
            system_resonance: 0.5,
            environment_resonance: 0.5,
            recent_adjustments: Vec::new(),
        }
    }
}

/// Record of a resonance adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceAdjustment {
    /// When adjustment was made
    pub timestamp: DateTime<Utc>,
    /// Stakeholder affected
    pub stakeholder: String,
    /// Adjustment delta
    pub delta: f64,
    /// Reason for adjustment
    pub reason: String,
}

/// Context metrics derived from system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    /// Recent error rate (errors per hour)
    pub error_rate: f64,
    /// CPU load average (0.0-1.0)
    pub cpu_load: f64,
    /// Memory pressure (0.0-1.0)
    pub memory_pressure: f64,
    /// Recent user activity level (0.0-1.0)
    pub user_activity: f64,
    /// Time since last user interaction (seconds)
    pub time_since_user_interaction: u64,
    /// Detected user fatigue indicators
    pub fatigue_indicators: Vec<String>,
}

impl Default for ContextMetrics {
    fn default() -> Self {
        Self {
            error_rate: 0.0,
            cpu_load: 0.0,
            memory_pressure: 0.0,
            user_activity: 0.0,
            time_since_user_interaction: 0,
            fatigue_indicators: Vec::new(),
        }
    }
}

/// Empathy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyConfig {
    /// Enable empathy kernel
    pub enabled: bool,
    /// Empathy sensitivity level (0.0-1.0)
    pub sensitivity: f64,
    /// Strain threshold for deferring actions (0.0-1.0)
    pub strain_threshold: f64,
    /// Context weights
    pub context_weights: ContextWeights,
    /// Adaptive response settings
    pub adaptive_response: AdaptiveResponseConfig,
}

impl Default for EmpathyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.7,
            strain_threshold: 0.6,
            context_weights: ContextWeights::default(),
            adaptive_response: AdaptiveResponseConfig::default(),
        }
    }
}

/// Weights for different contextual factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWeights {
    /// Weight for error rate (0.0-1.0)
    pub error_rate_weight: f64,
    /// Weight for CPU load (0.0-1.0)
    pub cpu_load_weight: f64,
    /// Weight for memory pressure (0.0-1.0)
    pub memory_pressure_weight: f64,
    /// Weight for user activity (0.0-1.0)
    pub user_activity_weight: f64,
    /// Weight for fatigue indicators (0.0-1.0)
    pub fatigue_weight: f64,
}

impl Default for ContextWeights {
    fn default() -> Self {
        Self {
            error_rate_weight: 0.3,
            cpu_load_weight: 0.2,
            memory_pressure_weight: 0.2,
            user_activity_weight: 0.2,
            fatigue_weight: 0.1,
        }
    }
}

/// Adaptive response configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveResponseConfig {
    /// Enable adaptive tone
    pub adaptive_tone: bool,
    /// Enable adaptive pacing
    pub adaptive_pacing: bool,
    /// Empathy threshold for tone adjustment (0.0-1.0)
    pub tone_threshold: f64,
    /// Delay multiplier when strain detected (1.0-5.0)
    pub strain_delay_multiplier: f64,
}

impl Default for AdaptiveResponseConfig {
    fn default() -> Self {
        Self {
            adaptive_tone: true,
            adaptive_pacing: true,
            tone_threshold: 0.6,
            strain_delay_multiplier: 2.0,
        }
    }
}

/// Sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    /// Overall sentiment score (-1.0 to 1.0)
    pub sentiment_score: f64,
    /// Token entropy (complexity indicator)
    pub token_entropy: f64,
    /// Anomaly delta from baseline
    pub anomaly_delta: f64,
    /// Detected patterns
    pub patterns: Vec<String>,
}

impl Default for SentimentAnalysis {
    fn default() -> Self {
        Self {
            sentiment_score: 0.0,
            token_entropy: 0.0,
            anomaly_delta: 0.0,
            patterns: Vec::new(),
        }
    }
}

/// Empathy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyEvaluation {
    /// Should this action be deferred?
    pub should_defer: bool,
    /// Deferral reason (if applicable)
    pub deferral_reason: Option<String>,
    /// Stakeholder impacts
    pub stakeholder_impacts: StakeholderImpacts,
    /// Contextual factors considered
    pub context_factors: Vec<String>,
    /// Recommended delay (seconds)
    pub recommended_delay: u64,
    /// Adaptive tone suggestion
    pub tone_adaptation: Option<String>,
}

/// Empathy pulse - current state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyPulse {
    /// Current timestamp
    pub timestamp: DateTime<Utc>,
    /// Empathy index
    pub empathy_index: f64,
    /// Strain index
    pub strain_index: f64,
    /// Resonance map
    pub resonance_map: ResonanceMap,
    /// Context summary
    pub context_summary: String,
    /// Recent perceptions (last 10)
    pub recent_perceptions: Vec<PerceptionRecord>,
}

/// Empathy simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathySimulation {
    /// Action being simulated
    pub action: String,
    /// Empathy evaluation
    pub evaluation: EmpathyEvaluation,
    /// Reasoning explanation
    pub reasoning: String,
    /// Would action proceed?
    pub would_proceed: bool,
}

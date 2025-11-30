//! Auto-Tuning Module v1.5.0 - Snow Leopard
//!
//! Provides gentle, data-driven auto-tuning of Anna's internal thresholds
//! based on benchmark results and telemetry data.
//!
//! ## Key Principles
//!
//! - No "black box magic" - all tuning is explainable and visible
//! - Tuning is slow and bounded - small steps only
//! - All changes are reflected in status
//! - Only triggered after successful Full Snow Leopard benchmarks
//!
//! ## What We Tune (v1.5.0)
//!
//! For now, only behavioral thresholds:
//! - Brain confidence threshold (when to trust Brain vs call LLM)
//!
//! We do NOT tune:
//! - Hard timeouts
//! - LLM model selection
//! - Probe execution behavior

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::bench_snow_leopard::{SnowLeopardResult, SkillStats};
use crate::telemetry::TelemetrySummaryComplete;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Path to auto-tune state file
const AUTO_TUNE_STATE_FILE: &str = "/var/lib/anna/knowledge/stats/auto_tune_state.json";

/// Default brain confidence threshold
const DEFAULT_BRAIN_CONF_THRESHOLD: f32 = 0.75;

/// Default LLM confidence threshold (for accepting Junior without Senior)
const DEFAULT_LLM_CONF_THRESHOLD: f32 = 0.65;

/// Auto-tuning configuration (bounds and targets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTuneConfig {
    /// Brain confidence threshold range (min, max)
    pub brain_threshold_range: (f32, f32),
    /// LLM confidence threshold range (min, max)
    pub llm_threshold_range: (f32, f32),
    /// Target maximum brain usage percentage (e.g., 0.70 = 70%)
    pub max_brain_usage_target: Option<f32>,
    /// Minimum success rate required before we tune (e.g., 0.80 = 80%)
    pub min_success_rate_for_tuning: f32,
    /// Maximum latency target in ms
    pub max_latency_target_ms: f32,
    /// Step size for threshold adjustments
    pub adjustment_step: f32,
}

impl Default for AutoTuneConfig {
    fn default() -> Self {
        Self {
            brain_threshold_range: (0.60, 0.95),
            llm_threshold_range: (0.50, 0.85),
            max_brain_usage_target: Some(0.70),
            min_success_rate_for_tuning: 0.80,
            max_latency_target_ms: 5000.0,
            adjustment_step: 0.02,
        }
    }
}

// ============================================================================
// STATE
// ============================================================================

/// Current auto-tuning state (persisted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTuneState {
    /// Current brain confidence threshold
    pub brain_conf_threshold: f32,
    /// Current LLM confidence threshold
    pub llm_conf_threshold: f32,
    /// Timestamp of last tuning
    pub last_tuned_at: Option<String>,
    /// Number of tuning steps applied
    pub tuning_steps_applied: u32,
    /// Last decision explanation
    pub last_decision: Option<String>,
}

impl Default for AutoTuneState {
    fn default() -> Self {
        Self {
            brain_conf_threshold: DEFAULT_BRAIN_CONF_THRESHOLD,
            llm_conf_threshold: DEFAULT_LLM_CONF_THRESHOLD,
            last_tuned_at: None,
            tuning_steps_applied: 0,
            last_decision: None,
        }
    }
}

impl AutoTuneState {
    /// Load state from disk (or return default if not found)
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string(AUTO_TUNE_STATE_FILE) {
            if let Ok(state) = serde_json::from_str(&data) {
                return state;
            }
        }
        Self::default()
    }

    /// Save state to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = Path::new(AUTO_TUNE_STATE_FILE).parent().unwrap();
        fs::create_dir_all(dir)?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(AUTO_TUNE_STATE_FILE, json)
    }

    /// Check if auto-tuning has ever been applied
    pub fn has_been_tuned(&self) -> bool {
        self.tuning_steps_applied > 0
    }

    /// Format for status display
    pub fn format_for_status(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "  Brain confidence threshold: {:.2}\n",
            self.brain_conf_threshold
        ));
        s.push_str(&format!(
            "  LLM confidence threshold:   {:.2}\n",
            self.llm_conf_threshold
        ));
        s.push_str(&format!("  Tuning steps applied:       {}\n", self.tuning_steps_applied));
        if let Some(ts) = &self.last_tuned_at {
            s.push_str(&format!("  Last tuned: {}\n", ts));
        }
        if let Some(decision) = &self.last_decision {
            s.push_str(&format!("\n  Last decision:\n    \"{}\"\n", decision));
        }
        s
    }
}

// ============================================================================
// DECISION
// ============================================================================

/// Describes what the auto-tuning decided to do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTuneDecision {
    /// Whether any change was made
    pub changed: bool,
    /// Which knob was adjusted (if any)
    pub knob_adjusted: Option<String>,
    /// Old value (if changed)
    pub old_value: Option<f32>,
    /// New value (if changed)
    pub new_value: Option<f32>,
    /// Human-readable explanation
    pub explanation: String,
    /// Reason for no change (if no change)
    pub no_change_reason: Option<String>,
}

impl AutoTuneDecision {
    /// Create a "no change" decision
    pub fn no_change(reason: &str) -> Self {
        Self {
            changed: false,
            knob_adjusted: None,
            old_value: None,
            new_value: None,
            explanation: format!("No tuning applied: {}", reason),
            no_change_reason: Some(reason.to_string()),
        }
    }

    /// Create a decision that adjusts brain threshold
    pub fn adjust_brain(old: f32, new: f32, explanation: &str) -> Self {
        Self {
            changed: true,
            knob_adjusted: Some("brain_conf_threshold".to_string()),
            old_value: Some(old),
            new_value: Some(new),
            explanation: explanation.to_string(),
            no_change_reason: None,
        }
    }
}

// ============================================================================
// TUNING LOGIC
// ============================================================================

/// Perform auto-tuning based on benchmark results and telemetry
///
/// This function analyzes recent performance data and makes small,
/// bounded adjustments to Anna's behavioral thresholds.
///
/// ## Tuning Rules
///
/// 1. If success rate is below minimum threshold, do nothing (too noisy)
/// 2. If success rate >= 90% AND brain reliability >= 95%, increase brain trust slightly
/// 3. If success rate < 75% OR benchmark shows brain failures, decrease brain trust slightly
/// 4. Always clamp to configured ranges
pub fn auto_tune_from_benchmark(
    telemetry: &TelemetrySummaryComplete,
    bench: &SnowLeopardResult,
    state: &mut AutoTuneState,
    config: &AutoTuneConfig,
) -> AutoTuneDecision {
    let success_rate = (bench.overall_success_rate() / 100.0) as f32; // Convert to 0-1 range
    let brain_usage = (bench.brain_usage_pct() / 100.0) as f32;
    let avg_latency = bench.overall_avg_latency() as f32;

    // Get brain-specific stats from telemetry if available
    let brain_success_rate = if telemetry.brain_stats.count > 0 {
        telemetry.brain_stats.successes as f32 / telemetry.brain_stats.count as f32
    } else {
        0.0
    };

    // Rule 0: Metrics too noisy if success rate below threshold
    if success_rate < config.min_success_rate_for_tuning {
        return AutoTuneDecision::no_change(&format!(
            "Success rate {:.0}% is below minimum {:.0}% required for tuning",
            success_rate * 100.0,
            config.min_success_rate_for_tuning * 100.0
        ));
    }

    // Rule 1: If success rate >= 90% AND brain reliability >= 95%, increase brain trust
    if success_rate >= 0.90 && brain_success_rate >= 0.95 && avg_latency < config.max_latency_target_ms {
        let old_threshold = state.brain_conf_threshold;
        let new_threshold = (old_threshold + config.adjustment_step)
            .min(config.brain_threshold_range.1);

        if new_threshold > old_threshold {
            state.brain_conf_threshold = new_threshold;
            state.tuning_steps_applied += 1;
            state.last_tuned_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

            let explanation = format!(
                "Increased brain confidence threshold from {:.2} to {:.2} because \
                 recent success rate was high ({:.0}%) and brain answers were reliably \
                 correct ({:.0}% brain success rate).",
                old_threshold,
                new_threshold,
                success_rate * 100.0,
                brain_success_rate * 100.0
            );
            state.last_decision = Some(explanation.clone());

            return AutoTuneDecision::adjust_brain(old_threshold, new_threshold, &explanation);
        }
    }

    // Rule 2: If success rate < 75% OR brain failures detected, decrease brain trust
    let brain_failure_detected = brain_success_rate < 0.70 && brain_usage > 0.30;

    if success_rate < 0.75 || brain_failure_detected {
        let old_threshold = state.brain_conf_threshold;
        let new_threshold = (old_threshold - config.adjustment_step)
            .max(config.brain_threshold_range.0);

        if new_threshold < old_threshold {
            state.brain_conf_threshold = new_threshold;
            state.tuning_steps_applied += 1;
            state.last_tuned_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

            let reason = if brain_failure_detected {
                format!("brain reliability was low ({:.0}%)", brain_success_rate * 100.0)
            } else {
                format!("overall success rate was low ({:.0}%)", success_rate * 100.0)
            };

            let explanation = format!(
                "Decreased brain confidence threshold from {:.2} to {:.2} because {}. \
                 Anna will now rely more on Junior/Senior for answers.",
                old_threshold,
                new_threshold,
                reason
            );
            state.last_decision = Some(explanation.clone());

            return AutoTuneDecision::adjust_brain(old_threshold, new_threshold, &explanation);
        }
    }

    // No change needed - performance is in acceptable range
    AutoTuneDecision::no_change(&format!(
        "Performance is stable (success: {:.0}%, brain: {:.0}%, latency: {:.0}ms). \
         No tuning needed.",
        success_rate * 100.0,
        brain_usage * 100.0,
        avg_latency
    ))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_snow_leopard::{
        BenchmarkMode, PhaseId, PhaseResult, QuestionResult, XpSnapshot, XpDelta, SkillStats,
    };
    use std::collections::HashMap;

    fn create_mock_benchmark(success_rate: f64, brain_pct: f64, latency_ms: u64) -> SnowLeopardResult {
        let num_questions = 10;
        let num_success = (success_rate / 100.0 * num_questions as f64) as usize;
        let brain_count = (brain_pct / 100.0 * num_questions as f64) as usize;

        let mut questions = Vec::new();
        for i in 0..num_questions {
            questions.push(QuestionResult {
                question_id: format!("q{}", i),
                question_text: format!("Question {}?", i),
                answer: "Mock answer".to_string(),
                origin: if i < brain_count { "Brain".to_string() } else { "LLM".to_string() },
                reliability: if i < num_success { 0.9 } else { 0.3 },
                latency_ms,
                probes_used: vec![],
                is_success: i < num_success,
                is_fallback: false,
                is_partial: false,
                xp_delta: if i < num_success { 10 } else { -5 },
                skill: "CpuInfo".to_string(),
            });
        }

        let mut origin_summary = HashMap::new();
        origin_summary.insert("Brain".to_string(), brain_count);
        origin_summary.insert("LLM".to_string(), num_questions - brain_count);

        let mut skill_summary = HashMap::new();
        skill_summary.insert("CpuInfo".to_string(), SkillStats {
            count: num_questions,
            success_count: num_success,
            fallback_count: 0,
            avg_latency_ms: latency_ms,
            avg_reliability: success_rate / 100.0,
            total_xp: 100,
        });

        SnowLeopardResult {
            mode: BenchmarkMode::Full,
            timestamp: "2025-11-29T10:00:00".to_string(),
            phases: vec![PhaseResult {
                phase_id: PhaseId::HardReset,
                phase_name: "Hard Reset".to_string(),
                questions,
                xp_before: XpSnapshot::default(),
                xp_after: XpSnapshot::default(),
                total_duration_ms: latency_ms * num_questions as u64,
                new_probes_created: 0,
                fallback_count: 0,
                partial_count: 0,
            }],
            total_questions: num_questions,
            total_xp: XpDelta { anna: 100, junior: 50, senior: 50 },
            total_probes_used: 5,
            latency_evolution: vec![],
            reliability_evolution: vec![],
            success_rate_evolution: vec![],
            origin_summary,
            skill_summary,
            ux_consistency_passed: success_rate >= 80.0,
            report_path: None,
            ascii_summary: "Mock".to_string(),
            warnings: vec![],
        }
    }

    fn create_mock_telemetry(brain_count: u64, brain_successes: u64) -> TelemetrySummaryComplete {
        use crate::telemetry::{TelemetrySummary, OriginStats};
        TelemetrySummaryComplete {
            has_data: true,
            lifetime: TelemetrySummary {
                total: 100,
                successes: 90,
                failures: 8,
                timeouts: 0,
                refusals: 2,
                success_rate: 0.9,
                avg_latency_ms: 300,
                brain_count: 50,
                junior_count: 30,
                senior_count: 20,
                top_failure: None,
                // v3.6.0: New stats fields
                ..Default::default()
            },
            window: TelemetrySummary {
                total: 20,
                successes: 18,
                failures: 2,
                timeouts: 0,
                refusals: 0,
                success_rate: 0.9,
                avg_latency_ms: 250,
                brain_count: 10,
                junior_count: 6,
                senior_count: 4,
                top_failure: None,
                // v3.6.0: New stats fields
                ..Default::default()
            },
            window_size: 20,
            brain_stats: OriginStats {
                count: brain_count,
                successes: brain_successes,
                success_rate: if brain_count > 0 { brain_successes as f64 / brain_count as f64 } else { 0.0 },
                avg_latency_ms: 50,
                min_latency_ms: 20,
                max_latency_ms: 100,
                avg_reliability: 0.95,
                // v3.6.0: New stats fields
                ..Default::default()
            },
            junior_stats: OriginStats {
                count: 5,
                successes: 4,
                success_rate: 0.8,
                avg_latency_ms: 500,
                min_latency_ms: 300,
                max_latency_ms: 800,
                avg_reliability: 0.85,
                // v3.6.0: New stats fields
                ..Default::default()
            },
            senior_stats: OriginStats {
                count: 5,
                successes: 5,
                success_rate: 1.0,
                avg_latency_ms: 800,
                min_latency_ms: 600,
                max_latency_ms: 1200,
                avg_reliability: 0.92,
                // v3.6.0: New stats fields
                ..Default::default()
            },
            status_hint: "Performing well".to_string(),
            // v3.4.0: Rolling stats for performance hints
            rolling_avg_latency_ms: 300,
            rolling_failure_rate: 0.05,
            rolling_timeout_rate: 0.02,
        }
    }

    #[test]
    fn test_auto_tune_no_change_low_success_rate() {
        let config = AutoTuneConfig::default();
        let mut state = AutoTuneState::default();

        // Success rate below threshold
        let bench = create_mock_benchmark(70.0, 50.0, 300);
        let telemetry = create_mock_telemetry(10, 9);

        let decision = auto_tune_from_benchmark(&telemetry, &bench, &mut state, &config);

        assert!(!decision.changed);
        assert!(decision.explanation.contains("below minimum"));
    }

    #[test]
    fn test_auto_tune_increase_brain_threshold() {
        let config = AutoTuneConfig::default();
        let mut state = AutoTuneState::default();
        let initial = state.brain_conf_threshold;

        // High success rate and brain reliability
        let bench = create_mock_benchmark(95.0, 50.0, 300);
        let telemetry = create_mock_telemetry(10, 10); // 100% brain success

        let decision = auto_tune_from_benchmark(&telemetry, &bench, &mut state, &config);

        assert!(decision.changed, "Should have made a change");
        assert_eq!(decision.knob_adjusted, Some("brain_conf_threshold".to_string()));
        assert!(state.brain_conf_threshold > initial);
        assert!(decision.explanation.contains("Increased"));
    }

    #[test]
    fn test_auto_tune_decrease_brain_threshold() {
        let config = AutoTuneConfig::default();
        let mut state = AutoTuneState::default();
        state.brain_conf_threshold = 0.85; // Start high
        let initial = state.brain_conf_threshold;

        // Low brain reliability with high brain usage
        let bench = create_mock_benchmark(85.0, 50.0, 300);
        let telemetry = create_mock_telemetry(10, 5); // 50% brain success (bad)

        let decision = auto_tune_from_benchmark(&telemetry, &bench, &mut state, &config);

        assert!(decision.changed, "Should have made a change");
        assert!(state.brain_conf_threshold < initial);
        assert!(decision.explanation.contains("Decreased"));
    }

    #[test]
    fn test_auto_tune_respects_bounds() {
        let config = AutoTuneConfig::default();
        let mut state = AutoTuneState::default();
        state.brain_conf_threshold = config.brain_threshold_range.1; // At max

        // High success - would normally increase, but already at max
        let bench = create_mock_benchmark(95.0, 50.0, 300);
        let telemetry = create_mock_telemetry(10, 10);

        let _decision = auto_tune_from_benchmark(&telemetry, &bench, &mut state, &config);

        // Should not increase beyond max
        assert!(state.brain_conf_threshold <= config.brain_threshold_range.1);
    }

    #[test]
    fn test_state_persistence() {
        let state = AutoTuneState {
            brain_conf_threshold: 0.80,
            llm_conf_threshold: 0.70,
            last_tuned_at: Some("2025-11-29T10:00:00".to_string()),
            tuning_steps_applied: 5,
            last_decision: Some("Test decision".to_string()),
        };

        let json = serde_json::to_string(&state).unwrap();
        let parsed: AutoTuneState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.brain_conf_threshold, 0.80);
        assert_eq!(parsed.tuning_steps_applied, 5);
    }

    #[test]
    fn test_config_defaults() {
        let config = AutoTuneConfig::default();

        assert!(config.brain_threshold_range.0 < config.brain_threshold_range.1);
        assert!(config.llm_threshold_range.0 < config.llm_threshold_range.1);
        assert!(config.min_success_rate_for_tuning > 0.0);
        assert!(config.adjustment_step > 0.0 && config.adjustment_step < 0.1);
    }
}

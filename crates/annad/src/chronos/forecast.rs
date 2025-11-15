//! Forecast - Probabilistic outcome engine
//!
//! Phase 1.5: Future state simulation using stochastic models
//! Citation: [archwiki:System_maintenance]

use super::timeline::{SystemMetrics, SystemSnapshot, Timeline, Trend, TrendDirection};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Forecast engine for predictive simulation
pub struct ForecastEngine {
    /// Historical timeline for learning
    timeline: Timeline,
    /// Forecast parameters
    config: ForecastConfig,
}

/// Forecast configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastConfig {
    /// Simulation steps per hour
    pub steps_per_hour: usize,
    /// Monte Carlo iterations
    pub monte_carlo_iterations: usize,
    /// Confidence threshold (0.0-1.0)
    pub confidence_threshold: f64,
    /// Maximum forecast horizon (hours)
    pub max_horizon_hours: u64,
}

impl Default for ForecastConfig {
    fn default() -> Self {
        Self {
            steps_per_hour: 4, // 15-minute intervals
            monte_carlo_iterations: 100,
            confidence_threshold: 0.7,
            max_horizon_hours: 168, // 1 week
        }
    }
}

impl ForecastEngine {
    /// Create new forecast engine
    pub fn new(timeline: Timeline, config: ForecastConfig) -> Self {
        Self { timeline, config }
    }

    /// Generate forecast for time window
    pub fn forecast(&self, window_hours: u64) -> ForecastResult {
        info!("Generating forecast for {} hours ahead", window_hours);

        if window_hours > self.config.max_horizon_hours {
            return ForecastResult {
                forecast_id: uuid::Uuid::new_v4().to_string(),
                generated_at: Utc::now(),
                horizon_hours: window_hours,
                scenarios: vec![],
                consensus_scenario: None,
                confidence: 0.0,
                divergence_warnings: vec!["Forecast horizon exceeds maximum".to_string()],
            };
        }

        // Get current state and trend
        let current = match self.timeline.latest() {
            Some(s) => s,
            None => {
                return ForecastResult {
                    forecast_id: uuid::Uuid::new_v4().to_string(),
                    generated_at: Utc::now(),
                    horizon_hours: window_hours,
                    scenarios: vec![],
                    consensus_scenario: None,
                    confidence: 0.0,
                    divergence_warnings: vec!["No historical data available".to_string()],
                };
            }
        };

        let trend = self.timeline.trend(10).unwrap_or_else(|| Trend {
            health: TrendDirection::Stable,
            empathy: TrendDirection::Stable,
            strain: TrendDirection::Stable,
            coherence: TrendDirection::Stable,
            health_velocity: 0.0,
            empathy_velocity: 0.0,
            strain_velocity: 0.0,
            coherence_velocity: 0.0,
        });

        // Run Monte Carlo simulations
        let mut scenarios = Vec::new();
        for i in 0..self.config.monte_carlo_iterations {
            let scenario = self.simulate_scenario(current, &trend, window_hours, i);
            scenarios.push(scenario);
        }

        // Calculate consensus scenario (median of all simulations)
        let consensus = self.calculate_consensus(&scenarios);

        // Calculate overall confidence
        let confidence = self.calculate_confidence(&scenarios, &consensus);

        // Detect divergence warnings
        let warnings = self.detect_divergences(&consensus);

        ForecastResult {
            forecast_id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            horizon_hours: window_hours,
            scenarios,
            consensus_scenario: Some(consensus),
            confidence,
            divergence_warnings: warnings,
        }
    }

    /// Simulate single scenario
    fn simulate_scenario(
        &self,
        start: &SystemSnapshot,
        trend: &Trend,
        window_hours: u64,
        seed: usize,
    ) -> ForecastScenario {
        let steps = (window_hours * self.config.steps_per_hour as u64) as usize;
        let mut snapshots = Vec::new();

        let mut current_metrics = start.metrics.clone();

        // Use seed for deterministic randomness
        let noise_factor = ((seed % 7) as f64 - 3.0) / 10.0; // -0.3 to +0.3

        for step in 0..steps {
            let hours_ahead = (step as f64) / (self.config.steps_per_hour as f64);

            // Apply trend with noise
            current_metrics.health_score = (current_metrics.health_score
                + trend.health_velocity + noise_factor * 0.02)
                .clamp(0.0, 1.0);
            current_metrics.empathy_index = (current_metrics.empathy_index
                + trend.empathy_velocity
                + noise_factor * 0.01)
                .clamp(0.0, 1.0);
            current_metrics.strain_index = (current_metrics.strain_index + trend.strain_velocity
                - noise_factor * 0.01)
                .clamp(0.0, 1.0);
            current_metrics.network_coherence = (current_metrics.network_coherence
                + trend.coherence_velocity
                + noise_factor * 0.01)
                .clamp(0.0, 1.0);

            // Create snapshot
            let snapshot = SystemSnapshot {
                id: format!("forecast_{}_{}", seed, step),
                timestamp: start.timestamp + Duration::hours(hours_ahead as i64),
                metrics: current_metrics.clone(),
                processes_count: start.processes_count,
                memory_usage_mb: start.memory_usage_mb,
                cpu_usage_percent: start.cpu_usage_percent,
                network_bytes_per_sec: start.network_bytes_per_sec,
                service_states: start.service_states.clone(),
                pending_actions: vec![],
            };

            snapshots.push(snapshot);
        }

        ForecastScenario {
            scenario_id: format!("scenario_{}", seed),
            probability: 1.0 / self.config.monte_carlo_iterations as f64,
            snapshots,
        }
    }

    /// Calculate consensus scenario from all simulations
    fn calculate_consensus(&self, scenarios: &[ForecastScenario]) -> ForecastScenario {
        if scenarios.is_empty() {
            return ForecastScenario {
                scenario_id: "consensus".to_string(),
                probability: 0.0,
                snapshots: vec![],
            };
        }

        let num_steps = scenarios[0].snapshots.len();
        let mut consensus_snapshots = Vec::new();

        for step in 0..num_steps {
            // Collect all metrics at this step
            let mut health_scores = Vec::new();
            let mut empathy_indices = Vec::new();
            let mut strain_indices = Vec::new();
            let mut coherences = Vec::new();

            for scenario in scenarios {
                if let Some(snapshot) = scenario.snapshots.get(step) {
                    health_scores.push(snapshot.metrics.health_score);
                    empathy_indices.push(snapshot.metrics.empathy_index);
                    strain_indices.push(snapshot.metrics.strain_index);
                    coherences.push(snapshot.metrics.network_coherence);
                }
            }

            // Calculate median values
            let consensus_metrics = SystemMetrics {
                health_score: median(&health_scores),
                empathy_index: median(&empathy_indices),
                strain_index: median(&strain_indices),
                network_coherence: median(&coherences),
                avg_trust_score: 0.8, // Placeholder
            };

            let template = &scenarios[0].snapshots[step];
            consensus_snapshots.push(SystemSnapshot {
                id: format!("consensus_{}", step),
                timestamp: template.timestamp,
                metrics: consensus_metrics,
                processes_count: template.processes_count,
                memory_usage_mb: template.memory_usage_mb,
                cpu_usage_percent: template.cpu_usage_percent,
                network_bytes_per_sec: template.network_bytes_per_sec,
                service_states: template.service_states.clone(),
                pending_actions: vec![],
            });
        }

        ForecastScenario {
            scenario_id: "consensus".to_string(),
            probability: 1.0,
            snapshots: consensus_snapshots,
        }
    }

    /// Calculate forecast confidence
    fn calculate_confidence(
        &self,
        scenarios: &[ForecastScenario],
        consensus: &ForecastScenario,
    ) -> f64 {
        if scenarios.is_empty() || consensus.snapshots.is_empty() {
            return 0.0;
        }

        // Calculate average deviation from consensus
        let mut total_deviation = 0.0;
        let mut count = 0;

        for scenario in scenarios {
            for (step, snapshot) in scenario.snapshots.iter().enumerate() {
                if let Some(consensus_snapshot) = consensus.snapshots.get(step) {
                    let deviation = (snapshot.metrics.health_score
                        - consensus_snapshot.metrics.health_score)
                        .abs()
                        + (snapshot.metrics.empathy_index
                            - consensus_snapshot.metrics.empathy_index)
                            .abs()
                        + (snapshot.metrics.strain_index - consensus_snapshot.metrics.strain_index)
                            .abs()
                        + (snapshot.metrics.network_coherence
                            - consensus_snapshot.metrics.network_coherence)
                            .abs();

                    total_deviation += deviation;
                    count += 1;
                }
            }
        }

        let avg_deviation = if count > 0 {
            total_deviation / count as f64
        } else {
            0.0
        };

        // Convert deviation to confidence (lower deviation = higher confidence)
        (1.0 - avg_deviation / 4.0).clamp(0.0, 1.0)
    }

    /// Detect ethical divergences in forecast
    fn detect_divergences(&self, consensus: &ForecastScenario) -> Vec<String> {
        let mut warnings = Vec::new();

        if let Some(final_state) = consensus.snapshots.last() {
            // Check for dangerous trends
            if final_state.metrics.health_score < 0.5 {
                warnings.push("Health score projected to drop below 50%".to_string());
            }

            if final_state.metrics.strain_index > 0.7 {
                warnings.push("Strain index projected to exceed 70%".to_string());
            }

            if final_state.metrics.network_coherence < 0.6 {
                warnings.push("Network coherence projected to degrade below 60%".to_string());
            }

            if final_state.metrics.empathy_index < 0.5 {
                warnings.push("Empathy index projected to drop below 50%".to_string());
            }
        }

        debug!("Divergence warnings: {:?}", warnings);
        warnings
    }
}

/// Forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    /// Forecast ID
    pub forecast_id: String,
    /// When forecast was generated
    pub generated_at: DateTime<Utc>,
    /// Forecast horizon (hours)
    pub horizon_hours: u64,
    /// All simulated scenarios
    pub scenarios: Vec<ForecastScenario>,
    /// Consensus scenario (median)
    pub consensus_scenario: Option<ForecastScenario>,
    /// Forecast confidence (0.0-1.0)
    pub confidence: f64,
    /// Divergence warnings
    pub divergence_warnings: Vec<String>,
}

/// Single forecast scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastScenario {
    /// Scenario ID
    pub scenario_id: String,
    /// Probability of this scenario
    pub probability: f64,
    /// Future snapshots
    pub snapshots: Vec<SystemSnapshot>,
}

/// Calculate median of values
fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_calculation() {
        assert_eq!(median(&[1.0, 2.0, 3.0]), 2.0);
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
        assert_eq!(median(&[]), 0.0);
    }

    #[test]
    fn test_forecast_engine_creation() {
        let timeline = Timeline::new("test".to_string(), 100);
        let config = ForecastConfig::default();
        let engine = ForecastEngine::new(timeline, config);

        assert_eq!(engine.config.steps_per_hour, 4);
    }
}

//! Ethics Projection - Predictive moral calculus
//!
//! Phase 1.5: Temporal empathy and ethical trajectory analysis
//! Citation: [archwiki:System_maintenance]

use super::forecast::{ForecastResult, ForecastScenario};
use super::timeline::SystemSnapshot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Ethics projection engine
pub struct EthicsProjector {
    /// Stakeholder weights
    stakeholder_weights: HashMap<String, f64>,
    /// Ethical thresholds
    thresholds: EthicalThresholds,
}

/// Ethical thresholds for warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalThresholds {
    /// Minimum acceptable health
    pub min_health: f64,
    /// Maximum acceptable strain
    pub max_strain: f64,
    /// Minimum network coherence
    pub min_coherence: f64,
    /// Minimum empathy index
    pub min_empathy: f64,
}

impl Default for EthicalThresholds {
    fn default() -> Self {
        Self {
            min_health: 0.6,
            max_strain: 0.7,
            min_coherence: 0.65,
            min_empathy: 0.6,
        }
    }
}

impl EthicsProjector {
    /// Create new ethics projector
    pub fn new() -> Self {
        let mut stakeholder_weights = HashMap::new();
        stakeholder_weights.insert("user".to_string(), 0.4);
        stakeholder_weights.insert("system".to_string(), 0.3);
        stakeholder_weights.insert("network".to_string(), 0.2);
        stakeholder_weights.insert("environment".to_string(), 0.1);

        Self {
            stakeholder_weights,
            thresholds: EthicalThresholds::default(),
        }
    }

    /// Project ethical outcomes for forecast
    pub fn project(&self, forecast: &ForecastResult) -> EthicsProjection {
        info!(
            "Projecting ethical outcomes for forecast {}",
            forecast.forecast_id
        );

        let consensus = match &forecast.consensus_scenario {
            Some(s) => s,
            None => {
                return EthicsProjection {
                    projection_id: uuid::Uuid::new_v4().to_string(),
                    forecast_id: forecast.forecast_id.clone(),
                    generated_at: Utc::now(),
                    temporal_empathy_index: 0.0,
                    stakeholder_impacts: HashMap::new(),
                    ethical_trajectory: EthicalTrajectory::Unknown,
                    intervention_recommendations: vec![
                        "No consensus scenario available".to_string()
                    ],
                    moral_cost: 0.0,
                };
            }
        };

        // Calculate temporal empathy index
        let temporal_empathy = self.calculate_temporal_empathy(consensus);

        // Project stakeholder impacts
        let stakeholder_impacts = self.project_stakeholder_impacts(consensus);

        // Determine ethical trajectory
        let trajectory = self.determine_trajectory(consensus);

        // Generate intervention recommendations
        let recommendations = self.generate_recommendations(consensus, &trajectory);

        // Calculate total moral cost
        let moral_cost = self.calculate_moral_cost(&stakeholder_impacts, &trajectory);

        EthicsProjection {
            projection_id: uuid::Uuid::new_v4().to_string(),
            forecast_id: forecast.forecast_id.clone(),
            generated_at: Utc::now(),
            temporal_empathy_index: temporal_empathy,
            stakeholder_impacts,
            ethical_trajectory: trajectory,
            intervention_recommendations: recommendations,
            moral_cost,
        }
    }

    /// Calculate temporal empathy index
    fn calculate_temporal_empathy(&self, scenario: &ForecastScenario) -> f64 {
        if scenario.snapshots.is_empty() {
            return 0.0;
        }

        // Average empathy across forecast window, weighted by recency
        let mut weighted_empathy = 0.0;
        let mut total_weight = 0.0;

        for (i, snapshot) in scenario.snapshots.iter().enumerate() {
            // Weight increases with time (future empathy weighted more)
            let weight = 1.0 + (i as f64 / scenario.snapshots.len() as f64);
            weighted_empathy += snapshot.metrics.empathy_index * weight;
            total_weight += weight;
        }

        weighted_empathy / total_weight
    }

    /// Project impacts on each stakeholder
    fn project_stakeholder_impacts(
        &self,
        scenario: &ForecastScenario,
    ) -> HashMap<String, StakeholderImpact> {
        let mut impacts = HashMap::new();

        if let (Some(first), Some(last)) = (scenario.snapshots.first(), scenario.snapshots.last()) {
            // User impact (based on empathy and strain)
            impacts.insert(
                "user".to_string(),
                StakeholderImpact {
                    stakeholder: "user".to_string(),
                    impact_score: self.calculate_user_impact(first, last),
                    emotional_cost: last.metrics.strain_index - first.metrics.strain_index,
                    description: "Impact on user experience and emotional well-being".to_string(),
                },
            );

            // System impact (based on health)
            impacts.insert(
                "system".to_string(),
                StakeholderImpact {
                    stakeholder: "system".to_string(),
                    impact_score: last.metrics.health_score - first.metrics.health_score,
                    emotional_cost: 0.0,
                    description: "Impact on system stability and performance".to_string(),
                },
            );

            // Network impact (based on coherence)
            impacts.insert(
                "network".to_string(),
                StakeholderImpact {
                    stakeholder: "network".to_string(),
                    impact_score: last.metrics.network_coherence - first.metrics.network_coherence,
                    emotional_cost: 0.0,
                    description: "Impact on network-wide ethical alignment".to_string(),
                },
            );

            // Environment impact (placeholder - would calculate resource usage)
            impacts.insert(
                "environment".to_string(),
                StakeholderImpact {
                    stakeholder: "environment".to_string(),
                    impact_score: 0.0,
                    emotional_cost: 0.0,
                    description: "Impact on resource consumption and sustainability".to_string(),
                },
            );
        }

        impacts
    }

    /// Calculate user impact score
    fn calculate_user_impact(&self, first: &SystemSnapshot, last: &SystemSnapshot) -> f64 {
        let empathy_change = last.metrics.empathy_index - first.metrics.empathy_index;
        let strain_change = -(last.metrics.strain_index - first.metrics.strain_index); // Negative strain is good

        // Weighted combination
        empathy_change * 0.6 + strain_change * 0.4
    }

    /// Determine overall ethical trajectory
    fn determine_trajectory(&self, scenario: &ForecastScenario) -> EthicalTrajectory {
        if let Some(last) = scenario.snapshots.last() {
            // Check if any thresholds are violated
            if last.metrics.health_score < self.thresholds.min_health {
                return EthicalTrajectory::DangerousDegradation;
            }

            if last.metrics.strain_index > self.thresholds.max_strain {
                return EthicalTrajectory::DangerousDegradation;
            }

            if last.metrics.network_coherence < self.thresholds.min_coherence {
                return EthicalTrajectory::MinorDegradation;
            }

            // Check for improvements
            if let Some(first) = scenario.snapshots.first() {
                let health_delta = last.metrics.health_score - first.metrics.health_score;
                let empathy_delta = last.metrics.empathy_index - first.metrics.empathy_index;
                let strain_delta = last.metrics.strain_index - first.metrics.strain_index;

                if health_delta > 0.1 && empathy_delta > 0.05 && strain_delta < -0.05 {
                    return EthicalTrajectory::SignificantImprovement;
                }

                if health_delta > 0.0 || empathy_delta > 0.0 {
                    return EthicalTrajectory::ModerateImprovement;
                }

                if health_delta < -0.1 || strain_delta > 0.1 {
                    return EthicalTrajectory::MinorDegradation;
                }
            }

            return EthicalTrajectory::Stable;
        }

        EthicalTrajectory::Unknown
    }

    /// Generate intervention recommendations
    fn generate_recommendations(
        &self,
        scenario: &ForecastScenario,
        trajectory: &EthicalTrajectory,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(last) = scenario.snapshots.last() {
            match trajectory {
                EthicalTrajectory::DangerousDegradation => {
                    recommendations.push("URGENT: Immediate intervention required".to_string());

                    if last.metrics.health_score < self.thresholds.min_health {
                        recommendations
                            .push("Critical: Health score below minimum threshold".to_string());
                        recommendations.push(
                            "Action: Run system diagnostics and address failures".to_string(),
                        );
                    }

                    if last.metrics.strain_index > self.thresholds.max_strain {
                        recommendations.push("Critical: Excessive strain detected".to_string());
                        recommendations.push(
                            "Action: Reduce workload and defer non-critical tasks".to_string(),
                        );
                    }
                }
                EthicalTrajectory::MinorDegradation => {
                    recommendations.push("Warning: Degradation trend detected".to_string());
                    recommendations
                        .push("Action: Monitor closely and prepare contingency plans".to_string());
                }
                EthicalTrajectory::Stable => {
                    recommendations.push("System trajectory stable".to_string());
                    recommendations
                        .push("Action: Maintain current operational parameters".to_string());
                }
                EthicalTrajectory::ModerateImprovement
                | EthicalTrajectory::SignificantImprovement => {
                    recommendations.push("Positive trajectory detected".to_string());
                    recommendations.push("Action: Continue current approach".to_string());
                }
                EthicalTrajectory::Unknown => {
                    recommendations.push("Insufficient data for recommendation".to_string());
                }
            }

            // Specific metric warnings
            if last.metrics.network_coherence < self.thresholds.min_coherence {
                recommendations.push(
                    "Network coherence below threshold - initiate mirror consensus".to_string(),
                );
            }

            if last.metrics.empathy_index < self.thresholds.min_empathy {
                recommendations
                    .push("Empathy index low - review user impact assessments".to_string());
            }
        }

        debug!("Generated {} recommendations", recommendations.len());
        recommendations
    }

    /// Calculate total moral cost of projected trajectory
    fn calculate_moral_cost(
        &self,
        impacts: &HashMap<String, StakeholderImpact>,
        trajectory: &EthicalTrajectory,
    ) -> f64 {
        let mut cost = 0.0;

        // Sum weighted negative impacts
        for (stakeholder, impact) in impacts {
            if let Some(&weight) = self.stakeholder_weights.get(stakeholder) {
                if impact.impact_score < 0.0 {
                    cost += impact.impact_score.abs() * weight;
                }
                cost += impact.emotional_cost.abs() * weight * 0.5;
            }
        }

        // Add trajectory penalty
        let trajectory_penalty = match trajectory {
            EthicalTrajectory::DangerousDegradation => 1.0,
            EthicalTrajectory::MinorDegradation => 0.5,
            EthicalTrajectory::Stable => 0.0,
            EthicalTrajectory::ModerateImprovement => -0.2,
            EthicalTrajectory::SignificantImprovement => -0.5,
            EthicalTrajectory::Unknown => 0.3,
        };

        cost + trajectory_penalty
    }
}

/// Ethics projection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsProjection {
    /// Projection ID
    pub projection_id: String,
    /// Associated forecast ID
    pub forecast_id: String,
    /// When projection was generated
    pub generated_at: DateTime<Utc>,
    /// Temporal empathy index (weighted future empathy)
    pub temporal_empathy_index: f64,
    /// Projected impacts per stakeholder
    pub stakeholder_impacts: HashMap<String, StakeholderImpact>,
    /// Overall ethical trajectory
    pub ethical_trajectory: EthicalTrajectory,
    /// Intervention recommendations
    pub intervention_recommendations: Vec<String>,
    /// Total moral cost (higher = worse)
    pub moral_cost: f64,
}

/// Stakeholder impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderImpact {
    /// Stakeholder name
    pub stakeholder: String,
    /// Impact score (-1.0 to +1.0, negative = harmful)
    pub impact_score: f64,
    /// Emotional cost (strain, distress)
    pub emotional_cost: f64,
    /// Description
    pub description: String,
}

/// Ethical trajectory classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EthicalTrajectory {
    SignificantImprovement,
    ModerateImprovement,
    Stable,
    MinorDegradation,
    DangerousDegradation,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethics_projector_creation() {
        let projector = EthicsProjector::new();
        assert_eq!(projector.stakeholder_weights.len(), 4);
    }

    #[test]
    fn test_trajectory_determination() {
        let projector = EthicsProjector::new();

        // Test dangerous degradation
        let mut scenario = ForecastScenario {
            scenario_id: "test".to_string(),
            probability: 1.0,
            snapshots: vec![],
        };

        scenario.snapshots.push(SystemSnapshot {
            id: "1".to_string(),
            timestamp: Utc::now(),
            metrics: super::super::timeline::SystemMetrics {
                health_score: 0.4,
                empathy_index: 0.5,
                strain_index: 0.8,
                network_coherence: 0.5,
                avg_trust_score: 0.7,
            },
            processes_count: 100,
            memory_usage_mb: 1000,
            cpu_usage_percent: 50.0,
            network_bytes_per_sec: 1000,
            service_states: HashMap::new(),
            pending_actions: vec![],
        });

        let trajectory = projector.determine_trajectory(&scenario);
        assert_eq!(trajectory, EthicalTrajectory::DangerousDegradation);
    }
}

//! Resonance mapping for stakeholder perception
//!
//! Phase 1.2: Models how well Anna resonates with different stakeholders
//! Citation: [archwiki:System_maintenance]

use super::types::{
    ContextMetrics, ResonanceAdjustment, ResonanceMap, SentimentAnalysis,
    StakeholderImpact, StakeholderImpacts,
};
use crate::sentinel::SentinelAction;
use chrono::Utc;
use tracing::debug;

/// Resonance mapper - tracks and adjusts stakeholder perception
pub struct ResonanceMapper {
    /// Current resonance state
    map: ResonanceMap,
}

impl ResonanceMapper {
    /// Create new resonance mapper
    pub fn new(map: ResonanceMap) -> Self {
        Self { map }
    }

    /// Analyze action impact on stakeholders
    pub fn analyze_stakeholder_impact(
        &self,
        action: &SentinelAction,
        context: &ContextMetrics,
    ) -> StakeholderImpacts {
        debug!("Analyzing stakeholder impact for action: {:?}", action);

        let user_impact = self.analyze_user_impact(action, context);
        let system_impact = self.analyze_system_impact(action, context);
        let environment_impact = self.analyze_environment_impact(action, context);

        StakeholderImpacts {
            user: user_impact,
            system: system_impact,
            environment: environment_impact,
        }
    }

    /// Analyze impact on user
    fn analyze_user_impact(
        &self,
        action: &SentinelAction,
        context: &ContextMetrics,
    ) -> StakeholderImpact {
        match action {
            SentinelAction::None => StakeholderImpact {
                score: 0.0,
                impact_type: "none".to_string(),
                reasoning: "No user impact".to_string(),
            },

            SentinelAction::RestartService { service } => {
                let disruption_score = if service.contains("display") || service.contains("desktop") {
                    0.7 // High disruption for display services
                } else if service.contains("network") {
                    0.5 // Moderate disruption for network
                } else {
                    0.2 // Low disruption for background services
                };

                StakeholderImpact {
                    score: disruption_score,
                    impact_type: "disruption".to_string(),
                    reasoning: format!(
                        "Service restart may cause temporary disruption for: {}",
                        service
                    ),
                }
            }

            SentinelAction::SystemUpdate { dry_run } => {
                if *dry_run {
                    StakeholderImpact {
                        score: 0.0,
                        impact_type: "none".to_string(),
                        reasoning: "Dry-run has no user impact".to_string(),
                    }
                } else {
                    // Consider user activity - don't update during active sessions
                    let impact_score = if context.user_activity > 0.5 {
                        0.8 // High impact during active use
                    } else {
                        0.3 // Lower impact during idle
                    };

                    StakeholderImpact {
                        score: impact_score,
                        impact_type: "cognitive_load".to_string(),
                        reasoning: format!(
                            "System update requires attention (user activity: {:.0}%)",
                            context.user_activity * 100.0
                        ),
                    }
                }
            }

            SentinelAction::SendNotification { title, body } => {
                // Notifications have cognitive load, especially during high activity
                let impact_score = if context.user_activity > 0.7 {
                    0.6 // Higher impact during focused work
                } else {
                    0.2 // Lower impact during idle
                };

                StakeholderImpact {
                    score: impact_score,
                    impact_type: "cognitive_load".to_string(),
                    reasoning: "Notification interrupts user attention".to_string(),
                }
            }

            SentinelAction::LogWarning { .. } => StakeholderImpact {
                score: 0.0,
                impact_type: "none".to_string(),
                reasoning: "Logging has no direct user impact".to_string(),
            },

            _ => StakeholderImpact {
                score: 0.2,
                impact_type: "minor".to_string(),
                reasoning: "Minor potential user impact".to_string(),
            },
        }
    }

    /// Analyze impact on system
    fn analyze_system_impact(
        &self,
        action: &SentinelAction,
        context: &ContextMetrics,
    ) -> StakeholderImpact {
        match action {
            SentinelAction::None => StakeholderImpact::default(),

            SentinelAction::RestartService { .. } => {
                // Service restart uses resources, especially under load
                let impact_score = if context.cpu_load > 0.7 || context.memory_pressure > 0.8 {
                    0.6 // Higher impact under strain
                } else {
                    0.2 // Normal impact
                };

                StakeholderImpact {
                    score: impact_score,
                    impact_type: "resource".to_string(),
                    reasoning: format!(
                        "Service restart consumes resources (CPU: {:.0}%, Mem: {:.0}%)",
                        context.cpu_load * 100.0,
                        context.memory_pressure * 100.0
                    ),
                }
            }

            SentinelAction::SystemUpdate { dry_run } => {
                if *dry_run {
                    StakeholderImpact {
                        score: 0.1,
                        impact_type: "resource".to_string(),
                        reasoning: "Dry-run uses minimal system resources".to_string(),
                    }
                } else {
                    let impact_score = if context.memory_pressure > 0.8 {
                        0.9 // Very high impact under memory pressure
                    } else if context.cpu_load > 0.7 {
                        0.7 // High impact under CPU load
                    } else {
                        0.4 // Moderate impact normally
                    };

                    StakeholderImpact {
                        score: impact_score,
                        impact_type: "resource".to_string(),
                        reasoning: "System update requires significant resources".to_string(),
                    }
                }
            }

            SentinelAction::SyncDatabases => StakeholderImpact {
                score: 0.1,
                impact_type: "resource".to_string(),
                reasoning: "Database sync uses minimal resources".to_string(),
            },

            _ => StakeholderImpact {
                score: 0.1,
                impact_type: "minor".to_string(),
                reasoning: "Minor system impact".to_string(),
            },
        }
    }

    /// Analyze impact on environment (network, dependencies, etc.)
    fn analyze_environment_impact(
        &self,
        action: &SentinelAction,
        _context: &ContextMetrics,
    ) -> StakeholderImpact {
        match action {
            SentinelAction::None => StakeholderImpact::default(),

            SentinelAction::SystemUpdate { dry_run } => {
                if *dry_run {
                    StakeholderImpact {
                        score: 0.3,
                        impact_type: "network".to_string(),
                        reasoning: "Dry-run checks network for updates".to_string(),
                    }
                } else {
                    StakeholderImpact {
                        score: 0.7,
                        impact_type: "network".to_string(),
                        reasoning: "Update downloads packages from mirrors".to_string(),
                    }
                }
            }

            SentinelAction::SyncDatabases => StakeholderImpact {
                score: 0.2,
                impact_type: "network".to_string(),
                reasoning: "Database sync requires network access".to_string(),
            },

            _ => StakeholderImpact::default(),
        }
    }

    /// Update resonance based on outcomes
    pub fn update_resonance(
        &mut self,
        impacts: &StakeholderImpacts,
        sentiment: &SentimentAnalysis,
    ) {
        debug!("Updating resonance map");

        // Adjust user resonance based on user impact and sentiment
        let user_delta = self.calculate_resonance_delta(
            impacts.user.score,
            sentiment.sentiment_score,
            self.map.user_resonance,
        );

        if user_delta.abs() > 0.01 {
            self.map.user_resonance = (self.map.user_resonance + user_delta).max(0.0).min(1.0);
            self.map.recent_adjustments.push(ResonanceAdjustment {
                timestamp: Utc::now(),
                stakeholder: "user".to_string(),
                delta: user_delta,
                reason: format!("Impact: {:.2}, Sentiment: {:.2}", impacts.user.score, sentiment.sentiment_score),
            });
        }

        // Adjust system resonance based on system impact
        let system_delta = self.calculate_resonance_delta(
            impacts.system.score,
            sentiment.sentiment_score,
            self.map.system_resonance,
        );

        if system_delta.abs() > 0.01 {
            self.map.system_resonance = (self.map.system_resonance + system_delta).max(0.0).min(1.0);
            self.map.recent_adjustments.push(ResonanceAdjustment {
                timestamp: Utc::now(),
                stakeholder: "system".to_string(),
                delta: system_delta,
                reason: format!("Impact: {:.2}", impacts.system.score),
            });
        }

        // Adjust environment resonance
        let env_delta = self.calculate_resonance_delta(
            impacts.environment.score,
            0.0,
            self.map.environment_resonance,
        );

        if env_delta.abs() > 0.01 {
            self.map.environment_resonance = (self.map.environment_resonance + env_delta).max(0.0).min(1.0);
            self.map.recent_adjustments.push(ResonanceAdjustment {
                timestamp: Utc::now(),
                stakeholder: "environment".to_string(),
                delta: env_delta,
                reason: format!("Impact: {:.2}", impacts.environment.score),
            });
        }

        // Keep only recent adjustments (last 100)
        if self.map.recent_adjustments.len() > 100 {
            self.map.recent_adjustments = self.map.recent_adjustments.split_off(
                self.map.recent_adjustments.len() - 100
            );
        }
    }

    /// Calculate resonance delta (change in resonance)
    fn calculate_resonance_delta(
        &self,
        impact_score: f64,
        sentiment_score: f64,
        current_resonance: f64,
    ) -> f64 {
        // High impact or negative sentiment reduces resonance
        // Low impact or positive sentiment increases resonance

        let impact_factor = -impact_score * 0.05; // Max -0.05 per high-impact action
        let sentiment_factor = sentiment_score * 0.02; // Max ±0.02 per sentiment

        // Learning rate decreases as resonance approaches extremes
        let learning_rate = if current_resonance < 0.2 || current_resonance > 0.8 {
            0.5
        } else {
            1.0
        };

        (impact_factor + sentiment_factor) * learning_rate
    }

    /// Get current resonance map
    pub fn get_map(&self) -> &ResonanceMap {
        &self.map
    }

    /// Get mutable resonance map
    pub fn get_map_mut(&mut self) -> &mut ResonanceMap {
        &mut self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_impact_service_restart() {
        let mapper = ResonanceMapper::new(ResonanceMap::default());
        let context = ContextMetrics::default();

        let action = SentinelAction::RestartService {
            service: "display-manager".to_string(),
        };

        let impact = mapper.analyze_user_impact(&action, &context);
        assert!(impact.score > 0.5); // High disruption for display services
    }

    #[test]
    fn test_system_impact_under_load() {
        let mapper = ResonanceMapper::new(ResonanceMap::default());
        let mut context = ContextMetrics::default();
        context.cpu_load = 0.9; // High load

        let action = SentinelAction::SystemUpdate { dry_run: false };

        let impact = mapper.analyze_system_impact(&action, &context);
        assert!(impact.score > 0.6); // High impact under load
    }

    #[test]
    fn test_resonance_update() {
        let mut mapper = ResonanceMapper::new(ResonanceMap::default());
        let initial_resonance = mapper.map.user_resonance;

        let impacts = StakeholderImpacts {
            user: StakeholderImpact {
                score: 0.8,
                impact_type: "disruption".to_string(),
                reasoning: "Test".to_string(),
            },
            system: StakeholderImpact::default(),
            environment: StakeholderImpact::default(),
        };

        let sentiment = SentimentAnalysis {
            sentiment_score: -0.5,
            ..Default::default()
        };

        mapper.update_resonance(&impacts, &sentiment);

        // Resonance should decrease due to high impact and negative sentiment
        assert!(mapper.map.user_resonance < initial_resonance);
    }
}

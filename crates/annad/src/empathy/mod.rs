//! Empathy Kernel - Contextual awareness and human-centered design
//!
//! Phase 1.2: Empathy Kernel that models stakeholder perception and adapts behavior
//! Citation: [archwiki:System_maintenance]

pub mod context;
pub mod resonance;
pub mod sentiment;
pub mod types;

use anyhow::{Context as AnyhowContext, Result};
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::sentinel::SentinelAction;
use context::ContextAnalyzer;
use resonance::ResonanceMapper;
use sentiment::SentimentAnalyzer;
use types::*;

/// Empathy Kernel - Main daemon for contextual awareness
pub struct EmpathyKernel {
    /// Kernel configuration
    config: EmpathyConfig,
    /// Current empathy state
    state: Arc<RwLock<EmpathyState>>,
    /// Context analyzer
    context_analyzer: Arc<RwLock<ContextAnalyzer>>,
    /// Sentiment analyzer
    sentiment_analyzer: Arc<RwLock<SentimentAnalyzer>>,
    /// Resonance mapper
    resonance_mapper: Arc<RwLock<ResonanceMapper>>,
}

impl EmpathyKernel {
    /// Create new empathy kernel
    pub async fn new() -> Result<Self> {
        Self::with_config(EmpathyConfig::default()).await
    }

    /// Create empathy kernel with custom config
    pub async fn with_config(config: EmpathyConfig) -> Result<Self> {
        info!("Initializing Empathy Kernel v1.2.0");

        // Load or create state
        let state = Self::load_or_create_state().await?;

        // Initialize analyzers
        let context_analyzer = Arc::new(RwLock::new(ContextAnalyzer::new(config.clone())));
        let sentiment_analyzer = Arc::new(RwLock::new(SentimentAnalyzer::new()));
        let resonance_mapper = Arc::new(RwLock::new(ResonanceMapper::new(
            state.resonance_map.clone(),
        )));

        info!(
            "Empathy Kernel initialized - Empathy: {:.1}%, Strain: {:.1}%",
            state.empathy_index * 100.0,
            state.strain_index * 100.0
        );

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            context_analyzer,
            sentiment_analyzer,
            resonance_mapper,
        })
    }

    /// Evaluate action with empathy - main entry point
    pub async fn evaluate_with_empathy(
        &self,
        action: &SentinelAction,
    ) -> Result<EmpathyEvaluation> {
        debug!("Evaluating action with empathy: {:?}", action);

        // Analyze context
        let context = {
            let mut analyzer = self.context_analyzer.write().await;
            analyzer.analyze().await?
        };

        // Analyze sentiment
        let sentiment = {
            let mut analyzer = self.sentiment_analyzer.write().await;
            analyzer.analyze().await?
        };

        // Analyze stakeholder impacts
        let stakeholder_impacts = {
            let mapper = self.resonance_mapper.read().await;
            mapper.analyze_stakeholder_impact(action, &context)
        };

        // Update resonance based on analysis
        {
            let mut mapper = self.resonance_mapper.write().await;
            mapper.update_resonance(&stakeholder_impacts, &sentiment);
        }

        // Calculate strain index
        let strain_index = {
            let analyzer = self.context_analyzer.read().await;
            analyzer.calculate_strain_index(&context, &self.config.context_weights)
        };

        // Calculate empathy index from resonance map
        let empathy_index = {
            let mapper = self.resonance_mapper.read().await;
            let map = mapper.get_map();
            (map.user_resonance + map.system_resonance + map.environment_resonance) / 3.0
        };

        // Update state
        {
            let mut state = self.state.write().await;
            state.empathy_index = empathy_index;
            state.strain_index = strain_index;
            state.context_metrics = context.clone();
            state.resonance_map = self.resonance_mapper.read().await.get_map().clone();
        }

        // Determine if action should be deferred
        let should_defer = self.should_defer_action(
            action,
            &context,
            &stakeholder_impacts,
            strain_index,
        );

        let deferral_reason = if should_defer {
            Some(self.explain_deferral(&context, &stakeholder_impacts, strain_index))
        } else {
            None
        };

        // Calculate recommended delay
        let recommended_delay = if strain_index > self.config.strain_threshold {
            let base_delay = 300; // 5 minutes
            (base_delay as f64 * self.config.adaptive_response.strain_delay_multiplier) as u64
        } else {
            0
        };

        // Generate adaptive tone suggestion
        let tone_adaptation = if empathy_index < self.config.adaptive_response.tone_threshold
            && self.config.adaptive_response.adaptive_tone
        {
            Some(self.suggest_tone_adaptation(empathy_index, &sentiment))
        } else {
            None
        };

        // Collect context factors
        let mut context_factors = Vec::new();
        if context.error_rate > 10.0 {
            context_factors.push(format!("High error rate: {:.1}/hour", context.error_rate));
        }
        if context.cpu_load > 0.7 {
            context_factors.push(format!("High CPU load: {:.0}%", context.cpu_load * 100.0));
        }
        if context.memory_pressure > 0.8 {
            context_factors.push(format!(
                "High memory pressure: {:.0}%",
                context.memory_pressure * 100.0
            ));
        }
        if context.user_activity < 0.3 {
            context_factors.push("Low user activity detected".to_string());
        }
        context_factors.extend(context.fatigue_indicators.clone());
        context_factors.extend(sentiment.patterns.clone());

        // Record perception
        self.record_perception(
            action.clone(),
            stakeholder_impacts.clone(),
            context_factors.clone(),
            deferral_reason.clone(),
        )
        .await?;

        let evaluation = EmpathyEvaluation {
            should_defer,
            deferral_reason,
            stakeholder_impacts,
            context_factors,
            recommended_delay,
            tone_adaptation,
        };

        debug!(
            "Empathy evaluation complete - Defer: {}, Delay: {}s",
            should_defer, recommended_delay
        );

        Ok(evaluation)
    }

    /// Determine if action should be deferred
    fn should_defer_action(
        &self,
        action: &SentinelAction,
        context: &ContextMetrics,
        impacts: &StakeholderImpacts,
        strain_index: f64,
    ) -> bool {
        // Never defer critical safety actions
        if Self::is_critical_action(action) {
            return false;
        }

        // Defer if strain is too high
        if strain_index > self.config.strain_threshold {
            return true;
        }

        // Defer high-impact actions during active user sessions
        if impacts.user.score > 0.6 && context.user_activity > 0.7 {
            return true;
        }

        // Defer resource-intensive actions under system stress
        if impacts.system.score > 0.7
            && (context.cpu_load > 0.8 || context.memory_pressure > 0.8)
        {
            return true;
        }

        false
    }

    /// Check if action is critical and should never be deferred
    fn is_critical_action(action: &SentinelAction) -> bool {
        matches!(
            action,
            SentinelAction::LogWarning { .. }
                | SentinelAction::SendNotification { .. }
                | SentinelAction::None
        )
    }

    /// Explain why action was deferred
    fn explain_deferral(
        &self,
        context: &ContextMetrics,
        impacts: &StakeholderImpacts,
        strain_index: f64,
    ) -> String {
        let mut reasons = Vec::new();

        if strain_index > self.config.strain_threshold {
            reasons.push(format!(
                "System strain index ({:.0}%) exceeds threshold ({:.0}%)",
                strain_index * 100.0,
                self.config.strain_threshold * 100.0
            ));
        }

        if impacts.user.score > 0.6 && context.user_activity > 0.7 {
            reasons.push(format!(
                "High user impact ({:.0}%) during active session",
                impacts.user.score * 100.0
            ));
        }

        if impacts.system.score > 0.7
            && (context.cpu_load > 0.8 || context.memory_pressure > 0.8)
        {
            reasons.push(format!(
                "System under stress (CPU: {:.0}%, Mem: {:.0}%)",
                context.cpu_load * 100.0,
                context.memory_pressure * 100.0
            ));
        }

        if reasons.is_empty() {
            "Action deferred based on contextual analysis".to_string()
        } else {
            reasons.join("; ")
        }
    }

    /// Suggest tone adaptation for human-facing messages
    fn suggest_tone_adaptation(&self, empathy_index: f64, sentiment: &SentimentAnalysis) -> String {
        if sentiment.sentiment_score < -0.3 {
            // Negative sentiment - be more supportive
            "Use reassuring, supportive tone. Acknowledge difficulties and offer concrete help.".to_string()
        } else if empathy_index < 0.4 {
            // Low empathy - be more careful
            "Use cautious, considerate tone. Minimize technical jargon and explain impacts clearly.".to_string()
        } else if sentiment.patterns.iter().any(|p| p.contains("Strain")) {
            // System strain - be gentle
            "Use gentle, patient tone. Avoid adding cognitive load with complex explanations.".to_string()
        } else {
            "Use clear, professional tone with empathetic framing.".to_string()
        }
    }

    /// Record perception in history
    async fn record_perception(
        &self,
        action: SentinelAction,
        stakeholder_impacts: StakeholderImpacts,
        context_factors: Vec<String>,
        adaptation: Option<String>,
    ) -> Result<()> {
        let mut state = self.state.write().await;

        let record = PerceptionRecord {
            timestamp: Utc::now(),
            action,
            stakeholder_impacts,
            context_factors,
            adaptation,
        };

        state.perception_history.push(record);

        // Keep only last 1000 perceptions
        if state.perception_history.len() > 1000 {
            let history_len = state.perception_history.len();
            state.perception_history = state.perception_history.split_off(history_len - 1000);
        }

        Ok(())
    }

    /// Get current empathy pulse
    pub async fn get_pulse(&self) -> EmpathyPulse {
        let state = self.state.read().await;

        let context_summary = format!(
            "Strain: {:.0}%, Empathy: {:.0}%, CPU: {:.0}%, Mem: {:.0}%, Errors: {:.1}/h, Activity: {:.0}%",
            state.strain_index * 100.0,
            state.empathy_index * 100.0,
            state.context_metrics.cpu_load * 100.0,
            state.context_metrics.memory_pressure * 100.0,
            state.context_metrics.error_rate,
            state.context_metrics.user_activity * 100.0
        );

        let recent_perceptions = state
            .perception_history
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        EmpathyPulse {
            timestamp: Utc::now(),
            empathy_index: state.empathy_index,
            strain_index: state.strain_index,
            resonance_map: state.resonance_map.clone(),
            context_summary,
            recent_perceptions,
        }
    }

    /// Simulate empathy evaluation for an action
    pub async fn simulate(&self, action: &SentinelAction) -> Result<EmpathySimulation> {
        let evaluation = self.evaluate_with_empathy(action).await?;

        // Capture values before moving evaluation
        let would_proceed = !evaluation.should_defer;

        let reasoning = format!(
            "Action: {:?}\n\nStakeholder Impacts:\n- User: {:.0}% ({})\n- System: {:.0}% ({})\n- Environment: {:.0}% ({})\n\nContext Factors:\n{}\n\nRecommendation: {}",
            action,
            evaluation.stakeholder_impacts.user.score * 100.0,
            evaluation.stakeholder_impacts.user.reasoning,
            evaluation.stakeholder_impacts.system.score * 100.0,
            evaluation.stakeholder_impacts.system.reasoning,
            evaluation.stakeholder_impacts.environment.score * 100.0,
            evaluation.stakeholder_impacts.environment.reasoning,
            evaluation.context_factors.join("\n"),
            if evaluation.should_defer {
                format!("Defer - {}", evaluation.deferral_reason.as_ref().unwrap())
            } else {
                "Proceed".to_string()
            }
        );

        Ok(EmpathySimulation {
            action: format!("{:?}", action),
            evaluation,
            reasoning,
            would_proceed,
        })
    }

    /// Save current state to disk
    pub async fn save_state(&self) -> Result<()> {
        let state = self.state.read().await;

        let state_path = Path::new("/var/lib/anna/empathy");
        fs::create_dir_all(state_path).await.context(
            "Failed to create empathy state directory (check permissions)",
        )?;

        let state_file = state_path.join("state.json");
        let json = serde_json::to_string_pretty(&*state)
            .context("Failed to serialize empathy state")?;

        fs::write(&state_file, json)
            .await
            .context("Failed to write empathy state file")?;

        debug!("Empathy state saved to {:?}", state_file);
        Ok(())
    }

    /// Load state from disk or create default
    async fn load_or_create_state() -> Result<EmpathyState> {
        let state_file = Path::new("/var/lib/anna/empathy/state.json");

        if state_file.exists() {
            match fs::read_to_string(state_file).await {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(state) => {
                        info!("Loaded empathy state from {:?}", state_file);
                        return Ok(state);
                    }
                    Err(e) => {
                        warn!("Failed to parse empathy state: {}, using default", e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read empathy state: {}, using default", e);
                }
            }
        }

        info!("Creating new empathy state");
        Ok(EmpathyState::default())
    }

    /// Get current state (for IPC)
    pub async fn get_state(&self) -> EmpathyState {
        self.state.read().await.clone()
    }

    /// Generate weekly empathy digest
    pub async fn generate_weekly_digest(&self) -> Result<String> {
        let state = self.state.read().await;

        let total_perceptions = state.perception_history.len();
        let deferred_count = state
            .perception_history
            .iter()
            .filter(|p| p.adaptation.is_some())
            .count();

        let avg_empathy = state.empathy_index;
        let avg_strain = state.strain_index;

        let digest = format!(
            "=== Anna Empathy Kernel - Weekly Digest ===\n\
             Generated: {}\n\n\
             Summary:\n\
             - Total perceptions: {}\n\
             - Actions deferred: {} ({:.1}%)\n\
             - Average empathy index: {:.1}%\n\
             - Average strain index: {:.1}%\n\n\
             Resonance Map:\n\
             - User resonance: {:.1}%\n\
             - System resonance: {:.1}%\n\
             - Environment resonance: {:.1}%\n\n\
             Recent Adjustments: {}\n\n\
             Context Metrics:\n\
             - CPU load: {:.0}%\n\
             - Memory pressure: {:.0}%\n\
             - Error rate: {:.1}/hour\n\
             - User activity: {:.0}%\n\n\
             Fatigue Indicators: {}\n\
             ",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            total_perceptions,
            deferred_count,
            if total_perceptions > 0 {
                (deferred_count as f64 / total_perceptions as f64) * 100.0
            } else {
                0.0
            },
            avg_empathy * 100.0,
            avg_strain * 100.0,
            state.resonance_map.user_resonance * 100.0,
            state.resonance_map.system_resonance * 100.0,
            state.resonance_map.environment_resonance * 100.0,
            state.resonance_map.recent_adjustments.len(),
            state.context_metrics.cpu_load * 100.0,
            state.context_metrics.memory_pressure * 100.0,
            state.context_metrics.error_rate,
            state.context_metrics.user_activity * 100.0,
            state.context_metrics.fatigue_indicators.join(", ")
        );

        // Write digest to log
        let digest_path = Path::new("/var/log/anna");
        if let Err(e) = fs::create_dir_all(digest_path).await {
            warn!("Failed to create log directory: {}", e);
        } else {
            let digest_file = digest_path.join("empathy-digest.log");
            if let Err(e) = fs::write(&digest_file, &digest).await {
                warn!("Failed to write empathy digest: {}", e);
            }
        }

        Ok(digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empathy_kernel_creation() {
        let kernel = EmpathyKernel::new().await;
        assert!(kernel.is_ok());
    }

    #[tokio::test]
    async fn test_evaluate_none_action() {
        let kernel = EmpathyKernel::new().await.unwrap();
        let action = SentinelAction::None;

        let evaluation = kernel.evaluate_with_empathy(&action).await;
        assert!(evaluation.is_ok());

        let eval = evaluation.unwrap();
        assert!(!eval.should_defer); // None action should not be deferred
    }

    #[tokio::test]
    async fn test_get_pulse() {
        let kernel = EmpathyKernel::new().await.unwrap();
        let pulse = kernel.get_pulse().await;

        assert!(pulse.empathy_index >= 0.0 && pulse.empathy_index <= 1.0);
        assert!(pulse.strain_index >= 0.0 && pulse.strain_index <= 1.0);
    }

    #[tokio::test]
    async fn test_simulate() {
        let kernel = EmpathyKernel::new().await.unwrap();
        let action = SentinelAction::SystemUpdate { dry_run: false };

        let simulation = kernel.simulate(&action).await;
        assert!(simulation.is_ok());

        let sim = simulation.unwrap();
        assert!(!sim.reasoning.is_empty());
    }

    #[tokio::test]
    async fn test_critical_actions_not_deferred() {
        let kernel = EmpathyKernel::new().await.unwrap();

        // Critical actions should never be deferred
        let action = SentinelAction::LogWarning {
            message: "test".to_string(),
        };

        let evaluation = kernel.evaluate_with_empathy(&action).await.unwrap();
        assert!(!evaluation.should_defer);
    }
}

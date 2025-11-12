//! Mirror Protocol - Recursive introspection and self-validation
//!
//! Phase 1.4: Metacognition for collective ethical coherence
//! Citation: [archwiki:System_maintenance]

pub mod critique;
pub mod mirror_consensus;
pub mod reflection;
pub mod repair;
pub mod types;

use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use critique::CritiqueEvaluator;
use mirror_consensus::MirrorConsensusCoordinator;
use reflection::ReflectionGenerator;
use repair::{RemediationEngine, RemediationReport};
use types::*;

/// Mirror Protocol daemon
pub struct MirrorProtocol {
    /// Configuration
    config: MirrorConfig,
    /// Current state
    state: Arc<RwLock<MirrorState>>,
    /// Reflection generator
    reflection_gen: Arc<RwLock<ReflectionGenerator>>,
    /// Critique evaluator
    critique_eval: Arc<RwLock<CritiqueEvaluator>>,
    /// Consensus coordinator
    consensus_coord: Arc<RwLock<MirrorConsensusCoordinator>>,
    /// Remediation engine
    remediation_engine: Arc<RwLock<RemediationEngine>>,
}

impl MirrorProtocol {
    /// Create new mirror protocol instance
    pub async fn new(node_id: PeerId, private_key: String) -> Result<Self> {
        Self::with_config(node_id, private_key, MirrorConfig::default()).await
    }

    /// Create with custom configuration
    pub async fn with_config(
        node_id: PeerId,
        private_key: String,
        config: MirrorConfig,
    ) -> Result<Self> {
        info!("Initializing Mirror Protocol v1.4.0");

        // Load or create state
        let mut state = Self::load_or_create_state().await?;
        state.node_id = node_id.clone();

        // Initialize components
        let reflection_gen = ReflectionGenerator::new(node_id.clone(), private_key.clone());
        let critique_eval = CritiqueEvaluator::new(node_id.clone(), private_key.clone(), 0.8);
        let consensus_coord = MirrorConsensusCoordinator::new(
            node_id.clone(),
            config.min_consensus_nodes,
            config.coherence_threshold,
        );
        let remediation_engine =
            RemediationEngine::new(node_id.clone(), config.auto_remediation);

        info!(
            "Mirror Protocol initialized - Node ID: {}, Coherence: {:.2}",
            state.node_id, state.current_coherence
        );

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            reflection_gen: Arc::new(RwLock::new(reflection_gen)),
            critique_eval: Arc::new(RwLock::new(critique_eval)),
            consensus_coord: Arc::new(RwLock::new(consensus_coord)),
            remediation_engine: Arc::new(RwLock::new(remediation_engine)),
        })
    }

    /// Start the mirror protocol daemon
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Mirror Protocol is disabled in configuration");
            return Ok(());
        }

        info!("Starting Mirror Protocol daemon");

        // Spawn periodic reflection task
        self.spawn_reflection_task().await;

        // Spawn periodic consensus task
        self.spawn_consensus_task().await;

        // Spawn state persistence task
        self.spawn_persistence_task().await;

        info!("Mirror Protocol daemon started");

        Ok(())
    }

    /// Spawn periodic reflection generation task
    async fn spawn_reflection_task(&self) {
        let reflection_gen = Arc::clone(&self.reflection_gen);
        let state = Arc::clone(&self.state);
        let interval_hours = self.config.reflection_interval_hours;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_hours * 3600));

            loop {
                ticker.tick().await;

                info!("Generating periodic reflection");
                let gen = reflection_gen.read().await;
                let reflection = gen.generate_reflection(interval_hours as i64);

                let mut state_guard = state.write().await;
                state_guard.recent_reflections.push(reflection);
                state_guard.last_reflection = Some(Utc::now());

                // Keep only last 10 reflections
                if state_guard.recent_reflections.len() > 10 {
                    state_guard.recent_reflections.remove(0);
                }

                debug!("Periodic reflection generated and stored");
            }
        });
    }

    /// Spawn periodic consensus task
    async fn spawn_consensus_task(&self) {
        let consensus_coord = Arc::clone(&self.consensus_coord);
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            // Run consensus every 7 days
            let mut ticker = interval(Duration::from_secs(7 * 24 * 3600));

            loop {
                ticker.tick().await;

                info!("Running periodic mirror consensus");
                let state_guard = state.read().await;

                if state_guard.recent_reflections.len() < 3 {
                    debug!("Insufficient reflections for consensus");
                    continue;
                }

                let coord = consensus_coord.read().await;
                let session = coord.initiate_session(
                    state_guard.recent_reflections.clone(),
                    state_guard.received_critiques.clone(),
                );

                drop(state_guard);
                let mut state_guard = state.write().await;
                state_guard.consensus_sessions.push(session.clone());
                state_guard.last_consensus = Some(Utc::now());
                state_guard.current_coherence = session.network_coherence;

                // Keep only last 5 sessions
                if state_guard.consensus_sessions.len() > 5 {
                    state_guard.consensus_sessions.remove(0);
                }

                debug!("Periodic consensus session completed");
            }
        });
    }

    /// Spawn state persistence task
    async fn spawn_persistence_task(&self) {
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600)); // Every hour

            loop {
                ticker.tick().await;

                if let Err(e) = Self::save_state_internal(&state).await {
                    error!("Failed to save mirror state: {}", e);
                }
            }
        });
    }

    /// Generate immediate reflection (manual trigger)
    pub async fn generate_reflection(&self) -> Result<ReflectionReport> {
        info!("Generating manual reflection");

        let gen = self.reflection_gen.read().await;
        let reflection = gen.generate_reflection(24);

        let mut state = self.state.write().await;
        state.recent_reflections.push(reflection.clone());
        state.last_reflection = Some(Utc::now());

        if state.recent_reflections.len() > 10 {
            state.recent_reflections.remove(0);
        }

        Ok(reflection)
    }

    /// Receive and evaluate peer reflection
    pub async fn receive_peer_reflection(
        &self,
        reflection: ReflectionReport,
    ) -> Result<PeerCritique> {
        info!(
            "Receiving peer reflection {} from {}",
            reflection.id, reflection.node_id
        );

        let evaluator = self.critique_eval.read().await;
        let critique = evaluator.evaluate_reflection(&reflection);

        let mut state = self.state.write().await;
        state.received_critiques.push(critique.clone());

        // Keep only last 50 critiques
        if state.received_critiques.len() > 50 {
            state.received_critiques.remove(0);
        }

        Ok(critique)
    }

    /// Run mirror consensus manually
    pub async fn run_consensus(&self) -> Result<MirrorConsensusSession> {
        info!("Running manual mirror consensus");

        let state_guard = self.state.read().await;
        let reflections = state_guard.recent_reflections.clone();
        let critiques = state_guard.received_critiques.clone();
        drop(state_guard);

        if reflections.len() < self.config.min_consensus_nodes {
            return Err(anyhow::anyhow!(
                "Insufficient reflections for consensus ({}/{})",
                reflections.len(),
                self.config.min_consensus_nodes
            ));
        }

        let coord = self.consensus_coord.read().await;
        let session = coord.initiate_session(reflections, critiques);

        let mut state = self.state.write().await;
        state.consensus_sessions.push(session.clone());
        state.last_consensus = Some(Utc::now());
        state.current_coherence = session.network_coherence;

        if state.consensus_sessions.len() > 5 {
            state.consensus_sessions.remove(0);
        }

        Ok(session)
    }

    /// Apply remediations from consensus
    pub async fn apply_remediations(
        &self,
        session: &MirrorConsensusSession,
    ) -> Result<RemediationReport> {
        info!(
            "Applying {} remediations from consensus {}",
            session.approved_remediations.len(),
            session.id
        );

        let engine = self.remediation_engine.read().await;
        let mut results = Vec::new();

        for action in &session.approved_remediations {
            // Validate before applying
            if let Err(e) = engine.validate_remediation(action) {
                warn!("Remediation validation failed: {}", e);
                continue;
            }

            match engine.apply_remediation(action).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to apply remediation {}: {}", action.id, e);
                }
            }
        }

        let report = engine.generate_report(&results);

        // Store applied remediations
        let mut state = self.state.write().await;
        state.applied_remediations.extend(
            session.approved_remediations.clone()
        );

        // Keep only last 20 remediations
        if state.applied_remediations.len() > 20 {
            let excess = state.applied_remediations.len() - 20;
            state.applied_remediations.drain(0..excess);
        }

        Ok(report)
    }

    /// Get current status
    pub async fn get_status(&self) -> MirrorStatus {
        let state = self.state.read().await;

        MirrorStatus {
            enabled: self.config.enabled,
            current_coherence: state.current_coherence,
            last_reflection: state.last_reflection.map(|t| t.to_rfc3339()),
            last_consensus: state.last_consensus.map(|t| t.to_rfc3339()),
            recent_reflections_count: state.recent_reflections.len(),
            received_critiques_count: state.received_critiques.len(),
            active_remediations_count: state.applied_remediations.len(),
            network_coherence: state.consensus_sessions.last().map(|s| s.network_coherence),
        }
    }

    /// Get audit summary for RPC
    pub async fn get_audit_summary(&self) -> Result<AuditSummary> {
        let state = self.state.read().await;

        // Convert recent critiques to simplified format
        let recent_critiques: Vec<SimplifiedCritiqueSummary> = state
            .received_critiques
            .iter()
            .take(5)
            .map(|c| SimplifiedCritiqueSummary {
                critic_id: c.critic_id.clone(),
                coherence_assessment: c.coherence_assessment,
                inconsistencies_count: c.inconsistencies.len(),
                biases_count: c.identified_biases.len(),
                recommendations: c.recommendations.clone(),
            })
            .collect();

        Ok(AuditSummary {
            enabled: self.config.enabled,
            current_coherence: state.current_coherence,
            last_reflection: state.last_reflection,
            last_consensus: state.last_consensus,
            recent_reflections_count: state.recent_reflections.len(),
            received_critiques_count: state.received_critiques.len(),
            active_remediations_count: state.applied_remediations.len(),
            network_coherence: state.consensus_sessions.last().map(|s| s.network_coherence),
            recent_critiques,
        })
    }

    /// Apply pending remediations
    pub async fn apply_pending_remediations(&self) -> Result<RemediationReport> {
        info!("Applying pending remediations");

        let state = self.state.read().await;

        // Clone the last consensus session if available
        let last_session = state.consensus_sessions.last().cloned();

        // Drop the read lock before calling apply_remediations
        drop(state);

        if let Some(session) = last_session {
            // Apply remediations from the last consensus
            self.apply_remediations(&session).await
        } else {
            // No consensus session available - return empty report
            Ok(RemediationReport {
                timestamp: Utc::now(),
                total_remediations: 0,
                successful_remediations: 0,
                failed_remediations: 0,
                summary: "No consensus session available - no remediations to apply".to_string(),
                details: vec![],
            })
        }
    }

    /// Load or create state
    async fn load_or_create_state() -> Result<MirrorState> {
        let state_path = Path::new("/var/lib/anna/mirror/state.json");

        if state_path.exists() {
            match fs::read_to_string(state_path).await {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(state) => {
                        info!("Loaded mirror state from {:?}", state_path);
                        return Ok(state);
                    }
                    Err(e) => {
                        warn!("Failed to parse mirror state: {}, using default", e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read mirror state: {}, using default", e);
                }
            }
        }

        info!("Creating new mirror state");
        Ok(MirrorState::default())
    }

    /// Save state to disk
    pub async fn save_state(&self) -> Result<()> {
        Self::save_state_internal(&self.state).await
    }

    /// Internal state save helper
    async fn save_state_internal(state: &Arc<RwLock<MirrorState>>) -> Result<()> {
        let state_guard = state.read().await;

        let state_path = Path::new("/var/lib/anna/mirror/state.json");
        fs::create_dir_all(state_path.parent().unwrap()).await?;

        let json = serde_json::to_string_pretty(&*state_guard)?;
        fs::write(state_path, json).await?;

        debug!("Mirror state saved to {:?}", state_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mirror_protocol_creation() {
        let result = MirrorProtocol::new("test_node".to_string(), "test_key".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_reflection() {
        let mirror = MirrorProtocol::new("test_node".to_string(), "test_key".to_string())
            .await
            .unwrap();

        let reflection = mirror.generate_reflection().await.unwrap();
        assert_eq!(reflection.node_id, "test_node");

        let status = mirror.get_status().await;
        assert_eq!(status.recent_reflections_count, 1);
    }
}

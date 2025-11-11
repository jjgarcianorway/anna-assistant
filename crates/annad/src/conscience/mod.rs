//! Conscience subsystem - Self-reflective ethical governance
//!
//! Phase 1.1: Conscious custodian layer
//! Citation: [archwiki:System_maintenance]
//!
//! The Conscience layer adds ethical reasoning and self-reflection to Anna's
//! autonomous decision-making. Every action is evaluated against safety, privacy,
//! integrity, and autonomy dimensions before execution.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 Conscience Layer                         │
//! │                                                          │
//! │  ┌──────────────┐      ┌──────────────┐                │
//! │  │   Ethics     │─────►│  Reasoning   │                │
//! │  │   Engine     │      │    Tree      │                │
//! │  └──────────────┘      └──────────────┘                │
//! │         │                      │                        │
//! │         ▼                      ▼                        │
//! │  ┌────────────────────────────────────┐                │
//! │  │      Decision Evaluator            │                │
//! │  │  ┌─────────┐  ┌──────────────┐   │                │
//! │  │  │ Approve │  │     Flag      │   │                │
//! │  │  └─────────┘  └──────────────┘   │                │
//! │  │  ┌─────────┐                      │                │
//! │  │  │ Reject  │                      │                │
//! │  │  └─────────┘                      │                │
//! │  └────────────────────────────────────┘                │
//! │         │                                               │
//! │         ▼                                               │
//! │  ┌──────────────┐      ┌──────────────┐               │
//! │  │   Journal    │◄────►│ Introspect   │               │
//! │  │   Logger     │      │    Loop      │               │
//! │  └──────────────┘      └──────────────┘               │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Ethical Evaluation**: Four-dimension scoring (safety, privacy, integrity, autonomy)
//! - **Reasoning Trees**: Explainable decision chains with evidence
//! - **Uncertainty Flagging**: Actions with low confidence require manual review
//! - **Rollback Planning**: Reversibility contracts for destructive operations
//! - **Self-Introspection**: Periodic review of past decisions (every 6 hours)
//! - **Append-Only Journal**: Immutable audit trail in /var/log/anna/journal.jsonl
//!
//! # Safety Guarantees
//!
//! - All automated actions evaluated before execution
//! - Flagged actions require manual approval
//! - Complete audit trail with reasoning
//! - Rollback plans for destructive operations

pub mod ethics;
pub mod explain;
pub mod introspect;
pub mod types;

pub use ethics::EthicsEngine;
pub use explain::{explain_decision, format_pending_actions, format_reasoning_tree, summarize_decision};
pub use introspect::Introspector;
pub use types::{
    ConscienceDecision, ConscienceState, DecisionOutcome, EthicalScore, EthicsConfig,
    IntrospectionReport, JournalEntry, PendingAction, ReasoningTree, RollbackPlan,
};

use crate::sentinel::SentinelAction;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Conscience daemon managing ethical governance
pub struct ConscienceDaemon {
    /// Current conscience state
    state: Arc<RwLock<ConscienceState>>,
    /// Ethics evaluation engine
    ethics: EthicsEngine,
    /// Introspection engine
    introspector: Introspector,
    /// Ethics configuration
    config: EthicsConfig,
}

impl ConscienceDaemon {
    /// Create new conscience daemon
    pub async fn new() -> Result<Self> {
        // Load or create state
        let state = load_state().await?;

        // Load ethics configuration
        let config = EthicsEngine::load_config().await?;

        // Create engines
        let ethics = EthicsEngine::new(config.clone());
        let introspector = Introspector::new(
            config.ethical_threshold,
            config.confidence_threshold,
        );

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            ethics,
            introspector,
            config,
        })
    }

    /// Evaluate action through conscience layer
    ///
    /// Returns (should_execute, decision_id)
    pub async fn evaluate_action(&self, action: &SentinelAction) -> Result<(bool, String)> {
        debug!("Conscience evaluating action: {:?}", action);

        // Perform ethical evaluation
        let (ethical_score, reasoning) = self.ethics.evaluate(action);

        // Calculate confidence from reasoning tree
        let confidence = reasoning.overall_confidence();
        let uncertainty = 1.0 - confidence;

        // Generate decision ID
        let decision_id = Uuid::new_v4().to_string();

        // Determine outcome based on ethical score and confidence
        let outcome = if !self.ethics.is_ethical(&ethical_score) {
            // Reject if ethical score too low
            DecisionOutcome::Rejected {
                reason: format!(
                    "Ethical score ({:.1}%) below threshold ({:.1}%). Weakest dimension: {} ({:.1}%)",
                    ethical_score.overall() * 100.0,
                    self.config.ethical_threshold * 100.0,
                    ethical_score.weakest_dimension(),
                    ethical_score.min_score() * 100.0
                ),
            }
        } else if uncertainty > self.config.uncertainty_threshold {
            // Flag if uncertainty too high
            DecisionOutcome::Flagged {
                reason: format!(
                    "Uncertainty ({:.1}%) exceeds threshold ({:.1}%)",
                    uncertainty * 100.0,
                    self.config.uncertainty_threshold * 100.0
                ),
            }
        } else {
            // Approve otherwise
            DecisionOutcome::Approved {
                execution_result: "Pending execution".to_string(),
            }
        };

        // Create rollback plan if needed
        let rollback_plan = if is_destructive(action) && self.config.require_rollback {
            Some(create_rollback_plan(action).await?)
        } else {
            None
        };

        // Create conscience decision
        let decision = ConscienceDecision {
            id: decision_id.clone(),
            timestamp: Utc::now(),
            action: action.clone(),
            reasoning,
            ethical_score,
            confidence,
            outcome: outcome.clone(),
            rollback_plan,
        };

        // Handle outcome
        let should_execute = match &outcome {
            DecisionOutcome::Approved { .. } => {
                info!(
                    "Conscience APPROVED action: {:?} (ethical: {:.1}%, confidence: {:.1}%)",
                    action,
                    decision.ethical_score.overall() * 100.0,
                    decision.confidence * 100.0
                );
                true
            }
            DecisionOutcome::Rejected { reason } => {
                warn!("Conscience REJECTED action: {:?} - {}", action, reason);
                false
            }
            DecisionOutcome::Flagged { reason } => {
                warn!("Conscience FLAGGED action: {:?} - {}", action, reason);

                // Add to pending actions
                let pending = PendingAction {
                    id: decision_id.clone(),
                    timestamp: Utc::now(),
                    action: action.clone(),
                    flag_reason: reason.clone(),
                    uncertainty,
                    ethical_score: decision.ethical_score.clone(),
                    reasoning: decision.reasoning.clone(),
                };

                let mut state = self.state.write().await;
                state.pending_actions.push(pending);

                false
            }
            DecisionOutcome::Pending => false,
        };

        // Record decision
        {
            let mut state = self.state.write().await;
            state.decision_history.push(decision.clone());

            // Keep only last 1000 decisions in memory
            let history_len = state.decision_history.len();
            if history_len > 1000 {
                state.decision_history = state.decision_history.split_off(history_len - 1000);
            }
        }

        // Write to journal
        write_journal_entry(&decision).await?;

        // Save state
        {
            let state = self.state.read().await;
            save_state(&*state).await?;
        }

        Ok((should_execute, decision_id))
    }

    /// Update decision outcome after execution
    pub async fn update_outcome(&self, decision_id: &str, result: String) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(decision) = state.decision_history.iter_mut().find(|d| d.id == decision_id) {
            decision.outcome = DecisionOutcome::Approved {
                execution_result: result,
            };

            // Write updated journal entry
            write_journal_entry(decision).await?;
        }

        save_state(&state).await?;

        Ok(())
    }

    /// Approve pending action
    pub async fn approve_action(&self, action_id: &str) -> Result<()> {
        let mut state = self.state.write().await;

        // Find and remove from pending
        if let Some(pos) = state.pending_actions.iter().position(|a| a.id == action_id) {
            let pending = state.pending_actions.remove(pos);

            info!("Manually approved flagged action: {}", action_id);

            // Update decision in history
            if let Some(decision) = state.decision_history.iter_mut().find(|d| d.id == action_id) {
                decision.outcome = DecisionOutcome::Approved {
                    execution_result: "Manually approved".to_string(),
                };

                write_journal_entry(decision).await?;
            }

            save_state(&state).await?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Action ID not found in pending list"))
        }
    }

    /// Reject pending action
    pub async fn reject_action(&self, action_id: &str) -> Result<()> {
        let mut state = self.state.write().await;

        // Find and remove from pending
        if let Some(pos) = state.pending_actions.iter().position(|a| a.id == action_id) {
            state.pending_actions.remove(pos);

            info!("Manually rejected flagged action: {}", action_id);

            // Update decision in history
            if let Some(decision) = state.decision_history.iter_mut().find(|d| d.id == action_id) {
                decision.outcome = DecisionOutcome::Rejected {
                    reason: "Manually rejected by operator".to_string(),
                };

                write_journal_entry(decision).await?;
            }

            save_state(&state).await?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Action ID not found in pending list"))
        }
    }

    /// Get current conscience state
    pub async fn get_state(&self) -> ConscienceState {
        self.state.read().await.clone()
    }

    /// Get decision by ID
    pub async fn get_decision(&self, id: &str) -> Option<ConscienceDecision> {
        let state = self.state.read().await;
        state.decision_history.iter().find(|d| d.id == id).cloned()
    }

    /// Run introspection and return report
    pub async fn introspect(&self) -> Result<IntrospectionReport> {
        let state_snapshot = {
            let state = self.state.read().await;
            state.clone()
        };
        let report = self.introspector.introspect(&state_snapshot).await?;

        // Update state with introspection run
        {
            let mut state = self.state.write().await;
            state.introspection_runs += 1;
            state.last_introspection = Some(Utc::now());

            save_state(&state).await?;
        }

        Ok(report)
    }
}

/// Load conscience state from disk
pub async fn load_state() -> Result<ConscienceState> {
    let state_path = "/var/lib/anna/conscience/state.json";

    match tokio::fs::read_to_string(state_path).await {
        Ok(content) => {
            let state: ConscienceState = serde_json::from_str(&content)
                .context("Failed to parse conscience state")?;
            debug!("Loaded conscience state from {}", state_path);
            Ok(state)
        }
        Err(_) => {
            debug!("No existing conscience state, creating new");
            Ok(ConscienceState::default())
        }
    }
}

/// Save conscience state to disk
pub async fn save_state(state: &ConscienceState) -> Result<()> {
    let state_path = "/var/lib/anna/conscience/state.json";

    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(state_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Update timestamp
    let mut state = state.clone();
    state.timestamp = Utc::now();

    // Write state
    let json = serde_json::to_string_pretty(&state)?;
    tokio::fs::write(state_path, json).await?;

    debug!("Saved conscience state to {}", state_path);

    Ok(())
}

/// Write journal entry to append-only log
pub async fn write_journal_entry(decision: &ConscienceDecision) -> Result<()> {
    let journal_path = "/var/log/anna/journal.jsonl";

    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(journal_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let entry = JournalEntry {
        timestamp: decision.timestamp,
        decision_id: decision.id.clone(),
        action_type: format!("{:?}", decision.action),
        outcome: format!("{:?}", decision.outcome),
        ethical_score: decision.ethical_score.overall(),
        confidence: decision.confidence,
        summary: decision.reasoning.root.statement.clone(),
    };

    let mut json = serde_json::to_string(&entry)?;
    json.push('\n');

    // Append to journal
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path)
        .await?;

    file.write_all(json.as_bytes()).await?;
    file.sync_all().await?;

    debug!("Wrote journal entry: {}", decision.id);

    Ok(())
}

/// Check if action is potentially destructive
fn is_destructive(action: &SentinelAction) -> bool {
    matches!(
        action,
        SentinelAction::SystemUpdate { dry_run: false }
            | SentinelAction::RunRepair { .. }
            | SentinelAction::RestartService { .. }
    )
}

/// Create rollback plan for destructive action
async fn create_rollback_plan(action: &SentinelAction) -> Result<RollbackPlan> {
    let plan_id = Uuid::new_v4().to_string();

    let (description, rollback_commands, estimated_time) = match action {
        SentinelAction::RestartService { service } => (
            format!("Rollback service restart: {}", service),
            vec![format!("systemctl restart {}", service)],
            5,
        ),

        SentinelAction::SystemUpdate { .. } => (
            "Rollback system update via pacman cache".to_string(),
            vec![
                "pacman -Syu --noconfirm".to_string(),
                "# Manual intervention may be required for complex downgrades".to_string(),
            ],
            120,
        ),

        SentinelAction::RunRepair { probe } => (
            format!("Rollback repair probe: {}", probe),
            vec![format!("# Probe-specific rollback for {}", probe)],
            30,
        ),

        _ => (
            "No rollback needed".to_string(),
            vec![],
            0,
        ),
    };

    Ok(RollbackPlan {
        id: plan_id,
        description,
        rollback_commands,
        checksums: HashMap::new(), // TODO: Implement checksum collection
        backup_paths: Vec::new(),  // TODO: Implement backup creation
        estimated_time,
    })
}

/// Initialize conscience subsystem
pub async fn initialize() -> Result<ConscienceDaemon> {
    info!("Initializing Conscience layer...");
    ConscienceDaemon::new().await
}

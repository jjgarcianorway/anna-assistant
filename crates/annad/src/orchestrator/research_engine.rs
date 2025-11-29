//! DEPRECATED: Research Loop Engine v0.15.0
//!
//! **WARNING**: This file is deprecated. Use `engine_v90.rs` (UnifiedEngine) instead.
//! See `docs/architecture.md` Section 9 for details.
//!
//! This module remains for backward compatibility but is not called in production.
//!
//! Junior (LLM-A) / Senior (LLM-B) architecture with:
//! - Command whitelist (no arbitrary shell)
//! - Max 6 LLM-A iterations
//! - Max 2 LLM-B review passes
//! - User question flow (ask_user)
//! - Reasoning trace logging

use anna_common::{
    command_whitelist::{CommandRegistry, CommandRisk, WhitelistError},
    CheckApproval, CheckRequest, CheckResult, CheckRisk, CheckRiskEval, CoreProbeId, DynamicCheck,
    FactSource, LlmAResponseV15, LlmBResponseV15, LlmBVerdict, ReasoningTrace, TraceStep,
    TrackedFact, UserQuestion,
};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Maximum LLM-A iterations per request
pub const MAX_LLM_A_ITERATIONS: usize = 6;

/// Maximum LLM-B review passes
pub const MAX_LLM_B_PASSES: usize = 2;

/// Default command timeout in seconds
pub const DEFAULT_COMMAND_TIMEOUT: u64 = 30;

/// Research loop state
#[derive(Debug, Clone)]
pub struct ResearchState {
    /// Current LLM-A iteration (1-based)
    pub iteration: usize,
    /// LLM-B pass count
    pub llm_b_passes: usize,
    /// Collected evidence from executed checks
    pub evidence: Vec<CheckResult>,
    /// Known facts (measured, user-asserted, inferred)
    pub facts: Vec<TrackedFact>,
    /// Pending user question (if any)
    pub pending_question: Option<UserQuestion>,
    /// Last mentor feedback from LLM-B
    pub mentor_feedback: Option<String>,
    /// Reasoning trace for debugging
    pub trace: ReasoningTrace,
    /// Is the loop complete?
    pub completed: bool,
    /// Final answer (if complete)
    pub final_answer: Option<String>,
    /// Final confidence
    pub final_confidence: f64,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            iteration: 0,
            llm_b_passes: 0,
            evidence: vec![],
            facts: vec![],
            pending_question: None,
            mentor_feedback: None,
            trace: ReasoningTrace::new(),
            completed: false,
            final_answer: None,
            final_confidence: 0.0,
        }
    }
}

impl ResearchState {
    pub fn can_continue(&self) -> bool {
        !self.completed && self.iteration < MAX_LLM_A_ITERATIONS && self.pending_question.is_none()
    }

    pub fn next_iteration(&mut self) {
        self.iteration += 1;
    }
}

/// Research engine configuration
#[derive(Debug, Clone)]
pub struct ResearchConfig {
    /// System mode (normal or dev)
    pub mode: String,
    /// Auto-approve low-risk commands in normal mode
    pub auto_approve_low: bool,
    /// Auto-approve medium-risk commands in dev mode
    pub auto_approve_medium_dev: bool,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            mode: "normal".to_string(),
            auto_approve_low: true,
            auto_approve_medium_dev: true,
        }
    }
}

/// Research Loop Engine v0.15.0
pub struct ResearchEngine {
    /// Command whitelist registry
    registry: CommandRegistry,
    /// Configuration
    config: ResearchConfig,
}

impl ResearchEngine {
    pub fn new(config: ResearchConfig) -> Self {
        Self {
            registry: CommandRegistry::new(),
            config,
        }
    }

    /// Process check requests from LLM-A, filtering through whitelist
    pub fn process_check_requests(&self, requests: &[CheckRequest]) -> Vec<ProcessedCheck> {
        let mut processed = Vec::new();

        for request in requests {
            match request {
                CheckRequest::CoreProbe { probe_id, reason } => {
                    // Map core probe to whitelist command
                    if let Some(cmd_id) = self.core_probe_to_command(probe_id) {
                        if let Some(cmd) = self.registry.get(cmd_id) {
                            processed.push(ProcessedCheck {
                                id: probe_id.clone(),
                                command: cmd.template.to_string(),
                                risk: self.check_risk_to_command_risk(CheckRisk::ReadOnlyLow),
                                reason: reason.clone(),
                                approved: false,
                                denial_reason: None,
                            });
                        }
                    } else {
                        warn!("Unknown core probe: {}", probe_id);
                    }
                }
                CheckRequest::ReuseCheck { check_id, reason } => {
                    // Look up existing check - for now, treat as dynamic
                    if let Some(cmd) = self.registry.get(check_id) {
                        processed.push(ProcessedCheck {
                            id: check_id.clone(),
                            command: cmd.template.to_string(),
                            risk: cmd.risk,
                            reason: reason.clone(),
                            approved: false,
                            denial_reason: None,
                        });
                    }
                }
                CheckRequest::NewCheck {
                    name,
                    command,
                    risk,
                    reason,
                    tags: _,
                } => {
                    // Validate against whitelist
                    match self.registry.matches_whitelist(command) {
                        Some(matched) => {
                            processed.push(ProcessedCheck {
                                id: name.clone(),
                                command: command.clone(),
                                risk: matched.command.risk,
                                reason: reason.clone(),
                                approved: false,
                                denial_reason: None,
                            });
                        }
                        None => {
                            // Command not in whitelist - deny
                            processed.push(ProcessedCheck {
                                id: name.clone(),
                                command: command.clone(),
                                risk: self.check_risk_to_command_risk(*risk),
                                reason: reason.clone(),
                                approved: false,
                                denial_reason: Some(
                                    "Command not in whitelist - cannot execute".to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        processed
    }

    /// Apply LLM-B approvals to processed checks
    pub fn apply_approvals(&self, checks: &mut [ProcessedCheck], approvals: &[CheckRiskEval]) {
        for check in checks.iter_mut() {
            // Find matching approval
            if let Some(eval) = approvals.iter().find(|a| a.check_ref == check.id) {
                match eval.approval {
                    CheckApproval::AllowNow => {
                        check.approved = true;
                    }
                    CheckApproval::AllowAfterUserConfirm => {
                        // Will need user confirmation - mark as pending
                        check.approved = false;
                        check.denial_reason = Some("Requires user confirmation".to_string());
                    }
                    CheckApproval::Deny => {
                        check.approved = false;
                        check.denial_reason = Some(eval.explanation.clone());
                    }
                }
            } else {
                // No explicit approval - apply default policy
                check.approved = self.auto_approve(&check.risk);
                if !check.approved && check.denial_reason.is_none() {
                    check.denial_reason = Some("Not approved by LLM-B".to_string());
                }
            }
        }
    }

    /// Check if a command risk level can be auto-approved
    fn auto_approve(&self, risk: &CommandRisk) -> bool {
        match risk {
            CommandRisk::Low => self.config.auto_approve_low,
            CommandRisk::Medium => self.config.mode == "dev" && self.config.auto_approve_medium_dev,
            CommandRisk::High => false, // Never auto-approve high risk
        }
    }

    /// Execute approved checks and return results
    pub async fn execute_checks(&self, checks: &[ProcessedCheck]) -> Vec<CheckResult> {
        let mut results = Vec::new();

        for check in checks {
            if !check.approved {
                debug!("Skipping unapproved check: {}", check.id);
                continue;
            }

            info!("Executing check: {} - {}", check.id, check.command);

            match self
                .execute_command(&check.command, DEFAULT_COMMAND_TIMEOUT)
                .await
            {
                Ok((exit_code, stdout, stderr)) => {
                    results.push(CheckResult {
                        check_id: check.id.clone(),
                        command: check.command.clone(),
                        exit_code,
                        stdout: Some(Self::truncate_output(&stdout, 4000)),
                        stderr: if stderr.is_empty() {
                            None
                        } else {
                            Some(Self::truncate_output(&stderr, 1000))
                        },
                        executed_at: Utc::now(),
                    });
                }
                Err(e) => {
                    error!("Check {} failed: {}", check.id, e);
                    results.push(CheckResult {
                        check_id: check.id.clone(),
                        command: check.command.clone(),
                        exit_code: -1,
                        stdout: None,
                        stderr: Some(format!("Execution error: {}", e)),
                        executed_at: Utc::now(),
                    });
                }
            }
        }

        results
    }

    /// Execute a single command with timeout
    async fn execute_command(
        &self,
        command: &str,
        timeout_secs: u64,
    ) -> Result<(i32, String, String)> {
        let cmd = command.to_string();

        // Run in blocking thread
        let result = tokio::task::spawn_blocking(move || {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .context("Failed to execute command")?;

            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok::<_, anyhow::Error>((exit_code, stdout, stderr))
        });

        // Apply timeout
        match timeout(Duration::from_secs(timeout_secs), result).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(anyhow::anyhow!("Command timed out after {}s", timeout_secs)),
        }
    }

    /// Map core probe ID to whitelist command ID
    fn core_probe_to_command(&self, probe_id: &str) -> Option<&'static str> {
        match probe_id {
            "core.cpu_info" => Some("cpu_info"),
            "core.mem_info" => Some("mem_info"),
            "core.disk_layout" => Some("disk_layout"),
            "core.fs_usage_root" => Some("disk_usage_root"),
            "core.net_links" => Some("net_interfaces"),
            "core.net_addr" => Some("net_addresses"),
            "core.dns_resolv" => Some("dns_config"),
            _ => None,
        }
    }

    /// Convert CheckRisk to CommandRisk
    fn check_risk_to_command_risk(&self, risk: CheckRisk) -> CommandRisk {
        match risk {
            CheckRisk::ReadOnlyLow => CommandRisk::Low,
            CheckRisk::ReadOnlyMedium => CommandRisk::Medium,
            CheckRisk::WriteLow => CommandRisk::Medium,
            CheckRisk::WriteMedium => CommandRisk::High,
            CheckRisk::WriteHigh => CommandRisk::High,
        }
    }

    /// Truncate output to max length
    fn truncate_output(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}... [truncated]", &s[..max_len])
        }
    }

    /// Build a parameterized command from whitelist
    pub fn build_command(
        &self,
        cmd_id: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, WhitelistError> {
        self.registry.build_command(cmd_id, params)
    }

    /// Get the command registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get available core probes as info
    pub fn core_probes_info(&self) -> Vec<(String, String)> {
        CoreProbeId::all()
            .iter()
            .map(|p| (p.as_str().to_string(), p.description().to_string()))
            .collect()
    }
}

impl Default for ResearchEngine {
    fn default() -> Self {
        Self::new(ResearchConfig::default())
    }
}

/// A check that has been processed through the whitelist
#[derive(Debug, Clone)]
pub struct ProcessedCheck {
    /// Check identifier
    pub id: String,
    /// The actual command to execute
    pub command: String,
    /// Risk level
    pub risk: CommandRisk,
    /// Reason for this check
    pub reason: String,
    /// Whether this check is approved for execution
    pub approved: bool,
    /// If not approved, why
    pub denial_reason: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = ResearchEngine::default();
        assert!(!engine.registry().all().is_empty());
    }

    #[test]
    fn test_core_probe_mapping() {
        let engine = ResearchEngine::default();

        assert_eq!(
            engine.core_probe_to_command("core.cpu_info"),
            Some("cpu_info")
        );
        assert_eq!(
            engine.core_probe_to_command("core.mem_info"),
            Some("mem_info")
        );
        assert_eq!(engine.core_probe_to_command("unknown"), None);
    }

    #[test]
    fn test_auto_approve_policy() {
        let engine = ResearchEngine::default();

        // Low risk should auto-approve
        assert!(engine.auto_approve(&CommandRisk::Low));

        // High risk should never auto-approve
        assert!(!engine.auto_approve(&CommandRisk::High));
    }

    #[test]
    fn test_auto_approve_dev_mode() {
        let config = ResearchConfig {
            mode: "dev".to_string(),
            auto_approve_low: true,
            auto_approve_medium_dev: true,
        };
        let engine = ResearchEngine::new(config);

        // Medium risk should auto-approve in dev mode
        assert!(engine.auto_approve(&CommandRisk::Medium));
    }

    #[test]
    fn test_auto_approve_normal_mode() {
        let config = ResearchConfig {
            mode: "normal".to_string(),
            auto_approve_low: true,
            auto_approve_medium_dev: true,
        };
        let engine = ResearchEngine::new(config);

        // Medium risk should NOT auto-approve in normal mode
        assert!(!engine.auto_approve(&CommandRisk::Medium));
    }

    #[test]
    fn test_process_core_probe_request() {
        let engine = ResearchEngine::default();

        let requests = vec![CheckRequest::CoreProbe {
            probe_id: "core.cpu_info".to_string(),
            reason: "Need CPU info".to_string(),
        }];

        let processed = engine.process_check_requests(&requests);
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].id, "core.cpu_info");
        assert!(processed[0].command.contains("lscpu"));
    }

    #[test]
    fn test_process_invalid_command() {
        let engine = ResearchEngine::default();

        let requests = vec![CheckRequest::NewCheck {
            name: "evil_check".to_string(),
            command: "rm -rf /".to_string(), // Not in whitelist!
            risk: CheckRisk::WriteHigh,
            reason: "Destroy everything".to_string(),
            tags: vec![],
        }];

        let processed = engine.process_check_requests(&requests);
        assert_eq!(processed.len(), 1);
        assert!(processed[0].denial_reason.is_some());
        assert!(processed[0]
            .denial_reason
            .as_ref()
            .unwrap()
            .contains("not in whitelist"));
    }

    #[test]
    fn test_truncate_output() {
        let short = "hello";
        assert_eq!(ResearchEngine::truncate_output(short, 100), "hello");

        let long = "a".repeat(200);
        let truncated = ResearchEngine::truncate_output(&long, 50);
        assert!(truncated.len() < 200);
        assert!(truncated.ends_with("... [truncated]"));
    }

    #[test]
    fn test_research_state_can_continue() {
        let mut state = ResearchState::default();
        assert!(state.can_continue());

        state.iteration = MAX_LLM_A_ITERATIONS;
        assert!(!state.can_continue());

        state.iteration = 1;
        state.pending_question = Some(UserQuestion::free_text("test", "reason"));
        assert!(!state.can_continue());

        state.pending_question = None;
        state.completed = true;
        assert!(!state.can_continue());
    }
}

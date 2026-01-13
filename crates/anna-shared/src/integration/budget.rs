//! Budget and Stopping Policy - Prevent runaway "smartness".
//!
//! Hard caps on:
//! - Commands executed
//! - Elapsed time
//! - Disk writes
//! - Network actions
//! - Log volume per task
//!
//! A graceful "unable within budget" result is better than thrashing.
//! Budgets differ by mode: Solver < Investigator < Lab

use super::OperationMode;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Budget limits for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBudget {
    /// Maximum commands to execute
    pub max_commands: usize,
    /// Maximum elapsed time in seconds
    pub max_time_secs: u64,
    /// Maximum disk writes in bytes
    pub max_disk_writes: u64,
    /// Maximum network actions
    pub max_network_actions: usize,
    /// Maximum log volume in bytes
    pub max_log_bytes: u64,
    /// Maximum LLM tokens
    pub max_tokens: usize,
}

impl TaskBudget {
    /// Create budget for a mode
    pub fn for_mode(mode: OperationMode) -> Self {
        match mode {
            OperationMode::Solver => Self {
                max_commands: 10,
                max_time_secs: 30,
                max_disk_writes: 1024 * 1024, // 1MB
                max_network_actions: 3,
                max_log_bytes: 100 * 1024, // 100KB
                max_tokens: 4000,
            },
            OperationMode::Investigator => Self {
                max_commands: 30,
                max_time_secs: 120,
                max_disk_writes: 10 * 1024 * 1024, // 10MB
                max_network_actions: 10,
                max_log_bytes: 1024 * 1024, // 1MB
                max_tokens: 16000,
            },
            OperationMode::Lab => Self {
                max_commands: 100,
                max_time_secs: 600,
                max_disk_writes: 100 * 1024 * 1024, // 100MB
                max_network_actions: 50,
                max_log_bytes: 10 * 1024 * 1024, // 10MB
                max_tokens: 64000,
            },
        }
    }

    /// Check if an action would exceed budget
    pub fn would_exceed(&self, usage: &BudgetUsage, action: &BudgetAction) -> Option<BudgetExceeded> {
        match action {
            BudgetAction::Command => {
                if usage.commands_executed >= self.max_commands {
                    return Some(BudgetExceeded::Commands);
                }
            }
            BudgetAction::DiskWrite(bytes) => {
                if usage.disk_writes + bytes > self.max_disk_writes {
                    return Some(BudgetExceeded::DiskWrites);
                }
            }
            BudgetAction::NetworkAction => {
                if usage.network_actions >= self.max_network_actions {
                    return Some(BudgetExceeded::NetworkActions);
                }
            }
            BudgetAction::LogWrite(bytes) => {
                if usage.log_bytes + bytes > self.max_log_bytes {
                    return Some(BudgetExceeded::LogVolume);
                }
            }
            BudgetAction::TokenUse(tokens) => {
                if usage.tokens_used + tokens > self.max_tokens {
                    return Some(BudgetExceeded::Tokens);
                }
            }
        }

        // Check time
        if let Some(start) = usage.start_time {
            if start.elapsed().as_secs() > self.max_time_secs {
                return Some(BudgetExceeded::Time);
            }
        }

        None
    }
}

/// An action that consumes budget
#[derive(Debug, Clone)]
pub enum BudgetAction {
    /// Execute a command
    Command,
    /// Write to disk (bytes)
    DiskWrite(u64),
    /// Network action
    NetworkAction,
    /// Write to log (bytes)
    LogWrite(u64),
    /// Use LLM tokens
    TokenUse(usize),
}

/// Budget exceeded reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetExceeded {
    Commands,
    Time,
    DiskWrites,
    NetworkActions,
    LogVolume,
    Tokens,
}

impl BudgetExceeded {
    /// Human-readable message
    pub fn message(&self) -> &'static str {
        match self {
            BudgetExceeded::Commands => "Maximum command count reached",
            BudgetExceeded::Time => "Time budget exceeded",
            BudgetExceeded::DiskWrites => "Disk write budget exceeded",
            BudgetExceeded::NetworkActions => "Network action budget exceeded",
            BudgetExceeded::LogVolume => "Log volume budget exceeded",
            BudgetExceeded::Tokens => "Token budget exceeded",
        }
    }
}

/// Current budget usage
#[derive(Debug, Clone, Default)]
pub struct BudgetUsage {
    /// When the task started
    pub start_time: Option<Instant>,
    /// Commands executed
    pub commands_executed: usize,
    /// Bytes written to disk
    pub disk_writes: u64,
    /// Network actions performed
    pub network_actions: usize,
    /// Bytes written to logs
    pub log_bytes: u64,
    /// Tokens used
    pub tokens_used: usize,
}

impl BudgetUsage {
    /// Create new usage tracker
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Record a command execution
    pub fn record_command(&mut self) {
        self.commands_executed += 1;
    }

    /// Record disk write
    pub fn record_disk_write(&mut self, bytes: u64) {
        self.disk_writes += bytes;
    }

    /// Record network action
    pub fn record_network_action(&mut self) {
        self.network_actions += 1;
    }

    /// Record log write
    pub fn record_log_write(&mut self, bytes: u64) {
        self.log_bytes += bytes;
    }

    /// Record token usage
    pub fn record_tokens(&mut self, tokens: usize) {
        self.tokens_used += tokens;
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.map(|s| s.elapsed()).unwrap_or_default()
    }

    /// Format usage summary
    pub fn summary(&self) -> String {
        format!(
            "Commands: {}, Time: {:.1}s, Disk: {}KB, Network: {}, Logs: {}KB, Tokens: {}",
            self.commands_executed,
            self.elapsed().as_secs_f32(),
            self.disk_writes / 1024,
            self.network_actions,
            self.log_bytes / 1024,
            self.tokens_used
        )
    }
}

/// The budget controller
#[derive(Debug, Clone, Default)]
pub struct BudgetController {
    /// Current budget
    budget: Option<TaskBudget>,
    /// Current usage
    usage: BudgetUsage,
    /// Mode
    mode: OperationMode,
}

impl BudgetController {
    /// Create new controller
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new task with a mode
    pub fn start_task(&mut self, mode: OperationMode) {
        self.mode = mode;
        self.budget = Some(TaskBudget::for_mode(mode));
        self.usage = BudgetUsage::new();
    }

    /// Get budget for a mode
    pub fn get_budget(&self, mode: OperationMode) -> ActiveBudget {
        let budget = TaskBudget::for_mode(mode);
        let usage = if self.budget.is_some() {
            self.usage.clone()
        } else {
            BudgetUsage::new()
        };

        ActiveBudget {
            budget,
            usage,
            mode,
        }
    }

    /// Check if an action is allowed
    pub fn check_action(&self, action: &BudgetAction) -> Result<(), BudgetExceeded> {
        if let Some(ref budget) = self.budget {
            if let Some(exceeded) = budget.would_exceed(&self.usage, action) {
                return Err(exceeded);
            }
        }
        Ok(())
    }

    /// Record an action (fails if budget exceeded)
    pub fn record_action(&mut self, action: BudgetAction) -> Result<(), BudgetExceeded> {
        self.check_action(&action)?;

        match action {
            BudgetAction::Command => self.usage.record_command(),
            BudgetAction::DiskWrite(bytes) => self.usage.record_disk_write(bytes),
            BudgetAction::NetworkAction => self.usage.record_network_action(),
            BudgetAction::LogWrite(bytes) => self.usage.record_log_write(bytes),
            BudgetAction::TokenUse(tokens) => self.usage.record_tokens(tokens),
        }

        Ok(())
    }

    /// Get remaining budget
    pub fn remaining(&self) -> Option<RemainingBudget> {
        let budget = self.budget.as_ref()?;
        let elapsed = self.usage.elapsed().as_secs();

        Some(RemainingBudget {
            commands: budget.max_commands.saturating_sub(self.usage.commands_executed),
            time_secs: budget.max_time_secs.saturating_sub(elapsed),
            disk_bytes: budget.max_disk_writes.saturating_sub(self.usage.disk_writes),
            network_actions: budget
                .max_network_actions
                .saturating_sub(self.usage.network_actions),
            log_bytes: budget.max_log_bytes.saturating_sub(self.usage.log_bytes),
            tokens: budget.max_tokens.saturating_sub(self.usage.tokens_used),
        })
    }

    /// Get usage summary
    pub fn usage_summary(&self) -> String {
        self.usage.summary()
    }

    /// Generate graceful stop result
    pub fn graceful_stop(&self, reason: BudgetExceeded) -> GracefulStopResult {
        GracefulStopResult {
            reason,
            message: reason.message().to_string(),
            usage_summary: self.usage.summary(),
            mode: self.mode,
            recommendation: match self.mode {
                OperationMode::Solver => "Consider using investigator mode for complex tasks".to_string(),
                OperationMode::Investigator => "Task may require lab mode or manual intervention".to_string(),
                OperationMode::Lab => "Task exceeds maximum budget - requires manual handling".to_string(),
            },
        }
    }
}

/// Active budget with current usage
#[derive(Debug, Clone)]
pub struct ActiveBudget {
    pub budget: TaskBudget,
    pub usage: BudgetUsage,
    pub mode: OperationMode,
}

impl ActiveBudget {
    /// Check if we can proceed
    pub fn can_proceed(&self) -> bool {
        if let Some(start) = self.usage.start_time {
            if start.elapsed().as_secs() > self.budget.max_time_secs {
                return false;
            }
        }
        self.usage.commands_executed < self.budget.max_commands
    }
}

/// Remaining budget amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemainingBudget {
    pub commands: usize,
    pub time_secs: u64,
    pub disk_bytes: u64,
    pub network_actions: usize,
    pub log_bytes: u64,
    pub tokens: usize,
}

/// Result when gracefully stopping due to budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracefulStopResult {
    pub reason: BudgetExceeded,
    pub message: String,
    pub usage_summary: String,
    pub mode: OperationMode,
    pub recommendation: String,
}

/// Acceptance test: in a deliberately unsolvable scenario,
/// Anna must stop without damaging actions and output a bounded diagnostic summary
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_has_tightest_budget() {
        let solver = TaskBudget::for_mode(OperationMode::Solver);
        let investigator = TaskBudget::for_mode(OperationMode::Investigator);
        let lab = TaskBudget::for_mode(OperationMode::Lab);

        assert!(solver.max_commands < investigator.max_commands);
        assert!(investigator.max_commands < lab.max_commands);
        assert!(solver.max_time_secs < investigator.max_time_secs);
    }

    #[test]
    fn test_budget_exceeded_detection() {
        let budget = TaskBudget::for_mode(OperationMode::Solver);
        let mut usage = BudgetUsage::new();

        // Execute max commands
        for _ in 0..budget.max_commands {
            usage.record_command();
        }

        // Next command should exceed
        let result = budget.would_exceed(&usage, &BudgetAction::Command);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), BudgetExceeded::Commands);
    }

    #[test]
    fn test_graceful_stop() {
        let mut controller = BudgetController::new();
        controller.start_task(OperationMode::Solver);

        let result = controller.graceful_stop(BudgetExceeded::Commands);
        assert!(!result.message.is_empty());
        assert!(!result.recommendation.is_empty());
    }

    #[test]
    fn test_remaining_budget() {
        let mut controller = BudgetController::new();
        controller.start_task(OperationMode::Solver);

        controller.record_action(BudgetAction::Command).unwrap();
        controller.record_action(BudgetAction::Command).unwrap();

        let remaining = controller.remaining().unwrap();
        assert!(remaining.commands < TaskBudget::for_mode(OperationMode::Solver).max_commands);
    }

    #[test]
    fn test_controller_blocks_over_budget() {
        let mut controller = BudgetController::new();
        controller.start_task(OperationMode::Solver);

        let budget = TaskBudget::for_mode(OperationMode::Solver);

        // Use all commands
        for _ in 0..budget.max_commands {
            assert!(controller.record_action(BudgetAction::Command).is_ok());
        }

        // Next should fail
        let result = controller.record_action(BudgetAction::Command);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BudgetExceeded::Commands);
    }
}

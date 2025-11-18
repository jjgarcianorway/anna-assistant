//! Autonomy command - set Anna's autonomy level
//!
//! Real Anna: `annactl autonomy [level]`
//! Purpose: Control how much Anna can do automatically
//! Levels:
//! - manual: No automatic changes, only on explicit `annactl repair`
//! - assisted: Safe, non-destructive auto-fixes (cache cleanup, log rotation, restart user services)
//! - proactive: Small set of autonomous actions, never risks data loss (no fsck, partitioning, config rewrites)

use anyhow::Result;
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;

/// Autonomy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyLevel {
    Manual,
    Assisted,
    Proactive,
}

impl AutonomyLevel {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "manual" => Ok(AutonomyLevel::Manual),
            "assisted" => Ok(AutonomyLevel::Assisted),
            "proactive" => Ok(AutonomyLevel::Proactive),
            _ => Err(anyhow::anyhow!(
                "Invalid autonomy level. Valid levels: manual, assisted, proactive"
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AutonomyLevel::Manual => "manual",
            AutonomyLevel::Assisted => "assisted",
            AutonomyLevel::Proactive => "proactive",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AutonomyLevel::Manual => {
                "No automatic changes. All repairs require explicit confirmation."
            }
            AutonomyLevel::Assisted => {
                "Safe automatic fixes only (cache cleanup, log rotation, service restarts)."
            }
            AutonomyLevel::Proactive => {
                "Autonomous actions allowed, but never anything that risks data loss."
            }
        }
    }
}

/// Execute 'autonomy' command - set or show autonomy level
pub async fn execute_autonomy_command(
    level: Option<String>,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // TODO: Implement autonomy level persistence in context DB
    // This should:
    // - If no level provided, show current level
    // - If level provided, validate and set it
    // - Persist to context database
    // - Show clear explanation of what each level means
    // - annad daemon must respect this setting

    if let Some(level_str) = level {
        let level = AutonomyLevel::from_str(&level_str)?;

        println!("Autonomy Level Set");
        println!("==================\n");
        println!("Level: {}", level.as_str());
        println!("{}\n", level.description());
        println!("[TODO: Persist to context database]\n");

        // Log command
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id: req_id.to_string(),
            state: state.to_string(),
            command: "autonomy".to_string(),
            allowed: Some(true),
            args: vec![level_str],
            exit_code: EXIT_SUCCESS,
            citation: "[archwiki:System_maintenance]".to_string(),
            duration_ms,
            ok: true,
            error: None,
        };
        let _ = log_entry.write();
    } else {
        // Show current level
        println!("Current Autonomy Level");
        println!("======================\n");
        println!("[TODO: Read from context database]\n");
        println!("Default: manual\n");
        println!("Available levels:");
        println!("  manual     - {}", AutonomyLevel::Manual.description());
        println!("  assisted   - {}", AutonomyLevel::Assisted.description());
        println!("  proactive  - {}", AutonomyLevel::Proactive.description());
        println!("\nSet level with: annactl autonomy <level>\n");

        // Log command
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id: req_id.to_string(),
            state: state.to_string(),
            command: "autonomy".to_string(),
            allowed: Some(true),
            args: vec![],
            exit_code: EXIT_SUCCESS,
            citation: "[archwiki:System_maintenance]".to_string(),
            duration_ms,
            ok: true,
            error: None,
        };
        let _ = log_entry.write();
    }

    Ok(())
}

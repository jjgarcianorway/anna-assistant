//! Probe executor - runs probes and parses output

use crate::parser;
use anna_common::{ProbeDefinition, ProbeResult};
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error};

/// Probe executor
pub struct ProbeExecutor;

impl ProbeExecutor {
    /// Execute a probe and parse its output
    pub async fn execute(probe: &ProbeDefinition) -> Result<ProbeResult> {
        let timestamp = Utc::now();

        // Build command
        let (program, args) = probe
            .cmd
            .split_first()
            .context("Probe command is empty")?;

        debug!("  Executing: {} {:?}", program, args);

        // Run command
        let output = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute probe command")?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("  Probe failed: {}", stderr);
            return Ok(ProbeResult {
                id: probe.id.clone(),
                success: false,
                data: serde_json::Value::Null,
                cached: false,
                timestamp,
                error: Some(stderr.to_string()),
            });
        }

        // Get raw output
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse output
        let data = parser::parse(&probe.parser, &stdout).context("Failed to parse probe output")?;

        Ok(ProbeResult {
            id: probe.id.clone(),
            success: true,
            data,
            cached: false,
            timestamp,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::CachePolicy;

    #[tokio::test]
    async fn test_execute_echo() {
        let probe = ProbeDefinition {
            id: "test.echo".to_string(),
            cmd: vec!["echo".to_string(), "hello".to_string()],
            parser: "raw_v1".to_string(),
            cache_policy: CachePolicy::Volatile,
            ttl: 0,
        };

        let result = ProbeExecutor::execute(&probe).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data.as_str().unwrap().trim(), "hello");
    }

    #[tokio::test]
    async fn test_execute_failure() {
        let probe = ProbeDefinition {
            id: "test.fail".to_string(),
            cmd: vec!["false".to_string()],
            parser: "raw_v1".to_string(),
            cache_policy: CachePolicy::Volatile,
            ttl: 0,
        };

        let result = ProbeExecutor::execute(&probe).await.unwrap();
        assert!(!result.success);
    }
}

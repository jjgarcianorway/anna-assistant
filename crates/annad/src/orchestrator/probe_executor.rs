//! Probe Executor v0.10.0
//!
//! Executes probes from the catalog and wraps results in evidence.

use anna_common::{EvidenceStatus, ProbeCatalog, ProbeEvidenceV10};
use anyhow::Result;
use std::process::Command;
use tracing::{debug, error, info};

/// Execute a probe and return structured evidence
pub async fn execute_probe(catalog: &ProbeCatalog, probe_id: &str) -> Result<ProbeEvidenceV10> {
    let probe_def = catalog
        .get(probe_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown probe: {}", probe_id))?;

    info!("Executing probe: {}", probe_id);

    let timestamp = chrono::Utc::now().to_rfc3339();

    // Handle internal probes
    if probe_def
        .commands
        .first()
        .map(|c| c.starts_with("internal:"))
        .unwrap_or(false)
    {
        return execute_internal_probe(probe_id, &timestamp).await;
    }

    // Execute shell command(s)
    let mut combined_output = String::new();
    let mut last_status = EvidenceStatus::Ok;

    for cmd in &probe_def.commands {
        match execute_shell_command(cmd).await {
            Ok(output) => {
                if !combined_output.is_empty() {
                    combined_output.push_str("\n---\n");
                }
                combined_output.push_str(&output);
            }
            Err(e) => {
                error!("Probe {} command failed: {}", probe_id, e);
                last_status = EvidenceStatus::Error;
                combined_output.push_str(&format!("Error: {}", e));
            }
        }
    }

    // Try to parse as JSON if possible
    let parsed = try_parse_json(&combined_output);

    // Truncate raw output if too large (keep first 4KB)
    let raw = if combined_output.len() > 4096 {
        Some(format!("{}... (truncated)", &combined_output[..4000]))
    } else {
        Some(combined_output)
    };

    Ok(ProbeEvidenceV10 {
        probe_id: probe_id.to_string(),
        timestamp,
        status: last_status,
        command: probe_def.commands.join(" && "),
        raw,
        parsed,
    })
}

/// Execute a shell command
async fn execute_shell_command(cmd: &str) -> Result<String> {
    debug!("Executing: {}", cmd);

    // Use tokio::spawn_blocking for shell command
    let cmd_owned = cmd.to_string();
    let output =
        tokio::task::spawn_blocking(move || Command::new("sh").arg("-c").arg(&cmd_owned).output())
            .await??;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Command failed: {}", stderr))
    }
}

/// Execute an internal probe (e.g., self-health)
async fn execute_internal_probe(probe_id: &str, timestamp: &str) -> Result<ProbeEvidenceV10> {
    match probe_id {
        "anna.self_health" => {
            // Check daemon self-health
            let health = anna_common::run_all_probes();
            let parsed = serde_json::to_value(&health).ok();

            Ok(ProbeEvidenceV10 {
                probe_id: probe_id.to_string(),
                timestamp: timestamp.to_string(),
                status: if health.overall.is_healthy() {
                    EvidenceStatus::Ok
                } else {
                    EvidenceStatus::Error
                },
                command: "internal:self_health".to_string(),
                raw: Some(anna_common::summary_line(&health)),
                parsed,
            })
        }
        _ => Err(anyhow::anyhow!("Unknown internal probe: {}", probe_id)),
    }
}

/// Try to parse output as JSON
fn try_parse_json(text: &str) -> Option<serde_json::Value> {
    // Try direct parse
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        return Some(value);
    }

    // Try to find JSON in the text
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text[start..=end]) {
                return Some(value);
            }
        }
    }

    // Try array
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text[start..=end]) {
                return Some(value);
            }
        }
    }

    None
}

/// Execute multiple probes in parallel
pub async fn execute_probes(catalog: &ProbeCatalog, probe_ids: &[String]) -> Vec<ProbeEvidenceV10> {
    let mut results = Vec::new();

    for probe_id in probe_ids {
        if !catalog.is_valid(probe_id) {
            results.push(ProbeEvidenceV10 {
                probe_id: probe_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                status: EvidenceStatus::NotFound,
                command: "".to_string(),
                raw: Some(format!("Probe '{}' not in catalog", probe_id)),
                parsed: None,
            });
            continue;
        }

        match execute_probe(catalog, probe_id).await {
            Ok(evidence) => results.push(evidence),
            Err(e) => {
                results.push(ProbeEvidenceV10 {
                    probe_id: probe_id.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    status: EvidenceStatus::Error,
                    command: "".to_string(),
                    raw: Some(format!("Execution error: {}", e)),
                    parsed: None,
                });
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_json_valid() {
        let json = r#"{"key": "value"}"#;
        let result = try_parse_json(json);
        assert!(result.is_some());
    }

    #[test]
    fn test_try_parse_json_with_prefix() {
        let text = r#"Some text before {"key": "value"} and after"#;
        let result = try_parse_json(text);
        assert!(result.is_some());
    }

    #[test]
    fn test_try_parse_json_invalid() {
        let text = "not json at all";
        let result = try_parse_json(text);
        assert!(result.is_none());
    }
}

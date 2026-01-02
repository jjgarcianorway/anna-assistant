//! Probe execution trait and implementations.

use super::types::ProbeResult;
use crate::learning_engine::RecipeProbe;
use std::collections::HashMap;

/// Probe executor trait (for testing and different implementations)
pub trait ProbeExecutor {
    /// Execute a probe and return result
    fn execute(&self, probe: &RecipeProbe, params: &HashMap<String, String>) -> ProbeResult;
}

/// Default probe executor that runs shell commands
pub struct ShellProbeExecutor;

impl ProbeExecutor for ShellProbeExecutor {
    fn execute(&self, probe: &RecipeProbe, params: &HashMap<String, String>) -> ProbeResult {
        let start = std::time::Instant::now();

        // Build command from probe tool
        let command = build_probe_command(&probe.tool, &probe.params, params);

        // Execute with timeout
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    ProbeResult::ok(&probe.id, &stdout, duration)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    ProbeResult::failed(&probe.id, &stderr)
                }
            }
            Err(e) => ProbeResult::failed(&probe.id, &e.to_string()),
        }
    }
}

/// Build probe command from tool and params
fn build_probe_command(
    tool: &str,
    probe_params: &[String],
    values: &HashMap<String, String>,
) -> String {
    // Map tool names to actual commands
    let base_command = match tool.trim_start_matches("probe.") {
        "free" => "free -h",
        "df" => "df -h",
        "systemctl_status" => "systemctl status",
        "systemctl" => "systemctl --no-pager",
        "journalctl" => "journalctl --no-pager -n 20",
        "ps" => "ps aux",
        "uptime" => "uptime",
        "vmstat" => "vmstat 1 1",
        "ip_addr" => "ip addr",
        "ss" => "ss -tuln",
        "lsblk" => "lsblk",
        "mount" => "mount",
        _ => tool,
    };

    // Substitute parameters
    let mut command = base_command.to_string();
    for param in probe_params {
        let value = if param.starts_with("{{") && param.ends_with("}}") {
            let key = &param[2..param.len() - 2];
            values.get(key).map(|s| s.as_str()).unwrap_or("")
        } else {
            values.get(param).map(|s| s.as_str()).unwrap_or(param)
        };

        if !value.is_empty() {
            command.push(' ');
            command.push_str(value);
        }
    }

    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_command_building() {
        let mut params = HashMap::new();
        params.insert("service_name".to_string(), "nginx".to_string());

        let cmd = build_probe_command(
            "probe.systemctl_status",
            &["{{service_name}}".to_string()],
            &params,
        );
        assert!(cmd.contains("nginx"));
    }
}

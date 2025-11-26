//! Anna Brain v10.0.0 - Generic Tool Catalog
//!
//! ONLY generic, reusable tools. No hardcoded detection logic.
//! The LLM decides which commands to run via run_shell.

use crate::brain_v10::contracts::EvidenceItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// Tool definition in catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameter schema
    pub parameters: serde_json::Value,
}

/// The generic tool catalog - LLM chooses what to run
pub struct ToolCatalog {
    tools: Vec<ToolSchema>,
}

impl ToolCatalog {
    /// Create the catalog with generic tools only
    pub fn new() -> Self {
        let tools = vec![
            // The main generic tool - LLM decides the command
            ToolSchema {
                name: "run_shell".to_string(),
                description: "Run any shell command. Use this to query the system. Examples: \
                    'pacman -Qs steam' (check package), 'free -m' (memory), 'lscpu' (CPU info), \
                    'df -h' (disk usage), 'which firefox' (find binary).".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Shell command to execute"
                        }
                    },
                    "required": ["command"]
                }),
            },
            // Read file contents
            ToolSchema {
                name: "read_file".to_string(),
                description: "Read contents of a text file. Useful for config files, logs, \
                    and system files like /proc/cpuinfo, /etc/hostname.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file"
                        }
                    },
                    "required": ["path"]
                }),
            },
            // Get telemetry snapshot (already collected by Anna)
            ToolSchema {
                name: "get_cached_snapshot".to_string(),
                description: "Get the pre-collected system telemetry snapshot. Includes: \
                    CPU model, RAM total, machine type, desktop environment, etc. \
                    Use this FIRST for basic system info queries.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // List processes
            ToolSchema {
                name: "list_processes".to_string(),
                description: "List running processes with CPU and memory usage (ps aux). \
                    Use to check if a program is running.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // Block devices
            ToolSchema {
                name: "list_block_devices".to_string(),
                description: "List block devices, partitions, and mount points (lsblk). \
                    Use for disk/storage queries.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        Self { tools }
    }

    /// Get the tool catalog for the LLM
    pub fn to_schema_list(&self) -> &[ToolSchema] {
        &self.tools
    }

    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing a tool (internal use)
#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Execute a tool and return result
pub fn execute_tool(
    tool_name: &str,
    arguments: &HashMap<String, String>,
    telemetry: Option<&serde_json::Value>,
) -> ToolOutput {
    match tool_name {
        "run_shell" => {
            let command = arguments.get("command").cloned().unwrap_or_default();
            if command.is_empty() {
                return ToolOutput {
                    stdout: String::new(),
                    stderr: "Missing required argument: command".to_string(),
                    exit_code: -1,
                };
            }
            run_shell_command(&command)
        }
        "read_file" => {
            let path = arguments.get("path").cloned().unwrap_or_default();
            if path.is_empty() {
                return ToolOutput {
                    stdout: String::new(),
                    stderr: "Missing required argument: path".to_string(),
                    exit_code: -1,
                };
            }
            read_file(&path)
        }
        "get_cached_snapshot" => {
            if let Some(t) = telemetry {
                ToolOutput {
                    stdout: serde_json::to_string_pretty(t).unwrap_or_else(|_| t.to_string()),
                    stderr: String::new(),
                    exit_code: 0,
                }
            } else {
                ToolOutput {
                    stdout: String::new(),
                    stderr: "No telemetry snapshot available".to_string(),
                    exit_code: 1,
                }
            }
        }
        "list_processes" => run_shell_command("ps aux --sort=-%mem | head -30"),
        "list_block_devices" => run_shell_command("lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINTS"),
        _ => ToolOutput {
            stdout: String::new(),
            stderr: format!("Unknown tool: {}", tool_name),
            exit_code: -1,
        },
    }
}

/// Run a shell command and return result
fn run_shell_command(command: &str) -> ToolOutput {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(out) => ToolOutput {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        },
        Err(e) => ToolOutput {
            stdout: String::new(),
            stderr: format!("Failed to execute: {}", e),
            exit_code: -1,
        },
    }
}

/// Read a file and return result
fn read_file(path: &str) -> ToolOutput {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            // Truncate very long files
            let truncated = if content.len() > 8000 {
                format!(
                    "{}...\n[truncated, {} bytes total]",
                    &content[..8000],
                    content.len()
                )
            } else {
                content
            };
            ToolOutput {
                stdout: truncated,
                stderr: String::new(),
                exit_code: 0,
            }
        }
        Err(e) => ToolOutput {
            stdout: String::new(),
            stderr: format!("Failed to read file: {}", e),
            exit_code: 1,
        },
    }
}

/// Convert tool output to evidence item
pub fn output_to_evidence(
    tool: &str,
    description: &str,
    output: &ToolOutput,
    evidence_id: &str,
) -> EvidenceItem {
    EvidenceItem::from_tool_result(
        evidence_id,
        tool,
        description,
        &output.stdout,
        output.exit_code,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_generic_tools() {
        let catalog = ToolCatalog::new();
        assert!(catalog.has_tool("run_shell"));
        assert!(catalog.has_tool("read_file"));
        assert!(catalog.has_tool("get_cached_snapshot"));
        assert!(catalog.has_tool("list_processes"));
        assert!(catalog.has_tool("list_block_devices"));
    }

    #[test]
    fn test_run_shell_command() {
        let mut args = HashMap::new();
        args.insert("command".to_string(), "echo hello".to_string());

        let result = execute_tool("run_shell", &args, None);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_get_cached_snapshot() {
        let telemetry = serde_json::json!({
            "cpu_model": "AMD Ryzen 9",
            "total_ram_mb": 32768
        });

        let result = execute_tool("get_cached_snapshot", &HashMap::new(), Some(&telemetry));
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("AMD Ryzen 9"));
    }

    #[test]
    fn test_unknown_tool() {
        let result = execute_tool("nonexistent", &HashMap::new(), None);
        assert!(result.stderr.contains("Unknown tool"));
    }
}

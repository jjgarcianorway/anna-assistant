//! Anna Brain Core v1.0 - Generic Tool Catalog
//!
//! ONLY generic, reusable tools. No hardcoded detection logic.
//! The LLM decides which commands to run via run_shell.

use crate::brain_v8::contracts::{ToolResult, ToolSchema};
use std::collections::HashMap;
use std::process::Command;

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
                description: "Run any shell command. Use this to query the system.".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute (e.g., 'free -m', 'pacman -Qs steam', 'lscpu')"
                        }
                    },
                    "required": ["command"]
                }),
            },
            // Read file contents
            ToolSchema {
                name: "read_file".to_string(),
                description: "Read contents of a text file.".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file (e.g., '/proc/cpuinfo', '/etc/hostname')"
                        }
                    },
                    "required": ["path"]
                }),
            },
            // List processes
            ToolSchema {
                name: "list_processes".to_string(),
                description: "List running processes (ps aux).".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // Block devices
            ToolSchema {
                name: "list_block_devices".to_string(),
                description: "List block devices and partitions (lsblk --json).".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // Network status
            ToolSchema {
                name: "network_status".to_string(),
                description: "Get network connection status.".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // Memory info
            ToolSchema {
                name: "memory_info".to_string(),
                description: "Get memory/RAM statistics (free -m).".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // CPU info
            ToolSchema {
                name: "cpu_info".to_string(),
                description: "Get CPU information (lscpu).".to_string(),
                schema: serde_json::json!({
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

/// Execute a tool with given arguments
pub fn execute_tool(
    tool_name: &str,
    arguments: &HashMap<String, String>,
) -> ToolResult {
    match tool_name {
        "run_shell" => {
            let command = arguments.get("command").cloned().unwrap_or_default();
            if command.is_empty() {
                return ToolResult {
                    tool: tool_name.to_string(),
                    arguments: arguments.clone(),
                    stdout: String::new(),
                    stderr: "Missing required argument: command".to_string(),
                    exit_code: -1,
                };
            }
            run_shell_command(&command, tool_name, arguments)
        }
        "read_file" => {
            let path = arguments.get("path").cloned().unwrap_or_default();
            if path.is_empty() {
                return ToolResult {
                    tool: tool_name.to_string(),
                    arguments: arguments.clone(),
                    stdout: String::new(),
                    stderr: "Missing required argument: path".to_string(),
                    exit_code: -1,
                };
            }
            read_file(&path, tool_name, arguments)
        }
        "list_processes" => run_shell_command("ps aux", tool_name, arguments),
        "list_block_devices" => run_shell_command("lsblk --json", tool_name, arguments),
        "network_status" => run_shell_command("nmcli -t -f TYPE,DEVICE,STATE device", tool_name, arguments),
        "memory_info" => run_shell_command("free -m", tool_name, arguments),
        "cpu_info" => run_shell_command("lscpu", tool_name, arguments),
        _ => ToolResult {
            tool: tool_name.to_string(),
            arguments: arguments.clone(),
            stdout: String::new(),
            stderr: format!("Unknown tool: {}", tool_name),
            exit_code: -1,
        },
    }
}

/// Run a shell command and return result
fn run_shell_command(
    command: &str,
    tool_name: &str,
    arguments: &HashMap<String, String>,
) -> ToolResult {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(out) => ToolResult {
            tool: tool_name.to_string(),
            arguments: arguments.clone(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        },
        Err(e) => ToolResult {
            tool: tool_name.to_string(),
            arguments: arguments.clone(),
            stdout: String::new(),
            stderr: format!("Failed to execute: {}", e),
            exit_code: -1,
        },
    }
}

/// Read a file and return result
fn read_file(
    path: &str,
    tool_name: &str,
    arguments: &HashMap<String, String>,
) -> ToolResult {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            // Truncate very long files
            let truncated = if content.len() > 10000 {
                format!("{}...[truncated, {} bytes total]", &content[..10000], content.len())
            } else {
                content
            };
            ToolResult {
                tool: tool_name.to_string(),
                arguments: arguments.clone(),
                stdout: truncated,
                stderr: String::new(),
                exit_code: 0,
            }
        }
        Err(e) => ToolResult {
            tool: tool_name.to_string(),
            arguments: arguments.clone(),
            stdout: String::new(),
            stderr: format!("Failed to read file: {}", e),
            exit_code: 1,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_generic_tools() {
        let catalog = ToolCatalog::new();
        assert!(catalog.has_tool("run_shell"));
        assert!(catalog.has_tool("read_file"));
        assert!(catalog.has_tool("memory_info"));
        assert!(catalog.has_tool("cpu_info"));
    }

    #[test]
    fn test_run_shell_command() {
        let mut args = HashMap::new();
        args.insert("command".to_string(), "echo hello".to_string());

        let result = execute_tool("run_shell", &args);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_memory_info() {
        let result = execute_tool("memory_info", &HashMap::new());
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Mem:"));
    }

    #[test]
    fn test_read_file() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "/etc/hostname".to_string());

        let result = execute_tool("read_file", &args);
        // Should succeed if file exists
        assert!(result.exit_code == 0 || result.stderr.contains("Failed"));
    }

    #[test]
    fn test_unknown_tool() {
        let result = execute_tool("nonexistent", &HashMap::new());
        assert!(result.stderr.contains("Unknown tool"));
    }
}

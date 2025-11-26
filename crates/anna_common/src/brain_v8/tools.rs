//! Brain v8 Tools - The hands of the machine
//!
//! Pure tool catalog. LLM sees only descriptions.
//! Rust owns execution. No hardcoded knowledge about what tools return.

use crate::brain_v8::contracts::ToolResult;
use std::collections::HashMap;
use std::process::Command;

/// A tool definition
#[derive(Debug, Clone)]
pub struct ToolDef {
    /// Tool name (what LLM requests)
    pub name: &'static str,
    /// Human-readable description for LLM
    pub description: &'static str,
    /// Parameters the tool accepts (name -> description)
    pub parameters: &'static [(&'static str, &'static str)],
    /// The actual command to run
    command: &'static str,
    /// Arguments (may include {param} placeholders)
    args: &'static [&'static str],
}

/// The tool catalog
pub struct ToolCatalog {
    tools: HashMap<&'static str, ToolDef>,
}

impl ToolCatalog {
    /// Create the catalog with all available tools
    pub fn new() -> Self {
        let tools = vec![
            // Hardware
            ToolDef {
                name: "mem_info",
                description: "Get memory/RAM information",
                parameters: &[],
                command: "free",
                args: &["-h"],
            },
            ToolDef {
                name: "cpu_info",
                description: "Get CPU information",
                parameters: &[],
                command: "lscpu",
                args: &[],
            },
            ToolDef {
                name: "gpu_info",
                description: "Get GPU/graphics card information",
                parameters: &[],
                command: "lspci",
                args: &["-v"],
            },
            // Packages
            ToolDef {
                name: "pacman_query",
                description: "Search for installed packages. USE THIS FIRST for 'is X installed?' questions. Returns package name and version if found, empty if not installed.",
                parameters: &[("pattern", "Package name to search for (e.g., 'steam', 'firefox')")],
                command: "pacman",
                args: &["-Qs", "{pattern}"],
            },
            ToolDef {
                name: "pacman_info",
                description: "Get detailed info (version, size, dependencies) about a package ALREADY KNOWN to be installed. Use pacman_query first to check if installed.",
                parameters: &[("package", "Exact package name (must be installed)")],
                command: "pacman",
                args: &["-Qi", "{package}"],
            },
            ToolDef {
                name: "pacman_updates",
                description: "Check for available package updates",
                parameters: &[],
                command: "checkupdates",
                args: &[],
            },
            ToolDef {
                name: "pacman_orphans",
                description: "List orphaned packages (installed but not required)",
                parameters: &[],
                command: "pacman",
                args: &["-Qdt"],
            },
            // Storage
            ToolDef {
                name: "disk_usage",
                description: "Show disk space usage for mounted filesystems",
                parameters: &[],
                command: "df",
                args: &["-h"],
            },
            ToolDef {
                name: "dir_size",
                description: "Get size of a directory",
                parameters: &[("path", "Directory path to measure")],
                command: "du",
                args: &["-sh", "{path}"],
            },
            // Network
            ToolDef {
                name: "ip_addresses",
                description: "Show network interface IP addresses",
                parameters: &[],
                command: "ip",
                args: &["-brief", "addr"],
            },
            ToolDef {
                name: "network_status",
                description: "Show network connection status",
                parameters: &[],
                command: "nmcli",
                args: &["general", "status"],
            },
            // System
            ToolDef {
                name: "systemd_failed",
                description: "List failed systemd services",
                parameters: &[],
                command: "systemctl",
                args: &["--failed", "--no-pager"],
            },
            ToolDef {
                name: "uptime",
                description: "Show system uptime",
                parameters: &[],
                command: "uptime",
                args: &["-p"],
            },
            ToolDef {
                name: "kernel_info",
                description: "Show kernel version",
                parameters: &[],
                command: "uname",
                args: &["-r"],
            },
            ToolDef {
                name: "journal_errors",
                description: "Show recent system errors from journal",
                parameters: &[],
                command: "journalctl",
                args: &["-p", "err", "-n", "20", "--no-pager"],
            },
            // Desktop
            ToolDef {
                name: "desktop_session",
                description: "Show current desktop environment",
                parameters: &[],
                command: "sh",
                args: &["-c", "echo $XDG_CURRENT_DESKTOP"],
            },
            // Files
            ToolDef {
                name: "file_exists",
                description: "Check if a file or directory exists",
                parameters: &[("path", "Path to check")],
                command: "test",
                args: &["-e", "{path}"],
            },
            ToolDef {
                name: "read_file",
                description: "Read contents of a file (first 50 lines)",
                parameters: &[("path", "File path to read")],
                command: "head",
                args: &["-n", "50", "{path}"],
            },
        ];

        let map: HashMap<&'static str, ToolDef> = tools
            .into_iter()
            .map(|t| (t.name, t))
            .collect();

        Self { tools: map }
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.get(name)
    }

    /// Build the catalog description for the LLM prompt
    pub fn to_prompt_string(&self) -> String {
        let mut lines: Vec<String> = self.tools
            .values()
            .map(|t| {
                if t.parameters.is_empty() {
                    format!("- {}: {}", t.name, t.description)
                } else {
                    let params: Vec<String> = t.parameters
                        .iter()
                        .map(|(name, desc)| format!("{}={}", name, desc))
                        .collect();
                    format!("- {}({}): {}", t.name, params.join(", "), t.description)
                }
            })
            .collect();
        lines.sort();
        lines.join("\n")
    }

    /// List all tool names
    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a tool with given arguments
pub fn execute_tool(
    catalog: &ToolCatalog,
    tool_name: &str,
    arguments: &HashMap<String, String>,
) -> ToolResult {
    let tool = match catalog.get(tool_name) {
        Some(t) => t,
        None => {
            return ToolResult {
                tool: tool_name.to_string(),
                success: false,
                stdout: String::new(),
                stderr: format!("Unknown tool: {}", tool_name),
                exit_code: -1,
            };
        }
    };

    // Build command with argument substitution
    let args: Vec<String> = tool.args
        .iter()
        .map(|arg| {
            let mut result = (*arg).to_string();
            for (key, value) in arguments {
                result = result.replace(&format!("{{{}}}", key), value);
            }
            result
        })
        .collect();

    // Check if any placeholders remain (missing required args)
    for arg in &args {
        if arg.contains('{') && arg.contains('}') {
            return ToolResult {
                tool: tool_name.to_string(),
                success: false,
                stdout: String::new(),
                stderr: format!("Missing required argument in: {}", arg),
                exit_code: -1,
            };
        }
    }

    // Execute the command
    let output = Command::new(tool.command)
        .args(&args)
        .output();

    match output {
        Ok(out) => ToolResult {
            tool: tool_name.to_string(),
            success: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        },
        Err(e) => ToolResult {
            tool: tool_name.to_string(),
            success: false,
            stdout: String::new(),
            stderr: format!("Failed to execute: {}", e),
            exit_code: -1,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_tools() {
        let catalog = ToolCatalog::new();
        assert!(catalog.get("mem_info").is_some());
        assert!(catalog.get("cpu_info").is_some());
        assert!(catalog.get("pacman_query").is_some());
    }

    #[test]
    fn test_unknown_tool() {
        let catalog = ToolCatalog::new();
        let result = execute_tool(&catalog, "nonexistent", &HashMap::new());
        assert!(!result.success);
        assert!(result.stderr.contains("Unknown tool"));
    }

    #[test]
    fn test_prompt_string_format() {
        let catalog = ToolCatalog::new();
        let prompt = catalog.to_prompt_string();

        // Should have tool descriptions
        assert!(prompt.contains("mem_info"));
        assert!(prompt.contains("cpu_info"));
        // Should not expose actual commands
        assert!(!prompt.contains("lscpu"));
        assert!(!prompt.contains("free -h"));
    }

    #[test]
    fn test_argument_substitution() {
        let catalog = ToolCatalog::new();
        let mut args = HashMap::new();
        args.insert("pattern".to_string(), "steam".to_string());

        let result = execute_tool(&catalog, "pacman_query", &args);
        // Result depends on whether steam is installed, but should execute
        assert!(result.exit_code >= 0 || result.stderr.contains("Failed"));
    }
}

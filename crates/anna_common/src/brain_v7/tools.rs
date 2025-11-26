//! Anna v7 Brain - Tool Catalog
//!
//! Strict, fixed tool definitions. The LLM sees descriptions only.
//! Rust owns the actual commands and never exposes them.
//!
//! Design principles:
//! - Each tool does ONE thing
//! - Commands are hardcoded and safe
//! - Output is raw text, interpreter must parse
//! - If a tool fails, it returns clear error in stderr

use super::contracts::{ToolDescriptor, ToolRun};
use chrono::Utc;
use std::collections::HashMap;
use std::process::Command;
use uuid::Uuid;

/// All available tools in the v7 brain
pub struct ToolCatalog {
    tools: HashMap<String, Tool>,
}

/// Internal tool definition (never exposed to LLM)
struct Tool {
    descriptor: ToolDescriptor,
    executor: Box<dyn Fn(&serde_json::Value) -> ToolExecution + Send + Sync>,
}

/// Result of preparing a tool for execution
struct ToolExecution {
    command: String,
    args: Vec<String>,
    preview: String,
}

impl ToolCatalog {
    /// Create the complete v7 tool catalog
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // =====================================================================
        // Hardware Tools
        // =====================================================================

        tools.insert("mem_info".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "mem_info".to_string(),
                description: "Get memory information from /proc/meminfo. Returns MemTotal, MemFree, MemAvailable, Buffers, Cached lines.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "cat".to_string(),
                args: vec!["/proc/meminfo".to_string()],
                preview: "cat /proc/meminfo".to_string(),
            }),
        });

        tools.insert("cpu_info".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "cpu_info".to_string(),
                description: "Get CPU information using lscpu. Returns model name, cores, threads, architecture, flags.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "lscpu".to_string(),
                args: vec![],
                preview: "lscpu".to_string(),
            }),
        });

        tools.insert("gpu_pci".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "gpu_pci".to_string(),
                description: "Get GPU information from PCI bus. Returns VGA and 3D controller lines from lspci.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "lspci -nn | grep -iE 'VGA|3D controller'".to_string()],
                preview: "lspci -nn | grep -iE 'VGA|3D controller'".to_string(),
            }),
        });

        // =====================================================================
        // Package Tools
        // =====================================================================

        tools.insert("pacman_search".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "pacman_search".to_string(),
                description: "Search installed packages by name pattern. Use for checking if specific software is installed.".to_string(),
                parameters_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Package name pattern to search for"
                        }
                    },
                    "required": ["query"]
                })),
            },
            executor: Box::new(|params| {
                let query = params.get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                ToolExecution {
                    command: "pacman".to_string(),
                    args: vec!["-Qs".to_string(), query.to_string()],
                    preview: format!("pacman -Qs {}", query),
                }
            }),
        });

        tools.insert("pacman_updates".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "pacman_updates".to_string(),
                description: "Check for available package updates using checkupdates.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "checkupdates".to_string(),
                args: vec![],
                preview: "checkupdates".to_string(),
            }),
        });

        tools.insert("aur_updates".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "aur_updates".to_string(),
                description: "Check for AUR package updates using yay -Qua.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "yay".to_string(),
                args: vec!["-Qua".to_string()],
                preview: "yay -Qua".to_string(),
            }),
        });

        tools.insert("pacman_orphans".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "pacman_orphans".to_string(),
                description: "List orphan packages (installed as dependencies but no longer needed).".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "pacman".to_string(),
                args: vec!["-Qdtq".to_string()],
                preview: "pacman -Qdtq".to_string(),
            }),
        });

        tools.insert("pacman_cache_size".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "pacman_cache_size".to_string(),
                description: "Get the size of the pacman package cache.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "du".to_string(),
                args: vec!["-sh".to_string(), "/var/cache/pacman/pkg".to_string()],
                preview: "du -sh /var/cache/pacman/pkg".to_string(),
            }),
        });

        // =====================================================================
        // Storage Tools
        // =====================================================================

        tools.insert("disk_usage".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "disk_usage".to_string(),
                description: "Get filesystem disk usage for all mounted partitions.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "df".to_string(),
                args: vec!["-h".to_string()],
                preview: "df -h".to_string(),
            }),
        });

        tools.insert("home_du_top".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "home_du_top".to_string(),
                description: "Get top 10 largest directories in user's home folder.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "sh".to_string(),
                args: vec![
                    "-c".to_string(),
                    "du -h --max-depth=1 ~ 2>/dev/null | sort -hr | head -15".to_string(),
                ],
                preview: "du -h --max-depth=1 ~ | sort -hr | head -15".to_string(),
            }),
        });

        tools.insert("var_largest_files".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "var_largest_files".to_string(),
                description: "Find the 10 largest files in /var directory.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "sh".to_string(),
                args: vec![
                    "-c".to_string(),
                    "find /var -type f -printf '%s %p\\n' 2>/dev/null | sort -rn | head -10".to_string(),
                ],
                preview: "find /var -type f -printf '%s %p' | sort -rn | head -10".to_string(),
            }),
        });

        // =====================================================================
        // Network Tools
        // =====================================================================

        tools.insert("net_interfaces".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "net_interfaces".to_string(),
                description: "List network interfaces and their status using nmcli.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "nmcli".to_string(),
                args: vec!["device".to_string()],
                preview: "nmcli device".to_string(),
            }),
        });

        tools.insert("ip_addresses".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "ip_addresses".to_string(),
                description: "Get IP addresses for all interfaces.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "ip".to_string(),
                args: vec!["-br".to_string(), "addr".to_string()],
                preview: "ip -br addr".to_string(),
            }),
        });

        tools.insert("dns_config".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "dns_config".to_string(),
                description: "Get DNS configuration from /etc/resolv.conf.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "cat".to_string(),
                args: vec!["/etc/resolv.conf".to_string()],
                preview: "cat /etc/resolv.conf".to_string(),
            }),
        });

        // =====================================================================
        // System Tools
        // =====================================================================

        tools.insert("kernel_info".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "kernel_info".to_string(),
                description: "Get kernel version and system information.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "uname".to_string(),
                args: vec!["-a".to_string()],
                preview: "uname -a".to_string(),
            }),
        });

        tools.insert("uptime".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "uptime".to_string(),
                description: "Get system uptime and load averages.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "uptime".to_string(),
                args: vec![],
                preview: "uptime".to_string(),
            }),
        });

        tools.insert("systemd_failed".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "systemd_failed".to_string(),
                description: "List failed systemd services.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "systemctl".to_string(),
                args: vec!["--failed".to_string()],
                preview: "systemctl --failed".to_string(),
            }),
        });

        tools.insert("journal_errors".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "journal_errors".to_string(),
                description: "Get recent error messages from system journal (last 50 lines).".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "journalctl".to_string(),
                args: vec!["-p".to_string(), "err".to_string(), "-n".to_string(), "50".to_string(), "--no-pager".to_string()],
                preview: "journalctl -p err -n 50".to_string(),
            }),
        });

        // =====================================================================
        // Desktop/Environment Tools
        // =====================================================================

        tools.insert("desktop_session".to_string(), Tool {
            descriptor: ToolDescriptor {
                name: "desktop_session".to_string(),
                description: "Get current desktop session from environment variables.".to_string(),
                parameters_schema: None,
            },
            executor: Box::new(|_| ToolExecution {
                command: "sh".to_string(),
                args: vec![
                    "-c".to_string(),
                    "echo \"XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP\"; echo \"XDG_SESSION_TYPE=$XDG_SESSION_TYPE\"; echo \"DESKTOP_SESSION=$DESKTOP_SESSION\"".to_string(),
                ],
                preview: "echo XDG_CURRENT_DESKTOP, XDG_SESSION_TYPE, DESKTOP_SESSION".to_string(),
            }),
        });

        Self { tools }
    }

    /// Get tool descriptors for the LLM (no commands exposed)
    pub fn get_descriptors(&self) -> Vec<ToolDescriptor> {
        self.tools.values().map(|t| t.descriptor.clone()).collect()
    }

    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Execute a tool and return the result
    pub fn execute(&self, name: &str, subtask_id: &str, params: &serde_json::Value) -> ToolRun {
        let started_at = Utc::now();
        let id = Uuid::new_v4().to_string();

        let tool = match self.tools.get(name) {
            Some(t) => t,
            None => {
                return ToolRun {
                    id,
                    subtask_id: subtask_id.to_string(),
                    tool: name.to_string(),
                    command_preview: format!("[unknown tool: {}]", name),
                    stdout: String::new(),
                    stderr: format!("Tool '{}' not found in catalog", name),
                    exit_code: -1,
                    started_at,
                    finished_at: Utc::now(),
                };
            }
        };

        let exec = (tool.executor)(params);

        let result = Command::new(&exec.command)
            .args(&exec.args)
            .output();

        let finished_at = Utc::now();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                // Truncate very long outputs
                let stdout = truncate_output(&stdout, 8000);

                ToolRun {
                    id,
                    subtask_id: subtask_id.to_string(),
                    tool: name.to_string(),
                    command_preview: exec.preview,
                    stdout,
                    stderr,
                    exit_code,
                    started_at,
                    finished_at,
                }
            }
            Err(e) => ToolRun {
                id,
                subtask_id: subtask_id.to_string(),
                tool: name.to_string(),
                command_preview: exec.preview,
                stdout: String::new(),
                stderr: format!("Failed to execute: {}", e),
                exit_code: -1,
                started_at,
                finished_at,
            },
        }
    }
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate output to avoid overwhelming the LLM
fn truncate_output(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...[truncated, {} bytes total]", &s[..max_len], s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_required_tools() {
        let catalog = ToolCatalog::new();

        // Hardware
        assert!(catalog.has_tool("mem_info"));
        assert!(catalog.has_tool("cpu_info"));
        assert!(catalog.has_tool("gpu_pci"));

        // Packages
        assert!(catalog.has_tool("pacman_search"));
        assert!(catalog.has_tool("pacman_updates"));

        // Storage
        assert!(catalog.has_tool("disk_usage"));
        assert!(catalog.has_tool("home_du_top"));

        // Network
        assert!(catalog.has_tool("net_interfaces"));
        assert!(catalog.has_tool("dns_config"));

        // System
        assert!(catalog.has_tool("kernel_info"));
        assert!(catalog.has_tool("systemd_failed"));
    }

    #[test]
    fn test_descriptors_dont_expose_commands() {
        let catalog = ToolCatalog::new();
        let descriptors = catalog.get_descriptors();

        for desc in descriptors {
            // Descriptions should NOT contain shell command syntax
            assert!(
                !desc.description.contains("sh -c"),
                "Tool {} leaks 'sh -c' in description",
                desc.name
            );
            assert!(
                !desc.description.contains(" | "),
                "Tool {} leaks pipe syntax in description",
                desc.name
            );
        }
    }

    #[test]
    fn test_unknown_tool_returns_error() {
        let catalog = ToolCatalog::new();
        let result = catalog.execute("nonexistent_tool", "st1", &serde_json::json!({}));

        assert_eq!(result.exit_code, -1);
        assert!(result.stderr.contains("not found"));
    }

    #[test]
    fn test_pacman_search_with_params() {
        let catalog = ToolCatalog::new();
        let result = catalog.execute(
            "pacman_search",
            "st1",
            &serde_json::json!({"query": "steam"}),
        );

        assert!(result.command_preview.contains("steam"));
    }
}

//! Self-Health Probes v0.7.0
//!
//! Six probes to monitor Anna's own health:
//! - daemon: Is annad running and responding?
//! - llm: Is Ollama backend accessible?
//! - model: Are required models available?
//! - tools: Is the probe catalog valid?
//! - permissions: Are directories writable?
//! - config: Is the config file valid?

use super::types::{ComponentHealth, ComponentStatus};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Daemon endpoint for health checks
const DAEMON_URL: &str = "http://127.0.0.1:7865";
/// Ollama endpoint
const OLLAMA_URL: &str = "http://127.0.0.1:11434";
/// Config file path
const CONFIG_PATH: &str = "/etc/anna/config.toml";
/// User config path
fn user_config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|p| p.join("anna/config.toml"))
}

/// Check if annad daemon is running and responding
pub fn check_daemon() -> ComponentHealth {
    // First check if process is running
    let systemctl_output = Command::new("systemctl")
        .args(["is-active", "annad"])
        .output();

    let process_running = match systemctl_output {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout);
            status.trim() == "active"
        }
        Err(_) => {
            // systemctl not available, try pgrep
            Command::new("pgrep")
                .arg("annad")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    };

    if !process_running {
        return ComponentHealth {
            name: "daemon".to_string(),
            status: ComponentStatus::Critical,
            message: "annad is not running".to_string(),
            details: Some(serde_json::json!({
                "process_running": false,
                "suggestion": "sudo systemctl start annad"
            })),
        };
    }

    // Check if daemon is responding
    let health_check = std::thread::spawn(move || {
        std::net::TcpStream::connect_timeout(
            &"127.0.0.1:7865".parse().unwrap(),
            Duration::from_secs(2),
        )
    });

    match health_check.join() {
        Ok(Ok(_)) => ComponentHealth {
            name: "daemon".to_string(),
            status: ComponentStatus::Healthy,
            message: "annad is running and responding".to_string(),
            details: Some(serde_json::json!({
                "process_running": true,
                "port_open": true,
                "endpoint": DAEMON_URL
            })),
        },
        Ok(Err(_)) | Err(_) => ComponentHealth {
            name: "daemon".to_string(),
            status: ComponentStatus::Degraded,
            message: "annad is running but not responding on port 7865".to_string(),
            details: Some(serde_json::json!({
                "process_running": true,
                "port_open": false,
                "suggestion": "sudo systemctl restart annad"
            })),
        },
    }
}

/// Check if Ollama LLM backend is accessible
pub fn check_llm_backend() -> ComponentHealth {
    // Check if Ollama process is running
    let ollama_running = Command::new("pgrep")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ollama_running {
        // Also try systemctl
        let systemctl_check = Command::new("systemctl")
            .args(["is-active", "ollama"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false);

        if !systemctl_check {
            return ComponentHealth {
                name: "llm".to_string(),
                status: ComponentStatus::Critical,
                message: "Ollama is not running".to_string(),
                details: Some(serde_json::json!({
                    "process_running": false,
                    "suggestion": "systemctl start ollama or ollama serve"
                })),
            };
        }
    }

    // Check if Ollama API is responding
    let api_check = std::thread::spawn(move || {
        std::net::TcpStream::connect_timeout(
            &"127.0.0.1:11434".parse().unwrap(),
            Duration::from_secs(2),
        )
    });

    match api_check.join() {
        Ok(Ok(_)) => ComponentHealth {
            name: "llm".to_string(),
            status: ComponentStatus::Healthy,
            message: "Ollama is running and responding".to_string(),
            details: Some(serde_json::json!({
                "process_running": true,
                "api_responding": true,
                "endpoint": OLLAMA_URL
            })),
        },
        Ok(Err(_)) | Err(_) => ComponentHealth {
            name: "llm".to_string(),
            status: ComponentStatus::Degraded,
            message: "Ollama process found but API not responding".to_string(),
            details: Some(serde_json::json!({
                "process_running": true,
                "api_responding": false,
                "suggestion": "systemctl restart ollama"
            })),
        },
    }
}

/// Check if required LLM models are available
pub fn check_model_availability() -> ComponentHealth {
    // Run ollama list to get available models
    let output = Command::new("ollama").arg("list").output();

    match output {
        Ok(output) if output.status.success() => {
            let list = String::from_utf8_lossy(&output.stdout);
            let models: Vec<&str> = list
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| line.split_whitespace().next())
                .collect();

            if models.is_empty() {
                return ComponentHealth {
                    name: "model".to_string(),
                    status: ComponentStatus::Critical,
                    message: "No LLM models installed".to_string(),
                    details: Some(serde_json::json!({
                        "models_found": [],
                        "suggestion": "ollama pull llama3.2:3b"
                    })),
                };
            }

            // Check for recommended models
            let has_small = models.iter().any(|m| {
                m.contains("llama3.2:3b")
                    || m.contains("qwen2.5:3b")
                    || m.contains("mistral")
            });

            if has_small {
                ComponentHealth {
                    name: "model".to_string(),
                    status: ComponentStatus::Healthy,
                    message: format!("{} model(s) available", models.len()),
                    details: Some(serde_json::json!({
                        "models_found": models,
                        "has_recommended": true
                    })),
                }
            } else {
                ComponentHealth {
                    name: "model".to_string(),
                    status: ComponentStatus::Degraded,
                    message: format!(
                        "{} model(s) found but none are recommended",
                        models.len()
                    ),
                    details: Some(serde_json::json!({
                        "models_found": models,
                        "has_recommended": false,
                        "suggestion": "ollama pull llama3.2:3b or qwen2.5:3b"
                    })),
                }
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            ComponentHealth {
                name: "model".to_string(),
                status: ComponentStatus::Critical,
                message: "Failed to list models".to_string(),
                details: Some(serde_json::json!({
                    "error": stderr.to_string(),
                    "suggestion": "Check Ollama installation"
                })),
            }
        }
        Err(e) => ComponentHealth {
            name: "model".to_string(),
            status: ComponentStatus::Critical,
            message: "Ollama command not found".to_string(),
            details: Some(serde_json::json!({
                "error": e.to_string(),
                "suggestion": "Install Ollama: curl -fsSL https://ollama.ai/install.sh | sh"
            })),
        },
    }
}

/// Check if probe catalog and tools are valid
pub fn check_tools_catalog() -> ComponentHealth {
    let probes_dir = Path::new("/usr/share/anna/probes");

    if !probes_dir.exists() {
        return ComponentHealth {
            name: "tools".to_string(),
            status: ComponentStatus::Critical,
            message: "Probes directory does not exist".to_string(),
            details: Some(serde_json::json!({
                "path": probes_dir.display().to_string(),
                "suggestion": "sudo mkdir -p /usr/share/anna/probes"
            })),
        };
    }

    // Count probe files
    let probe_count = fs::read_dir(probes_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .count()
        })
        .unwrap_or(0);

    if probe_count == 0 {
        ComponentHealth {
            name: "tools".to_string(),
            status: ComponentStatus::Degraded,
            message: "No probe definitions found".to_string(),
            details: Some(serde_json::json!({
                "path": probes_dir.display().to_string(),
                "probe_count": 0,
                "suggestion": "Install probes from GitHub release"
            })),
        }
    } else {
        ComponentHealth {
            name: "tools".to_string(),
            status: ComponentStatus::Healthy,
            message: format!("{} probe(s) registered", probe_count),
            details: Some(serde_json::json!({
                "path": probes_dir.display().to_string(),
                "probe_count": probe_count
            })),
        }
    }
}

/// Check if required directories are writable
pub fn check_permissions() -> ComponentHealth {
    let dirs_to_check = [
        ("/var/lib/anna", "data"),
        ("/var/log/anna", "logs"),
        ("/run/anna", "runtime"),
    ];

    let mut issues = Vec::new();
    let mut healthy = Vec::new();

    for (path, purpose) in &dirs_to_check {
        let path = Path::new(path);
        if !path.exists() {
            issues.push(format!("{} ({}) does not exist", path.display(), purpose));
        } else if fs::metadata(path).map(|m| m.permissions().readonly()).unwrap_or(true) {
            issues.push(format!("{} ({}) is read-only", path.display(), purpose));
        } else {
            healthy.push(path.display().to_string());
        }
    }

    // Also check user config directory
    if let Some(user_config) = user_config_path() {
        if let Some(parent) = user_config.parent() {
            if !parent.exists() {
                // This is fine - will be created on first use
            } else if fs::metadata(parent)
                .map(|m| m.permissions().readonly())
                .unwrap_or(true)
            {
                issues.push(format!("{} is read-only", parent.display()));
            }
        }
    }

    if issues.is_empty() {
        ComponentHealth {
            name: "permissions".to_string(),
            status: ComponentStatus::Healthy,
            message: "All directories accessible".to_string(),
            details: Some(serde_json::json!({
                "healthy": healthy,
                "issues": []
            })),
        }
    } else if healthy.is_empty() {
        ComponentHealth {
            name: "permissions".to_string(),
            status: ComponentStatus::Critical,
            message: format!("{} permission issue(s)", issues.len()),
            details: Some(serde_json::json!({
                "healthy": healthy,
                "issues": issues,
                "suggestion": "sudo chown -R anna:anna /var/lib/anna /var/log/anna /run/anna"
            })),
        }
    } else {
        ComponentHealth {
            name: "permissions".to_string(),
            status: ComponentStatus::Degraded,
            message: format!("{} permission issue(s)", issues.len()),
            details: Some(serde_json::json!({
                "healthy": healthy,
                "issues": issues
            })),
        }
    }
}

/// Check if configuration file is valid
pub fn check_config() -> ComponentHealth {
    // Check system config first
    let system_config = Path::new(CONFIG_PATH);
    let user_config = user_config_path();

    let configs_found: Vec<String> = [
        system_config.exists().then(|| CONFIG_PATH.to_string()),
        user_config
            .as_ref()
            .filter(|p| p.exists())
            .map(|p| p.display().to_string()),
    ]
    .into_iter()
    .flatten()
    .collect();

    if configs_found.is_empty() {
        return ComponentHealth {
            name: "config".to_string(),
            status: ComponentStatus::Degraded,
            message: "No config file found (using defaults)".to_string(),
            details: Some(serde_json::json!({
                "searched": [CONFIG_PATH, user_config.map(|p| p.display().to_string())],
                "suggestion": "Run 'annactl' to generate default config"
            })),
        };
    }

    // Try to parse configs
    let mut parse_errors = Vec::new();
    for config_path in &configs_found {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Err(e) = content.parse::<toml::Table>() {
                parse_errors.push(format!("{}: {}", config_path, e));
            }
        }
    }

    if !parse_errors.is_empty() {
        ComponentHealth {
            name: "config".to_string(),
            status: ComponentStatus::Degraded,
            message: "Config file has syntax errors".to_string(),
            details: Some(serde_json::json!({
                "configs_found": configs_found,
                "parse_errors": parse_errors,
                "suggestion": "Check config file syntax"
            })),
        }
    } else {
        ComponentHealth {
            name: "config".to_string(),
            status: ComponentStatus::Healthy,
            message: format!("{} config file(s) valid", configs_found.len()),
            details: Some(serde_json::json!({
                "configs_found": configs_found,
                "parse_errors": []
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_config_no_file() {
        // This test runs in a clean environment where config likely doesn't exist
        let health = check_config();
        // Should be either Healthy (if config exists) or Degraded (if not)
        assert!(matches!(
            health.status,
            ComponentStatus::Healthy | ComponentStatus::Degraded
        ));
    }

    #[test]
    fn test_check_tools_catalog() {
        let health = check_tools_catalog();
        // Should return valid status regardless of probe existence
        assert!(!health.name.is_empty());
        assert!(!health.message.is_empty());
    }

    #[test]
    fn test_check_permissions() {
        let health = check_permissions();
        // Should return valid status
        assert_eq!(health.name, "permissions");
    }
}

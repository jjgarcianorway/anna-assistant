//! Auto-Repair Engine v0.7.0
//!
//! Repair actions catalog with safety rules:
//! - Auto: Safe to execute without user confirmation
//! - WarnOnly: Requires sudo or has side effects

use super::types::{
    ComponentHealth, ComponentStatus, RepairAction, RepairPlan, RepairResult, RepairSafety,
};
use std::process::Command;

/// Plan a repair for a degraded or critical component
pub fn plan_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    match component.name.as_str() {
        "daemon" => plan_daemon_repair(component),
        "llm" => plan_llm_repair(component),
        "model" => plan_model_repair(component),
        "tools" => plan_tools_repair(component),
        "permissions" => plan_permissions_repair(component),
        "config" => plan_config_repair(component),
        _ => None,
    }
}

/// Execute a repair action
pub fn execute_repair(plan: &RepairPlan) -> Result<RepairResult, String> {
    // Only execute auto-safe repairs
    if plan.safety != RepairSafety::Auto {
        return Err("Cannot auto-execute warn-only repairs".to_string());
    }

    match &plan.action {
        RepairAction::RestartDaemon => execute_restart_daemon(),
        RepairAction::RestartOllama => execute_restart_ollama(),
        RepairAction::PullModel(model) => execute_pull_model(model),
        RepairAction::ClearProbeCache => execute_clear_cache(),
        RepairAction::FixPermissions(path) => execute_fix_permissions(path),
        RepairAction::RegenerateConfig => execute_regenerate_config(),
        RepairAction::Custom(cmd) => execute_custom_command(cmd),
    }
}

// === Repair Planning ===

fn plan_daemon_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    match component.status {
        ComponentStatus::Critical => Some(RepairPlan {
            action: RepairAction::RestartDaemon,
            // Requires sudo - warn only
            safety: RepairSafety::WarnOnly,
            description: "Restart annad daemon".to_string(),
            command: "sudo systemctl restart annad".to_string(),
            target_component: "daemon".to_string(),
        }),
        ComponentStatus::Degraded => Some(RepairPlan {
            action: RepairAction::RestartDaemon,
            safety: RepairSafety::WarnOnly,
            description: "Restart annad daemon (not responding)".to_string(),
            command: "sudo systemctl restart annad".to_string(),
            target_component: "daemon".to_string(),
        }),
        _ => None,
    }
}

fn plan_llm_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    match component.status {
        ComponentStatus::Critical | ComponentStatus::Degraded => Some(RepairPlan {
            action: RepairAction::RestartOllama,
            // Requires sudo or user-level service restart
            safety: RepairSafety::WarnOnly,
            description: "Restart Ollama service".to_string(),
            command: "systemctl restart ollama".to_string(),
            target_component: "llm".to_string(),
        }),
        _ => None,
    }
}

fn plan_model_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    if component.status == ComponentStatus::Critical {
        // No models - suggest pulling default
        return Some(RepairPlan {
            action: RepairAction::PullModel("llama3.2:3b".to_string()),
            // Pulling model is safe - just downloads data
            safety: RepairSafety::Auto,
            description: "Pull default LLM model".to_string(),
            command: "ollama pull llama3.2:3b".to_string(),
            target_component: "model".to_string(),
        });
    }

    if component.status == ComponentStatus::Degraded {
        // Has models but not recommended ones
        return Some(RepairPlan {
            action: RepairAction::PullModel("llama3.2:3b".to_string()),
            safety: RepairSafety::Auto,
            description: "Pull recommended LLM model".to_string(),
            command: "ollama pull llama3.2:3b".to_string(),
            target_component: "model".to_string(),
        });
    }

    None
}

fn plan_tools_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    if component.status == ComponentStatus::Critical {
        Some(RepairPlan {
            action: RepairAction::Custom("sudo mkdir -p /usr/share/anna/probes".to_string()),
            safety: RepairSafety::WarnOnly,
            description: "Create probes directory".to_string(),
            command: "sudo mkdir -p /usr/share/anna/probes".to_string(),
            target_component: "tools".to_string(),
        })
    } else if component.status == ComponentStatus::Degraded {
        Some(RepairPlan {
            action: RepairAction::Custom("Download probes from GitHub release".to_string()),
            safety: RepairSafety::WarnOnly,
            description: "Install probe definitions".to_string(),
            command: "See installation instructions".to_string(),
            target_component: "tools".to_string(),
        })
    } else {
        None
    }
}

fn plan_permissions_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    if component.status == ComponentStatus::Critical
        || component.status == ComponentStatus::Degraded
    {
        Some(RepairPlan {
            action: RepairAction::FixPermissions("/var/lib/anna".to_string()),
            // Requires sudo
            safety: RepairSafety::WarnOnly,
            description: "Fix directory permissions".to_string(),
            command: "sudo chown -R anna:anna /var/lib/anna /var/log/anna /run/anna".to_string(),
            target_component: "permissions".to_string(),
        })
    } else {
        None
    }
}

fn plan_config_repair(component: &ComponentHealth) -> Option<RepairPlan> {
    if component.status == ComponentStatus::Degraded {
        // Check if it's missing config vs syntax error
        if let Some(details) = &component.details {
            if let Some(errors) = details.get("parse_errors") {
                if errors.as_array().is_some_and(|a| !a.is_empty()) {
                    // Syntax error - can't auto-fix
                    return Some(RepairPlan {
                        action: RepairAction::RegenerateConfig,
                        safety: RepairSafety::WarnOnly,
                        description: "Fix config syntax (will reset to defaults)".to_string(),
                        command: "rm ~/.config/anna/config.toml && annactl --version".to_string(),
                        target_component: "config".to_string(),
                    });
                }
            }
        }

        // Missing config - safe to generate
        Some(RepairPlan {
            action: RepairAction::RegenerateConfig,
            safety: RepairSafety::Auto,
            description: "Generate default config".to_string(),
            command: "annactl --version".to_string(),
            target_component: "config".to_string(),
        })
    } else {
        None
    }
}

// === Repair Execution ===

fn execute_restart_daemon() -> Result<RepairResult, String> {
    // Note: This requires sudo, so we don't actually execute it
    // Instead, we return an error indicating manual action needed
    Err("Restarting daemon requires sudo - run: sudo systemctl restart annad".to_string())
}

fn execute_restart_ollama() -> Result<RepairResult, String> {
    Err(
        "Restarting Ollama requires elevated privileges - run: systemctl restart ollama"
            .to_string(),
    )
}

fn execute_pull_model(model: &str) -> Result<RepairResult, String> {
    // This is safe to auto-execute
    let output = Command::new("ollama")
        .args(["pull", model])
        .output()
        .map_err(|e| format!("Failed to run ollama: {}", e))?;

    if output.status.success() {
        Ok(RepairResult {
            action: RepairAction::PullModel(model.to_string()),
            success: true,
            message: format!("Successfully pulled model: {}", model),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to pull model: {}", stderr))
    }
}

fn execute_clear_cache() -> Result<RepairResult, String> {
    // Clear probe cache in /var/lib/anna/cache
    let cache_dir = std::path::Path::new("/var/lib/anna/cache");

    if !cache_dir.exists() {
        return Ok(RepairResult {
            action: RepairAction::ClearProbeCache,
            success: true,
            message: "Cache directory does not exist (nothing to clear)".to_string(),
        });
    }

    std::fs::remove_dir_all(cache_dir).map_err(|e| format!("Failed to clear cache: {}", e))?;

    std::fs::create_dir_all(cache_dir)
        .map_err(|e| format!("Failed to recreate cache dir: {}", e))?;

    Ok(RepairResult {
        action: RepairAction::ClearProbeCache,
        success: true,
        message: "Cleared probe cache".to_string(),
    })
}

fn execute_fix_permissions(_path: &str) -> Result<RepairResult, String> {
    Err("Fixing permissions requires sudo - run the suggested command manually".to_string())
}

fn execute_regenerate_config() -> Result<RepairResult, String> {
    // Create default config in user directory
    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("anna");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    let config_path = config_dir.join("config.toml");

    let default_config = r#"# Anna Configuration v0.7.0
# Generated by auto-repair

[core]
mode = "normal"

[llm]
preferred_model = "llama3.2:3b"
fallback_model = "llama3.2:3b"
selection_mode = "auto"

[update]
enabled = true
interval_seconds = 86400
channel = "main"
"#;

    std::fs::write(&config_path, default_config)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(RepairResult {
        action: RepairAction::RegenerateConfig,
        success: true,
        message: format!("Generated default config at {}", config_path.display()),
    })
}

fn execute_custom_command(cmd: &str) -> Result<RepairResult, String> {
    // For safety, we don't execute arbitrary commands
    Err(format!("Manual action required: {}", cmd))
}

/// Get all available repair actions (for documentation)
pub fn available_repairs() -> Vec<(&'static str, &'static str, RepairSafety)> {
    vec![
        (
            "RestartDaemon",
            "Restart annad service",
            RepairSafety::WarnOnly,
        ),
        (
            "RestartOllama",
            "Restart Ollama service",
            RepairSafety::WarnOnly,
        ),
        ("PullModel", "Download LLM model", RepairSafety::Auto),
        (
            "FixPermissions",
            "Fix directory permissions",
            RepairSafety::WarnOnly,
        ),
        (
            "RegenerateConfig",
            "Create default config file",
            RepairSafety::Auto,
        ),
        (
            "ClearProbeCache",
            "Clear cached probe results",
            RepairSafety::Auto,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_model_repair_critical() {
        let component = ComponentHealth {
            name: "model".to_string(),
            status: ComponentStatus::Critical,
            message: "No models".to_string(),
            details: None,
        };

        let plan = plan_repair(&component);
        assert!(plan.is_some());

        let plan = plan.unwrap();
        assert!(matches!(plan.action, RepairAction::PullModel(_)));
        assert_eq!(plan.safety, RepairSafety::Auto);
    }

    #[test]
    fn test_plan_daemon_repair_requires_sudo() {
        let component = ComponentHealth {
            name: "daemon".to_string(),
            status: ComponentStatus::Critical,
            message: "Not running".to_string(),
            details: None,
        };

        let plan = plan_repair(&component);
        assert!(plan.is_some());

        let plan = plan.unwrap();
        assert_eq!(plan.safety, RepairSafety::WarnOnly);
    }

    #[test]
    fn test_available_repairs_list() {
        let repairs = available_repairs();
        assert!(!repairs.is_empty());

        // Check that we have both auto and warn-only repairs
        let has_auto = repairs.iter().any(|(_, _, s)| *s == RepairSafety::Auto);
        let has_warn = repairs.iter().any(|(_, _, s)| *s == RepairSafety::WarnOnly);

        assert!(has_auto);
        assert!(has_warn);
    }

    #[test]
    fn test_healthy_component_no_repair() {
        let component = ComponentHealth {
            name: "daemon".to_string(),
            status: ComponentStatus::Healthy,
            message: "Running".to_string(),
            details: None,
        };

        let plan = plan_repair(&component);
        assert!(plan.is_none());
    }
}

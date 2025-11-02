// Anna v0.10.1 - Capability Management with Graceful Degradation
//
// Loads CAPABILITIES.toml and modules.yaml to determine which modules should run
// and whether their dependencies are satisfied. Reports status as Active/Degraded/Disabled.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

const CAPABILITIES_PATH: &str = "/usr/lib/anna/CAPABILITIES.toml";
const MODULES_CONFIG_PATH: &str = "/etc/anna/modules.yaml";

/// Module status after capability check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleStatus {
    Active,
    Degraded,
    Disabled,
}

/// Complete capability check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCheck {
    pub module_name: String,
    pub description: String,
    pub status: ModuleStatus,
    pub reason: Option<String>,
    pub action: Option<String>,
    pub impact: Option<String>,
    pub required: bool,
    pub enabled_by_user: bool,
    pub evidence: Vec<String>,
}

/// Capability registry loaded from CAPABILITIES.toml
#[derive(Debug, Deserialize)]
struct CapabilitiesRegistry {
    meta: CapabilityMeta,
    modules: HashMap<String, ModuleDef>,
}

#[derive(Debug, Deserialize)]
struct CapabilityMeta {
    version: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ModuleDef {
    name: String,
    description: String,
    category: String,
    required: bool,
    deps: ModuleDeps,
    checks: ModuleChecks,
    degraded: Option<DegradedInfo>,
}

#[derive(Debug, Deserialize)]
struct ModuleDeps {
    required: Option<DepsSpec>,
    optional: Option<DepsSpec>,
}

#[derive(Debug, Deserialize)]
struct DepsSpec {
    #[serde(default)]
    packages: Vec<String>,
    #[serde(default)]
    commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModuleChecks {
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DegradedInfo {
    reason: String,
    action: String,
    impact: String,
}

/// User configuration from modules.yaml
#[derive(Debug, Deserialize, Serialize)]
struct ModulesConfig {
    version: String,
    modules: HashMap<String, ModuleUserConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleUserConfig {
    state: String,
    reason: String,
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            version: "0.10.1".to_string(),
            modules: HashMap::new(),
        }
    }
}

pub struct CapabilityManager {
    registry: CapabilitiesRegistry,
    user_config: ModulesConfig,
}

impl CapabilityManager {
    /// Load capability registry and user config
    pub fn new() -> Result<Self> {
        let registry = Self::load_registry(CAPABILITIES_PATH)?;
        let user_config = Self::load_user_config(MODULES_CONFIG_PATH).unwrap_or_else(|e| {
            warn!("Failed to load modules.yaml: {}, using defaults", e);
            ModulesConfig::default()
        });

        Ok(Self {
            registry,
            user_config,
        })
    }

    fn load_registry(path: &str) -> Result<CapabilitiesRegistry> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path))
    }

    fn load_user_config(path: &str) -> Result<ModulesConfig> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path))?;
        serde_yaml::from_str(&content).with_context(|| format!("Failed to parse {}", path))
    }

    /// Check all module capabilities and return results
    pub fn check_all(&self) -> Vec<CapabilityCheck> {
        let mut results = Vec::new();

        for (module_name, module_def) in &self.registry.modules {
            let check = self.check_module(module_name, module_def);
            results.push(check);
        }

        // Sort: required first, then alphabetical
        results.sort_by(|a, b| match (a.required, b.required) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.module_name.cmp(&b.module_name),
        });

        results
    }

    /// Check a single module's capability
    fn check_module(&self, module_name: &str, module_def: &ModuleDef) -> CapabilityCheck {
        // Check if user disabled this module
        let enabled_by_user = self.is_module_enabled(module_name);

        if !enabled_by_user {
            return CapabilityCheck {
                module_name: module_name.to_string(),
                description: module_def.description.clone(),
                status: ModuleStatus::Disabled,
                reason: Some("Disabled by user in modules.yaml".to_string()),
                action: Some(format!("annactl module enable {}", module_name)),
                impact: Some("Module skipped entirely".to_string()),
                required: module_def.required,
                enabled_by_user: false,
                evidence: vec![],
            };
        }

        // Check required dependencies
        let required_ok = self.check_deps(module_def.deps.required.as_ref());

        // Check optional dependencies
        let optional_ok = self.check_deps(module_def.deps.optional.as_ref());

        // Check actual system files/commands
        let checks_ok = self.verify_checks(&module_def.checks);

        // Determine status
        let status = if required_ok && optional_ok && checks_ok {
            ModuleStatus::Active
        } else if required_ok {
            // Required deps satisfied but optional missing
            ModuleStatus::Degraded
        } else {
            // Required deps missing - critical for required modules
            if module_def.required {
                ModuleStatus::Degraded // Required modules can't be fully disabled
            } else {
                ModuleStatus::Degraded
            }
        };

        let (reason, action, impact) = if status == ModuleStatus::Degraded {
            if let Some(degraded_info) = &module_def.degraded {
                (
                    Some(degraded_info.reason.clone()),
                    Some(degraded_info.action.clone()),
                    Some(degraded_info.impact.clone()),
                )
            } else {
                (
                    Some("Dependencies not fully satisfied".to_string()),
                    Some("Check CAPABILITIES.toml for details".to_string()),
                    Some("Limited functionality".to_string()),
                )
            }
        } else {
            (None, None, None)
        };

        CapabilityCheck {
            module_name: module_name.to_string(),
            description: module_def.description.clone(),
            status,
            reason,
            action,
            impact,
            required: module_def.required,
            enabled_by_user: true,
            evidence: module_def.checks.evidence.clone(),
        }
    }

    fn is_module_enabled(&self, module_name: &str) -> bool {
        if let Some(user_cfg) = self.user_config.modules.get(module_name) {
            user_cfg.state == "enabled"
        } else {
            // Default to enabled if not specified
            true
        }
    }

    fn check_deps(&self, deps: Option<&DepsSpec>) -> bool {
        let Some(deps) = deps else {
            return true;
        };

        // Check commands exist in PATH
        for cmd in &deps.commands {
            if !self.command_exists(cmd) {
                debug!("Command '{}' not found in PATH", cmd);
                return false;
            }
        }

        // Note: We don't check package installation directly (too distro-specific)
        // Instead we rely on command checks and file checks

        true
    }

    fn verify_checks(&self, checks: &ModuleChecks) -> bool {
        // Check if commands can be executed
        for cmd in &checks.commands {
            if !self.can_execute_command(cmd) {
                debug!("Cannot execute check command: {}", cmd);
                return false;
            }
        }

        // Check if required files exist
        for file_path in &checks.files {
            if !Path::new(file_path).exists() {
                debug!("Required path does not exist: {}", file_path);
                return false;
            }
        }

        true
    }

    fn command_exists(&self, cmd: &str) -> bool {
        which::which(cmd).is_ok()
    }

    fn can_execute_command(&self, cmd: &str) -> bool {
        // Parse command (handle pipes, redirects, etc.)
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let result = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        result.is_ok() // Don't care about exit code, just that it can run
    }

    /// Get list of active modules (enabled + dependencies satisfied)
    pub fn active_modules(&self) -> Vec<String> {
        self.check_all()
            .into_iter()
            .filter(|c| c.status == ModuleStatus::Active)
            .map(|c| c.module_name)
            .collect()
    }

    /// Get list of degraded modules
    pub fn degraded_modules(&self) -> Vec<CapabilityCheck> {
        self.check_all()
            .into_iter()
            .filter(|c| c.status == ModuleStatus::Degraded)
            .collect()
    }
}

/// Enable a module in modules.yaml
pub fn enable_module(module_name: &str) -> Result<()> {
    let mut config = load_or_create_modules_config()?;

    if let Some(module_cfg) = config.modules.get_mut(module_name) {
        module_cfg.state = "enabled".to_string();
    } else {
        // Add new entry
        config.modules.insert(
            module_name.to_string(),
            ModuleUserConfig {
                state: "enabled".to_string(),
                reason: "Enabled by user".to_string(),
            },
        );
    }

    save_modules_config(&config)?;
    Ok(())
}

/// Disable a module in modules.yaml
pub fn disable_module(module_name: &str, reason: Option<String>) -> Result<()> {
    let mut config = load_or_create_modules_config()?;

    if let Some(module_cfg) = config.modules.get_mut(module_name) {
        module_cfg.state = "disabled".to_string();
        if let Some(r) = reason {
            module_cfg.reason = r;
        }
    } else {
        // Add new entry
        config.modules.insert(
            module_name.to_string(),
            ModuleUserConfig {
                state: "disabled".to_string(),
                reason: reason.unwrap_or_else(|| "Disabled by user".to_string()),
            },
        );
    }

    save_modules_config(&config)?;
    Ok(())
}

fn load_or_create_modules_config() -> Result<ModulesConfig> {
    match std::fs::read_to_string(MODULES_CONFIG_PATH) {
        Ok(content) => serde_yaml::from_str(&content).context("Failed to parse modules.yaml"),
        Err(_) => Ok(ModulesConfig::default()),
    }
}

fn save_modules_config(config: &ModulesConfig) -> Result<()> {
    let content = serde_yaml::to_string(config).context("Failed to serialize modules.yaml")?;
    std::fs::write(MODULES_CONFIG_PATH, content).context("Failed to write modules.yaml")?;
    Ok(())
}

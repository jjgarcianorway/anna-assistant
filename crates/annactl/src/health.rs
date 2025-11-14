//! Health Model and Self-Check System
//!
//! Centralized health checking and auto-repair for Anna's components.
//! Used by:
//! - REPL startup (auto-repair before entering interactive mode)
//! - annactl status (diagnostic report)
//! - annad periodic checks (self-healing)

use anyhow::{Context, Result};
use anna_common::terminal_format as fmt;
use chrono::{DateTime, Utc};
use std::process::Command;

/// Overall health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Everything working correctly
    Healthy,
    /// Some issues but Anna can function
    Degraded,
    /// Critical issues preventing Anna from working
    Broken,
}

/// Health report for Anna's components
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// Overall status
    pub status: HealthStatus,
    /// Daemon component health
    pub daemon: DaemonHealth,
    /// LLM backend health
    pub llm: LlmHealth,
    /// Permissions and groups
    pub permissions: PermissionsHealth,
    /// Last auto-repair (if any)
    pub last_repair: Option<RepairRecord>,
    /// Timestamp of this health check
    pub checked_at: DateTime<Utc>,
}

/// Daemon health details
#[derive(Debug, Clone)]
pub struct DaemonHealth {
    /// Service file exists
    pub installed: bool,
    /// Service enabled to start on boot
    pub enabled: bool,
    /// Service currently running
    pub running: bool,
    /// Recent errors from journald
    pub recent_errors: Vec<String>,
}

/// LLM backend health details
#[derive(Debug, Clone)]
pub struct LlmHealth {
    /// Backend type (Ollama, Remote API, etc.)
    pub backend: String,
    /// Backend process/service is running
    pub backend_running: bool,
    /// Can reach backend endpoint
    pub reachable: bool,
    /// Configured model name
    pub model: Option<String>,
    /// Model is available/downloaded
    pub model_available: bool,
}

/// Permissions and groups health
#[derive(Debug, Clone)]
pub struct PermissionsHealth {
    /// Required groups exist on system
    pub required_groups_exist: bool,
    /// Current user has required group memberships
    pub user_in_groups: bool,
    /// Anna data directories have correct permissions
    pub data_dirs_ok: bool,
    /// Missing groups or permissions details
    pub issues: Vec<String>,
}

/// Record of a repair operation
#[derive(Debug, Clone)]
pub struct RepairRecord {
    /// When the repair happened
    pub timestamp: DateTime<Utc>,
    /// What was repaired
    pub actions: Vec<String>,
    /// Whether repair was successful
    pub success: bool,
}

impl HealthReport {
    /// Check Anna's health (with optional auto-repair)
    pub async fn check(auto_repair: bool) -> Result<Self> {
        let daemon = check_daemon_health().await?;
        let llm = check_llm_health().await?;
        let permissions = check_permissions_health()?;

        // Determine overall status
        let status = determine_status(&daemon, &llm, &permissions);

        let mut report = HealthReport {
            status: status.clone(),
            daemon,
            llm,
            permissions,
            last_repair: None,
            checked_at: Utc::now(),
        };

        // Auto-repair if requested
        if auto_repair && status != HealthStatus::Healthy {
            if let Ok(repair_record) = perform_auto_repair(&report).await {
                report.last_repair = Some(repair_record);
                // Re-check health after repair (without auto-repair to avoid recursion)
                let updated_daemon = check_daemon_health().await?;
                let updated_llm = check_llm_health().await?;
                let updated_permissions = check_permissions_health()?;

                report.status = determine_status(&updated_daemon, &updated_llm, &updated_permissions);
                report.daemon = updated_daemon;
                report.llm = updated_llm;
                report.permissions = updated_permissions;
            }
        }

        Ok(report)
    }

    /// Display human-readable health summary
    pub fn display_summary(&self) {
        match self.status {
            HealthStatus::Healthy => {
                println!("{}", fmt::success("✓ Anna is healthy"));
            }
            HealthStatus::Degraded => {
                println!("{}", fmt::warning("⚠ Anna is degraded but functional"));
            }
            HealthStatus::Broken => {
                println!("{}", fmt::error("✗ Anna has critical issues"));
            }
        }
    }

    /// Get exit code for status command
    pub fn exit_code(&self) -> i32 {
        match self.status {
            HealthStatus::Healthy => 0,
            HealthStatus::Degraded | HealthStatus::Broken => 1,
        }
    }
}

/// Determine overall health status based on components
fn determine_status(daemon: &DaemonHealth, llm: &LlmHealth, permissions: &PermissionsHealth) -> HealthStatus {
    if !daemon.running || !llm.reachable {
        HealthStatus::Broken
    } else if !daemon.enabled || !llm.model_available || !permissions.user_in_groups {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    }
}

/// Check daemon health
async fn check_daemon_health() -> Result<DaemonHealth> {
    let status = crate::systemd::get_service_status()?;

    // Get recent errors from journald
    let recent_errors = get_recent_journal_errors().unwrap_or_default();

    Ok(DaemonHealth {
        installed: status.exists,
        enabled: status.enabled,
        running: status.active,
        recent_errors,
    })
}

/// Check LLM backend health
async fn check_llm_health() -> Result<LlmHealth> {
    use anna_common::context::db::{ContextDb, DbLocation};
    use anna_common::llm::LlmMode;

    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;
    let config = db.load_llm_config().await?;

    let (backend, backend_running, reachable, model, model_available) = match config.mode {
        LlmMode::Local => {
            // Check Ollama service
            let ollama_running = check_ollama_running();
            let reachable = if ollama_running {
                check_ollama_reachable().await
            } else {
                false
            };
            let model = config.model.clone();
            let model_available = if reachable && model.is_some() {
                check_ollama_model_available(model.as_ref().unwrap()).await
            } else {
                false
            };

            ("Ollama".to_string(), ollama_running, reachable, model, model_available)
        }
        LlmMode::Remote => {
            // Check remote API
            let reachable = config.base_url.is_some();
            ("Remote API".to_string(), true, reachable, config.model.clone(), reachable)
        }
        LlmMode::NotConfigured | LlmMode::Disabled => {
            ("None".to_string(), false, false, None, false)
        }
    };

    Ok(LlmHealth {
        backend,
        backend_running,
        reachable,
        model,
        model_available,
    })
}

/// Check permissions and groups
fn check_permissions_health() -> Result<PermissionsHealth> {
    let mut issues = Vec::new();

    // Check if anna group exists
    let anna_group_exists = check_group_exists("anna");
    if !anna_group_exists {
        issues.push("Group 'anna' does not exist".to_string());
    }

    // Check if current user is in anna group
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let user_in_anna_group = check_user_in_group(&user, "anna");
    if !user_in_anna_group {
        issues.push(format!("User '{}' is not in 'anna' group", user));
    }

    // Check data directory permissions
    let data_dirs_ok = check_data_dir_permissions();
    if !data_dirs_ok {
        issues.push("Anna data directories have incorrect permissions".to_string());
    }

    Ok(PermissionsHealth {
        required_groups_exist: anna_group_exists,
        user_in_groups: user_in_anna_group,
        data_dirs_ok,
        issues,
    })
}

/// Perform auto-repair based on health report
async fn perform_auto_repair(report: &HealthReport) -> Result<RepairRecord> {
    let mut actions = Vec::new();
    let mut success = true;

    // Repair daemon if needed
    if !report.daemon.installed || !report.daemon.enabled || !report.daemon.running {
        match crate::systemd::repair_service() {
            Ok(msg) => {
                actions.push(format!("Daemon: {}", msg));
            }
            Err(e) => {
                actions.push(format!("Daemon repair failed: {}", e));
                success = false;
            }
        }
    }

    // Repair LLM if needed
    if !report.llm.backend_running || !report.llm.reachable {
        match repair_llm_backend(&report.llm).await {
            Ok(msg) => {
                actions.push(format!("LLM: {}", msg));
            }
            Err(e) => {
                actions.push(format!("LLM repair failed: {}", e));
                success = false;
            }
        }
    }

    // Repair permissions if needed
    if !report.permissions.user_in_groups {
        actions.push("Permissions: User needs to be added to 'anna' group (run: sudo usermod -aG anna $USER)".to_string());
        success = false; // Can't auto-fix this
    }

    Ok(RepairRecord {
        timestamp: Utc::now(),
        actions,
        success,
    })
}

/// Check if Ollama is running
fn check_ollama_running() -> bool {
    Command::new("systemctl")
        .args(&["is-active", "--quiet", "ollama"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if Ollama endpoint is reachable
async fn check_ollama_reachable() -> bool {
    // Try to ping Ollama API
    let output = Command::new("curl")
        .args(&["-s", "-f", "http://localhost:11434/api/version"])
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Check if a specific model is available in Ollama
async fn check_ollama_model_available(model: &str) -> bool {
    let output = Command::new("ollama")
        .args(&["list"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains(model)
    } else {
        false
    }
}

/// Repair LLM backend
async fn repair_llm_backend(llm: &LlmHealth) -> Result<String> {
    let mut actions: Vec<String> = Vec::new();

    // Check network connectivity first
    let network_ok = Command::new("ping")
        .args(&["-c", "1", "-W", "2", "1.1.1.1"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !network_ok {
        return Err(anyhow::anyhow!("Network unavailable - cannot download LLM components"));
    }

    if llm.backend == "Ollama" || llm.backend == "None" {
        // Check if Ollama is installed
        let ollama_installed = Command::new("which")
            .arg("ollama")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !ollama_installed {
            actions.push("Installing Ollama...".to_string());
            // Install Ollama using official script
            let install_result = Command::new("sh")
                .arg("-c")
                .arg("curl -fsSL https://ollama.com/install.sh | sh")
                .status();

            if !install_result.map(|s| s.success()).unwrap_or(false) {
                return Err(anyhow::anyhow!("Failed to install Ollama"));
            }
            actions.push("✓ Ollama installed".to_string());
        }

        // Ensure Ollama service is enabled and running
        if !llm.backend_running {
            actions.push("Starting Ollama service...".to_string());
            Command::new("sudo")
                .args(&["systemctl", "enable", "--now", "ollama"])
                .status()
                .context("Failed to enable Ollama service")?;

            // Wait for service to be ready
            std::thread::sleep(std::time::Duration::from_secs(2));
            actions.push("✓ Ollama service started".to_string());
        }

        // Check if a model is available
        if !llm.model_available {
            actions.push("Downloading LLM model...".to_string());

            // Detect hardware to select appropriate model
            let ram_gb = get_system_ram_gb();
            let model = if ram_gb >= 8 { "llama3.2:3b" } else { "llama3.2:1b" };

            actions.push(format!("Downloading {}...", model));

            let pull_result = Command::new("ollama")
                .args(&["pull", model])
                .status();

            if !pull_result.map(|s| s.success()).unwrap_or(false) {
                return Err(anyhow::anyhow!("Failed to download model {}", model));
            }

            actions.push(format!("✓ Model {} downloaded", model));

            // Save LLM config to database
            use anna_common::context::db::{ContextDb, DbLocation};
            use anna_common::llm::LlmConfig;

            let db = ContextDb::open(DbLocation::auto_detect()).await?;
            let config = LlmConfig::local("http://127.0.0.1:11434/v1", model);
            db.save_llm_config(&config).await?;

            actions.push("✓ LLM configured in Anna".to_string());
        }

        // Restart daemon to pick up new config
        if !actions.is_empty() {
            actions.push("Restarting Anna daemon...".to_string());
            Command::new("sudo")
                .args(&["systemctl", "restart", "annad"])
                .status()
                .context("Failed to restart daemon")?;
            actions.push("✓ Daemon restarted".to_string());
        }

        Ok(actions.join("\n"))
    } else {
        Ok("No auto-repair available for this LLM backend".to_string())
    }
}

/// Get system RAM in GB
fn get_system_ram_gb() -> usize {
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(kb_str) = line.split_whitespace().nth(1) {
                if let Ok(kb) = kb_str.parse::<usize>() {
                    return kb / 1024 / 1024;
                }
            }
        }
    }
    4 // Default fallback
}

/// Check if a group exists
fn check_group_exists(group: &str) -> bool {
    Command::new("getent")
        .args(&["group", group])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if user is in a group
fn check_user_in_group(user: &str, group: &str) -> bool {
    let output = Command::new("id")
        .args(&["-nG", user])
        .output();

    if let Ok(output) = output {
        let groups = String::from_utf8_lossy(&output.stdout);
        groups.split_whitespace().any(|g| g == group)
    } else {
        false
    }
}

/// Check data directory permissions
fn check_data_dir_permissions() -> bool {
    use std::path::Path;

    let paths = ["/var/lib/anna", "/var/log/anna"];

    for path in &paths {
        if !Path::new(path).exists() {
            return false;
        }
    }

    true
}

/// Get recent journal errors
fn get_recent_journal_errors() -> Result<Vec<String>> {
    let output = Command::new("journalctl")
        .args(&[
            "-u", "annad",
            "-p", "warning..alert",
            "-n", "5",
            "--no-pager",
            "--output=cat",
        ])
        .output()
        .context("Failed to run journalctl")?;

    let stderr_output = String::from_utf8_lossy(&output.stdout);
    Ok(stderr_output
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

//! Handler implementations for each ExecutorRequest variant.
//!
//! Each handler:
//! - Validates its inputs against a static allowlist (no dynamic shell construction)
//! - Uses fixed Command::new() with explicit args (no sh -c, ever)
//! - Returns ExecutorResponse

use std::io::Write;
use std::process::Command;
use tracing::{info, warn};

use crate::policy::ExecutorPolicy;
use crate::protocol::{ExecutorRequest, ExecutorResponse};

const AUDIT_LOG: &str = "/var/lib/anna/executor_audit.jsonl";

fn audit_log(action: &str, outcome: &str) {
    let entry = format!(
        "{{\"ts\":\"{}\",\"action\":\"{}\",\"outcome\":\"{}\"}}\n",
        chrono::Utc::now().to_rfc3339(),
        action,
        outcome
    );
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(AUDIT_LOG)
    {
        f.write_all(entry.as_bytes()).ok();
    }
}

/// Services that anna-executor is permitted to restart.
/// This mirrors SAFE_TO_RESTART in annad/self_healing.rs — kept in sync manually.
const RESTARTABLE_SERVICES: &[&str] = &[
    "pipewire",
    "pipewire-pulse",
    "wireplumber",
    "xdg-desktop-portal",
    "xdg-desktop-portal-gtk",
    "xdg-desktop-portal-gnome",
    "xdg-desktop-portal-kde",
    "gvfs-daemon",
    "evolution-addressbook-factory",
    "evolution-calendar-factory",
    "evolution-source-registry",
    "tracker-miner-fs",
    "gnome-keyring-daemon",
];

/// Dispatch a request and return the response.
/// Policy is loaded on each call (supports hot-reload without restart).
pub fn handle(request: ExecutorRequest) -> ExecutorResponse {
    let policy = ExecutorPolicy::load();

    let (action, response) = match request {
        ExecutorRequest::RestartService { ref name } => {
            if !policy.allow_restart_service {
                audit_log(&format!("RestartService:{}", name), "denied");
                return ExecutorResponse::Denied {
                    reason: "RestartService denied by policy".to_string(),
                };
            }
            let r = restart_service(name);
            (format!("RestartService:{}", name), r)
        }
        ExecutorRequest::CleanJournal { keep_days } => {
            if !policy.allow_clean_journal {
                audit_log(&format!("CleanJournal:{}", keep_days), "denied");
                return ExecutorResponse::Denied {
                    reason: "CleanJournal denied by policy".to_string(),
                };
            }
            let effective_days = keep_days.max(policy.min_journal_keep_days);
            let r = clean_journal(effective_days);
            (format!("CleanJournal:{}", effective_days), r)
        }
        ExecutorRequest::CleanPackageCache { keep_versions } => {
            if !policy.allow_clean_package_cache {
                audit_log(&format!("CleanPackageCache:{}", keep_versions), "denied");
                return ExecutorResponse::Denied {
                    reason: "CleanPackageCache denied by policy".to_string(),
                };
            }
            let effective_k = keep_versions.max(policy.min_package_keep_versions);
            let r = clean_package_cache(effective_k);
            (format!("CleanPackageCache:{}", effective_k), r)
        }
        ExecutorRequest::CleanTmpFiles => {
            if !policy.allow_clean_tmp_files {
                audit_log("CleanTmpFiles", "denied");
                return ExecutorResponse::Denied {
                    reason: "CleanTmpFiles denied by policy".to_string(),
                };
            }
            let r = clean_tmp_files();
            ("CleanTmpFiles".to_string(), r)
        }
    };
    let outcome = match &response {
        ExecutorResponse::Ok { .. } => "ok",
        ExecutorResponse::Error { .. } => "error",
        ExecutorResponse::Denied { .. } => "denied",
    };
    audit_log(&action, outcome);
    response
}

fn restart_service(name: &str) -> ExecutorResponse {
    // Validate against static allowlist — no arbitrary service names
    if !RESTARTABLE_SERVICES.contains(&name) {
        warn!("Denied restart request for non-allowlisted service: {}", name);
        return ExecutorResponse::Denied {
            reason: format!("Service '{}' is not in the restart allowlist", name),
        };
    }

    info!("Restarting service: {}", name);
    let result = Command::new("systemctl")
        .args(["restart", name])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            info!("Successfully restarted: {}", name);
            ExecutorResponse::Ok {
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            warn!("Failed to restart {}: {}", name, stderr);
            ExecutorResponse::Error { message: stderr }
        }
        Err(e) => {
            warn!("Failed to run systemctl restart {}: {}", name, e);
            ExecutorResponse::Error { message: e.to_string() }
        }
    }
}

fn clean_journal(keep_days: u32) -> ExecutorResponse {
    // Cap keep_days to prevent accidental data loss (min 1 day)
    let days = keep_days.max(1);
    let vacuum_arg = format!("{}d", days);

    info!("Vacuuming journal, keeping {} days", days);
    let result = Command::new("journalctl")
        .args(["--vacuum-time", &vacuum_arg])
        .output();

    match result {
        Ok(output) if output.status.success() => ExecutorResponse::Ok {
            output: String::from_utf8_lossy(&output.stderr).to_string(), // journalctl reports to stderr
        },
        Ok(output) => ExecutorResponse::Error {
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(e) => ExecutorResponse::Error { message: e.to_string() },
    }
}

fn clean_package_cache(keep_versions: u32) -> ExecutorResponse {
    // Cap to at least 1 to avoid deleting all cached versions
    let k = keep_versions.max(1);

    info!("Running paccache -rk{}", k);
    let result = Command::new("paccache")
        .args(["-rk", &k.to_string()])
        .output();

    match result {
        Ok(output) if output.status.success() => ExecutorResponse::Ok {
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        },
        Ok(output) => ExecutorResponse::Error {
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(e) => ExecutorResponse::Error { message: e.to_string() },
    }
}

fn clean_tmp_files() -> ExecutorResponse {
    info!("Cleaning /tmp files older than 1 day");
    // Using find with explicit args — no shell, no globbing
    let result = Command::new("find")
        .args(["/tmp", "-type", "f", "-mtime", "+1", "-delete"])
        .output();

    match result {
        Ok(output) if output.status.success() => ExecutorResponse::Ok {
            output: "Cleaned stale files from /tmp".to_string(),
        },
        Ok(output) => ExecutorResponse::Error {
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(e) => ExecutorResponse::Error { message: e.to_string() },
    }
}

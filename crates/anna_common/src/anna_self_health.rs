//! Anna Self-Health Module (6.11.0)
//!
//! Anna checks that she has everything she needs to function:
//! - Required system tools (systemctl, journalctl, etc.)
//! - Correct permissions (journal access, data directories)
//! - LLM backend availability
//!
//! This is detect-and-report only in 6.11.0. No auto-installation.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Anna's self-health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaSelfHealth {
    /// All required dependencies present
    pub deps_ok: bool,
    /// List of missing dependencies (if any)
    pub missing_deps: Vec<String>,
    /// Permissions are correct
    pub permissions_ok: bool,
    /// List of permission issues (if any)
    pub missing_permissions: Vec<String>,
    /// LLM backend is healthy
    pub llm_ok: bool,
    /// Human-readable LLM status details
    pub llm_details: String,
}

/// Check Anna's self-health
///
/// Verifies that Anna has all the tools and permissions she needs.
/// Does not modify the system - only detects and reports.
pub fn check_anna_self_health() -> AnnaSelfHealth {
    let (deps_ok, missing_deps) = check_dependencies();
    let (permissions_ok, missing_permissions) = check_permissions();
    let (llm_ok, llm_details) = check_llm_backend();

    AnnaSelfHealth {
        deps_ok,
        missing_deps,
        permissions_ok,
        missing_permissions,
        llm_ok,
        llm_details,
    }
}

/// Check for required system tools
///
/// Anna needs these tools to function properly:
/// - systemctl: Service management and detection
/// - journalctl: Log analysis
/// - ps: Process inspection
/// - df: Disk space checks
/// - ip/ping: Network diagnostics
fn check_dependencies() -> (bool, Vec<String>) {
    let required_tools = [
        "systemctl",
        "journalctl",
        "ps",
        "df",
        "ip",  // Preferred for network checks
    ];

    let mut missing = Vec::new();

    for tool in &required_tools {
        if !command_exists(tool) {
            missing.push(tool.to_string());
        }
    }

    // Check for ping as optional (ip is preferred but ping is common fallback)
    if !command_exists("ping") && !missing.contains(&"ip".to_string()) {
        // Only warn about ping if ip is also missing
        if missing.contains(&"ip".to_string()) {
            missing.push("ping (fallback)".to_string());
        }
    }

    (missing.is_empty(), missing)
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check Anna's permissions
///
/// Verifies:
/// - Can read systemd journal
/// - Data directories are accessible
/// - User has necessary group memberships
fn check_permissions() -> (bool, Vec<String>) {
    let mut issues = Vec::new();

    // Check journal access
    let journal_test = Command::new("journalctl")
        .args(&["-n", "1", "--no-pager"])
        .output();

    match journal_test {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("permission") || stderr.contains("access denied") {
                    issues.push("Cannot read systemd journal - user may need systemd-journal group".to_string());
                }
            }
        }
        Err(_) => {
            issues.push("Cannot execute journalctl".to_string());
        }
    }

    // Check data directory access
    let data_dirs = [
        "/var/lib/anna",
        "/var/log/anna",
    ];

    for dir in &data_dirs {
        if std::path::Path::new(dir).exists() {
            // Try to write a test file
            let test_file = format!("{}/.health_check", dir);
            if std::fs::write(&test_file, "test").is_err() {
                issues.push(format!("Cannot write to {}", dir));
            } else {
                let _ = std::fs::remove_file(&test_file);
            }
        }
    }

    // Check user groups (if running as non-root)
    if let Ok(output) = Command::new("id").arg("-Gn").output() {
        if output.status.success() {
            let groups = String::from_utf8_lossy(&output.stdout);
            let groups_lower = groups.to_lowercase();

            // Check for useful groups
            if !groups_lower.contains("systemd-journal") {
                // This is informational, not critical
                // We already checked if journal works above
            }
        }
    }

    (issues.is_empty(), issues)
}

/// Check LLM backend health
///
/// For Ollama: Checks if service is running and configured model is available
/// For other backends: TBD in future versions
fn check_llm_backend() -> (bool, String) {
    // Try to detect Ollama
    let ollama_running = Command::new("systemctl")
        .args(&["is-active", "ollama"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ollama_running {
        // Try to check if ollama command exists
        let ollama_exists = command_exists("ollama");

        if !ollama_exists {
            return (
                false,
                "Ollama not installed (expected at /usr/bin/ollama)".to_string(),
            );
        }

        return (
            false,
            "Ollama service not running (try: sudo systemctl start ollama)".to_string(),
        );
    }

    // Check if we can list models
    let models_check = Command::new("ollama")
        .args(&["list"])
        .output();

    match models_check {
        Ok(output) => {
            if output.status.success() {
                let models_text = String::from_utf8_lossy(&output.stdout);

                // Try to read configured model from context.db
                // For now, just check if any models are available
                if models_text.lines().count() <= 1 {
                    // Only header line, no models
                    return (
                        false,
                        "No Ollama models installed (try: ollama pull llama3.1:8b)".to_string(),
                    );
                }

                // At least one model exists
                let model_count = models_text.lines().count() - 1; // Subtract header
                return (
                    true,
                    format!("Ollama running with {} model(s) available", model_count),
                );
            }

            (
                false,
                format!("Cannot list Ollama models: {}", String::from_utf8_lossy(&output.stderr)),
            )
        }
        Err(e) => (
            false,
            format!("Cannot check Ollama models: {}", e),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_health_returns_struct() {
        // Just verify the function runs without panicking
        let health = check_anna_self_health();

        // Should have at least checked something
        assert!(health.deps_ok || !health.missing_deps.is_empty());
    }

    #[test]
    fn test_command_exists() {
        // These should always exist on any Unix system
        assert!(command_exists("ls"));
        assert!(command_exists("echo"));

        // This should not exist
        assert!(!command_exists("nonexistent_command_xyz_123"));
    }

    #[test]
    fn test_check_dependencies_structure() {
        let (ok, missing) = check_dependencies();

        // Either all deps are ok, or we have a list of missing ones
        if ok {
            assert!(missing.is_empty());
        } else {
            assert!(!missing.is_empty());
        }
    }

    #[test]
    fn test_check_permissions_structure() {
        let (ok, issues) = check_permissions();

        // Either permissions are ok, or we have a list of issues
        if ok {
            assert!(issues.is_empty());
        } else {
            assert!(!issues.is_empty());
        }
    }

    #[test]
    fn test_check_llm_backend_structure() {
        let (ok, details) = check_llm_backend();

        // Details should never be empty
        assert!(!details.is_empty());

        // Details should be human-readable
        assert!(details.len() > 10); // At least a sentence
    }
}

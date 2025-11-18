//! Internal Repair Engine
//!
//! Self-healing for Anna's own components. Called by:
//! - REPL when user says "fix yourself", "repair anna", etc.
//! - annad on startup for automatic self-healing
//! - annactl status for trivial fixes
//!
//! NOT exposed as a public CLI command.

use anna_common::terminal_format as fmt;
use anyhow::Result;

use crate::systemd;

/// Result of a repair operation
#[derive(Debug, Clone)]
pub struct RepairReport {
    pub issues_found: Vec<RepairIssue>,
    pub fixes_applied: Vec<RepairFix>,
    pub all_healthy: bool,
}

#[derive(Debug, Clone)]
pub struct RepairIssue {
    pub description: String,
    pub severity: IssueSeverity,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct RepairFix {
    pub description: String,
    pub success: bool,
    pub details: String,
}

/// Check Anna's health and return issues
///
/// This is idempotent - safe to call repeatedly
pub fn check_health() -> Result<RepairReport> {
    let mut issues = Vec::new();

    // Check 1: Daemon status
    if let Ok(status) = systemd::get_service_status() {
        if !status.exists {
            issues.push(RepairIssue {
                description: "Anna daemon service is not installed".to_string(),
                severity: IssueSeverity::Critical,
                auto_fixable: true,
            });
        } else if !status.enabled {
            issues.push(RepairIssue {
                description: "Anna daemon is not enabled (won't start on boot)".to_string(),
                severity: IssueSeverity::Warning,
                auto_fixable: true,
            });
        } else if status.failed {
            issues.push(RepairIssue {
                description: "Anna daemon has failed".to_string(),
                severity: IssueSeverity::Critical,
                auto_fixable: true,
            });
        } else if !status.active {
            issues.push(RepairIssue {
                description: "Anna daemon is not running".to_string(),
                severity: IssueSeverity::Warning,
                auto_fixable: true,
            });
        }
    }

    // Check 2: Database accessibility
    // TODO: Add database health check

    // Check 3: Key directories
    // TODO: Add directory checks

    Ok(RepairReport {
        all_healthy: issues.is_empty(),
        issues_found: issues,
        fixes_applied: Vec::new(),
    })
}

/// Repair Anna's health issues
///
/// Returns a report of what was fixed
pub fn repair() -> Result<RepairReport> {
    let health = check_health()?;

    if health.all_healthy {
        return Ok(health);
    }

    let mut fixes = Vec::new();

    // Repair daemon issues
    for issue in &health.issues_found {
        if issue.description.contains("daemon") && issue.auto_fixable {
            match systemd::repair_service() {
                Ok(report) => {
                    fixes.push(RepairFix {
                        description: "Repaired daemon service".to_string(),
                        success: true,
                        details: report,
                    });
                }
                Err(e) => {
                    fixes.push(RepairFix {
                        description: "Failed to repair daemon".to_string(),
                        success: false,
                        details: e.to_string(),
                    });
                }
            }
        }
    }

    // Check final health
    let final_health = check_health()?;

    Ok(RepairReport {
        all_healthy: final_health.all_healthy,
        issues_found: health.issues_found,
        fixes_applied: fixes,
    })
}

/// Display repair report to user in conversational format
pub fn display_repair_report(report: &RepairReport) {
    if report.all_healthy && report.fixes_applied.is_empty() {
        println!("{}", fmt::success("I'm healthy! No issues found."));
        println!();
        return;
    }

    if !report.issues_found.is_empty() {
        println!("{}", fmt::bold("Issues I found:"));
        for issue in &report.issues_found {
            let icon = match issue.severity {
                IssueSeverity::Critical => "🔴",
                IssueSeverity::Warning => "🟡",
                IssueSeverity::Info => "ℹ️",
            };
            println!("  {} {}", icon, issue.description);
        }
        println!();
    }

    if !report.fixes_applied.is_empty() {
        println!("{}", fmt::bold("Fixes I applied:"));
        for fix in &report.fixes_applied {
            if fix.success {
                println!("  {} {}", fmt::success("✓"), fix.description);
                if !fix.details.is_empty() {
                    for line in fix.details.lines() {
                        if !line.is_empty() {
                            println!("    {}", fmt::dimmed(line));
                        }
                    }
                }
            } else {
                println!("  {} {}", fmt::error("✗"), fix.description);
                println!("    {}", fmt::error(&fix.details));
            }
        }
        println!();
    }

    if report.all_healthy {
        println!("{}", fmt::success("All fixed! I'm healthy now."));
    } else {
        println!(
            "{}",
            fmt::warning("Some issues remain. You may need to check manually.")
        );
    }
    println!();
}

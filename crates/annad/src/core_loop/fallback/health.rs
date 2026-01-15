//! Proactive health checks for fallback system.
//! v0.0.953: Health check result types and implementations
//! v0.0.992: Integrated with comprehensive monitoring system

use std::process::Command;
use std::sync::RwLock;
use tracing::{debug, info, warn};

use anna_shared::monitor::{run_checks, IssueStore, MonitorThresholds};

/// v0.0.953: Health check result
#[derive(Clone, Debug)]
pub struct HealthCheckResult {
    pub category: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<String>,
}

/// v0.0.953: Health status levels
#[derive(Clone, Debug, PartialEq)]
pub enum HealthStatus {
    Ok,
    Warning,
    Critical,
}

/// v0.0.953: Cached health status
static HEALTH_CACHE: RwLock<Option<Vec<HealthCheckResult>>> = RwLock::new(None);

/// v0.0.992: Run the comprehensive monitoring system and save issues
fn run_comprehensive_monitoring() {
    let thresholds = MonitorThresholds::default();
    let results = run_checks(&thresholds);

    // Save issues to store for display in REPL
    match IssueStore::load() {
        Ok(mut store) => {
            store.update(results);
            if let Err(e) = store.save() {
                warn!("Failed to save monitoring issues: {}", e);
            } else {
                let unnotified = store.get_unnotified().len();
                let critical = store.get_critical().len();
                if critical > 0 {
                    info!("Monitoring: {} critical issues detected", critical);
                } else if unnotified > 0 {
                    debug!("Monitoring: {} new issues detected", unnotified);
                }
            }
        }
        Err(e) => {
            warn!("Failed to load issue store: {}", e);
        }
    }
}

/// v0.0.953: Run proactive health checks and cache results
/// v0.0.992: Integrated with comprehensive monitoring system
/// Called at startup and can be called periodically
pub fn run_health_checks() -> Vec<HealthCheckResult> {
    info!("Running proactive health checks...");

    // v0.0.992: Run comprehensive monitoring and save issues
    run_comprehensive_monitoring();
    let mut results = Vec::new();

    // Check disk space
    if let Ok(output) = Command::new("df").arg("-h").arg("/").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            // Parse disk usage percentage
            if let Some(line) = out.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let usage_str = parts[4].trim_end_matches('%');
                    if let Ok(usage) = usage_str.parse::<u32>() {
                        let status = if usage >= 95 {
                            HealthStatus::Critical
                        } else if usage >= 85 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Ok
                        };
                        results.push(HealthCheckResult {
                            category: "disk".to_string(),
                            status,
                            message: format!("Root partition {}% used", usage),
                            details: Some(line.to_string()),
                        });
                    }
                }
            }
        }
    }

    // Check memory
    if let Ok(output) = Command::new("free").arg("-m").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = out.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                        let pct = (used * 100) / total.max(1);
                        let status = if pct >= 95 {
                            HealthStatus::Critical
                        } else if pct >= 85 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Ok
                        };
                        results.push(HealthCheckResult {
                            category: "memory".to_string(),
                            status,
                            message: format!("Memory {}% used ({}/{}MB)", pct, used, total),
                            details: None,
                        });
                    }
                }
            }
        }
    }

    // Check failed services
    if let Ok(output) = Command::new("systemctl").args(["--failed", "--no-pager"]).output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            let failed_count = out.lines().filter(|l| l.contains("failed")).count();
            let status = if failed_count > 0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Ok
            };
            results.push(HealthCheckResult {
                category: "services".to_string(),
                status,
                message: if failed_count > 0 {
                    format!("{} failed service(s)", failed_count)
                } else {
                    "All services running".to_string()
                },
                details: if failed_count > 0 { Some(out.to_string()) } else { None },
            });
        }
    }

    // Check system load
    if let Ok(output) = Command::new("cat").arg("/proc/loadavg").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            if let Some(first) = out.split_whitespace().next() {
                if let Ok(load) = first.parse::<f32>() {
                    // Get CPU count for context
                    let ncpu = num_cpus().unwrap_or(4) as f32;
                    let status = if load > ncpu * 2.0 {
                        HealthStatus::Critical
                    } else if load > ncpu {
                        HealthStatus::Warning
                    } else {
                        HealthStatus::Ok
                    };
                    results.push(HealthCheckResult {
                        category: "load".to_string(),
                        status,
                        message: format!("System load: {:.2}", load),
                        details: None,
                    });
                }
            }
        }
    }

    // Check for recent errors in journal
    if let Ok(output) = Command::new("journalctl")
        .args(["-p", "err", "-b", "--no-pager", "-n", "10"])
        .output()
    {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            let error_count = out.lines().filter(|l| !l.is_empty()).count();
            let status = if error_count > 5 {
                HealthStatus::Warning
            } else {
                HealthStatus::Ok
            };
            results.push(HealthCheckResult {
                category: "errors".to_string(),
                status,
                message: format!("{} recent error(s) in journal", error_count),
                details: if error_count > 0 { Some(out.lines().take(5).collect::<Vec<_>>().join("\n")) } else { None },
            });
        }
    }

    // Cache results
    if let Ok(mut guard) = HEALTH_CACHE.write() {
        *guard = Some(results.clone());
    }

    let warnings = results.iter().filter(|r| r.status == HealthStatus::Warning).count();
    let criticals = results.iter().filter(|r| r.status == HealthStatus::Critical).count();
    info!("Health checks complete: {} checks, {} warnings, {} critical", results.len(), warnings, criticals);

    results
}

/// v0.0.953: Get cached health check results
pub fn get_cached_health() -> Option<Vec<HealthCheckResult>> {
    if let Ok(guard) = HEALTH_CACHE.read() {
        guard.clone()
    } else {
        None
    }
}

/// v0.0.953: Get summary of system health for instant answers
pub fn get_health_summary() -> String {
    match get_cached_health() {
        Some(results) => {
            let mut summary = Vec::new();
            // v0.3.30: Use plain text instead of emojis
            for r in &results {
                let icon = match r.status {
                    HealthStatus::Ok => "[OK]",
                    HealthStatus::Warning => "[WARN]",
                    HealthStatus::Critical => "[CRIT]",
                };
                summary.push(format!("{} {}: {}", icon, r.category, r.message));
            }
            summary.join("\n")
        }
        None => "Health checks not yet run".to_string(),
    }
}

/// Helper to get CPU count
fn num_cpus() -> Option<u32> {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.matches("processor").count() as u32)
}

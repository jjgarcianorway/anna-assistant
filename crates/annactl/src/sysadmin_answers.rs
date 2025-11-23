//! Sysadmin Answer Composer - Beta.263 + Beta.264
//!
//! Deterministic answer patterns for system troubleshooting.
//! Beta.263: services, disk, logs
//! Beta.264: CPU, memory, processes, network
//!
//! These functions compose focused, sysadmin-grade answers using real system data.

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use anna_common::systemd_health::SystemdHealth;
use anna_common::telemetry::{CpuInfo, MemoryInfo};

/// Compose a focused service health answer
///
/// Provides sysadmin-grade service health analysis with:
/// - Clear summary of service state
/// - Top failed services with details
/// - Relevant systemctl commands
pub fn compose_service_health_answer(brain: &BrainAnalysisData, systemd: &SystemdHealth) -> String {
    let failed_count = systemd.failed_units.len();

    // Build summary
    let summary = if failed_count == 0 {
        "[SUMMARY]\nService health: all core services are running.".to_string()
    } else if failed_count == 1 {
        "[SUMMARY]\nService health: 1 failed service detected.".to_string()
    } else {
        format!("[SUMMARY]\nService health: {} failed services detected.", failed_count)
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if failed_count == 0 {
        details.push_str("\nNo failed systemd units.");
    } else {
        // Show top 3 failed units
        for (i, unit) in systemd.failed_units.iter().take(3).enumerate() {
            if i == 0 {
                details.push('\n');
            }
            details.push_str(&format!(
                "\n✗ {} ({})",
                unit.name,
                unit.active_state
            ));
        }

        if failed_count > 3 {
            details.push_str(&format!("\n... and {} more", failed_count - 3));
        }
    }

    // Build commands
    let mut commands = String::from("\n\n[COMMANDS]");
    if failed_count > 0 {
        commands.push_str("\n# List all failed services:");
        commands.push_str("\nsystemctl --failed");

        // Add status command for first failed service
        if let Some(first_unit) = systemd.failed_units.first() {
            commands.push_str(&format!("\n\n# Check specific service status:"));
            commands.push_str(&format!("\nsystemctl status {}", first_unit.name));
            commands.push_str(&format!("\n\n# View service logs:"));
            commands.push_str(&format!("\njournalctl -u {} -n 50", first_unit.name));
        }
    } else {
        commands.push_str("\n# Verify service health:");
        commands.push_str("\nsystemctl --failed");
        commands.push_str("\n\n# List all services:");
        commands.push_str("\nsystemctl list-units --type=service");
    }

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused disk health answer
///
/// Provides sysadmin-grade disk analysis with:
/// - Clear summary of disk state
/// - Affected mount points with usage
/// - Relevant df/du commands
pub fn compose_disk_health_answer(brain: &BrainAnalysisData) -> String {
    // Extract disk insights from brain analysis
    let disk_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("disk") ||
                    i.summary.to_lowercase().contains("filesystem"))
        .collect();

    let has_critical = disk_insights.iter().any(|i| i.severity == "critical");
    let has_warning = disk_insights.iter().any(|i| i.severity == "warning");

    // Build summary
    let summary = if has_critical {
        "[SUMMARY]\nDisk health: critical – filesystem usage requires attention.".to_string()
    } else if has_warning {
        "[SUMMARY]\nDisk health: warning – some filesystems approaching capacity.".to_string()
    } else {
        "[SUMMARY]\nDisk health: all monitored filesystems below 80% usage.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if disk_insights.is_empty() {
        details.push_str("\nAll filesystems are within normal usage thresholds.");
    } else {
        for insight in disk_insights.iter().take(3) {
            details.push_str(&format!("\n• {}", insight.summary));
        }
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check filesystem usage:\n\
df -h\n\
\n\
# Check inode usage:\n\
df -i\n\
\n\
# Find large directories:\n\
du -h /var | sort -h | tail -20\n\
\n\
# Check /tmp usage:\n\
du -sh /tmp/*";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused log health answer
///
/// Provides sysadmin-grade log analysis with:
/// - Clear summary of log state
/// - Representative error types
/// - Relevant journalctl commands
pub fn compose_log_health_answer(brain: &BrainAnalysisData) -> String {
    // Extract log-related insights
    let log_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("log") ||
                    i.summary.to_lowercase().contains("error") ||
                    i.summary.to_lowercase().contains("journal"))
        .collect();

    let has_critical = log_insights.iter().any(|i| i.severity == "critical");
    let has_errors = !log_insights.is_empty();

    // Build summary
    let summary = if has_critical {
        "[SUMMARY]\nLog health: critical errors detected in recent system logs.".to_string()
    } else if has_errors {
        "[SUMMARY]\nLog health: warnings detected in recent system logs.".to_string()
    } else {
        "[SUMMARY]\nLog health: no critical errors in recent system logs.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if log_insights.is_empty() {
        details.push_str("\nNo significant errors detected in systemd journal.");
    } else {
        for insight in log_insights.iter().take(3) {
            details.push_str(&format!("\n• {}", insight.summary));
        }
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check recent errors:\n\
journalctl -p err -n 20\n\
\n\
# Check critical errors since boot:\n\
journalctl -p crit -b\n\
\n\
# Check last hour of logs:\n\
journalctl --since \"1 hour ago\"\n\
\n\
# Check journal disk usage:\n\
journalctl --disk-usage";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused CPU health answer (Beta.264)
///
/// Provides sysadmin-grade CPU analysis with:
/// - Clear summary of CPU state
/// - Load average vs cores context
/// - Relevant top/ps commands
pub fn compose_cpu_health_answer(cpu: &CpuInfo, brain: &BrainAnalysisData) -> String {
    let usage_pct = cpu.usage_percent.unwrap_or(0.0);

    // Build summary
    let summary = if usage_pct > 80.0 {
        "[SUMMARY]\nCPU health: degraded – high utilization detected.".to_string()
    } else if usage_pct > 50.0 {
        "[SUMMARY]\nCPU health: moderate – CPU usage is elevated but manageable.".to_string()
    } else {
        "[SUMMARY]\nCPU health: all clear – CPU load within normal range.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");
    details.push_str(&format!("\nCPU usage: {:.1}%", usage_pct));
    details.push_str(&format!("\nCPU cores: {}", cpu.cores));
    details.push_str(&format!("\nLoad average: {:.2} (1min)", cpu.load_avg_1min));

    // Add CPU-related insights from brain
    let cpu_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("cpu") ||
                    i.summary.to_lowercase().contains("load"))
        .take(2)
        .collect();

    for insight in cpu_insights {
        details.push_str(&format!("\n• {}", insight.summary));
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check load average:\n\
uptime\n\
\n\
# Top CPU consumers:\n\
ps -eo pid,comm,%cpu --sort=-%cpu | head -10\n\
\n\
# Interactive monitoring:\n\
top";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused memory health answer (Beta.264)
///
/// Provides sysadmin-grade memory analysis with:
/// - Clear summary of memory state
/// - RAM and swap usage
/// - Relevant free/ps commands
pub fn compose_memory_health_answer(memory: &MemoryInfo, brain: &BrainAnalysisData) -> String {
    let mem_pct = memory.usage_percent;
    let swap_used = memory.swap_used_mb > 0;

    // Build summary
    let summary = if mem_pct > 90.0 && swap_used {
        "[SUMMARY]\nMemory health: degraded – high memory usage with active swap.".to_string()
    } else if mem_pct > 85.0 {
        "[SUMMARY]\nMemory health: warning – memory usage is high.".to_string()
    } else {
        "[SUMMARY]\nMemory health: all clear – free memory and cache levels are healthy.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");
    details.push_str(&format!("\nRAM: {:.1} GB used / {:.1} GB total ({:.0}%)",
        memory.used_mb as f64 / 1024.0,
        memory.total_mb as f64 / 1024.0,
        mem_pct));

    if swap_used {
        details.push_str(&format!("\nSwap: {:.1} GB in use (total {:.1} GB)",
            memory.swap_used_mb as f64 / 1024.0,
            memory.swap_total_mb as f64 / 1024.0));
    } else {
        details.push_str("\nSwap: not in use");
    }

    // Add memory-related insights
    let mem_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("memory") ||
                    i.summary.to_lowercase().contains("swap") ||
                    i.summary.to_lowercase().contains("ram"))
        .take(2)
        .collect();

    for insight in mem_insights {
        details.push_str(&format!("\n• {}", insight.summary));
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check memory usage:\n\
free -h\n\
\n\
# Detailed memory info:\n\
cat /proc/meminfo | head -20\n\
\n\
# Top memory consumers:\n\
ps -eo pid,comm,%mem --sort=-%mem | head -10";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused process health answer (Beta.264)
///
/// Provides sysadmin-grade process analysis with:
/// - Clear summary of process state
/// - Top resource consumers
/// - Relevant ps commands
pub fn compose_process_health_answer(brain: &BrainAnalysisData) -> String {
    // Extract process-related insights
    let process_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("process") ||
                    i.summary.to_lowercase().contains("consuming") ||
                    i.summary.to_lowercase().contains("using cpu") ||
                    i.summary.to_lowercase().contains("using memory"))
        .collect();

    let has_issues = !process_insights.is_empty();

    // Build summary
    let summary = if has_issues {
        if process_insights.len() == 1 {
            "[SUMMARY]\nProcess health: degraded – 1 process consuming disproportionate resources.".to_string()
        } else {
            format!("[SUMMARY]\nProcess health: degraded – {} processes consuming disproportionate resources.", process_insights.len())
        }
    } else {
        "[SUMMARY]\nProcess health: no obviously misbehaving processes detected.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if process_insights.is_empty() {
        details.push_str("\nNo processes showing anomalous resource consumption.");
    } else {
        for insight in process_insights.iter().take(3) {
            details.push_str(&format!("\n• {}", insight.summary));
        }
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Top CPU consumers:\n\
ps -eo pid,comm,%cpu,%mem --sort=-%cpu | head -10\n\
\n\
# Top memory consumers:\n\
ps -eo pid,comm,%cpu,%mem --sort=-%mem | head -10\n\
\n\
# Interactive process monitor:\n\
top";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused network health answer (Beta.264)
///
/// Provides sysadmin-grade network analysis with:
/// - Clear summary of network state
/// - Basic connectivity checks
/// - Relevant ip/ping commands
pub fn compose_network_health_answer(brain: &BrainAnalysisData) -> String {
    // Extract network-related insights
    let network_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("network") ||
                    i.summary.to_lowercase().contains("connectivity") ||
                    i.summary.to_lowercase().contains("dns") ||
                    i.summary.to_lowercase().contains("interface"))
        .collect();

    let has_issues = !network_insights.is_empty();

    // Build summary
    let summary = if has_issues {
        "[SUMMARY]\nNetwork health: degraded – connectivity issues detected.".to_string()
    } else {
        "[SUMMARY]\nNetwork health: all clear – local network and basic connectivity look healthy.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if network_insights.is_empty() {
        details.push_str("\nNo obvious network connectivity problems detected.");
    } else {
        for insight in network_insights.iter().take(3) {
            details.push_str(&format!("\n• {}", insight.summary));
        }
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check network interfaces:\n\
ip addr show\n\
\n\
# Check routing table:\n\
ip route\n\
\n\
# Test basic connectivity:\n\
ping -c 4 1.1.1.1\n\
\n\
# Test DNS resolution:\n\
ping -c 4 example.com";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused network conflict answer (Beta.265)
///
/// Detects and reports multi-interface collisions using NetworkDiagnostics.
/// Scenarios covered:
/// - Ethernet slower than WiFi but taking default route
/// - Duplicate default routes
/// - Multiple active interfaces causing unpredictable routing
/// - Interface priority mismatches
pub fn compose_network_conflict_answer(
    diagnostics: &crate::net_diagnostics::NetworkDiagnostics,
) -> String {
    use crate::net_diagnostics::{CollisionType, NetworkHealthStatus};

    let has_conflict = diagnostics.interface_collision.is_some();

    // Build summary
    let summary = if let Some(ref collision) = diagnostics.interface_collision {
        match collision.collision_type {
            CollisionType::EthernetSlowerThanWiFi => {
                "[SUMMARY]\nNetwork conflict: critical – Ethernet is slower than WiFi but taking default route priority.".to_string()
            }
            CollisionType::DuplicateDefaultRoutes => {
                "[SUMMARY]\nNetwork conflict: critical – Multiple default routes detected causing unpredictable routing.".to_string()
            }
            CollisionType::MultipleActiveInterfaces => {
                "[SUMMARY]\nNetwork conflict: warning – Both Ethernet and WiFi are active simultaneously.".to_string()
            }
        }
    } else if diagnostics.health_status == NetworkHealthStatus::Healthy {
        "[SUMMARY]\nNetwork conflict: none – Single active interface with proper routing.".to_string()
    } else {
        "[SUMMARY]\nNetwork conflict: no interface collision detected.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if let Some(ref collision) = diagnostics.interface_collision {
        details.push_str(&format!("\n{}", collision.description));

        // Add interface speeds if available
        if let Some(eth_speed) = collision.metrics.ethernet_speed_mbps {
            details.push_str(&format!("\nEthernet speed: {} Mbps", eth_speed));
        }
        if let Some(wifi_speed) = collision.metrics.wifi_speed_mbps {
            details.push_str(&format!("\nWiFi speed: {} Mbps", wifi_speed));
        }

        // Add error rates
        if collision.metrics.ethernet_error_rate > 0.0 {
            details.push_str(&format!("\nEthernet error rate: {:.2}%", collision.metrics.ethernet_error_rate));
        }
        if collision.metrics.wifi_error_rate > 0.0 {
            details.push_str(&format!("\nWiFi error rate: {:.2}%", collision.metrics.wifi_error_rate));
        }

        // Add default route info
        if let Some(ref default_iface) = collision.metrics.default_route_interface {
            details.push_str(&format!("\nDefault route interface: {}", default_iface));
        }

        // Add priority mismatch if present
        if let Some(ref mismatch) = diagnostics.priority_mismatch {
            details.push_str(&format!("\n\nPriority mismatch: {}", mismatch.description));
            details.push_str(&format!("\n  Expected: {} (rank {})", mismatch.expected_interface, mismatch.expected_rank));
            details.push_str(&format!("\n  Actual: {} (rank {})", mismatch.actual_interface, mismatch.actual_rank));
        }
    } else {
        details.push_str("\nNo multi-interface conflicts detected.");
        details.push_str("\nSingle active interface configuration is optimal for predictable routing.");
    }

    // Build commands
    let commands = if has_conflict {
        "\n\n[COMMANDS]\n\
# Check interface status and speeds:\n\
ip -brief link show\n\
ethtool <interface>  # Replace <interface> with eth0, wlan0, etc.\n\
\n\
# Check routing table and metrics:\n\
ip route show\n\
\n\
# Check NetworkManager connections:\n\
nmcli device show\n\
nmcli connection show --active\n\
\n\
# Disable slower interface (example for eth0):\n\
nmcli device disconnect eth0\n\
\n\
# Or adjust route metrics to prefer faster interface:\n\
# Lower metric = higher priority\n\
nmcli connection modify <connection-name> ipv4.route-metric 100"
    } else {
        "\n\n[COMMANDS]\n\
# Verify current network configuration:\n\
ip -brief link show\n\
ip route show\n\
nmcli device status"
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused network routing answer (Beta.265)
///
/// Detects and reports routing table misconfigurations.
/// Scenarios covered:
/// - Duplicate default routes with same metric
/// - Missing IPv4/IPv6 fallback routes
/// - High metric on default route
/// - Connectivity degradation (packet loss, latency, DNS)
pub fn compose_network_routing_answer(
    diagnostics: &crate::net_diagnostics::NetworkDiagnostics,
) -> String {
    use crate::net_diagnostics::{DiagnosticSeverity, NetworkHealthStatus};

    let has_routing_issues = !diagnostics.routing_issues.is_empty();
    let has_degradation = diagnostics.connectivity_degradation.is_some();

    // Build summary
    let summary = if diagnostics.health_status == NetworkHealthStatus::Critical {
        "[SUMMARY]\nNetwork routing: critical – Major routing or connectivity issues detected.".to_string()
    } else if diagnostics.health_status == NetworkHealthStatus::Degraded {
        "[SUMMARY]\nNetwork routing: degraded – Minor routing configuration issues detected.".to_string()
    } else {
        "[SUMMARY]\nNetwork routing: all clear – Routing table properly configured.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    // Report routing issues
    if has_routing_issues {
        details.push_str("\n\nRouting Table Issues:");
        for issue in &diagnostics.routing_issues {
            details.push_str(&format!("\n• {} - {}",
                match issue.severity {
                    DiagnosticSeverity::Critical => "CRITICAL",
                    DiagnosticSeverity::Warning => "WARNING",
                },
                issue.description
            ));
            if !issue.affected_routes.is_empty() {
                for route in &issue.affected_routes {
                    details.push_str(&format!("\n  - {}", route));
                }
            }
        }
    }

    // Report connectivity degradation
    if let Some(ref degradation) = diagnostics.connectivity_degradation {
        details.push_str("\n\nConnectivity Issues:");
        details.push_str(&format!("\n• {} - {}",
            match degradation.severity {
                DiagnosticSeverity::Critical => "CRITICAL",
                DiagnosticSeverity::Warning => "WARNING",
            },
            degradation.description
        ));

        // Add metrics
        if let Some(loss) = degradation.metrics.packet_loss_percent {
            details.push_str(&format!("\n  Packet loss: {:.1}%", loss));
        }
        if let Some(latency) = degradation.metrics.internet_latency_ms {
            details.push_str(&format!("\n  Internet latency: {:.1}ms", latency));
        }
        if let Some(gw_latency) = degradation.metrics.gateway_latency_ms {
            details.push_str(&format!("\n  Gateway latency: {:.1}ms", gw_latency));
        }
        if degradation.metrics.interface_errors > 0 {
            details.push_str(&format!("\n  Interface errors: {}", degradation.metrics.interface_errors));
        }
    }

    if !has_routing_issues && !has_degradation {
        details.push_str("\nRouting table configuration is correct.");
        details.push_str("\nNo connectivity degradation detected.");
    }

    // Build commands
    let commands = if has_routing_issues || has_degradation {
        let mut cmd = String::from("\n\n[COMMANDS]\n");

        if has_routing_issues {
            cmd.push_str("# Check routing table:\n\
ip route show\n\
ip -6 route show\n\
\n");
        }

        if has_degradation {
            cmd.push_str("# Test connectivity:\n\
ping -c 10 $(ip route | grep default | awk '{print $3}')  # Gateway\n\
ping -c 10 1.1.1.1  # Internet\n\
ping -c 10 example.com  # DNS resolution\n\
\n\
# Check interface statistics:\n\
ip -s link show\n\
\n");
        }

        cmd.push_str("# NetworkManager diagnostics:\n\
nmcli device show\n\
systemd-resolve --status\n\
\n\
# Check for interface flapping:\n\
journalctl -u NetworkManager --since \"1 hour ago\" | grep -E 'up|down|disconnect'");

        cmd
    } else {
        "\n\n[COMMANDS]\n\
# Verify routing configuration:\n\
ip route show\n\
ip -6 route show\n\
\n\
# Test basic connectivity:\n\
ping -c 4 1.1.1.1\n\
ping -c 4 example.com".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::systemd_health::FailedUnit;

    #[test]
    fn test_service_health_no_failures() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };
        let systemd = SystemdHealth {
            failed_units: vec![],
            essential_timers: vec![],
            journal_disk_usage_mb: Some(100),
            journal_rotation_configured: true,
        };

        let answer = compose_service_health_answer(&brain, &systemd);

        assert!(answer.contains("[SUMMARY]"));
        assert!(answer.contains("all core services are running"));
        assert!(answer.contains("[DETAILS]"));
        assert!(answer.contains("[COMMANDS]"));
        assert!(answer.contains("systemctl --failed"));
    }

    #[test]
    fn test_service_health_one_failure() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };
        let systemd = SystemdHealth {
            failed_units: vec![
                FailedUnit {
                    name: "docker.service".to_string(),
                    unit_type: "service".to_string(),
                    load_state: "loaded".to_string(),
                    active_state: "failed".to_string(),
                    sub_state: "failed".to_string(),
                }
            ],
            essential_timers: vec![],
            journal_disk_usage_mb: Some(100),
            journal_rotation_configured: true,
        };

        let answer = compose_service_health_answer(&brain, &systemd);

        assert!(answer.contains("1 failed service detected"));
        assert!(answer.contains("docker.service"));
        assert!(answer.contains("systemctl status docker.service"));
        assert!(answer.contains("journalctl -u docker.service"));
    }

    #[test]
    fn test_service_health_multiple_failures() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };
        let systemd = SystemdHealth {
            failed_units: vec![
                FailedUnit {
                    name: "docker.service".to_string(),
                    unit_type: "service".to_string(),
                    load_state: "loaded".to_string(),
                    active_state: "failed".to_string(),
                    sub_state: "failed".to_string(),
                },
                FailedUnit {
                    name: "NetworkManager.service".to_string(),
                    unit_type: "service".to_string(),
                    load_state: "loaded".to_string(),
                    active_state: "failed".to_string(),
                    sub_state: "failed".to_string(),
                }
            ],
            essential_timers: vec![],
            journal_disk_usage_mb: Some(100),
            journal_rotation_configured: true,
        };

        let answer = compose_service_health_answer(&brain, &systemd);

        assert!(answer.contains("2 failed services detected"));
        assert!(answer.contains("docker.service"));
        assert!(answer.contains("NetworkManager.service"));
    }

    #[test]
    fn test_disk_health_critical() {
        let brain = BrainAnalysisData {
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "disk-space".to_string(),
                    summary: "Disk usage critical on /: 95% full".to_string(),
                    details: "Root filesystem is at 95% capacity".to_string(),
                    evidence: "df shows 95% usage".to_string(),
                    severity: "critical".to_string(),
                    commands: vec!["df -h".to_string()],
                    citations: vec![],
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 1,
            warning_count: 0,
        };

        let answer = compose_disk_health_answer(&brain);

        assert!(answer.contains("[SUMMARY]"));
        assert!(answer.contains("critical"));
        assert!(answer.contains("Disk usage critical on /"));
        assert!(answer.contains("df -h"));
        assert!(answer.contains("df -i"));
    }

    #[test]
    fn test_disk_health_healthy() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_disk_health_answer(&brain);

        assert!(answer.contains("below 80% usage"));
        assert!(answer.contains("df -h"));
    }

    #[test]
    fn test_log_health_critical() {
        let brain = BrainAnalysisData {
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "journal-errors".to_string(),
                    summary: "Critical errors in systemd journal".to_string(),
                    details: "Multiple critical log entries found".to_string(),
                    evidence: "Found 5 critical log entries".to_string(),
                    severity: "critical".to_string(),
                    commands: vec!["journalctl -p crit".to_string()],
                    citations: vec![],
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 1,
            warning_count: 0,
        };

        let answer = compose_log_health_answer(&brain);

        assert!(answer.contains("critical errors detected"));
        assert!(answer.contains("Critical errors in systemd journal"));
        assert!(answer.contains("journalctl -p err"));
    }

    #[test]
    fn test_log_health_clean() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_log_health_answer(&brain);

        assert!(answer.contains("no critical errors"));
        assert!(answer.contains("journalctl"));
    }

    // Beta.264 tests for CPU/memory/process/network

    #[test]
    fn test_cpu_health_normal() {
        let cpu = CpuInfo {
            cores: 8,
            load_avg_1min: 2.5,
            load_avg_5min: 2.3,
            usage_percent: Some(25.5),
        };
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_cpu_health_answer(&cpu, &brain);

        assert!(answer.contains("[SUMMARY]"));
        assert!(answer.contains("all clear"));
        assert!(answer.contains("25.5%"));
        assert!(answer.contains("uptime"));
        assert!(answer.contains("ps -eo"));
    }

    #[test]
    fn test_cpu_health_high() {
        let cpu = CpuInfo {
            cores: 4,
            load_avg_1min: 3.8,
            load_avg_5min: 3.5,
            usage_percent: Some(85.0),
        };
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_cpu_health_answer(&cpu, &brain);

        assert!(answer.contains("degraded"));
        assert!(answer.contains("85.0%"));
    }

    #[test]
    fn test_memory_health_normal() {
        let memory = MemoryInfo {
            total_mb: 16384,
            used_mb: 8192,
            available_mb: 8192,
            swap_total_mb: 4096,
            swap_used_mb: 0,
            usage_percent: 50.0,
        };
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_memory_health_answer(&memory, &brain);

        assert!(answer.contains("[SUMMARY]"));
        assert!(answer.contains("all clear"));
        assert!(answer.contains("Swap: not in use"));
        assert!(answer.contains("free -h"));
    }

    #[test]
    fn test_memory_health_high_with_swap() {
        let memory = MemoryInfo {
            total_mb: 8192,
            used_mb: 7500,
            available_mb: 692,
            swap_total_mb: 4096,
            swap_used_mb: 1024,
            usage_percent: 91.6,
        };
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_memory_health_answer(&memory, &brain);

        assert!(answer.contains("degraded"));
        assert!(answer.contains("active swap"));
        assert!(answer.contains("1.0 GB in use"));
    }

    #[test]
    fn test_process_health_no_issues() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_process_health_answer(&brain);

        assert!(answer.contains("no obviously misbehaving"));
        assert!(answer.contains("ps -eo"));
    }

    #[test]
    fn test_process_health_with_issues() {
        let brain = BrainAnalysisData {
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "process-cpu".to_string(),
                    summary: "Process chrome consuming 85% CPU".to_string(),
                    details: "Single process using excessive CPU".to_string(),
                    evidence: "ps output shows high usage".to_string(),
                    severity: "warning".to_string(),
                    commands: vec!["ps aux".to_string()],
                    citations: vec![],
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 1,
        };

        let answer = compose_process_health_answer(&brain);

        assert!(answer.contains("1 process consuming"));
        assert!(answer.contains("chrome"));
    }

    #[test]
    fn test_network_health_normal() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_network_health_answer(&brain);

        assert!(answer.contains("all clear"));
        assert!(answer.contains("ip addr"));
        assert!(answer.contains("ping"));
    }

    #[test]
    fn test_network_health_with_issues() {
        let brain = BrainAnalysisData {
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "network-connectivity".to_string(),
                    summary: "Network connectivity issue detected".to_string(),
                    details: "Cannot reach default gateway".to_string(),
                    evidence: "ping failed".to_string(),
                    severity: "critical".to_string(),
                    commands: vec!["ip route".to_string()],
                    citations: vec![],
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 1,
            warning_count: 0,
        };

        let answer = compose_network_health_answer(&brain);

        assert!(answer.contains("degraded"));
        assert!(answer.contains("connectivity issue"));
    }
}

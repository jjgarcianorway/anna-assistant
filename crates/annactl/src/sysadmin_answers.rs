//! Sysadmin Answer Composer - Beta.263 + Beta.264 + Beta.268 + Beta.269
//!
//! Deterministic answer patterns for system troubleshooting.
//! Beta.263: services, disk, logs (diagnostic answers)
//! Beta.264: CPU, memory, processes, network (diagnostic answers)
//! Beta.268: network remediation actions
//! Beta.269: services, disk, logs, CPU, memory, processes remediation actions
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

/// Compose network priority mismatch remediation (Beta.268)
///
/// Provides actionable guidance when slow Ethernet is prioritized over fast WiFi.
/// Uses interface names and speeds from brain insights.
pub fn compose_network_priority_fix_answer(
    eth_name: &str,
    eth_speed: u32,
    wifi_name: &str,
    wifi_speed: u32,
) -> String {
    // Build summary
    let summary = "[SUMMARY]\nNetwork priority issue detected.".to_string();

    // Build details
    let details = format!(
        "\n\n[DETAILS]\n\
        Your system is currently using a slower Ethernet connection ({} at {} Mbps) \
        for routing instead of a faster WiFi connection ({} at {} Mbps).\n\
        \n\
        This typically happens when Ethernet is connected via a USB adapter or dock \
        while WiFi has better link quality. NetworkManager assigns priority based on \
        interface type by default, not speed.\n\
        \n\
        Recommended action: Disconnect the slower Ethernet interface and use WiFi, \
        or adjust route metrics to prefer the faster connection.",
        eth_name, eth_speed, wifi_name, wifi_speed
    );

    // Build commands
    let commands = format!(
        "\n\n[COMMANDS]\n\
        # Check current network configuration:\n\
        nmcli connection show\n\
        \n\
        # Disconnect slower Ethernet interface:\n\
        nmcli connection down {}\n\
        \n\
        # Verify WiFi is now the default route:\n\
        ip route\n\
        \n\
        # To permanently prefer WiFi, adjust route metrics:\n\
        # (Lower metric = higher priority)\n\
        nmcli connection modify {} ipv4.route-metric 100\n\
        nmcli connection modify {} ipv4.route-metric 200",
        eth_name, wifi_name, eth_name
    );

    format!("{}{}{}", summary, details, commands)
}

/// Compose network routing fix remediation (Beta.268)
///
/// Provides actionable guidance for duplicate or missing default routes.
/// Handles both duplicate route and missing route scenarios.
pub fn compose_network_route_fix_answer(
    issue_type: &str,
    interface_names: Vec<&str>,
) -> String {
    // Build summary
    let summary = if issue_type == "duplicate" {
        "[SUMMARY]\nDuplicate default routes detected.".to_string()
    } else {
        "[SUMMARY]\nRouting configuration issue detected.".to_string()
    };

    // Build details
    let details = if issue_type == "duplicate" {
        format!(
            "\n\n[DETAILS]\n\
            Multiple default routes are configured on interfaces: {}.\n\
            \n\
            This causes unpredictable routing behavior where network traffic may randomly \
            use different interfaces for outbound connections. This can result in:\n\
            • Connection timeouts and failures\n\
            • Inconsistent DNS resolution\n\
            • VPN and firewall issues\n\
            \n\
            Only one default route should be active. The system should use the fastest \
            or most reliable connection.",
            interface_names.join(", ")
        )
    } else {
        "\n\n[DETAILS]\n\
        Your system's routing table has a configuration issue that may affect connectivity.\n\
        \n\
        This typically indicates a missing default route or incorrect route priority. \
        Without a proper default route, outbound internet traffic cannot reach external networks.\n\
        \n\
        NetworkManager usually manages routes automatically, so this may indicate a service issue.".to_string()
    };

    // Build commands
    let commands = if issue_type == "duplicate" {
        format!(
            "\n\n[COMMANDS]\n\
            # View current routing table:\n\
            ip route\n\
            \n\
            # Check which interface should be preferred:\n\
            nmcli device status\n\
            \n\
            # Remove duplicate default route (replace <gateway> and <interface>):\n\
            sudo ip route del default via <gateway> dev <interface>\n\
            \n\
            # Restart NetworkManager to rebuild routes:\n\
            sudo systemctl restart NetworkManager\n\
            \n\
            # Verify single default route remains:\n\
            ip route | grep default"
        )
    } else {
        "\n\n[COMMANDS]\n\
        # View current routing table:\n\
        ip route\n\
        \n\
        # Check NetworkManager status:\n\
        sudo systemctl status NetworkManager\n\
        \n\
        # Restart NetworkManager to rebuild routes:\n\
        sudo systemctl restart NetworkManager\n\
        \n\
        # If issue persists, check connection configuration:\n\
        nmcli connection show\n\
        nmcli connection up <connection-name>".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose network quality fix remediation (Beta.268)
///
/// Provides actionable guidance for packet loss, latency, or interface errors.
/// Handles connectivity degradation issues.
pub fn compose_network_quality_fix_answer(
    issue_type: &str,
    metric_value: f64,
    interface_name: Option<&str>,
) -> String {
    // Build summary
    let summary = match issue_type {
        "packet_loss" => format!("[SUMMARY]\nHigh packet loss detected ({:.1}%).", metric_value),
        "latency" => format!("[SUMMARY]\nHigh network latency detected ({:.0} ms).", metric_value),
        "errors" => "[SUMMARY]\nNetwork interface errors detected.".to_string(),
        _ => "[SUMMARY]\nNetwork quality issue detected.".to_string(),
    };

    // Build details
    let details = match issue_type {
        "packet_loss" => format!(
            "\n\n[DETAILS]\n\
            Your network connection is experiencing {:.1}% packet loss.\n\
            \n\
            Packet loss above 5% indicates connectivity problems. Common causes:\n\
            • WiFi signal interference or weak signal strength\n\
            • Faulty network cable or connector\n\
            • Congested network or overloaded router\n\
            • Driver or hardware issues\n\
            \n\
            Packet loss degrades performance for real-time applications (video calls, gaming) \
            and can cause connection timeouts.",
            metric_value
        ),
        "latency" => format!(
            "\n\n[DETAILS]\n\
            Your network connection has high latency ({:.0} ms to gateway or internet).\n\
            \n\
            Normal latency should be under 50ms for local networks and under 100ms for most \
            internet connections. High latency causes:\n\
            • Slow page loads and sluggish response times\n\
            • Poor performance for interactive applications\n\
            • VPN and SSH session delays\n\
            \n\
            Common causes include WiFi distance, network congestion, or ISP routing issues.",
            metric_value
        ),
        "errors" => {
            let iface_detail = if let Some(iface) = interface_name {
                format!(" on interface {}", iface)
            } else {
                String::new()
            };
            format!(
                "\n\n[DETAILS]\n\
                Network interface errors detected{}.\n\
                \n\
                Interface errors (RX/TX errors) indicate hardware or driver problems:\n\
                • Faulty network cable or damaged connector\n\
                • Incompatible or buggy network driver\n\
                • Hardware failure (network card or port)\n\
                • Duplex mismatch (auto-negotiation issue)\n\
                \n\
                Error rates above 1% warrant investigation.",
                iface_detail
            )
        },
        _ => "\n\n[DETAILS]\nNetwork quality degradation detected.".to_string(),
    };

    // Build commands
    let commands = if issue_type == "packet_loss" {
        "\n\n[COMMANDS]\n\
        # Test packet loss to gateway:\n\
        ping -c 20 $(ip route | grep default | awk '{print $3}')\n\
        \n\
        # Test packet loss to internet:\n\
        ping -c 20 1.1.1.1\n\
        \n\
        # Check WiFi signal strength (if applicable):\n\
        nmcli device wifi\n\
        \n\
        # Check interface statistics:\n\
        ip -s link show\n\
        \n\
        # Restart NetworkManager:\n\
        sudo systemctl restart NetworkManager".to_string()
    } else if issue_type == "latency" {
        "\n\n[COMMANDS]\n\
        # Measure latency to gateway:\n\
        ping -c 10 $(ip route | grep default | awk '{print $3}')\n\
        \n\
        # Measure latency to internet:\n\
        ping -c 10 1.1.1.1\n\
        \n\
        # Trace route to identify slow hops:\n\
        traceroute -n 1.1.1.1\n\
        \n\
        # Check for interface congestion:\n\
        ip -s link show\n\
        \n\
        # Restart NetworkManager:\n\
        sudo systemctl restart NetworkManager".to_string()
    } else if let Some(iface) = interface_name {
        format!(
            "\n\n[COMMANDS]\n\
            # Check interface error statistics:\n\
            ip -s link show {}\n\
            \n\
            # Check interface speed and duplex:\n\
            ethtool {}\n\
            \n\
            # Force interface speed (example for 1000 Mbps full duplex):\n\
            sudo ethtool -s {} speed 1000 duplex full autoneg on\n\
            \n\
            # Restart NetworkManager:\n\
            sudo systemctl restart NetworkManager\n\
            \n\
            # Check kernel messages for hardware errors:\n\
            journalctl -k | grep -i {}",
            iface, iface, iface, iface
        )
    } else {
        "\n\n[COMMANDS]\n\
        # Check all interface statistics:\n\
        ip -s link show\n\
        \n\
        # Check for hardware errors in kernel log:\n\
        journalctl -k | grep -iE 'network|eth|wlan'\n\
        \n\
        # Restart NetworkManager:\n\
        sudo systemctl restart NetworkManager".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose systemd services fix remediation (Beta.269)
///
/// Provides actionable guidance when systemd services are failed or degraded.
/// Uses service names from brain insights.
pub fn compose_services_fix_answer(
    failed_services: Vec<&str>,
    is_critical: bool,
) -> String {
    let count = failed_services.len();

    // Build summary
    let summary = if is_critical {
        if count == 1 {
            "[SUMMARY]\nCritical: 1 systemd service has failed.".to_string()
        } else {
            format!("[SUMMARY]\nCritical: {} systemd services have failed.", count)
        }
    } else {
        if count == 1 {
            "[SUMMARY]\n1 systemd service is degraded.".to_string()
        } else {
            format!("[SUMMARY]\n{} systemd services are degraded.", count)
        }
    };

    // Build details
    let details = if is_critical {
        format!(
            "\n\n[DETAILS]\n\
            Failed systemd services prevent critical system functionality from working.\n\
            \n\
            Common causes:\n\
            • Configuration errors in service unit files\n\
            • Missing dependencies or files\n\
            • Permission issues\n\
            • Resource exhaustion (ports, memory, disk space)\n\
            \n\
            Inspect each service's logs to identify the root cause before attempting repairs."
        )
    } else {
        "\n\n[DETAILS]\n\
        Degraded services are running but not in their expected state.\n\
        \n\
        This typically indicates:\n\
        • Partial functionality or reduced performance\n\
        • Dependencies that couldn't be fully satisfied\n\
        • Non-critical errors during startup\n\
        \n\
        Check service status and logs to understand what functionality is affected.".to_string()
    };

    // Build commands
    let commands = if !failed_services.is_empty() {
        let first_service = failed_services[0];
        format!(
            "\n\n[COMMANDS]\n\
            # List all failed services:\n\
            systemctl --failed\n\
            \n\
            # Check specific service status:\n\
            systemctl status {}\n\
            \n\
            # View service logs (errors only):\n\
            journalctl -u {} -p err -n 50\n\
            \n\
            # View full service logs:\n\
            journalctl -u {} -n 100\n\
            \n\
            # After fixing the underlying issue, restart the service:\n\
            sudo systemctl restart {}",
            first_service, first_service, first_service, first_service
        )
    } else {
        "\n\n[COMMANDS]\n\
        # List all failed services:\n\
        systemctl --failed\n\
        \n\
        # Check specific service status:\n\
        systemctl status <service>\n\
        \n\
        # View service logs:\n\
        journalctl -u <service> -p err -n 50\n\
        \n\
        # Restart service after fixing issue:\n\
        sudo systemctl restart <service>".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose disk space fix remediation (Beta.269)
///
/// Provides actionable guidance when disk usage is high.
/// Uses mountpoint from brain insights.
pub fn compose_disk_fix_answer(
    mountpoint: &str,
    usage_percent: u32,
    is_critical: bool,
) -> String {
    // Build summary
    let summary = if is_critical {
        format!("[SUMMARY]\nCritical: Disk usage on {} is {}% (critical threshold exceeded).", mountpoint, usage_percent)
    } else {
        format!("[SUMMARY]\nWarning: Disk usage on {} is {}% (approaching capacity).", mountpoint, usage_percent)
    };

    // Build details
    let details = format!(
        "\n\n[DETAILS]\n\
        High disk usage on {} can cause:\n\
        • System instability and crashes\n\
        • Failed write operations and data loss\n\
        • Service failures (databases, logs, temporary files)\n\
        \n\
        Common culprits:\n\
        • Large log files in /var/log\n\
        • Package cache in /var/cache\n\
        • Old journal entries\n\
        • User data in /home\n\
        \n\
        IMPORTANT: Do not blindly delete files. Identify large directories first, \
        then carefully remove only unnecessary data.",
        mountpoint
    );

    // Build commands
    let commands = format!(
        "\n\n[COMMANDS]\n\
        # Check disk usage by filesystem:\n\
        df -h\n\
        \n\
        # Find largest directories on {}:\n\
        sudo du -h {} 2>/dev/null | sort -h | tail -20\n\
        \n\
        # Check log file sizes:\n\
        sudo du -h /var/log | sort -h | tail -10\n\
        \n\
        # Clean package cache (safe):\n\
        sudo pacman -Sc\n\
        \n\
        # Clean old journal entries (keeps last 7 days):\n\
        sudo journalctl --vacuum-time=7d\n\
        \n\
        # Check for failed journal entries:\n\
        journalctl -p err -n 50",
        mountpoint, mountpoint
    );

    format!("{}{}{}", summary, details, commands)
}

/// Compose log issues fix remediation (Beta.269)
///
/// Provides actionable guidance when critical log issues are detected.
pub fn compose_logs_fix_answer(
    error_count: usize,
    is_disk_space_issue: bool,
) -> String {
    // Build summary
    let summary = if error_count == 1 {
        "[SUMMARY]\nCritical log issue detected.".to_string()
    } else {
        format!("[SUMMARY]\n{} critical log issues detected.", error_count)
    };

    // Build details
    let details = if is_disk_space_issue {
        "\n\n[DETAILS]\n\
        System logs are consuming excessive disk space.\n\
        \n\
        This can happen due to:\n\
        • Services logging errors repeatedly\n\
        • Very verbose logging configuration\n\
        • Failed log rotation\n\
        \n\
        Action: Review recent errors to identify root cause, then clean old logs.".to_string()
    } else {
        "\n\n[DETAILS]\n\
        Critical errors are appearing in system logs.\n\
        \n\
        These errors indicate:\n\
        • Service failures or crashes\n\
        • Hardware issues (disk, memory, network)\n\
        • Configuration problems\n\
        \n\
        Action: Inspect the error messages to identify which component is failing, \
        then address the underlying issue.".to_string()
    };

    // Build commands
    let commands = if is_disk_space_issue {
        "\n\n[COMMANDS]\n\
        # Check journal disk usage:\n\
        journalctl --disk-usage\n\
        \n\
        # View recent errors:\n\
        journalctl -p err -n 50\n\
        \n\
        # Clean old journal entries (keeps last 7 days):\n\
        sudo journalctl --vacuum-time=7d\n\
        \n\
        # Or limit by size (keeps last 500MB):\n\
        sudo journalctl --vacuum-size=500M\n\
        \n\
        # Rotate journal files:\n\
        sudo journalctl --rotate".to_string()
    } else {
        "\n\n[COMMANDS]\n\
        # View recent errors:\n\
        journalctl -p err -n 50\n\
        \n\
        # View recent warnings:\n\
        journalctl -p warning -n 100\n\
        \n\
        # View errors from specific service:\n\
        journalctl -u <service> -p err -n 50\n\
        \n\
        # View errors since last boot:\n\
        journalctl -p err -b\n\
        \n\
        # Check kernel errors:\n\
        journalctl -k -p err".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose CPU usage fix remediation (Beta.269)
///
/// Provides actionable guidance when CPU usage is high.
pub fn compose_cpu_fix_answer(
    usage_percent: f64,
    is_sustained: bool,
    top_process: Option<&str>,
) -> String {
    // Build summary
    let summary = if is_sustained {
        format!("[SUMMARY]\nCPU usage sustained at {:.1}% (critical load).", usage_percent)
    } else {
        format!("[SUMMARY]\nCPU usage elevated at {:.1}%.", usage_percent)
    };

    // Build details
    let details = if is_sustained {
        "\n\n[DETAILS]\n\
        Sustained high CPU usage degrades system responsiveness and can indicate:\n\
        • Runaway processes or infinite loops\n\
        • Insufficient resources for workload\n\
        • Cryptocurrency miners or malware\n\
        • Misconfigured services\n\
        \n\
        Identify the top CPU consumers and determine if their usage is expected.".to_string()
    } else {
        "\n\n[DETAILS]\n\
        Elevated CPU usage is normal during intensive tasks, but prolonged high usage may indicate:\n\
        • Background tasks running unexpectedly\n\
        • System maintenance (indexing, backups)\n\
        • Compilation or data processing\n\
        \n\
        Monitor CPU usage to see if it returns to normal or requires intervention.".to_string()
    };

    // Build commands
    let commands = if let Some(process) = top_process {
        format!(
            "\n\n[COMMANDS]\n\
            # Monitor CPU usage in real-time:\n\
            top\n\
            # or for better interface:\n\
            htop\n\
            \n\
            # List processes by CPU usage:\n\
            ps aux --sort=-%cpu | head -20\n\
            \n\
            # Check if top process is a service:\n\
            systemctl status {}\n\
            \n\
            # View process details:\n\
            ps -p $(pgrep {}) -o pid,ppid,cmd,%cpu,%mem",
            process, process
        )
    } else {
        "\n\n[COMMANDS]\n\
        # Monitor CPU usage in real-time:\n\
        top\n\
        # or:\n\
        htop\n\
        \n\
        # List processes by CPU usage:\n\
        ps aux --sort=-%cpu | head -20\n\
        \n\
        # Check system load:\n\
        uptime\n\
        \n\
        # View CPU info:\n\
        lscpu".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose memory usage fix remediation (Beta.269)
///
/// Provides actionable guidance when memory usage is high.
pub fn compose_memory_fix_answer(
    mem_percent: f64,
    swap_percent: Option<f64>,
    is_critical: bool,
) -> String {
    // Build summary
    let summary = if is_critical {
        if let Some(swap) = swap_percent {
            format!("[SUMMARY]\nCritical: Memory at {:.1}%, swap at {:.1}% (system under pressure).", mem_percent, swap)
        } else {
            format!("[SUMMARY]\nCritical: Memory usage at {:.1}% (no swap configured).", mem_percent)
        }
    } else {
        format!("[SUMMARY]\nWarning: Memory usage at {:.1}%.", mem_percent)
    };

    // Build details
    let details = if let Some(swap) = swap_percent {
        if swap > 50.0 {
            "\n\n[DETAILS]\n\
            High memory usage combined with heavy swap usage indicates memory pressure.\n\
            \n\
            When swap is heavily used:\n\
            • System becomes very slow (disk is much slower than RAM)\n\
            • Applications may freeze or crash\n\
            • Risk of OOM (Out Of Memory) killer terminating processes\n\
            \n\
            Action: Identify memory-hungry processes and consider closing unnecessary applications \
            or adding more RAM.".to_string()
        } else {
            "\n\n[DETAILS]\n\
            High memory usage but swap is not heavily used yet.\n\
            \n\
            This is often normal for systems that cache aggressively, but monitor for:\n\
            • Gradual memory leaks in long-running processes\n\
            • Unexpected memory consumers\n\
            • Swap usage increasing over time\n\
            \n\
            Linux uses available RAM for filesystem cache, which is released when needed.".to_string()
        }
    } else {
        "\n\n[DETAILS]\n\
        High memory usage without swap configured.\n\
        \n\
        Without swap, the system has no buffer when RAM fills up:\n\
        • OOM killer will terminate processes to free memory\n\
        • No warning before critical failures\n\
        • Important processes may be killed\n\
        \n\
        Consider configuring swap as a safety buffer, especially on systems with limited RAM.".to_string()
    };

    // Build commands
    let commands = if swap_percent.is_some() {
        "\n\n[COMMANDS]\n\
        # Check memory and swap usage:\n\
        free -h\n\
        \n\
        # Monitor memory in real-time:\n\
        top\n\
        # or:\n\
        htop\n\
        \n\
        # List processes by memory usage:\n\
        ps aux --sort=-%mem | head -20\n\
        \n\
        # Check swap configuration:\n\
        swapon --show\n\
        \n\
        # View memory statistics:\n\
        vmstat 1 5".to_string()
    } else {
        "\n\n[COMMANDS]\n\
        # Check memory usage:\n\
        free -h\n\
        \n\
        # List processes by memory usage:\n\
        ps aux --sort=-%mem | head -20\n\
        \n\
        # Monitor memory in real-time:\n\
        top\n\
        \n\
        # Check if swap is configured:\n\
        swapon --show\n\
        \n\
        # Consider creating swap file (requires root):\n\
        # sudo fallocate -l 2G /swapfile\n\
        # sudo chmod 600 /swapfile\n\
        # sudo mkswap /swapfile\n\
        # sudo swapon /swapfile".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Compose process fix remediation (Beta.269)
///
/// Provides actionable guidance when runaway processes are detected.
pub fn compose_process_fix_answer(
    process_name: Option<&str>,
    issue_type: &str, // "cpu" or "memory"
) -> String {
    // Build summary
    let summary = if let Some(proc) = process_name {
        if issue_type == "cpu" {
            format!("[SUMMARY]\nRunaway process detected: {} consuming excessive CPU.", proc)
        } else {
            format!("[SUMMARY]\nMemory-heavy process detected: {} consuming excessive memory.", proc)
        }
    } else {
        "[SUMMARY]\nRunaway process detected consuming excessive resources.".to_string()
    };

    // Build details
    let details = "\n\n[DETAILS]\n\
    Runaway processes can degrade system performance or cause instability.\n\
    \n\
    Before terminating a process:\n\
    • Identify what the process does (is it a critical service?)\n\
    • Check if it's temporary (compilation, backup, indexing)\n\
    • Look for error messages in logs\n\
    • Consider if the process is legitimately busy\n\
    \n\
    WARNING: Killing processes can cause data loss or service disruption. \
    Only terminate processes you understand and can safely restart.".to_string();

    // Build commands
    let commands = if let Some(proc) = process_name {
        format!(
            "\n\n[COMMANDS]\n\
            # Find process ID:\n\
            pgrep {}\n\
            \n\
            # View process details:\n\
            ps aux | grep {}\n\
            \n\
            # Check if it's a systemd service:\n\
            systemctl status {}\n\
            \n\
            # Monitor the process:\n\
            top -p $(pgrep {})\n\
            \n\
            # If it's a service, restart via systemd (preferred):\n\
            sudo systemctl restart {}\n\
            \n\
            # As a last resort, terminate the process (CAUTION):\n\
            # sudo kill <pid>\n\
            # or force kill:\n\
            # sudo kill -9 <pid>",
            proc, proc, proc, proc, proc
        )
    } else {
        "\n\n[COMMANDS]\n\
        # List processes by CPU usage:\n\
        ps aux --sort=-%cpu | head -20\n\
        \n\
        # List processes by memory usage:\n\
        ps aux --sort=-%mem | head -20\n\
        \n\
        # Monitor processes in real-time:\n\
        top\n\
        \n\
        # Check if runaway process is a service:\n\
        systemctl status <service>\n\
        \n\
        # Terminate process (CAUTION - only if you know what it is):\n\
        # sudo kill <pid>".to_string()
    };

    format!("{}{}{}", summary, details, commands)
}

/// Route network insights to remediation composers (Beta.268)
///
/// Dispatches brain analysis network insights to appropriate remediation composers.
/// This is deterministic routing based on insight rule_id from Beta.267 brain analysis.
///
/// Routes:
/// - network_priority_mismatch → compose_network_priority_fix_answer()
/// - duplicate_default_routes → compose_network_route_fix_answer("duplicate")
/// - high_packet_loss → compose_network_quality_fix_answer("packet_loss")
/// - high_latency → compose_network_quality_fix_answer("latency")
///
/// Returns None if no network remediation is applicable.
pub fn route_network_remediation(brain: &BrainAnalysisData) -> Option<String> {
    // Find first network insight requiring remediation
    for insight in &brain.insights {
        match insight.rule_id.as_str() {
            "network_priority_mismatch" => {
                // Extract interface names and speeds from evidence
                // Evidence format: "Ethernet eth0 (100 Mbps) has default route, WiFi wlan0 (866 Mbps) does not"
                if let Some((eth_name, eth_speed, wifi_name, wifi_speed)) = parse_priority_mismatch_evidence(&insight.evidence) {
                    return Some(compose_network_priority_fix_answer(
                        eth_name,
                        eth_speed,
                        wifi_name,
                        wifi_speed,
                    ));
                }
            }
            "duplicate_default_routes" => {
                // Extract interface names from evidence or details
                let interface_names = extract_interface_names_from_insight(insight);
                return Some(compose_network_route_fix_answer(
                    "duplicate",
                    interface_names,
                ));
            }
            "high_packet_loss" => {
                // Extract packet loss percentage from summary
                if let Some(percentage) = extract_percentage_from_summary(&insight.summary) {
                    return Some(compose_network_quality_fix_answer(
                        "packet_loss",
                        percentage,
                        None, // Interface name not currently tracked
                    ));
                }
            }
            "high_latency" => {
                // Extract latency value from summary
                if let Some(latency_ms) = extract_latency_from_summary(&insight.summary) {
                    return Some(compose_network_quality_fix_answer(
                        "latency",
                        latency_ms,
                        None, // Interface name not currently tracked
                    ));
                }
            }
            _ => {
                // Not a network remediation insight
                continue;
            }
        }
    }

    None // No remediable network insights found
}

/// Parse priority mismatch evidence to extract interface details
///
/// Expected format: "Ethernet eth0 (100 Mbps) has default route, WiFi wlan0 (866 Mbps) does not"
fn parse_priority_mismatch_evidence(evidence: &str) -> Option<(&str, u32, &str, u32)> {
    // Extract ethernet interface name and speed
    let eth_start = evidence.find("Ethernet ")?;
    let eth_end = evidence[eth_start + 9..].find(" (")?;
    let eth_name = &evidence[eth_start + 9..eth_start + 9 + eth_end];

    let eth_speed_start = eth_start + 9 + eth_end + 2;
    let eth_speed_end = evidence[eth_speed_start..].find(" Mbps")?;
    let eth_speed: u32 = evidence[eth_speed_start..eth_speed_start + eth_speed_end].parse().ok()?;

    // Extract wifi interface name and speed
    let wifi_start = evidence.find("WiFi ")?;
    let wifi_end = evidence[wifi_start + 5..].find(" (")?;
    let wifi_name = &evidence[wifi_start + 5..wifi_start + 5 + wifi_end];

    let wifi_speed_start = wifi_start + 5 + wifi_end + 2;
    let wifi_speed_end = evidence[wifi_speed_start..].find(" Mbps")?;
    let wifi_speed: u32 = evidence[wifi_speed_start..wifi_speed_start + wifi_speed_end].parse().ok()?;

    Some((eth_name, eth_speed, wifi_name, wifi_speed))
}

/// Extract interface names from duplicate route insight
fn extract_interface_names_from_insight(insight: &DiagnosticInsightData) -> Vec<&str> {
    // Try to parse from details which contains interface list
    // Format in check_network_issues: "interfaces: eth0, wlan0"
    if let Some(interfaces_pos) = insight.details.find("interfaces: ") {
        let start = interfaces_pos + 12;
        if let Some(end_pos) = insight.details[start..].find('.') {
            let interfaces_str = &insight.details[start..start + end_pos];
            return interfaces_str.split(", ").collect();
        }
    }

    // Fallback: empty list
    vec![]
}

/// Extract percentage value from summary text
fn extract_percentage_from_summary(summary: &str) -> Option<f64> {
    // Look for pattern like "35%" or "35.5%"
    let percent_idx = summary.find('%')?;
    let mut start = percent_idx;

    // Walk backwards to find start of number
    while start > 0 && (summary.as_bytes()[start - 1].is_ascii_digit() || summary.as_bytes()[start - 1] == b'.') {
        start -= 1;
    }

    summary[start..percent_idx].parse().ok()
}

/// Extract latency milliseconds from summary text
fn extract_latency_from_summary(summary: &str) -> Option<f64> {
    // Look for pattern like "350ms" or "350 ms"
    if let Some(ms_idx) = summary.find("ms") {
        let mut start = ms_idx;

        // Skip space if present
        if start > 0 && summary.as_bytes()[start - 1] == b' ' {
            start -= 1;
        }

        // Walk backwards to find start of number
        while start > 0 && (summary.as_bytes()[start - 1].is_ascii_digit() || summary.as_bytes()[start - 1] == b'.') {
            start -= 1;
        }

        return summary[start..ms_idx].trim().parse().ok();
    }

    None
}

/// Route all system insights to remediation composers (Beta.269)
///
/// Extended routing that covers all core system domains:
/// - Services (failed/degraded)
/// - Disk space (critical/warning)
/// - Logs (critical issues)
/// - CPU (overload/high load)
/// - Memory (pressure critical/warning)
/// - Processes (runaway)
/// - Network (from Beta.268)
///
/// Returns the most critical remediation answer, or None if no remediations apply.
pub fn route_system_remediation(brain: &BrainAnalysisData) -> Option<String> {
    // Priority order: services > disk > memory > CPU > logs > processes > network
    // This ensures critical system failures are addressed first

    for insight in &brain.insights {
        match insight.rule_id.as_str() {
            // Services (highest priority)
            "failed_services" => {
                // Extract service names from details
                let services = extract_service_names_from_insight(insight);
                return Some(compose_services_fix_answer(services, true));
            }
            "degraded_services" => {
                let services = extract_service_names_from_insight(insight);
                return Some(compose_services_fix_answer(services, false));
            }

            // Disk space (second priority)
            "disk_space_critical" => {
                if let Some((mountpoint, percent)) = extract_disk_info_from_insight(insight) {
                    return Some(compose_disk_fix_answer(mountpoint, percent, true));
                }
            }
            "disk_space_warning" => {
                if let Some((mountpoint, percent)) = extract_disk_info_from_insight(insight) {
                    return Some(compose_disk_fix_answer(mountpoint, percent, false));
                }
            }

            // Memory pressure (third priority)
            "memory_pressure_critical" | "memory_pressure_warning" => {
                let is_critical = insight.rule_id == "memory_pressure_critical";
                // Extract memory percentage from summary
                if let Some(mem_percent) = extract_percentage_from_summary(&insight.summary) {
                    // Try to extract swap info from details
                    let swap_percent = extract_swap_percent_from_details(&insight.details);
                    return Some(compose_memory_fix_answer(mem_percent, swap_percent, is_critical));
                }
            }

            // CPU load (fourth priority)
            "cpu_overload_critical" => {
                if let Some(cpu_percent) = extract_percentage_from_summary(&insight.summary) {
                    let top_process = extract_process_name_from_details(&insight.details);
                    return Some(compose_cpu_fix_answer(cpu_percent, true, top_process));
                }
            }
            "cpu_high_load" => {
                if let Some(cpu_percent) = extract_percentage_from_summary(&insight.summary) {
                    let top_process = extract_process_name_from_details(&insight.details);
                    return Some(compose_cpu_fix_answer(cpu_percent, false, top_process));
                }
            }

            // Logs (fifth priority)
            "critical_log_issues" => {
                // Count can be extracted from summary or default to 1
                let count = extract_count_from_summary(&insight.summary).unwrap_or(1);
                // Check if it's a disk space issue
                let is_disk_issue = insight.details.contains("disk space") || insight.details.contains("consuming");
                return Some(compose_logs_fix_answer(count, is_disk_issue));
            }

            // Network (from Beta.268 - sixth priority)
            "network_priority_mismatch" => {
                if let Some((eth_name, eth_speed, wifi_name, wifi_speed)) = parse_priority_mismatch_evidence(&insight.evidence) {
                    return Some(compose_network_priority_fix_answer(eth_name, eth_speed, wifi_name, wifi_speed));
                }
            }
            "duplicate_default_routes" => {
                let interface_names = extract_interface_names_from_insight(insight);
                return Some(compose_network_route_fix_answer("duplicate", interface_names));
            }
            "high_packet_loss" | "elevated_packet_loss" => {
                if let Some(percentage) = extract_percentage_from_summary(&insight.summary) {
                    return Some(compose_network_quality_fix_answer("packet_loss", percentage, None));
                }
            }
            "high_latency" | "critical_latency" => {
                if let Some(latency_ms) = extract_latency_from_summary(&insight.summary) {
                    return Some(compose_network_quality_fix_answer("latency", latency_ms, None));
                }
            }

            _ => continue,
        }
    }

    None // No remediable insights found
}

/// Extract service names from failed/degraded services insight
fn extract_service_names_from_insight(insight: &DiagnosticInsightData) -> Vec<&str> {
    // Services are typically listed in details
    // Look for service.service pattern
    let mut services = Vec::new();

    for line in insight.details.lines() {
        if line.contains(".service") {
            // Extract service name from lines like "• sshd.service - OpenSSH Daemon"
            // Find the .service suffix
            if let Some(service_end_pos) = line.find(".service") {
                // Service name ends at .service, walk backwards to find start
                let before_service = &line[..service_end_pos];
                // Find last whitespace before service name
                if let Some(ws_pos) = before_service.rfind(|c: char| c.is_whitespace()) {
                    let service_name = &line[ws_pos + 1..service_end_pos + 8]; // Include ".service"
                    services.push(service_name.trim());
                } else {
                    // No whitespace, take from beginning
                    let service_name = &line[..service_end_pos + 8];
                    services.push(service_name.trim());
                }
            }
        }
    }

    // Fallback: empty list means generic commands
    services
}

/// Extract mountpoint and usage percentage from disk insight
fn extract_disk_info_from_insight(insight: &DiagnosticInsightData) -> Option<(&str, u32)> {
    // Summary format: "Disk usage on / is 92%"
    // Extract mountpoint
    let mountpoint_start = insight.summary.find("on ")? + 3;
    let mountpoint_end = insight.summary[mountpoint_start..].find(" is")?;
    let mountpoint = &insight.summary[mountpoint_start..mountpoint_start + mountpoint_end];

    // Extract percentage
    let percent = extract_percentage_from_summary(&insight.summary)? as u32;

    Some((mountpoint, percent))
}

/// Extract swap percentage from details
fn extract_swap_percent_from_details(details: &str) -> Option<f64> {
    // Look for "swap" followed by percentage
    if let Some(swap_pos) = details.find("swap") {
        let after_swap = &details[swap_pos..];
        if let Some(percent_pos) = after_swap.find('%') {
            // Walk backwards from % to find number
            let mut start = percent_pos;
            while start > 0 && (after_swap.as_bytes()[start - 1].is_ascii_digit() || after_swap.as_bytes()[start - 1] == b'.') {
                start -= 1;
            }
            return after_swap[start..percent_pos].parse().ok();
        }
    }
    None
}

/// Extract process name from details
fn extract_process_name_from_details(details: &str) -> Option<&str> {
    // Look for "process: <name>" or similar patterns
    if let Some(proc_pos) = details.find("process: ") {
        let start = proc_pos + 9;
        if let Some(end) = details[start..].find(|c: char| c.is_whitespace() || c == ')' || c == ',') {
            return Some(&details[start..start + end]);
        }
    }
    None
}

/// Extract count from summary (e.g., "3 critical log issues")
fn extract_count_from_summary(summary: &str) -> Option<usize> {
    // Look for number at start or after space
    for word in summary.split_whitespace() {
        if let Ok(num) = word.parse::<usize>() {
            return Some(num);
        }
    }
    None
}

/// Beta.272: Compose remediation answer for top proactive issue
///
/// Maps proactive engine root_cause to the appropriate remediation composer.
/// Returns None if no remediation mapping exists yet.
pub fn compose_top_proactive_remediation(
    issue: &anna_common::ipc::ProactiveIssueSummaryData,
    _brain: &anna_common::ipc::BrainAnalysisData,
) -> Option<String> {
    // Map root_cause to remediation domain
    match issue.root_cause.as_str() {
        // Network root causes
        "network_routing_conflict" | "network_priority_mismatch" => {
            // Generic network routing remediation
            Some(format!(
                "[SUMMARY]\n\
                Network routing issue detected: {}\n\n\
                [DETAILS]\n\
                The proactive engine detected a network routing or priority problem.\n\
                This typically indicates:\n\
                - Slow interface has default route while faster interface doesn't\n\
                - Multiple default routes causing routing conflicts\n\
                - Interface priority misconfiguration\n\n\
                [COMMANDS]\n\
                # Check current routing table\n\
                $ ip route show\n\n\
                # Check interface speeds and status\n\
                $ ip link show\n\
                $ ethtool <interface> | grep Speed\n\n\
                # Review NetworkManager connections\n\
                $ nmcli connection show\n\
                $ nmcli connection modify <connection> ipv4.route-metric <value>\n\n\
                # Or edit netctl profile priority\n\
                $ sudo nano /etc/netctl/<profile>\n\
                # Set Priority=10 (higher = preferred)",
                issue.summary
            ))
        }
        "network_quality_degradation" => {
            Some(format!(
                "[SUMMARY]\n\
                Network quality degradation detected: {}\n\n\
                [DETAILS]\n\
                The proactive engine detected packet loss or high latency.\n\
                Common causes:\n\
                - Physical cable issues\n\
                - WiFi interference or weak signal\n\
                - Router/switch problems\n\
                - Network congestion\n\n\
                [COMMANDS]\n\
                # Test network quality\n\
                $ ping -c 20 8.8.8.8\n\
                $ mtr --report 8.8.8.8\n\n\
                # Check interface statistics\n\
                $ ip -s link\n\
                $ ethtool -S <interface>\n\n\
                # WiFi signal strength (if applicable)\n\
                $ iwconfig\n\
                $ nmcli dev wifi",
                issue.summary
            ))
        }

        // Disk root causes
        "disk_pressure" | "disk_log_growth" => {
            // Use generic disk remediation
            Some(compose_disk_fix_answer("/", 85, true))
        }

        // Service root causes
        "service_flapping" | "service_under_load" | "service_config_error" => {
            // Generic service remediation
            Some(format!(
                "[SUMMARY]\n\
                Service issue detected: {}\n\n\
                [DETAILS]\n\
                The proactive engine detected a service problem:\n\
                - Service flapping: Repeated restarts indicate instability\n\
                - Service under load: Resource exhaustion or performance issues\n\
                - Service config error: Misconfiguration preventing proper operation\n\n\
                [COMMANDS]\n\
                # Check failed and restarting services\n\
                $ systemctl --failed\n\
                $ systemctl list-units --state=failed,restarting\n\n\
                # View service logs\n\
                $ journalctl -u <service> -n 50\n\
                $ journalctl -u <service> --since \"1 hour ago\"\n\n\
                # Check service status and resource usage\n\
                $ systemctl status <service>\n\
                $ systemd-cgtop",
                issue.summary
            ))
        }

        // Memory root causes
        "memory_pressure" => {
            Some(compose_memory_fix_answer(85.0, Some(50.0), true))
        }

        // CPU root causes
        "cpu_overload" => {
            Some(compose_cpu_fix_answer(90.0, true, Some("unknown")))
        }

        // Device root causes
        "kernel_regression" | "device_hotplug" => {
            Some(format!(
                "[SUMMARY]\n\
                System hardware or kernel issue detected: {}\n\n\
                [DETAILS]\n\
                The proactive engine detected a hardware or kernel-level problem.\n\
                This may indicate:\n\
                - Kernel regression after update\n\
                - Device hotplug issues (USB, power management)\n\
                - Driver problems\n\n\
                [COMMANDS]\n\
                # Check kernel messages\n\
                $ dmesg | tail -50\n\
                $ journalctl -k -n 50\n\n\
                # Check for hardware errors\n\
                $ lspci -v\n\
                $ lsusb -v\n\n\
                # Review recent package updates\n\
                $ pacman -Qi linux | grep \"Install Date\"\n\
                $ grep -i \"upgraded.*linux\" /var/log/pacman.log | tail -5",
                issue.summary
            ))
        }

        // Unknown or unmapped root cause
        _ => {
            // Return generic proactive issue answer
            Some(format!(
                "[SUMMARY]\n\
                Correlated issue detected: {}\n\n\
                [DETAILS]\n\
                The proactive engine detected a pattern (root cause: {}) but specific \
                remediation guidance is not yet available for this issue type.\n\n\
                Confidence: {:.0}%\n\
                First seen: {}\n\
                Last seen: {}\n\n\
                [COMMANDS]\n\
                # Check overall system health\n\
                $ annactl \"check my system health\"\n\
                $ annactl status\n\n\
                # Review system logs\n\
                $ journalctl -xe -n 50",
                issue.summary,
                issue.root_cause,
                issue.confidence * 100.0,
                issue.first_seen,
                issue.last_seen
            ))
        }
    }
}

/// Beta.278: Compose full sysadmin report answer
///
/// Provides a comprehensive system briefing combining:
/// - Health summary and diagnostic insights
/// - Daily snapshot (session deltas, kernel, packages)
/// - Proactive correlated issues
/// - Key domain highlights (services, disk, network, resources)
///
/// This is the "sysadmin start-of-shift briefing" - concise but complete.
/// Target: 20-40 lines max.
pub fn compose_sysadmin_report_answer(
    brain: &BrainAnalysisData,
    daily_snapshot_text: Option<&str>,
    proactive_issues: &[anna_common::ipc::ProactiveIssueSummaryData],
    proactive_health_score: u8,
) -> String {
    let mut report = String::new();

    // ========================================================================
    // [SUMMARY] - One-liner overall status
    // ========================================================================
    report.push_str("[SUMMARY]\n");

    let overall_status = if brain.critical_count > 0 {
        "degraded"
    } else if brain.warning_count > 0 {
        "stable with warnings"
    } else {
        "all clear"
    };

    if brain.critical_count > 0 || brain.warning_count > 0 {
        report.push_str(&format!(
            "Overall status: {}, {} critical and {} warning level issues detected.\n",
            overall_status, brain.critical_count, brain.warning_count
        ));
    } else {
        report.push_str("Overall status: all clear, no critical issues detected.\n");
    }

    // ========================================================================
    // [HEALTH] - Diagnostic engine summary
    // ========================================================================
    report.push_str("\n[HEALTH]\n");

    if brain.insights.is_empty() {
        report.push_str("All diagnostic checks passed.\n");
    } else {
        // Show top 3 insights, prioritizing critical first
        let mut insights_sorted = brain.insights.clone();
        insights_sorted.sort_by(|a, b| {
            let a_priority = match a.severity.as_str() {
                "critical" => 0,
                "warning" => 1,
                _ => 2,
            };
            let b_priority = match b.severity.as_str() {
                "critical" => 0,
                "warning" => 1,
                _ => 2,
            };
            a_priority.cmp(&b_priority)
        });

        for insight in insights_sorted.iter().take(3) {
            let marker = match insight.severity.as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };
            report.push_str(&format!("{} {}\n", marker, insight.summary));
        }

        if brain.insights.len() > 3 {
            report.push_str(&format!("... and {} more issues\n", brain.insights.len() - 3));
        }
    }

    // ========================================================================
    // [SESSION] - Daily snapshot if available
    // ========================================================================
    if let Some(snapshot) = daily_snapshot_text {
        report.push_str("\n[SESSION]\n");
        // Extract just the key lines (kernel, packages, reboots)
        // The snapshot is already formatted, just include it directly
        let snapshot_lines: Vec<&str> = snapshot.lines().collect();
        for line in snapshot_lines.iter().take(5) {
            if !line.is_empty() && !line.starts_with('[') {
                report.push_str(line);
                report.push('\n');
            }
        }
    }

    // ========================================================================
    // [PROACTIVE] - Top correlated issues
    // ========================================================================
    report.push_str("\n[PROACTIVE]\n");

    if proactive_issues.is_empty() {
        report.push_str("No correlated issues detected in the last 24 hours.\n");
        report.push_str(&format!("Health score: {}/100\n", proactive_health_score));
    } else {
        report.push_str(&format!("Health score: {}/100\n", proactive_health_score));

        // Show top 3 proactive issues
        for issue in proactive_issues.iter().take(3) {
            let marker = match issue.severity.as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                "info" => "ℹ",
                _ => "~",
            };
            report.push_str(&format!(
                "{} {} (confidence: {:.0}%)\n",
                marker, issue.summary, issue.confidence * 100.0
            ));
        }

        if proactive_issues.len() > 3 {
            report.push_str(&format!("... and {} more correlated issues\n", proactive_issues.len() - 3));
        }
    }

    // ========================================================================
    // [KEY DOMAINS] - Highlights from sysadmin domains
    // ========================================================================
    report.push_str("\n[KEY DOMAINS]\n");

    let mut domain_highlights = Vec::new();

    // Services: extract from insights
    let service_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.rule_id.contains("service") || i.summary.to_lowercase().contains("service"))
        .collect();
    if !service_insights.is_empty() {
        let failed_count = service_insights.len();
        if failed_count == 1 {
            domain_highlights.push(format!("Services: 1 failed service detected"));
        } else {
            domain_highlights.push(format!("Services: {} issues detected", failed_count));
        }
    }

    // Disk: extract from insights
    let disk_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.rule_id.contains("disk") || i.summary.to_lowercase().contains("disk") || i.summary.to_lowercase().contains("partition"))
        .collect();
    if !disk_insights.is_empty() {
        for insight in disk_insights.iter().take(1) {
            domain_highlights.push(format!("Disk: {}", insight.summary.to_lowercase()));
        }
    }

    // Network: extract from insights
    let network_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.rule_id.contains("network") || i.summary.to_lowercase().contains("network") || i.summary.to_lowercase().contains("connectivity"))
        .collect();
    if !network_insights.is_empty() {
        for insight in network_insights.iter().take(1) {
            domain_highlights.push(format!("Network: {}", insight.summary.to_lowercase()));
        }
    }

    // CPU/Memory: extract from insights
    let resource_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.rule_id.contains("cpu") || i.rule_id.contains("memory") ||
                   i.summary.to_lowercase().contains("cpu") || i.summary.to_lowercase().contains("memory"))
        .collect();
    if !resource_insights.is_empty() {
        for insight in resource_insights.iter().take(1) {
            domain_highlights.push(format!("Resources: {}", insight.summary.to_lowercase()));
        }
    }

    // If no specific domain issues, say so
    if domain_highlights.is_empty() {
        report.push_str("All key domains (services, disk, network, resources) are nominal.\n");
    } else {
        for highlight in domain_highlights.iter().take(5) {
            report.push_str(&format!("• {}\n", highlight));
        }
    }

    // ========================================================================
    // [COMMANDS] - Next actions
    // ========================================================================
    report.push_str("\n[COMMANDS]\n");

    if brain.critical_count > 0 {
        report.push_str("# For detailed health analysis\n");
        report.push_str("$ annactl \"check my system health\"\n\n");
        report.push_str("# To see remediation for top issue\n");
        report.push_str("$ annactl \"what should I fix first\"\n");
    } else if brain.warning_count > 0 {
        report.push_str("# For detailed health analysis\n");
        report.push_str("$ annactl \"check my system health\"\n\n");
        report.push_str("# To focus on specific domains\n");
        report.push_str("$ annactl \"check my network\"\n");
        report.push_str("$ annactl \"check my disk\"\n");
    } else {
        report.push_str("# System is healthy - run specific checks as needed\n");
        report.push_str("$ annactl \"check my network\"\n");
        report.push_str("$ annactl \"check my disk\"\n");
        report.push_str("$ annactl status\n");
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::systemd_health::FailedUnit;

    #[test]
    fn test_service_health_no_failures() {
        let brain = BrainAnalysisData {
            insights: vec![],
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            proactive_health_score: 100,
        };

        let answer = compose_process_health_answer(&brain);

        assert!(answer.contains("1 process consuming"));
        assert!(answer.contains("chrome"));
    }

    #[test]
    fn test_network_health_normal() {
        let brain = BrainAnalysisData {
            insights: vec![],
            proactive_issues: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_health_score: 100,
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
            proactive_issues: vec![],
            proactive_health_score: 100,
        };

        let answer = compose_network_health_answer(&brain);

        assert!(answer.contains("degraded"));
        assert!(answer.contains("connectivity issue"));
    }
}

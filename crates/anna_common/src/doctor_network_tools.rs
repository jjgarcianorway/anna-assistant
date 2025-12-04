//! Doctor Network Evidence Tools v0.0.49
//!
//! Specialized network evidence collection for NetworkingDoctor.
//! These tools return structured data matching doctor_lifecycle types.

use crate::doctor_lifecycle::{
    DnsSummary, NetInterfaceSummary, NetworkErrorsSummary, NmSummary, RouteSummary, WifiSummary,
};
use crate::tools::ToolResult;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Execute net_interfaces_summary tool
pub fn execute_net_interfaces_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let mut interfaces = Vec::new();

    // Read /sys/class/net for interface list
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue; // Skip loopback
            }

            let base = format!("/sys/class/net/{}", name);

            // Read operstate
            let operstate = fs::read_to_string(format!("{}/operstate", base))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            // Read carrier (1 = cable connected)
            let carrier = fs::read_to_string(format!("{}/carrier", base))
                .map(|s| s.trim() == "1")
                .unwrap_or(false);

            // Read MAC address
            let mac = fs::read_to_string(format!("{}/address", base))
                .map(|s| s.trim().to_string())
                .ok();

            // Check if wireless
            let is_wireless = fs::metadata(format!("{}/wireless", base)).is_ok();

            // Determine state (UP/DOWN)
            let state = if operstate == "up" { "UP" } else { "DOWN" }.to_string();

            // Get IP addresses via ip addr
            let (ip4, ip6) = get_interface_ips(&name);

            interfaces.push(NetInterfaceSummary {
                name: name.clone(),
                state,
                operstate,
                carrier,
                mac,
                ip4,
                ip6,
                is_wireless,
            });
        }
    }

    let iface_count = interfaces.len();
    let up_count = interfaces.iter().filter(|i| i.state == "UP").count();
    let human_summary = format!("{} interfaces ({} up)", iface_count, up_count);

    ToolResult {
        tool_name: "net_interfaces_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({ "interfaces": interfaces }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Get IPv4 and IPv6 addresses for an interface
fn get_interface_ips(iface: &str) -> (Vec<String>, Vec<String>) {
    let mut ip4 = Vec::new();
    let mut ip6 = Vec::new();

    if let Ok(output) = Command::new("ip")
        .args(["-o", "addr", "show", "dev", iface])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("inet ") {
                // Extract IPv4: "2: eth0    inet 192.168.1.100/24 ..."
                if let Some(addr) = line.split_whitespace().skip_while(|s| *s != "inet").nth(1) {
                    ip4.push(addr.to_string());
                }
            } else if line.contains("inet6 ") {
                // Extract IPv6
                if let Some(addr) = line.split_whitespace().skip_while(|s| *s != "inet6").nth(1) {
                    ip6.push(addr.to_string());
                }
            }
        }
    }

    (ip4, ip6)
}

/// Execute net_routes_summary tool
pub fn execute_net_routes_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let mut has_default = false;
    let mut default_gw: Option<String> = None;
    let mut default_iface: Option<String> = None;
    let mut route_count = 0;

    if let Ok(output) = Command::new("ip").args(["route", "show"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            route_count += 1;
            if line.starts_with("default") {
                has_default = true;
                // "default via 192.168.1.1 dev eth0 proto dhcp metric 100"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(idx) = parts.iter().position(|&s| s == "via") {
                    default_gw = parts.get(idx + 1).map(|s| s.to_string());
                }
                if let Some(idx) = parts.iter().position(|&s| s == "dev") {
                    default_iface = parts.get(idx + 1).map(|s| s.to_string());
                }
            }
        }
    }

    let summary = RouteSummary {
        has_default_route: has_default,
        default_gateway: default_gw.clone(),
        default_interface: default_iface.clone(),
        route_count,
    };

    let human_summary = if has_default {
        format!(
            "Default route via {} on {} ({} routes)",
            default_gw.as_deref().unwrap_or("?"),
            default_iface.as_deref().unwrap_or("?"),
            route_count
        )
    } else {
        format!("No default route ({} routes)", route_count)
    };

    ToolResult {
        tool_name: "net_routes_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!(summary),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Execute dns_summary tool
pub fn execute_dns_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let mut servers = Vec::new();
    let mut source = "unknown".to_string();
    let mut is_stub = false;
    let mut domains = Vec::new();

    // Check systemd-resolved first
    if let Ok(output) = Command::new("resolvectl")
        .args(["status", "--no-pager"])
        .output()
    {
        if output.status.success() {
            source = "systemd-resolved".to_string();
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("DNS Servers:") || line.contains("Current DNS Server:") {
                    // Extract servers after the colon
                    if let Some(part) = line.split(':').nth(1) {
                        for server in part.split_whitespace() {
                            if !servers.contains(&server.to_string()) {
                                servers.push(server.to_string());
                            }
                        }
                    }
                }
                if line.contains("DNS Domain:") {
                    if let Some(part) = line.split(':').nth(1) {
                        for domain in part.split_whitespace() {
                            domains.push(domain.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fall back to /etc/resolv.conf
    if servers.is_empty() {
        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            source = "resolv.conf".to_string();
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        servers.push(server.to_string());
                        // Check for stub resolver
                        if server == "127.0.0.53" || server == "127.0.0.1" {
                            is_stub = true;
                        }
                    }
                }
                if line.starts_with("search") || line.starts_with("domain") {
                    for domain in line.split_whitespace().skip(1) {
                        domains.push(domain.to_string());
                    }
                }
            }
        }
    }

    let summary = DnsSummary {
        servers: servers.clone(),
        source: source.clone(),
        is_stub_resolver: is_stub,
        domains: domains.clone(),
    };

    let human_summary = format!(
        "{} DNS servers from {} (stub: {})",
        servers.len(),
        source,
        if is_stub { "yes" } else { "no" }
    );

    ToolResult {
        tool_name: "dns_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!(summary),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Execute iw_summary tool (wireless status)
pub fn execute_iw_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Find wireless interface
    let wireless_iface = find_wireless_interface();

    if wireless_iface.is_none() {
        return ToolResult {
            tool_name: "iw_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "connected": false,
                "reason": "no_wireless_interface"
            }),
            human_summary: "No wireless interface found".to_string(),
            success: true,
            error: None,
            timestamp,
        };
    }

    let iface = wireless_iface.unwrap();

    // Get wireless info via iw
    let mut summary = WifiSummary {
        connected: false,
        ssid: None,
        signal_dbm: None,
        signal_quality: None,
        frequency: None,
        bitrate: None,
    };

    if let Ok(output) = Command::new("iw").args(["dev", &iface, "link"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("Not connected") {
            // Leave defaults
        } else {
            summary.connected = true;

            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("SSID:") {
                    summary.ssid = Some(line.trim_start_matches("SSID:").trim().to_string());
                } else if line.starts_with("signal:") {
                    // "signal: -55 dBm"
                    if let Some(db) = line.split_whitespace().nth(1) {
                        if let Ok(val) = db.parse::<i32>() {
                            summary.signal_dbm = Some(val);
                            summary.signal_quality = Some(signal_quality(val));
                        }
                    }
                } else if line.starts_with("freq:") {
                    summary.frequency = Some(line.trim_start_matches("freq:").trim().to_string());
                } else if line.starts_with("tx bitrate:") {
                    summary.bitrate = line.split(':').nth(1).map(|s| s.trim().to_string());
                }
            }
        }
    }

    let human_summary = if summary.connected {
        format!(
            "Connected to {} ({} dBm, {})",
            summary.ssid.as_deref().unwrap_or("?"),
            summary
                .signal_dbm
                .map(|d| d.to_string())
                .unwrap_or_else(|| "?".into()),
            summary.signal_quality.as_deref().unwrap_or("?")
        )
    } else {
        "WiFi not connected".to_string()
    };

    ToolResult {
        tool_name: "iw_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!(summary),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Find the first wireless interface
fn find_wireless_interface() -> Option<String> {
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if fs::metadata(format!("/sys/class/net/{}/wireless", name)).is_ok() {
                return Some(name);
            }
        }
    }
    None
}

/// Convert signal dBm to quality string
fn signal_quality(dbm: i32) -> String {
    if dbm >= -50 {
        "excellent".to_string()
    } else if dbm >= -60 {
        "good".to_string()
    } else if dbm >= -70 {
        "fair".to_string()
    } else {
        "poor".to_string()
    }
}

/// Execute recent_network_errors tool
pub fn execute_recent_network_errors(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let minutes = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(30) as u32;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Query journalctl for network-related units
    let units = [
        "NetworkManager",
        "systemd-networkd",
        "systemd-resolved",
        "wpa_supplicant",
        "dhcpcd",
    ];

    for unit in &units {
        if let Ok(output) = Command::new("journalctl")
            .args([
                "-u",
                unit,
                "--since",
                &format!("{} minutes ago", minutes),
                "-p",
                "warning",
                "--no-pager",
                "-q",
                "-o",
                "short-iso",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().take(10) {
                if line.to_lowercase().contains("error") || line.to_lowercase().contains("fail") {
                    errors.push(format!("[{}] {}", unit, line));
                } else {
                    warnings.push(format!("[{}] {}", unit, line));
                }
            }
        }
    }

    let summary = NetworkErrorsSummary {
        error_count: errors.len(),
        warning_count: warnings.len(),
        recent_errors: errors.iter().take(5).cloned().collect(),
        recent_warnings: warnings.iter().take(5).cloned().collect(),
        time_range_minutes: minutes,
    };

    let human_summary = format!(
        "{} errors, {} warnings in last {} minutes",
        summary.error_count, summary.warning_count, minutes
    );

    ToolResult {
        tool_name: "recent_network_errors".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!(summary),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Execute ping_check tool
pub fn execute_ping_check(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let target = params
        .get("target")
        .and_then(|v| v.as_str())
        .unwrap_or("1.1.1.1");

    // Single ping with 2 second timeout
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "2", target])
        .output();

    match output {
        Ok(result) => {
            let success = result.status.success();
            let stdout = String::from_utf8_lossy(&result.stdout);

            // Parse latency from "time=X.XX ms"
            let latency_ms: Option<f64> =
                stdout.lines().find(|l| l.contains("time=")).and_then(|l| {
                    l.split("time=")
                        .nth(1)
                        .and_then(|t| t.split_whitespace().next())
                        .and_then(|t| t.parse().ok())
                });

            let human_summary = if success {
                format!(
                    "Ping {} OK ({}ms)",
                    target,
                    latency_ms
                        .map(|l| format!("{:.1}", l))
                        .unwrap_or_else(|| "?".into())
                )
            } else {
                format!("Ping {} failed", target)
            };

            ToolResult {
                tool_name: "ping_check".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "target": target,
                    "success": success,
                    "latency_ms": latency_ms,
                }),
                human_summary,
                success: true, // Tool executed successfully even if ping failed
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "ping_check".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "target": target,
                "success": false,
                "error": e.to_string(),
            }),
            human_summary: format!("Ping failed: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_quality() {
        assert_eq!(signal_quality(-45), "excellent");
        assert_eq!(signal_quality(-55), "good");
        assert_eq!(signal_quality(-65), "fair");
        assert_eq!(signal_quality(-80), "poor");
    }
}

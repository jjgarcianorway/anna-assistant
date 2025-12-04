//! Networking Department Playbook v0.0.67
//!
//! Evidence-based networking diagnosis with human-readable summaries.
//! - net_link_summary: Physical/wireless link state
//! - net_addr_route_summary: IP addresses and routing
//! - net_dns_summary: DNS resolution configuration
//! - net_nm_iwd_status: NetworkManager / iwd service status
//! - net_recent_errors: Recent journal errors
//!
//! Each tool returns PlaybookEvidence with:
//! - summary_human: No tool names, no IDs
//! - summary_debug: With tool names and details
//! - raw_refs: Commands used (debug only)

use crate::evidence_playbook::{
    NetworkCauseCategory, NetworkingDiagnosis, PlaybookBundle, PlaybookEvidence, PlaybookTopic,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::time::Instant;

// ============================================================================
// Evidence Collection Types
// ============================================================================

/// Link state for an interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEvidence {
    pub name: String,
    pub state: String,     // UP/DOWN
    pub operstate: String, // up/down/unknown
    pub carrier: bool,     // cable connected
    pub is_wireless: bool,
    pub mac: Option<String>,
}

/// Address/route evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddrRouteEvidence {
    pub has_ipv4: bool,
    pub has_ipv6: bool,
    pub ipv4_addrs: Vec<String>,
    pub ipv6_addrs: Vec<String>,
    pub has_default_route: bool,
    pub default_gw: Option<String>,
    pub default_iface: Option<String>,
}

/// DNS configuration evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEvidence {
    pub servers: Vec<String>,
    pub source: String, // systemd-resolved, resolv.conf
    pub is_stub: bool,
    pub domains: Vec<String>,
}

/// Network manager status evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerEvidence {
    pub nm_running: bool,
    pub nm_state: Option<String>,
    pub iwd_running: bool,
    pub systemd_networkd_running: bool,
    pub conflict_detected: bool,
    pub conflict_reason: Option<String>,
}

/// Recent network errors evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkErrorsEvidence {
    pub error_count: usize,
    pub warning_count: usize,
    pub recent_errors: Vec<String>,
    pub time_range_minutes: u32,
}

// ============================================================================
// Playbook Topics (What to check)
// ============================================================================

/// Get the networking playbook topics
pub fn networking_topics() -> Vec<PlaybookTopic> {
    vec![
        PlaybookTopic::required(
            "net_link",
            "Link State",
            "Physical or wireless link must be UP for any connectivity",
            vec!["ip link show", "/sys/class/net"],
        ),
        PlaybookTopic::required(
            "net_addr_route",
            "IP Addressing & Routing",
            "Must have IP address and default route for internet access",
            vec!["ip addr show", "ip route show"],
        ),
        PlaybookTopic::required(
            "net_dns",
            "DNS Resolution",
            "DNS must be configured to resolve hostnames",
            vec!["resolvectl status", "cat /etc/resolv.conf"],
        ),
        PlaybookTopic::required(
            "net_manager",
            "Network Manager Status",
            "Detect conflicting network managers (NM vs iwd vs networkd)",
            vec!["systemctl status NetworkManager", "systemctl status iwd"],
        ),
        PlaybookTopic::optional(
            "net_errors",
            "Recent Errors",
            "Journal errors may reveal intermittent issues",
            vec!["journalctl -u NetworkManager -p warning"],
        ),
    ]
}

// ============================================================================
// Evidence Collection Functions
// ============================================================================

/// Collect link state evidence
pub fn collect_link_evidence() -> (LinkEvidence, PlaybookEvidence) {
    let start = Instant::now();
    let mut primary_iface: Option<LinkEvidence> = None;
    let mut all_up = 0;
    let mut all_down = 0;

    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue;
            }

            let base = format!("/sys/class/net/{}", name);

            let operstate = fs::read_to_string(format!("{}/operstate", base))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let carrier = fs::read_to_string(format!("{}/carrier", base))
                .map(|s| s.trim() == "1")
                .unwrap_or(false);

            let mac = fs::read_to_string(format!("{}/address", base))
                .map(|s| s.trim().to_string())
                .ok();

            let is_wireless = fs::metadata(format!("{}/wireless", base)).is_ok();

            let state = if operstate == "up" { "UP" } else { "DOWN" }.to_string();

            if state == "UP" {
                all_up += 1;
            } else {
                all_down += 1;
            }

            let link = LinkEvidence {
                name: name.clone(),
                state: state.clone(),
                operstate,
                carrier,
                is_wireless,
                mac,
            };

            // Prefer first UP interface, or first interface
            if primary_iface.is_none() || state == "UP" {
                if primary_iface.is_none() || primary_iface.as_ref().unwrap().state != "UP" {
                    primary_iface = Some(link);
                }
            }
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    let link = primary_iface.clone().unwrap_or(LinkEvidence {
        name: "none".to_string(),
        state: "DOWN".to_string(),
        operstate: "unknown".to_string(),
        carrier: false,
        is_wireless: false,
        mac: None,
    });

    // Human summary (no technical details)
    let human = if all_up > 0 {
        let iface_type = if link.is_wireless { "WiFi" } else { "Ethernet" };
        format!("{} connection is active", iface_type)
    } else {
        "No active network connections".to_string()
    };

    // Debug summary (with interface names)
    let debug = format!(
        "{}: {} (operstate={}, carrier={})",
        link.name, link.state, link.operstate, link.carrier
    );

    let evidence = PlaybookEvidence::success("net_link", &human, &debug)
        .with_refs(vec![
            "ip link show".to_string(),
            format!("/sys/class/net/{}/operstate", link.name),
        ])
        .with_duration(duration);

    (link, evidence)
}

/// Collect address and route evidence
pub fn collect_addr_route_evidence() -> (AddrRouteEvidence, PlaybookEvidence) {
    let start = Instant::now();

    let mut ipv4_addrs = Vec::new();
    let mut ipv6_addrs = Vec::new();

    // Get addresses
    if let Ok(output) = Command::new("ip").args(["-o", "addr", "show"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(" lo ") {
                continue;
            }
            if line.contains("inet ") && !line.contains("127.0.0.1") {
                if let Some(addr) = line.split_whitespace().skip_while(|s| *s != "inet").nth(1) {
                    ipv4_addrs.push(addr.to_string());
                }
            } else if line.contains("inet6 ") && !line.contains("::1") && !line.contains("fe80:") {
                if let Some(addr) = line.split_whitespace().skip_while(|s| *s != "inet6").nth(1) {
                    ipv6_addrs.push(addr.to_string());
                }
            }
        }
    }

    // Get default route
    let mut has_default = false;
    let mut default_gw: Option<String> = None;
    let mut default_iface: Option<String> = None;

    if let Ok(output) = Command::new("ip").args(["route", "show"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("default") {
                has_default = true;
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(idx) = parts.iter().position(|&s| s == "via") {
                    default_gw = parts.get(idx + 1).map(|s| s.to_string());
                }
                if let Some(idx) = parts.iter().position(|&s| s == "dev") {
                    default_iface = parts.get(idx + 1).map(|s| s.to_string());
                }
                break;
            }
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    let addr_route = AddrRouteEvidence {
        has_ipv4: !ipv4_addrs.is_empty(),
        has_ipv6: !ipv6_addrs.is_empty(),
        ipv4_addrs: ipv4_addrs.clone(),
        ipv6_addrs: ipv6_addrs.clone(),
        has_default_route: has_default,
        default_gw: default_gw.clone(),
        default_iface: default_iface.clone(),
    };

    // Human summary
    let human = if !ipv4_addrs.is_empty() && has_default {
        "IP address configured, internet route present".to_string()
    } else if !ipv4_addrs.is_empty() {
        "IP address configured, but no internet route".to_string()
    } else {
        "No IP address assigned".to_string()
    };

    // Debug summary
    let debug = format!(
        "IPv4: {:?}, IPv6: {:?}, default via {} on {}",
        ipv4_addrs,
        ipv6_addrs,
        default_gw.as_deref().unwrap_or("none"),
        default_iface.as_deref().unwrap_or("none")
    );

    let evidence = PlaybookEvidence::success("net_addr_route", &human, &debug)
        .with_refs(vec![
            "ip addr show".to_string(),
            "ip route show".to_string(),
        ])
        .with_duration(duration);

    (addr_route, evidence)
}

/// Collect DNS evidence
pub fn collect_dns_evidence() -> (DnsEvidence, PlaybookEvidence) {
    let start = Instant::now();

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
                    if let Some(part) = line.split(':').nth(1) {
                        for server in part.split_whitespace() {
                            if !servers.contains(&server.to_string()) {
                                servers.push(server.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to resolv.conf
    if servers.is_empty() {
        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            source = "resolv.conf".to_string();
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        servers.push(server.to_string());
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

    let duration = start.elapsed().as_millis() as u64;

    let dns = DnsEvidence {
        servers: servers.clone(),
        source: source.clone(),
        is_stub,
        domains,
    };

    // Human summary
    let human = if servers.is_empty() {
        "No DNS servers configured".to_string()
    } else {
        format!("{} DNS server(s) configured", servers.len())
    };

    // Debug summary
    let debug = format!(
        "DNS servers: {:?} from {} (stub: {})",
        servers, source, is_stub
    );

    let evidence = PlaybookEvidence::success("net_dns", &human, &debug)
        .with_refs(vec![
            "resolvectl status".to_string(),
            "cat /etc/resolv.conf".to_string(),
        ])
        .with_duration(duration);

    (dns, evidence)
}

/// Collect network manager status evidence
pub fn collect_manager_evidence() -> (ManagerEvidence, PlaybookEvidence) {
    let start = Instant::now();

    let nm_running = is_service_active("NetworkManager");
    let iwd_running = is_service_active("iwd");
    let networkd_running = is_service_active("systemd-networkd");

    // Get NM state if running
    let nm_state = if nm_running {
        Command::new("nmcli")
            .args(["general", "status"])
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .nth(1)
                    .and_then(|l| l.split_whitespace().next())
                    .map(|s| s.to_string())
            })
    } else {
        None
    };

    // Detect conflicts
    let mut conflict_detected = false;
    let mut conflict_reason = None;

    if nm_running && iwd_running {
        // This is OK if NM is using iwd as backend
        // Check NM config for wifi.backend=iwd
        let uses_iwd_backend = fs::read_to_string("/etc/NetworkManager/conf.d/wifi_backend.conf")
            .map(|c| c.contains("wifi.backend=iwd"))
            .unwrap_or(false);

        if !uses_iwd_backend {
            conflict_detected = true;
            conflict_reason =
                Some("NetworkManager and iwd both running without backend integration".to_string());
        }
    }

    if nm_running && networkd_running {
        conflict_detected = true;
        conflict_reason = Some("NetworkManager and systemd-networkd both running".to_string());
    }

    let duration = start.elapsed().as_millis() as u64;

    let mgr = ManagerEvidence {
        nm_running,
        nm_state: nm_state.clone(),
        iwd_running,
        systemd_networkd_running: networkd_running,
        conflict_detected,
        conflict_reason: conflict_reason.clone(),
    };

    // Human summary
    let human = if conflict_detected {
        conflict_reason
            .clone()
            .unwrap_or("Network manager conflict detected".to_string())
    } else if nm_running {
        format!(
            "NetworkManager active ({})",
            nm_state.as_deref().unwrap_or("unknown")
        )
    } else if iwd_running {
        "iwd managing WiFi".to_string()
    } else if networkd_running {
        "systemd-networkd managing network".to_string()
    } else {
        "No network manager running".to_string()
    };

    // Debug summary
    let debug = format!(
        "NM: {} ({:?}), iwd: {}, networkd: {}, conflict: {}",
        nm_running, nm_state, iwd_running, networkd_running, conflict_detected
    );

    let evidence = PlaybookEvidence::success("net_manager", &human, &debug)
        .with_refs(vec![
            "systemctl is-active NetworkManager".to_string(),
            "systemctl is-active iwd".to_string(),
            "nmcli general status".to_string(),
        ])
        .with_duration(duration);

    (mgr, evidence)
}

/// Collect recent network errors
pub fn collect_errors_evidence(minutes: u32) -> (NetworkErrorsEvidence, PlaybookEvidence) {
    let start = Instant::now();

    let mut error_count = 0;
    let mut warning_count = 0;
    let mut recent_errors = Vec::new();

    let units = [
        "NetworkManager",
        "systemd-networkd",
        "systemd-resolved",
        "wpa_supplicant",
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
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().take(20) {
                let lower = line.to_lowercase();
                if lower.contains("error") || lower.contains("fail") {
                    error_count += 1;
                    if recent_errors.len() < 5 {
                        recent_errors.push(line.to_string());
                    }
                } else {
                    warning_count += 1;
                }
            }
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    let errors = NetworkErrorsEvidence {
        error_count,
        warning_count,
        recent_errors: recent_errors.clone(),
        time_range_minutes: minutes,
    };

    // Human summary
    let human = if error_count == 0 && warning_count == 0 {
        format!("No network issues in the last {} minutes", minutes)
    } else if error_count > 0 {
        format!(
            "{} network error(s) in the last {} minutes",
            error_count, minutes
        )
    } else {
        format!(
            "{} network warning(s) in the last {} minutes",
            warning_count, minutes
        )
    };

    // Debug summary
    let debug = format!(
        "{} errors, {} warnings in {} min, samples: {:?}",
        error_count,
        warning_count,
        minutes,
        recent_errors.first()
    );

    let evidence = PlaybookEvidence::success("net_errors", &human, &debug)
        .with_refs(vec![format!(
            "journalctl -u NetworkManager --since '{} minutes ago' -p warning",
            minutes
        )])
        .with_duration(duration);

    (errors, evidence)
}

// ============================================================================
// Diagnosis Engine
// ============================================================================

/// Run the full networking diagnosis playbook
pub fn run_networking_playbook() -> NetworkingDiagnosis {
    let topics = networking_topics();
    let mut bundle = PlaybookBundle::new("networking");
    let mut findings = Vec::new();

    // Collect all evidence
    let (link, link_ev) = collect_link_evidence();
    bundle.add(link_ev);

    let (addr_route, addr_route_ev) = collect_addr_route_evidence();
    bundle.add(addr_route_ev);

    let (dns, dns_ev) = collect_dns_evidence();
    bundle.add(dns_ev);

    let (manager, manager_ev) = collect_manager_evidence();
    bundle.add(manager_ev);

    let (errors, errors_ev) = collect_errors_evidence(30);
    bundle.add(errors_ev);

    // Finalize coverage
    bundle.finalize(&topics);

    // Analyze and classify cause
    let (cause, confidence) = classify_network_cause(&link, &addr_route, &dns, &manager, &errors);

    // Build findings
    if link.state != "UP" {
        findings.push(format!("Network interface {} is DOWN", link.name));
    }
    if !addr_route.has_ipv4 {
        findings.push("No IPv4 address assigned".to_string());
    }
    if !addr_route.has_default_route {
        findings.push("No default route - cannot reach internet".to_string());
    }
    if dns.servers.is_empty() {
        findings.push("No DNS servers configured".to_string());
    }
    if manager.conflict_detected {
        findings.push(
            manager
                .conflict_reason
                .unwrap_or("Manager conflict".to_string()),
        );
    }
    if errors.error_count > 0 {
        findings.push(format!(
            "{} recent network errors in journal",
            errors.error_count
        ));
    }

    NetworkingDiagnosis::new(bundle)
        .with_findings(findings)
        .with_cause(cause, confidence)
}

/// Classify the root cause based on evidence
fn classify_network_cause(
    link: &LinkEvidence,
    addr_route: &AddrRouteEvidence,
    dns: &DnsEvidence,
    manager: &ManagerEvidence,
    _errors: &NetworkErrorsEvidence,
) -> (NetworkCauseCategory, u8) {
    // Priority order: Link -> DHCP -> Manager -> DNS -> Unknown

    if link.state != "UP" || !link.carrier {
        return (NetworkCauseCategory::Link, 90);
    }

    if !addr_route.has_ipv4 || !addr_route.has_default_route {
        return (NetworkCauseCategory::Dhcp, 85);
    }

    if manager.conflict_detected {
        return (NetworkCauseCategory::ManagerConflict, 80);
    }

    if dns.servers.is_empty() {
        return (NetworkCauseCategory::Dns, 85);
    }

    (NetworkCauseCategory::Unknown, 30)
}

// ============================================================================
// Helpers
// ============================================================================

fn is_service_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_networking_topics() {
        let topics = networking_topics();
        assert_eq!(topics.len(), 5);
        assert!(topics.iter().filter(|t| t.required).count() >= 4);
    }

    #[test]
    fn test_classify_link_down() {
        let link = LinkEvidence {
            name: "eth0".to_string(),
            state: "DOWN".to_string(),
            operstate: "down".to_string(),
            carrier: false,
            is_wireless: false,
            mac: None,
        };
        let addr = AddrRouteEvidence {
            has_ipv4: false,
            has_ipv6: false,
            ipv4_addrs: vec![],
            ipv6_addrs: vec![],
            has_default_route: false,
            default_gw: None,
            default_iface: None,
        };
        let dns = DnsEvidence {
            servers: vec!["8.8.8.8".to_string()],
            source: "resolv.conf".to_string(),
            is_stub: false,
            domains: vec![],
        };
        let mgr = ManagerEvidence {
            nm_running: true,
            nm_state: Some("connected".to_string()),
            iwd_running: false,
            systemd_networkd_running: false,
            conflict_detected: false,
            conflict_reason: None,
        };
        let errors = NetworkErrorsEvidence {
            error_count: 0,
            warning_count: 0,
            recent_errors: vec![],
            time_range_minutes: 30,
        };

        let (cause, conf) = classify_network_cause(&link, &addr, &dns, &mgr, &errors);
        assert_eq!(cause, NetworkCauseCategory::Link);
        assert!(conf >= 80);
    }

    #[test]
    fn test_classify_no_ip() {
        let link = LinkEvidence {
            name: "eth0".to_string(),
            state: "UP".to_string(),
            operstate: "up".to_string(),
            carrier: true,
            is_wireless: false,
            mac: None,
        };
        let addr = AddrRouteEvidence {
            has_ipv4: false,
            has_ipv6: false,
            ipv4_addrs: vec![],
            ipv6_addrs: vec![],
            has_default_route: false,
            default_gw: None,
            default_iface: None,
        };
        let dns = DnsEvidence {
            servers: vec!["8.8.8.8".to_string()],
            source: "resolv.conf".to_string(),
            is_stub: false,
            domains: vec![],
        };
        let mgr = ManagerEvidence {
            nm_running: true,
            nm_state: Some("connected".to_string()),
            iwd_running: false,
            systemd_networkd_running: false,
            conflict_detected: false,
            conflict_reason: None,
        };
        let errors = NetworkErrorsEvidence {
            error_count: 0,
            warning_count: 0,
            recent_errors: vec![],
            time_range_minutes: 30,
        };

        let (cause, _) = classify_network_cause(&link, &addr, &dns, &mgr, &errors);
        assert_eq!(cause, NetworkCauseCategory::Dhcp);
    }
}

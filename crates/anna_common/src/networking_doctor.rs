//! Arch Networking Doctor v0.0.38
//!
//! Specialized diagnosis and fix system for Arch Linux networking issues.
//! Supports: NetworkManager, iwd, systemd-networkd, wpa_supplicant
//!
//! Features:
//! - Deterministic diagnosis flow (link -> IP -> route -> connectivity -> DNS)
//! - Evidence-backed hypotheses (1-3 max)
//! - Fix playbooks with preflight, confirmation, post-check, rollback
//! - Recipe creation on successful fix (>= 80% reliability)
//! - Case file integration (networking_doctor.json)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

use crate::generate_request_id;

// =============================================================================
// Constants
// =============================================================================

/// Default ping timeout in seconds
pub const PING_TIMEOUT_SECS: u32 = 3;

/// Default ping count
pub const PING_COUNT: u32 = 2;

/// DNS test domains
pub const DNS_TEST_DOMAINS: &[&str] = &["archlinux.org", "google.com"];

/// Default gateway for raw IP connectivity test
pub const RAW_IP_TEST: &str = "1.1.1.1";

/// Fix confirmation phrase
pub const FIX_CONFIRMATION: &str = "I CONFIRM (apply fix)";

/// Max journal lines to collect per service
pub const MAX_JOURNAL_LINES: usize = 50;

// =============================================================================
// Network Manager Detection
// =============================================================================

/// Supported network managers on Arch Linux
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkManager {
    /// NetworkManager (most common)
    NetworkManager,
    /// iwd (Intel wireless daemon)
    Iwd,
    /// systemd-networkd
    SystemdNetworkd,
    /// wpa_supplicant (usually with dhcpcd)
    WpaSupplicant,
    /// No recognized manager detected
    None,
}

impl NetworkManager {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkManager::NetworkManager => "NetworkManager",
            NetworkManager::Iwd => "iwd",
            NetworkManager::SystemdNetworkd => "systemd-networkd",
            NetworkManager::WpaSupplicant => "wpa_supplicant",
            NetworkManager::None => "none",
        }
    }

    pub fn service_name(&self) -> Option<&'static str> {
        match self {
            NetworkManager::NetworkManager => Some("NetworkManager.service"),
            NetworkManager::Iwd => Some("iwd.service"),
            NetworkManager::SystemdNetworkd => Some("systemd-networkd.service"),
            NetworkManager::WpaSupplicant => Some("wpa_supplicant.service"),
            NetworkManager::None => None,
        }
    }
}

/// Detected network managers and their status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkManagerStatus {
    pub manager: NetworkManager,
    pub is_running: bool,
    pub is_enabled: bool,
    pub has_errors: bool,
    pub error_summary: Option<String>,
}

/// Detect which network manager is active
pub fn detect_network_manager() -> NetworkManagerStatus {
    // Check managers in order of preference
    let managers = [
        ("NetworkManager.service", NetworkManager::NetworkManager),
        ("iwd.service", NetworkManager::Iwd),
        ("systemd-networkd.service", NetworkManager::SystemdNetworkd),
        ("wpa_supplicant.service", NetworkManager::WpaSupplicant),
    ];

    for (service, manager) in managers {
        if let Some(status) = check_service_status(service) {
            if status.is_running {
                return NetworkManagerStatus {
                    manager,
                    is_running: status.is_running,
                    is_enabled: status.is_enabled,
                    has_errors: status.has_errors,
                    error_summary: status.error_summary,
                };
            }
        }
    }

    // Check if any are enabled but not running
    for (service, manager) in managers {
        if let Some(status) = check_service_status(service) {
            if status.is_enabled {
                return NetworkManagerStatus {
                    manager,
                    is_running: false,
                    is_enabled: true,
                    has_errors: true,
                    error_summary: Some("Service enabled but not running".to_string()),
                };
            }
        }
    }

    NetworkManagerStatus {
        manager: NetworkManager::None,
        is_running: false,
        is_enabled: false,
        has_errors: true,
        error_summary: Some("No network manager detected".to_string()),
    }
}

/// Check all network managers for conflicts
pub fn detect_manager_conflicts() -> Vec<String> {
    let mut conflicts = Vec::new();
    let mut running = Vec::new();

    let managers = [
        ("NetworkManager.service", "NetworkManager"),
        ("iwd.service", "iwd"),
        ("systemd-networkd.service", "systemd-networkd"),
        ("wpa_supplicant.service", "wpa_supplicant"),
    ];

    for (service, name) in managers {
        if let Some(status) = check_service_status(service) {
            if status.is_running {
                running.push(name);
            }
        }
    }

    // NetworkManager + iwd is OK (NM can use iwd as backend)
    // NetworkManager + systemd-networkd is a conflict
    // Multiple managers generally conflict
    if running.contains(&"NetworkManager") && running.contains(&"systemd-networkd") {
        conflicts.push(
            "NetworkManager and systemd-networkd are both running - this may cause conflicts"
                .to_string(),
        );
    }

    if running.len() > 2 {
        conflicts.push(format!(
            "Multiple network managers running: {} - this may cause conflicts",
            running.join(", ")
        ));
    }

    conflicts
}

// =============================================================================
// Diagnosis Steps
// =============================================================================

/// Step result in the diagnosis flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStepResult {
    Pass,
    Fail,
    Partial,
    Skipped,
}

impl DiagnosisStepResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosisStepResult::Pass => "PASS",
            DiagnosisStepResult::Fail => "FAIL",
            DiagnosisStepResult::Partial => "PARTIAL",
            DiagnosisStepResult::Skipped => "SKIPPED",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            DiagnosisStepResult::Pass => "[OK]",
            DiagnosisStepResult::Fail => "[FAIL]",
            DiagnosisStepResult::Partial => "[WARN]",
            DiagnosisStepResult::Skipped => "[SKIP]",
        }
    }
}

/// A single diagnosis step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStep {
    pub name: String,
    pub description: String,
    pub result: DiagnosisStepResult,
    pub details: String,
    pub evidence_id: String,
    pub implication: String,
    pub timestamp: DateTime<Utc>,
}

impl DiagnosisStep {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: DiagnosisStepResult::Skipped,
            details: String::new(),
            evidence_id: generate_request_id(),
            implication: String::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn pass(mut self, details: &str, implication: &str) -> Self {
        self.result = DiagnosisStepResult::Pass;
        self.details = details.to_string();
        self.implication = implication.to_string();
        self
    }

    pub fn fail(mut self, details: &str, implication: &str) -> Self {
        self.result = DiagnosisStepResult::Fail;
        self.details = details.to_string();
        self.implication = implication.to_string();
        self
    }

    pub fn partial(mut self, details: &str, implication: &str) -> Self {
        self.result = DiagnosisStepResult::Partial;
        self.details = details.to_string();
        self.implication = implication.to_string();
        self
    }

    pub fn format_summary(&self) -> String {
        format!("{} {} - {}", self.result.symbol(), self.name, self.details)
    }

    pub fn format_detail(&self) -> String {
        format!(
            "{} {} [{}]\n  Description: {}\n  Details: {}\n  Implication: {}",
            self.result.symbol(),
            self.name,
            self.evidence_id,
            self.description,
            self.details,
            self.implication
        )
    }
}

// =============================================================================
// Evidence Collection
// =============================================================================

/// Network evidence bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvidence {
    pub evidence_id: String,
    pub timestamp: DateTime<Utc>,

    // Interface inventory
    pub interfaces: Vec<InterfaceEvidence>,

    // Routing
    pub default_gateway: Option<String>,
    pub routes: Vec<String>,

    // DNS
    pub dns_servers: Vec<String>,
    pub dns_config_source: String,

    // Manager status
    pub manager_status: NetworkManagerStatus,
    pub manager_conflicts: Vec<String>,

    // Connectivity tests
    pub gateway_reachable: Option<bool>,
    pub internet_reachable: Option<bool>,
    pub dns_working: Option<bool>,

    // Logs
    pub recent_errors: Vec<String>,
    pub recent_warnings: Vec<String>,
}

/// Evidence for a single interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceEvidence {
    pub name: String,
    pub iface_type: String,
    pub is_up: bool,
    pub has_carrier: bool,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub driver: Option<String>,

    // WiFi specific
    pub wifi_ssid: Option<String>,
    pub wifi_signal_dbm: Option<i32>,
    pub wifi_frequency: Option<String>,

    // Ethernet specific
    pub link_speed: Option<String>,
}

impl NetworkEvidence {
    pub fn new() -> Self {
        Self {
            evidence_id: format!("NET-{}", generate_request_id()),
            timestamp: Utc::now(),
            interfaces: Vec::new(),
            default_gateway: None,
            routes: Vec::new(),
            dns_servers: Vec::new(),
            dns_config_source: String::new(),
            manager_status: NetworkManagerStatus {
                manager: NetworkManager::None,
                is_running: false,
                is_enabled: false,
                has_errors: false,
                error_summary: None,
            },
            manager_conflicts: Vec::new(),
            gateway_reachable: None,
            internet_reachable: None,
            dns_working: None,
            recent_errors: Vec::new(),
            recent_warnings: Vec::new(),
        }
    }
}

impl Default for NetworkEvidence {
    fn default() -> Self {
        Self::new()
    }
}

/// Collect all network evidence
pub fn collect_network_evidence() -> NetworkEvidence {
    let mut evidence = NetworkEvidence::new();

    // Collect interface info
    evidence.interfaces = collect_interface_evidence();

    // Collect routing info
    let (gateway, routes) = collect_routing_evidence();
    evidence.default_gateway = gateway;
    evidence.routes = routes;

    // Collect DNS info
    let (servers, source) = collect_dns_evidence();
    evidence.dns_servers = servers;
    evidence.dns_config_source = source;

    // Detect manager
    evidence.manager_status = detect_network_manager();
    evidence.manager_conflicts = detect_manager_conflicts();

    // Collect logs
    let (errors, warnings) = collect_network_logs(&evidence.manager_status.manager);
    evidence.recent_errors = errors;
    evidence.recent_warnings = warnings;

    evidence
}

fn collect_interface_evidence() -> Vec<InterfaceEvidence> {
    let mut interfaces = Vec::new();

    // Use ip link show
    if let Ok(output) = Command::new("ip").args(["-o", "link", "show"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(iface) = parse_interface_line(line) {
                    interfaces.push(iface);
                }
            }
        }
    }

    // Enrich with IP addresses
    if let Ok(output) = Command::new("ip").args(["-o", "addr", "show"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[1].trim_end_matches(':');
                    if parts[2] == "inet" || parts[2] == "inet6" {
                        let addr = parts[3].to_string();
                        if let Some(iface) = interfaces.iter_mut().find(|i| i.name == name) {
                            if !addr.starts_with("fe80:") && iface.ip_addresses.len() < 5 {
                                iface.ip_addresses.push(addr);
                            }
                        }
                    }
                }
            }
        }
    }

    // Enrich WiFi interfaces with iw info
    for iface in &mut interfaces {
        if iface.iface_type == "wifi" {
            enrich_wifi_info(iface);
        }
    }

    // Enrich Ethernet interfaces with ethtool
    for iface in &mut interfaces {
        if iface.iface_type == "ethernet" {
            enrich_ethernet_info(iface);
        }
    }

    interfaces
}

fn parse_interface_line(line: &str) -> Option<InterfaceEvidence> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[1].trim_end_matches(':').to_string();

    // Skip loopback
    if name == "lo" {
        return None;
    }

    let is_up = line.contains("state UP") || line.contains(",UP");
    let has_carrier = line.contains("state UP"); // More accurate check

    // Determine type from name
    let iface_type = if name.starts_with("wl") || name.starts_with("wlan") {
        "wifi"
    } else if name.starts_with("en") || name.starts_with("eth") {
        "ethernet"
    } else if name.starts_with("br") || name.starts_with("virbr") {
        "bridge"
    } else if name.starts_with("docker") || name.starts_with("veth") {
        "virtual"
    } else {
        "unknown"
    }
    .to_string();

    // Extract MAC address
    let mac_address = if let Some(pos) = line.find("link/ether") {
        line[pos..].split_whitespace().nth(1).map(|s| s.to_string())
    } else {
        None
    };

    Some(InterfaceEvidence {
        name,
        iface_type,
        is_up,
        has_carrier,
        ip_addresses: Vec::new(),
        mac_address,
        driver: None,
        wifi_ssid: None,
        wifi_signal_dbm: None,
        wifi_frequency: None,
        link_speed: None,
    })
}

fn enrich_wifi_info(iface: &mut InterfaceEvidence) {
    // Use iw dev <name> link
    if let Ok(output) = Command::new("iw")
        .args(["dev", &iface.name, "link"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("SSID:") {
                    iface.wifi_ssid = Some(line.trim_start_matches("SSID:").trim().to_string());
                } else if line.starts_with("signal:") {
                    if let Some(dbm) = line.split_whitespace().nth(1) {
                        iface.wifi_signal_dbm = dbm.parse().ok();
                    }
                } else if line.starts_with("freq:") {
                    iface.wifi_frequency =
                        Some(line.trim_start_matches("freq:").trim().to_string());
                }
            }
        }
    }
}

fn enrich_ethernet_info(iface: &mut InterfaceEvidence) {
    // Use ethtool if available
    if let Ok(output) = Command::new("ethtool").arg(&iface.name).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("Speed:") {
                    iface.link_speed = Some(line.trim_start_matches("Speed:").trim().to_string());
                }
            }
        }
    }
}

fn collect_routing_evidence() -> (Option<String>, Vec<String>) {
    let mut routes = Vec::new();
    let mut default_gateway = None;

    if let Ok(output) = Command::new("ip").args(["route", "show"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                routes.push(line.to_string());
                if line.starts_with("default via") {
                    // Extract gateway IP
                    if let Some(gw) = line.split_whitespace().nth(2) {
                        default_gateway = Some(gw.to_string());
                    }
                }
            }
        }
    }

    (default_gateway, routes)
}

fn collect_dns_evidence() -> (Vec<String>, String) {
    let mut servers = Vec::new();
    let mut source = "unknown".to_string();

    // Try resolvectl first (systemd-resolved)
    if let Ok(output) = Command::new("resolvectl").args(["status"]).output() {
        if output.status.success() {
            source = "systemd-resolved".to_string();
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("DNS Servers:") || line.starts_with("Current DNS Server:") {
                    if let Some(server) = line.split(':').nth(1) {
                        for s in server.split_whitespace() {
                            if !servers.contains(&s.to_string()) {
                                servers.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to /etc/resolv.conf
    if servers.is_empty() {
        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            source = "/etc/resolv.conf".to_string();
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        servers.push(server.to_string());
                    }
                }
            }
        }
    }

    (servers, source)
}

fn collect_network_logs(manager: &NetworkManager) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let service = match manager {
        NetworkManager::NetworkManager => "NetworkManager",
        NetworkManager::Iwd => "iwd",
        NetworkManager::SystemdNetworkd => "systemd-networkd",
        NetworkManager::WpaSupplicant => "wpa_supplicant",
        NetworkManager::None => return (errors, warnings),
    };

    // Get recent logs
    if let Ok(output) = Command::new("journalctl")
        .args([
            "-u",
            service,
            "--no-pager",
            "-n",
            &MAX_JOURNAL_LINES.to_string(),
            "--since",
            "30 min ago",
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("error") || lower.contains("failed") {
                    if errors.len() < 10 {
                        errors.push(truncate_log_line(line, 200));
                    }
                } else if lower.contains("warn") {
                    if warnings.len() < 10 {
                        warnings.push(truncate_log_line(line, 200));
                    }
                }
            }
        }
    }

    (errors, warnings)
}

fn truncate_log_line(line: &str, max_len: usize) -> String {
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len])
    }
}

// =============================================================================
// Diagnosis Flow
// =============================================================================

/// Full diagnosis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    pub evidence: NetworkEvidence,
    pub steps: Vec<DiagnosisStep>,
    pub hypotheses: Vec<NetworkHypothesis>,
    pub recommended_fix: Option<FixPlaybook>,
    pub summary: String,
    pub overall_status: DiagnosisStatus,
}

/// Overall diagnosis status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStatus {
    /// Network is fully working
    Healthy,
    /// Network has issues but is partially working
    Degraded,
    /// Network is not working
    Broken,
    /// Could not determine status
    Unknown,
}

impl DiagnosisStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosisStatus::Healthy => "healthy",
            DiagnosisStatus::Degraded => "degraded",
            DiagnosisStatus::Broken => "broken",
            DiagnosisStatus::Unknown => "unknown",
        }
    }
}

/// Run the full diagnosis flow
pub fn run_diagnosis() -> DiagnosisResult {
    let evidence = collect_network_evidence();
    let mut steps = Vec::new();

    // Step 1: Physical Link
    let link_step = check_physical_link(&evidence);
    steps.push(link_step.clone());

    // Step 2: IP Address
    let ip_step = check_ip_address(&evidence);
    steps.push(ip_step.clone());

    // Step 3: Default Route
    let route_step = check_default_route(&evidence);
    steps.push(route_step.clone());

    // Step 4: Raw IP Connectivity
    let mut ip_connectivity = DiagnosisStep::new("ip_connectivity", "Test raw IP connectivity");
    if route_step.result == DiagnosisStepResult::Pass {
        ip_connectivity = check_ip_connectivity(&evidence);
    } else {
        ip_connectivity = ip_connectivity.fail(
            "Skipped - no default route",
            "Cannot test IP connectivity without a route",
        );
    }
    steps.push(ip_connectivity.clone());

    // Step 5: DNS
    let mut dns_step = DiagnosisStep::new("dns", "Test DNS resolution");
    if ip_connectivity.result == DiagnosisStepResult::Pass {
        dns_step = check_dns(&evidence);
    } else {
        dns_step = dns_step.fail(
            "Skipped - no IP connectivity",
            "Cannot test DNS without IP connectivity",
        );
    }
    steps.push(dns_step.clone());

    // Step 6: Manager Health
    let manager_step = check_manager_health(&evidence);
    steps.push(manager_step.clone());

    // Generate hypotheses
    let hypotheses = generate_hypotheses(&evidence, &steps);

    // Determine recommended fix
    let recommended_fix = select_fix_playbook(&hypotheses, &evidence);

    // Determine overall status
    let overall_status = determine_status(&steps);

    // Generate summary
    let summary = generate_summary(&steps, &hypotheses, overall_status);

    DiagnosisResult {
        evidence,
        steps,
        hypotheses,
        recommended_fix,
        summary,
        overall_status,
    }
}

fn check_physical_link(evidence: &NetworkEvidence) -> DiagnosisStep {
    let mut step = DiagnosisStep::new("physical_link", "Check physical link (carrier/association)");

    let physical: Vec<_> = evidence
        .interfaces
        .iter()
        .filter(|i| i.iface_type == "wifi" || i.iface_type == "ethernet")
        .collect();

    if physical.is_empty() {
        return step.fail(
            "No physical interfaces found",
            "System has no WiFi or Ethernet interfaces - check hardware",
        );
    }

    let up_count = physical.iter().filter(|i| i.is_up).count();
    let wifi_associated: Vec<_> = physical
        .iter()
        .filter(|i| i.iface_type == "wifi" && i.wifi_ssid.is_some())
        .collect();

    if up_count > 0 {
        let details = physical
            .iter()
            .filter(|i| i.is_up)
            .map(|i| {
                if let Some(ref ssid) = i.wifi_ssid {
                    format!("{} (WiFi: {})", i.name, ssid)
                } else {
                    format!("{} ({})", i.name, i.iface_type)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        step.pass(
            &format!("{} interface(s) up: {}", up_count, details),
            "Physical layer is working",
        )
    } else if !wifi_associated.is_empty() {
        step.partial(
            "WiFi associated but interface not fully up",
            "WiFi connected but may have issues",
        )
    } else {
        let iface_list = physical
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        step.fail(
            &format!("All interfaces down: {}", iface_list),
            "No network connectivity at physical layer",
        )
    }
}

fn check_ip_address(evidence: &NetworkEvidence) -> DiagnosisStep {
    let mut step = DiagnosisStep::new("ip_address", "Check for IP address (v4/v6)");

    let with_ip: Vec<_> = evidence
        .interfaces
        .iter()
        .filter(|i| !i.ip_addresses.is_empty())
        .filter(|i| i.iface_type == "wifi" || i.iface_type == "ethernet")
        .collect();

    if with_ip.is_empty() {
        return step.fail(
            "No IP address on any physical interface",
            "DHCP may have failed or static IP not configured",
        );
    }

    let details = with_ip
        .iter()
        .map(|i| format!("{}: {}", i.name, i.ip_addresses.join(", ")))
        .collect::<Vec<_>>()
        .join("; ");

    step.pass(
        &details,
        "IP address acquired - DHCP or static config working",
    )
}

fn check_default_route(evidence: &NetworkEvidence) -> DiagnosisStep {
    let mut step = DiagnosisStep::new("default_route", "Check for default route (gateway)");

    if let Some(ref gw) = evidence.default_gateway {
        step.pass(
            &format!("Default gateway: {}", gw),
            "Routing table has a default route",
        )
    } else {
        step.fail(
            "No default route configured",
            "Cannot reach the internet without a default route",
        )
    }
}

fn check_ip_connectivity(evidence: &NetworkEvidence) -> DiagnosisStep {
    let mut step = DiagnosisStep::new("ip_connectivity", "Test raw IP connectivity");

    // Ping gateway first
    if let Some(ref gw) = evidence.default_gateway {
        if ping_host(gw) {
            // Try internet IP
            if ping_host(RAW_IP_TEST) {
                step = step.pass(
                    &format!("Gateway {} and {} reachable", gw, RAW_IP_TEST),
                    "IP connectivity working - can reach internet",
                );
            } else {
                step = step.partial(
                    &format!("Gateway {} reachable but {} not", gw, RAW_IP_TEST),
                    "Local network works but internet blocked (firewall?)",
                );
            }
        } else {
            step = step.fail(
                &format!("Cannot ping gateway {}", gw),
                "Gateway unreachable - check cable/WiFi or gateway device",
            );
        }
    } else {
        step = step.fail(
            "No gateway to test",
            "Cannot test IP connectivity without a gateway",
        );
    }

    step
}

fn check_dns(evidence: &NetworkEvidence) -> DiagnosisStep {
    let mut step = DiagnosisStep::new("dns", "Test DNS resolution");

    if evidence.dns_servers.is_empty() {
        return step.fail(
            "No DNS servers configured",
            "DNS resolution will not work without nameservers",
        );
    }

    // Test DNS resolution
    let mut resolved = false;
    let mut test_results = Vec::new();

    for domain in DNS_TEST_DOMAINS {
        if let Ok(output) = Command::new("getent").args(["hosts", domain]).output() {
            if output.status.success() {
                resolved = true;
                test_results.push(format!("{}: OK", domain));
                break;
            } else {
                test_results.push(format!("{}: FAIL", domain));
            }
        }
    }

    if resolved {
        step.pass(
            &format!(
                "DNS working (servers: {}, tests: {})",
                evidence.dns_servers.join(", "),
                test_results.join(", ")
            ),
            "Name resolution is working",
        )
    } else {
        step.fail(
            &format!(
                "DNS resolution failed (servers: {}, tests: {})",
                evidence.dns_servers.join(", "),
                test_results.join(", ")
            ),
            "Cannot resolve domain names - check DNS config",
        )
    }
}

fn check_manager_health(evidence: &NetworkEvidence) -> DiagnosisStep {
    let mut step = DiagnosisStep::new("manager_health", "Check network manager health");

    let status = &evidence.manager_status;

    if status.manager == NetworkManager::None {
        return step.fail(
            "No network manager detected",
            "Install and enable NetworkManager, iwd, or systemd-networkd",
        );
    }

    if !evidence.manager_conflicts.is_empty() {
        return step.partial(
            &format!(
                "{} running but conflicts detected: {}",
                status.manager.as_str(),
                evidence.manager_conflicts.join("; ")
            ),
            "Multiple network managers may cause issues",
        );
    }

    if !status.is_running {
        return step.fail(
            &format!("{} is not running", status.manager.as_str()),
            "Start the network manager service",
        );
    }

    if status.has_errors {
        let err_msg = status.error_summary.as_deref().unwrap_or("unknown errors");
        return step.partial(
            &format!(
                "{} running with errors: {}",
                status.manager.as_str(),
                err_msg
            ),
            "Check service logs for details",
        );
    }

    step.pass(
        &format!("{} running and healthy", status.manager.as_str()),
        "Network manager is functioning normally",
    )
}

fn ping_host(host: &str) -> bool {
    Command::new("ping")
        .args([
            "-c",
            &PING_COUNT.to_string(),
            "-W",
            &PING_TIMEOUT_SECS.to_string(),
            host,
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn determine_status(steps: &[DiagnosisStep]) -> DiagnosisStatus {
    let fails = steps
        .iter()
        .filter(|s| s.result == DiagnosisStepResult::Fail)
        .count();
    let partials = steps
        .iter()
        .filter(|s| s.result == DiagnosisStepResult::Partial)
        .count();
    let passes = steps
        .iter()
        .filter(|s| s.result == DiagnosisStepResult::Pass)
        .count();

    if fails == 0 && partials == 0 {
        DiagnosisStatus::Healthy
    } else if fails == 0 && partials > 0 {
        DiagnosisStatus::Degraded
    } else if fails > 0 && passes > 0 {
        // Some pass, some fail = degraded
        DiagnosisStatus::Degraded
    } else if fails > 0 && passes == 0 {
        // All fail = broken
        DiagnosisStatus::Broken
    } else {
        DiagnosisStatus::Unknown
    }
}

fn generate_summary(
    steps: &[DiagnosisStep],
    hypotheses: &[NetworkHypothesis],
    status: DiagnosisStatus,
) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "Network Status: {}",
        status.as_str().to_uppercase()
    ));
    lines.push(String::new());

    lines.push("Diagnosis Steps:".to_string());
    for step in steps {
        lines.push(format!("  {}", step.format_summary()));
    }

    if !hypotheses.is_empty() {
        lines.push(String::new());
        lines.push("Hypotheses:".to_string());
        for (i, h) in hypotheses.iter().enumerate() {
            lines.push(format!(
                "  {}. {} (confidence: {}%)",
                i + 1,
                h.description,
                h.confidence
            ));
        }
    }

    lines.join("\n")
}

// =============================================================================
// Hypothesis Generation
// =============================================================================

/// A hypothesis about the network problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHypothesis {
    pub id: String,
    pub description: String,
    pub confidence: u8,
    pub supporting_evidence: Vec<String>,
    pub refuting_evidence: Vec<String>,
    pub next_test: String,
    pub fix_playbook: Option<String>,
}

fn generate_hypotheses(
    evidence: &NetworkEvidence,
    steps: &[DiagnosisStep],
) -> Vec<NetworkHypothesis> {
    let mut hypotheses = Vec::new();

    // Find failing steps
    let failing_steps: Vec<_> = steps
        .iter()
        .filter(|s| s.result == DiagnosisStepResult::Fail)
        .collect();

    if failing_steps.is_empty() {
        return hypotheses;
    }

    // Generate hypotheses based on first failure point
    for step in failing_steps.iter().take(1) {
        match step.name.as_str() {
            "physical_link" => {
                // WiFi issues
                if evidence.interfaces.iter().any(|i| i.iface_type == "wifi") {
                    hypotheses.push(NetworkHypothesis {
                        id: generate_request_id(),
                        description:
                            "WiFi not connected - may need to reconnect or check credentials"
                                .to_string(),
                        confidence: 70,
                        supporting_evidence: vec![step.evidence_id.clone()],
                        refuting_evidence: Vec::new(),
                        next_test: "Check WiFi signal and try reconnecting".to_string(),
                        fix_playbook: Some("restart_manager".to_string()),
                    });
                }

                // Ethernet issues
                if evidence
                    .interfaces
                    .iter()
                    .any(|i| i.iface_type == "ethernet")
                {
                    hypotheses.push(NetworkHypothesis {
                        id: generate_request_id(),
                        description: "Ethernet cable disconnected or link down".to_string(),
                        confidence: 60,
                        supporting_evidence: vec![step.evidence_id.clone()],
                        refuting_evidence: Vec::new(),
                        next_test: "Check cable connection and switch/router".to_string(),
                        fix_playbook: None,
                    });
                }
            }

            "ip_address" => {
                hypotheses.push(NetworkHypothesis {
                    id: generate_request_id(),
                    description: "DHCP failure - no IP address assigned".to_string(),
                    confidence: 80,
                    supporting_evidence: vec![step.evidence_id.clone()],
                    refuting_evidence: Vec::new(),
                    next_test: "Check DHCP server or try renewing lease".to_string(),
                    fix_playbook: Some("renew_dhcp".to_string()),
                });
            }

            "default_route" => {
                hypotheses.push(NetworkHypothesis {
                    id: generate_request_id(),
                    description: "Missing default route - routing misconfigured".to_string(),
                    confidence: 75,
                    supporting_evidence: vec![step.evidence_id.clone()],
                    refuting_evidence: Vec::new(),
                    next_test: "Check network manager configuration".to_string(),
                    fix_playbook: Some("restart_manager".to_string()),
                });
            }

            "ip_connectivity" => {
                hypotheses.push(NetworkHypothesis {
                    id: generate_request_id(),
                    description: "Gateway unreachable - network path issue".to_string(),
                    confidence: 70,
                    supporting_evidence: vec![step.evidence_id.clone()],
                    refuting_evidence: Vec::new(),
                    next_test: "Check gateway device (router) is working".to_string(),
                    fix_playbook: Some("restart_manager".to_string()),
                });
            }

            "dns" => {
                hypotheses.push(NetworkHypothesis {
                    id: generate_request_id(),
                    description: "DNS resolution failure - resolver misconfigured".to_string(),
                    confidence: 85,
                    supporting_evidence: vec![step.evidence_id.clone()],
                    refuting_evidence: Vec::new(),
                    next_test: "Check DNS server availability".to_string(),
                    fix_playbook: Some("restart_resolver".to_string()),
                });

                // If using systemd-resolved, suggest restart
                if evidence.dns_config_source == "systemd-resolved" {
                    hypotheses.push(NetworkHypothesis {
                        id: generate_request_id(),
                        description: "systemd-resolved may need restart".to_string(),
                        confidence: 75,
                        supporting_evidence: vec![step.evidence_id.clone()],
                        refuting_evidence: Vec::new(),
                        next_test: "Restart systemd-resolved service".to_string(),
                        fix_playbook: Some("restart_resolver".to_string()),
                    });
                }
            }

            "manager_health" => {
                if !evidence.manager_status.is_running {
                    hypotheses.push(NetworkHypothesis {
                        id: generate_request_id(),
                        description: format!(
                            "{} service not running",
                            evidence.manager_status.manager.as_str()
                        ),
                        confidence: 90,
                        supporting_evidence: vec![step.evidence_id.clone()],
                        refuting_evidence: Vec::new(),
                        next_test: "Start the network manager service".to_string(),
                        fix_playbook: Some("start_manager".to_string()),
                    });
                }

                if !evidence.manager_conflicts.is_empty() {
                    hypotheses.push(NetworkHypothesis {
                        id: generate_request_id(),
                        description: "Conflicting network managers detected".to_string(),
                        confidence: 85,
                        supporting_evidence: evidence
                            .manager_conflicts
                            .iter()
                            .map(|c| c.clone())
                            .collect(),
                        refuting_evidence: Vec::new(),
                        next_test: "Disable conflicting services".to_string(),
                        fix_playbook: Some("disable_conflicting".to_string()),
                    });
                }
            }

            _ => {}
        }
    }

    // Sort by confidence
    hypotheses.sort_by(|a, b| b.confidence.cmp(&a.confidence));

    // Limit to 3
    hypotheses.truncate(3);

    hypotheses
}

// =============================================================================
// Fix Playbooks
// =============================================================================

/// A fix playbook that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlaybook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub risk_level: FixRiskLevel,
    pub confirmation_required: bool,
    pub confirmation_phrase: String,
    pub steps: Vec<FixStep>,
    pub post_checks: Vec<String>,
    pub rollback_steps: Vec<FixStep>,
}

/// Risk level for a fix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixRiskLevel {
    Low,
    Medium,
    High,
}

impl FixRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            FixRiskLevel::Low => "low",
            FixRiskLevel::Medium => "medium",
            FixRiskLevel::High => "high",
        }
    }
}

/// A single step in a fix playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStep {
    pub description: String,
    pub command: String,
    pub requires_root: bool,
}

/// Result of applying a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub playbook_id: String,
    pub success: bool,
    pub steps_executed: usize,
    pub post_check_passed: bool,
    pub rolled_back: bool,
    pub error: Option<String>,
    pub reliability_score: u8,
}

/// Get available fix playbooks
pub fn get_fix_playbooks() -> HashMap<String, FixPlaybook> {
    let mut playbooks = HashMap::new();

    // Restart network manager
    playbooks.insert(
        "restart_manager".to_string(),
        FixPlaybook {
            id: "restart_manager".to_string(),
            name: "Restart Network Manager".to_string(),
            description: "Restart the active network manager service to reset connections"
                .to_string(),
            risk_level: FixRiskLevel::Low,
            confirmation_required: true,
            confirmation_phrase: FIX_CONFIRMATION.to_string(),
            steps: vec![FixStep {
                description: "Restart network manager service".to_string(),
                command: "systemctl restart {MANAGER_SERVICE}".to_string(),
                requires_root: true,
            }],
            post_checks: vec!["check_ip_connectivity".to_string(), "check_dns".to_string()],
            rollback_steps: Vec::new(), // No rollback needed - just restart again
        },
    );

    // Renew DHCP lease
    playbooks.insert(
        "renew_dhcp".to_string(),
        FixPlaybook {
            id: "renew_dhcp".to_string(),
            name: "Renew DHCP Lease".to_string(),
            description: "Request a new IP address from the DHCP server".to_string(),
            risk_level: FixRiskLevel::Low,
            confirmation_required: true,
            confirmation_phrase: FIX_CONFIRMATION.to_string(),
            steps: vec![FixStep {
                description: "Renew DHCP lease via network manager".to_string(),
                command: "nmcli connection down {CONNECTION} && nmcli connection up {CONNECTION}"
                    .to_string(),
                requires_root: false,
            }],
            post_checks: vec![
                "check_ip_address".to_string(),
                "check_default_route".to_string(),
            ],
            rollback_steps: Vec::new(),
        },
    );

    // Restart resolver
    playbooks.insert(
        "restart_resolver".to_string(),
        FixPlaybook {
            id: "restart_resolver".to_string(),
            name: "Restart DNS Resolver".to_string(),
            description: "Restart systemd-resolved to fix DNS issues".to_string(),
            risk_level: FixRiskLevel::Low,
            confirmation_required: true,
            confirmation_phrase: FIX_CONFIRMATION.to_string(),
            steps: vec![
                FixStep {
                    description: "Flush DNS cache".to_string(),
                    command: "resolvectl flush-caches".to_string(),
                    requires_root: false,
                },
                FixStep {
                    description: "Restart systemd-resolved".to_string(),
                    command: "systemctl restart systemd-resolved".to_string(),
                    requires_root: true,
                },
            ],
            post_checks: vec!["check_dns".to_string()],
            rollback_steps: Vec::new(),
        },
    );

    // Start network manager
    playbooks.insert(
        "start_manager".to_string(),
        FixPlaybook {
            id: "start_manager".to_string(),
            name: "Start Network Manager".to_string(),
            description: "Start the network manager service".to_string(),
            risk_level: FixRiskLevel::Low,
            confirmation_required: true,
            confirmation_phrase: FIX_CONFIRMATION.to_string(),
            steps: vec![FixStep {
                description: "Start network manager service".to_string(),
                command: "systemctl start {MANAGER_SERVICE}".to_string(),
                requires_root: true,
            }],
            post_checks: vec![
                "check_physical_link".to_string(),
                "check_ip_address".to_string(),
            ],
            rollback_steps: Vec::new(),
        },
    );

    // Disable conflicting service (high risk)
    playbooks.insert(
        "disable_conflicting".to_string(),
        FixPlaybook {
            id: "disable_conflicting".to_string(),
            name: "Disable Conflicting Network Manager".to_string(),
            description: "Stop and disable a conflicting network manager service".to_string(),
            risk_level: FixRiskLevel::High,
            confirmation_required: true,
            confirmation_phrase: FIX_CONFIRMATION.to_string(),
            steps: vec![
                FixStep {
                    description: "Stop conflicting service".to_string(),
                    command: "systemctl stop {CONFLICTING_SERVICE}".to_string(),
                    requires_root: true,
                },
                FixStep {
                    description: "Disable conflicting service".to_string(),
                    command: "systemctl disable {CONFLICTING_SERVICE}".to_string(),
                    requires_root: true,
                },
            ],
            post_checks: vec!["check_manager_health".to_string()],
            rollback_steps: vec![
                FixStep {
                    description: "Re-enable conflicting service".to_string(),
                    command: "systemctl enable {CONFLICTING_SERVICE}".to_string(),
                    requires_root: true,
                },
                FixStep {
                    description: "Start conflicting service".to_string(),
                    command: "systemctl start {CONFLICTING_SERVICE}".to_string(),
                    requires_root: true,
                },
            ],
        },
    );

    // Toggle WiFi power saving
    playbooks.insert(
        "toggle_wifi_powersave".to_string(),
        FixPlaybook {
            id: "toggle_wifi_powersave".to_string(),
            name: "Disable WiFi Power Saving".to_string(),
            description: "Disable WiFi power saving which can cause disconnects".to_string(),
            risk_level: FixRiskLevel::Low,
            confirmation_required: true,
            confirmation_phrase: FIX_CONFIRMATION.to_string(),
            steps: vec![FixStep {
                description: "Disable power saving on WiFi interface".to_string(),
                command: "iw dev {WIFI_IFACE} set power_save off".to_string(),
                requires_root: true,
            }],
            post_checks: vec!["check_physical_link".to_string()],
            rollback_steps: vec![FixStep {
                description: "Re-enable power saving".to_string(),
                command: "iw dev {WIFI_IFACE} set power_save on".to_string(),
                requires_root: true,
            }],
        },
    );

    playbooks
}

fn select_fix_playbook(
    hypotheses: &[NetworkHypothesis],
    _evidence: &NetworkEvidence,
) -> Option<FixPlaybook> {
    let playbooks = get_fix_playbooks();

    // Find first hypothesis with a fix playbook
    for hypothesis in hypotheses {
        if let Some(ref playbook_id) = hypothesis.fix_playbook {
            if let Some(playbook) = playbooks.get(playbook_id) {
                return Some(playbook.clone());
            }
        }
    }

    None
}

// =============================================================================
// Case File Integration
// =============================================================================

/// Networking doctor case file data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingDoctorCase {
    pub timestamp: DateTime<Utc>,
    pub diagnosis: DiagnosisResult,
    pub fix_applied: Option<String>,
    pub fix_result: Option<FixResult>,
    pub recipe_created: Option<String>,
}

impl NetworkingDoctorCase {
    pub fn new(diagnosis: DiagnosisResult) -> Self {
        Self {
            timestamp: Utc::now(),
            diagnosis,
            fix_applied: None,
            fix_result: None,
            recipe_created: None,
        }
    }

    pub fn with_fix(mut self, playbook_id: &str, result: FixResult) -> Self {
        self.fix_applied = Some(playbook_id.to_string());
        self.fix_result = Some(result);
        self
    }

    pub fn with_recipe(mut self, recipe_id: &str) -> Self {
        self.recipe_created = Some(recipe_id.to_string());
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// =============================================================================
// Service Status Helper
// =============================================================================

#[derive(Debug, Clone)]
struct ServiceStatus {
    is_running: bool,
    is_enabled: bool,
    has_errors: bool,
    error_summary: Option<String>,
}

fn check_service_status(service: &str) -> Option<ServiceStatus> {
    let output = Command::new("systemctl")
        .args(["is-active", service])
        .output()
        .ok()?;

    let is_running = output.status.success();

    let enabled_output = Command::new("systemctl")
        .args(["is-enabled", service])
        .output()
        .ok()?;

    let is_enabled = enabled_output.status.success();

    // Check for recent failures
    let has_errors = if is_running {
        Command::new("systemctl")
            .args(["is-failed", service])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        true
    };

    let error_summary = if has_errors && !is_running {
        Some("Service is not running".to_string())
    } else {
        None
    };

    Some(ServiceStatus {
        is_running,
        is_enabled,
        has_errors: !is_running,
        error_summary,
    })
}

// =============================================================================
// Formatting
// =============================================================================

impl DiagnosisResult {
    /// Format for transcript display
    pub fn format_transcript(&self) -> String {
        let mut lines = Vec::new();

        lines.push("=== NETWORKING DOCTOR DIAGNOSIS ===".to_string());
        lines.push(String::new());

        // Manager info
        lines.push(format!(
            "Network Manager: {} ({})",
            self.evidence.manager_status.manager.as_str(),
            if self.evidence.manager_status.is_running {
                "running"
            } else {
                "NOT RUNNING"
            }
        ));

        // Interfaces
        lines.push(String::new());
        lines.push("Interfaces:".to_string());
        for iface in &self.evidence.interfaces {
            let status = if iface.is_up { "UP" } else { "DOWN" };
            let ips = if iface.ip_addresses.is_empty() {
                "no IP".to_string()
            } else {
                iface.ip_addresses.join(", ")
            };
            let extra = if let Some(ref ssid) = iface.wifi_ssid {
                format!(" [WiFi: {}]", ssid)
            } else {
                String::new()
            };
            lines.push(format!(
                "  {} ({}) - {} - {}{}",
                iface.name, iface.iface_type, status, ips, extra
            ));
        }

        // Diagnosis steps
        lines.push(String::new());
        lines.push("Diagnosis Flow:".to_string());
        for step in &self.steps {
            lines.push(format!("  {} {}", step.result.symbol(), step.name));
            lines.push(format!("    {}", step.details));
            if !step.implication.is_empty() {
                lines.push(format!("    -> {}", step.implication));
            }
        }

        // Hypotheses
        if !self.hypotheses.is_empty() {
            lines.push(String::new());
            lines.push("Hypotheses:".to_string());
            for (i, h) in self.hypotheses.iter().enumerate() {
                lines.push(format!(
                    "  {}. {} ({}% confidence)",
                    i + 1,
                    h.description,
                    h.confidence
                ));
                if let Some(ref fix) = h.fix_playbook {
                    lines.push(format!("     Fix available: {}", fix));
                }
            }
        }

        // Recommended fix
        if let Some(ref fix) = self.recommended_fix {
            lines.push(String::new());
            lines.push("Recommended Fix:".to_string());
            lines.push(format!("  {} - {}", fix.name, fix.description));
            lines.push(format!("  Risk: {}", fix.risk_level.as_str()));
            if fix.confirmation_required {
                lines.push(format!(
                    "  To apply, confirm with: {}",
                    fix.confirmation_phrase
                ));
            }
        }

        // Overall status
        lines.push(String::new());
        lines.push(format!(
            "Overall Status: {}",
            self.overall_status.as_str().to_uppercase()
        ));

        lines.join("\n")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_manager_detection() {
        // Just test the enum works
        assert_eq!(NetworkManager::NetworkManager.as_str(), "NetworkManager");
        assert_eq!(NetworkManager::Iwd.as_str(), "iwd");
        assert_eq!(
            NetworkManager::NetworkManager.service_name(),
            Some("NetworkManager.service")
        );
    }

    #[test]
    fn test_diagnosis_step_result() {
        assert_eq!(DiagnosisStepResult::Pass.symbol(), "[OK]");
        assert_eq!(DiagnosisStepResult::Fail.symbol(), "[FAIL]");
        assert_eq!(DiagnosisStepResult::Partial.symbol(), "[WARN]");
    }

    #[test]
    fn test_diagnosis_step_creation() {
        let step = DiagnosisStep::new("test", "A test step").pass("All good", "Everything works");

        assert_eq!(step.name, "test");
        assert_eq!(step.result, DiagnosisStepResult::Pass);
        assert_eq!(step.details, "All good");
    }

    #[test]
    fn test_network_evidence_creation() {
        let evidence = NetworkEvidence::new();
        assert!(evidence.evidence_id.starts_with("NET-"));
        assert!(evidence.interfaces.is_empty());
    }

    #[test]
    fn test_fix_playbooks_exist() {
        let playbooks = get_fix_playbooks();
        assert!(playbooks.contains_key("restart_manager"));
        assert!(playbooks.contains_key("renew_dhcp"));
        assert!(playbooks.contains_key("restart_resolver"));
        assert!(playbooks.contains_key("disable_conflicting"));
    }

    #[test]
    fn test_fix_playbook_confirmation() {
        let playbooks = get_fix_playbooks();
        let restart = playbooks.get("restart_manager").unwrap();
        assert!(restart.confirmation_required);
        assert_eq!(restart.confirmation_phrase, FIX_CONFIRMATION);
    }

    #[test]
    fn test_diagnosis_status() {
        assert_eq!(DiagnosisStatus::Healthy.as_str(), "healthy");
        assert_eq!(DiagnosisStatus::Broken.as_str(), "broken");
    }

    #[test]
    fn test_hypothesis_generation_empty() {
        let evidence = NetworkEvidence::new();
        let steps = vec![DiagnosisStep::new("test", "test").pass("ok", "good")];
        let hypotheses = generate_hypotheses(&evidence, &steps);
        assert!(hypotheses.is_empty()); // No failures = no hypotheses
    }

    #[test]
    fn test_parse_interface_line() {
        let line = "2: enp0s3: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP mode DEFAULT group default qlen 1000 link/ether 08:00:27:12:34:56 brd ff:ff:ff:ff:ff:ff";
        let iface = parse_interface_line(line);
        assert!(iface.is_some());
        let iface = iface.unwrap();
        assert_eq!(iface.name, "enp0s3");
        assert_eq!(iface.iface_type, "ethernet");
        assert!(iface.is_up);
    }

    #[test]
    fn test_parse_interface_line_wifi() {
        let line = "3: wlan0: <BROADCAST,MULTICAST> mtu 1500 qdisc noop state DOWN mode DEFAULT group default qlen 1000 link/ether aa:bb:cc:dd:ee:ff brd ff:ff:ff:ff:ff:ff";
        let iface = parse_interface_line(line);
        assert!(iface.is_some());
        let iface = iface.unwrap();
        assert_eq!(iface.name, "wlan0");
        assert_eq!(iface.iface_type, "wifi");
        assert!(!iface.is_up);
    }

    #[test]
    fn test_networking_doctor_case() {
        let evidence = NetworkEvidence::new();
        let result = DiagnosisResult {
            evidence,
            steps: Vec::new(),
            hypotheses: Vec::new(),
            recommended_fix: None,
            summary: "Test".to_string(),
            overall_status: DiagnosisStatus::Healthy,
        };

        let case = NetworkingDoctorCase::new(result);
        let json = case.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_fix_risk_levels() {
        let playbooks = get_fix_playbooks();

        // restart_manager should be low risk
        let restart = playbooks.get("restart_manager").unwrap();
        assert_eq!(restart.risk_level, FixRiskLevel::Low);

        // disable_conflicting should be high risk
        let disable = playbooks.get("disable_conflicting").unwrap();
        assert_eq!(disable.risk_level, FixRiskLevel::High);
    }

    #[test]
    fn test_determine_status() {
        // All pass = healthy
        let steps = vec![
            DiagnosisStep::new("a", "a").pass("ok", ""),
            DiagnosisStep::new("b", "b").pass("ok", ""),
        ];
        assert_eq!(determine_status(&steps), DiagnosisStatus::Healthy);

        // Some fail = broken
        let steps = vec![
            DiagnosisStep::new("a", "a").fail("bad", ""),
            DiagnosisStep::new("b", "b").pass("ok", ""),
        ];
        assert_eq!(determine_status(&steps), DiagnosisStatus::Degraded);

        // All fail = broken
        let steps = vec![
            DiagnosisStep::new("a", "a").fail("bad", ""),
            DiagnosisStep::new("b", "b").fail("bad", ""),
        ];
        assert_eq!(determine_status(&steps), DiagnosisStatus::Broken);
    }
}

//! Network Diagnostics and Configuration Recipe
//!
//! Beta.152: Deterministic recipe for network diagnostics and configuration guidance
//!
//! This module focuses on read-only diagnostics and provides safe configuration guidance.
//! It does NOT directly modify network configuration (too risky for deterministic recipes).
//! Instead, it provides step-by-step instructions for manual configuration.
//!
//! Covers:
//! - Network connectivity diagnostics
//! - Interface status checks
//! - WiFi status and available networks
//! - DNS configuration checks
//! - Static IP configuration guidance (instructional, not automated)

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// Network diagnostics and configuration scenario detector
pub struct NetworkRecipe;

/// Network query types
#[derive(Debug, Clone, PartialEq)]
enum NetworkQuery {
    Connectivity,       // "am I connected to the internet"
    InterfaceStatus,    // "show my network interfaces"
    WifiStatus,         // "why is my wifi not working"
    WifiList,           // "show available wifi networks"
    DnsCheck,           // "check my DNS settings"
    StaticIpGuide,      // "configure static IP" (guidance only)
}

impl NetworkRecipe {
    /// Check if user request matches network operations
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Network-related keywords
        let has_network_context = input_lower.contains("network")
            || input_lower.contains("internet")
            || input_lower.contains("wifi")
            || input_lower.contains("ethernet")
            || input_lower.contains("connection")
            || input_lower.contains("connected")
            || input_lower.contains("ip")
            || input_lower.contains("dns")
            || input_lower.contains("ping");

        // Query/action keywords
        let has_query = input_lower.contains("check")
            || input_lower.contains("show")
            || input_lower.contains("status")
            || input_lower.contains("why")
            || input_lower.contains("not working")
            || input_lower.contains("configure")
            || input_lower.contains("setup")
            || input_lower.contains("static")
            || input_lower.contains("available")
            || input_lower.contains("list")
            || input_lower.contains("am i")
            || input_lower.contains("do i");

        has_network_context && has_query
    }

    /// Generate network diagnostics ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let query_type = Self::detect_query_type(
            telemetry
                .get("user_request")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );

        match query_type {
            NetworkQuery::Connectivity => Self::build_connectivity_plan(),
            NetworkQuery::InterfaceStatus => Self::build_interface_status_plan(),
            NetworkQuery::WifiStatus => Self::build_wifi_status_plan(),
            NetworkQuery::WifiList => Self::build_wifi_list_plan(),
            NetworkQuery::DnsCheck => Self::build_dns_check_plan(),
            NetworkQuery::StaticIpGuide => Self::build_static_ip_guide_plan(),
        }
    }

    fn detect_query_type(user_input: &str) -> NetworkQuery {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("static") && input_lower.contains("ip") {
            NetworkQuery::StaticIpGuide
        } else if input_lower.contains("dns") {
            NetworkQuery::DnsCheck
        } else if (input_lower.contains("wifi") || input_lower.contains("wireless"))
            && (input_lower.contains("available") || input_lower.contains("list"))
        {
            NetworkQuery::WifiList
        } else if input_lower.contains("wifi") || input_lower.contains("wireless") {
            NetworkQuery::WifiStatus
        } else if input_lower.contains("interface")
            || input_lower.contains("ethernet")
            || input_lower.contains("adapter")
        {
            NetworkQuery::InterfaceStatus
        } else {
            // Default to connectivity check
            NetworkQuery::Connectivity
        }
    }

    fn build_connectivity_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "ping-gateway".to_string(),
                description: "Test connectivity to default gateway (local network)".to_string(),
                command: "ping -c 3 $(ip route | grep default | awk '{print $3}')".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "ping-external-ip".to_string(),
                description: "Test connectivity to external IP (1.1.1.1 - Cloudflare DNS)".to_string(),
                command: "ping -c 3 1.1.1.1".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "ping-domain".to_string(),
                description: "Test DNS resolution and connectivity to archlinux.org".to_string(),
                command: "ping -c 3 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-routes".to_string(),
                description: "Show routing table".to_string(),
                command: "ip route show".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests network connectivity diagnostics. Running layered tests: \
                        gateway (local network) → external IP → domain (DNS).".to_string();

        let goals = vec![
            "Test local network connectivity (gateway)".to_string(),
            "Test external connectivity (internet)".to_string(),
            "Test DNS resolution".to_string(),
            "Display routing information".to_string(),
        ];

        let notes_for_user = "This diagnostic checks connectivity at multiple layers:\n\n\
             1. Gateway ping - Tests local network (router/gateway reachable)\n\
             2. External IP ping (1.1.1.1) - Tests internet connectivity without DNS\n\
             3. Domain ping (archlinux.org) - Tests DNS resolution + internet\n\n\
             Interpretation:\n\
             • All 3 succeed → Network fully functional\n\
             • Gateway succeeds, others fail → Local network OK, ISP/internet issue\n\
             • Gateway fails → Local network problem (check cable, WiFi, router)\n\
             • External IP succeeds, domain fails → DNS problem\n\n\
             Common fixes:\n\
             • Restart NetworkManager: sudo systemctl restart NetworkManager\n\
             • Check DNS: cat /etc/resolv.conf\n\
             • Reset network interface: sudo ip link set <interface> down && sudo ip link set <interface> up\n\n\
             Risk: INFO - Read-only diagnostics, no system changes".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "network_connectivity",
        )
    }

    fn build_interface_status_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "list-interfaces".to_string(),
                description: "List all network interfaces".to_string(),
                command: "ip link show".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-ip-addresses".to_string(),
                description: "Show IP addresses assigned to interfaces".to_string(),
                command: "ip addr show".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-network-manager-status".to_string(),
                description: "Check NetworkManager connection status".to_string(),
                command: "nmcli device status".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-active-connections".to_string(),
                description: "Show active network connections".to_string(),
                command: "nmcli connection show --active".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests network interface status information. \
                        Showing interface states, IP addresses, and NetworkManager status.".to_string();

        let goals = vec![
            "List all network interfaces and their states".to_string(),
            "Show IP address assignments".to_string(),
            "Display NetworkManager connection status".to_string(),
        ];

        let notes_for_user = "Network interface information:\n\n\
             Interface states:\n\
             • UP - Interface is enabled\n\
             • DOWN - Interface is disabled\n\
             • LOWER_UP - Physical link detected (cable plugged in)\n\n\
             Common interface names:\n\
             • eth0, enp* - Ethernet interfaces\n\
             • wlan0, wlp* - Wireless interfaces\n\
             • lo - Loopback interface (localhost)\n\n\
             NetworkManager commands:\n\
             • Show all connections: nmcli connection show\n\
             • Connect to WiFi: nmcli device wifi connect <SSID> password <PASSWORD>\n\
             • Disconnect: nmcli device disconnect <interface>\n\
             • Reconnect: nmcli connection up <connection-name>\n\n\
             Risk: INFO - Read-only status checks".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "network_interface_status",
        )
    }

    fn build_wifi_status_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "check-wifi-device".to_string(),
                description: "Check if WiFi device is detected and enabled".to_string(),
                command: "nmcli radio wifi".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-wifi-device-status".to_string(),
                description: "Show WiFi device details".to_string(),
                command: "nmcli device | grep wifi".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-wifi-connection".to_string(),
                description: "Show current WiFi connection details".to_string(),
                command: "nmcli connection show --active | grep wifi".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-driver".to_string(),
                description: "Check WiFi driver and kernel module".to_string(),
                command: "lspci -k | grep -A 3 -i network".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User reports WiFi connectivity issues. Running diagnostics to check \
                        WiFi device status, driver, and current connections.".to_string();

        let goals = vec![
            "Verify WiFi device is detected and enabled".to_string(),
            "Check current WiFi connection status".to_string(),
            "Verify WiFi driver is loaded".to_string(),
        ];

        let notes_for_user = "WiFi troubleshooting steps:\n\n\
             1. Check if WiFi is enabled:\n\
             nmcli radio wifi on\n\n\
             2. Scan for available networks:\n\
             nmcli device wifi list\n\n\
             3. Connect to a network:\n\
             nmcli device wifi connect \"<SSID>\" password \"<PASSWORD>\"\n\n\
             4. If device not detected:\n\
             • Check if kernel module is loaded: lsmod | grep -i wifi\n\
             • Check for hardware kill switch (laptop)\n\
             • Try restarting NetworkManager: sudo systemctl restart NetworkManager\n\n\
             5. If driver issues:\n\
             • Identify chipset: lspci -k | grep -A 3 -i network\n\
             • Install firmware (if needed): sudo pacman -S linux-firmware\n\
             • Reboot after installing firmware\n\n\
             6. Check logs for errors:\n\
             journalctl -u NetworkManager -n 50\n\n\
             Risk: INFO - Read-only diagnostics".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "network_wifi_status",
        )
    }

    fn build_wifi_list_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "scan-wifi-networks".to_string(),
                description: "Scan and list available WiFi networks".to_string(),
                command: "nmcli device wifi list".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "rescan-wifi".to_string(),
                description: "Force rescan of WiFi networks".to_string(),
                command: "nmcli device wifi rescan && sleep 2 && nmcli device wifi list".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests list of available WiFi networks. \
                        Scanning for nearby networks and displaying signal strength.".to_string();

        let goals = vec![
            "Scan for available WiFi networks".to_string(),
            "Display network SSIDs, signal strength, and security".to_string(),
        ];

        let notes_for_user = "WiFi network list shows:\n\
             • SSID - Network name\n\
             • SIGNAL - Signal strength (0-100, higher is better)\n\
             • SECURITY - Encryption type (WPA2, WPA3, etc.)\n\
             • RATE - Maximum connection speed\n\
             • FREQ - Frequency (2.4GHz or 5GHz)\n\n\
             To connect to a network:\n\
             nmcli device wifi connect \"<SSID>\" password \"<PASSWORD>\"\n\n\
             To connect and save connection:\n\
             nmcli device wifi connect \"<SSID>\" password \"<PASSWORD>\" name \"<connection-name>\"\n\n\
             For hidden networks:\n\
             nmcli connection add type wifi con-name \"<connection-name>\" ifname wlan0 ssid \"<SSID>\" wifi-sec.key-mgmt wpa-psk wifi-sec.psk \"<PASSWORD>\"\n\n\
             Risk: INFO - Read-only scan operation".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "network_wifi_list",
        )
    }

    fn build_dns_check_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "show-resolv-conf".to_string(),
                description: "Show current DNS configuration".to_string(),
                command: "cat /etc/resolv.conf".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-dns-resolution".to_string(),
                description: "Test DNS resolution with dig".to_string(),
                command: "dig archlinux.org +short || nslookup archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-systemd-resolved".to_string(),
                description: "Check if systemd-resolved is managing DNS".to_string(),
                command: "systemctl status systemd-resolved --no-pager -l".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-networkmanager-dns".to_string(),
                description: "Show NetworkManager DNS settings".to_string(),
                command: "nmcli device show | grep -i dns".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests DNS configuration diagnostics. \
                        Checking DNS servers, resolution capability, and management system.".to_string();

        let goals = vec![
            "Display current DNS server configuration".to_string(),
            "Test DNS resolution functionality".to_string(),
            "Identify DNS management system (systemd-resolved, NetworkManager, etc.)".to_string(),
        ];

        let notes_for_user = "DNS configuration on Arch Linux:\n\n\
             /etc/resolv.conf typically managed by:\n\
             • systemd-resolved (stub resolver at 127.0.0.53)\n\
             • NetworkManager (writes directly to /etc/resolv.conf)\n\
             • dhcpcd (for DHCP-managed systems)\n\n\
             To change DNS servers:\n\n\
             Option 1 - NetworkManager (temporary):\n\
             nmcli connection modify <connection-name> ipv4.dns \"1.1.1.1 8.8.8.8\"\n\
             nmcli connection up <connection-name>\n\n\
             Option 2 - NetworkManager (permanent):\n\
             nmcli connection modify <connection-name> ipv4.ignore-auto-dns yes\n\
             nmcli connection modify <connection-name> ipv4.dns \"1.1.1.1 8.8.8.8\"\n\n\
             Option 3 - systemd-resolved:\n\
             Edit /etc/systemd/resolved.conf:\n\
             [Resolve]\n\
             DNS=1.1.1.1 8.8.8.8\n\
             Then: sudo systemctl restart systemd-resolved\n\n\
             Popular DNS servers:\n\
             • Cloudflare: 1.1.1.1, 1.0.0.1\n\
             • Google: 8.8.8.8, 8.8.4.4\n\
             • Quad9: 9.9.9.9, 149.112.112.112\n\n\
             Risk: INFO - Read-only diagnostics".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "network_dns_check",
        )
    }

    fn build_static_ip_guide_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "show-current-config".to_string(),
                description: "Show current IP configuration".to_string(),
                command: "ip addr show && ip route show".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-connection-name".to_string(),
                description: "Show NetworkManager connection name".to_string(),
                command: "nmcli connection show".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests static IP configuration. This recipe provides step-by-step \
                        guidance but does NOT automatically configure (too risky for deterministic recipe).".to_string();

        let goals = vec![
            "Display current network configuration".to_string(),
            "Provide step-by-step static IP configuration instructions".to_string(),
        ];

        let notes_for_user = "⚠️ IMPORTANT: Static IP configuration is NOT automated by this recipe.\n\
             Follow these steps carefully:\n\n\
             Method 1: NetworkManager (Recommended for desktop/laptop)\n\
             ================================================================\n\n\
             1. Identify your connection name from the output above\n\
             2. Note your current gateway (ip route | grep default)\n\
             3. Configure static IP:\n\n\
             # Replace <CONNECTION> with your connection name\n\
             # Replace <IP> with desired static IP (e.g., 192.168.1.100)\n\
             # Replace <GATEWAY> with your gateway IP (e.g., 192.168.1.1)\n\n\
             sudo nmcli connection modify <CONNECTION> ipv4.addresses <IP>/24\n\
             sudo nmcli connection modify <CONNECTION> ipv4.gateway <GATEWAY>\n\
             sudo nmcli connection modify <CONNECTION> ipv4.dns \"1.1.1.1 8.8.8.8\"\n\
             sudo nmcli connection modify <CONNECTION> ipv4.method manual\n\
             sudo nmcli connection up <CONNECTION>\n\n\
             Example:\n\
             sudo nmcli connection modify \"Wired connection 1\" ipv4.addresses 192.168.1.100/24\n\
             sudo nmcli connection modify \"Wired connection 1\" ipv4.gateway 192.168.1.1\n\
             sudo nmcli connection modify \"Wired connection 1\" ipv4.dns \"1.1.1.1 8.8.8.8\"\n\
             sudo nmcli connection modify \"Wired connection 1\" ipv4.method manual\n\
             sudo nmcli connection up \"Wired connection 1\"\n\n\
             Method 2: systemd-networkd (For servers/minimal systems)\n\
             ================================================================\n\n\
             1. Create /etc/systemd/network/20-wired.network:\n\n\
             [Match]\n\
             Name=enp*\n\n\
             [Network]\n\
             Address=192.168.1.100/24\n\
             Gateway=192.168.1.1\n\
             DNS=1.1.1.1\n\
             DNS=8.8.8.8\n\n\
             2. Enable and start systemd-networkd:\n\
             sudo systemctl enable systemd-networkd\n\
             sudo systemctl start systemd-networkd\n\n\
             To revert to DHCP (NetworkManager):\n\
             ================================================================\n\
             sudo nmcli connection modify <CONNECTION> ipv4.method auto\n\
             sudo nmcli connection up <CONNECTION>\n\n\
             ⚠️ WARNING: Incorrect static IP configuration can lose network access!\n\
             Make sure you have physical access to the machine before proceeding.\n\n\
             Risk: This recipe is INFO-only (no changes made).\n\
             Actual configuration is HIGH RISK - follow instructions carefully!".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "network_static_ip_guide",
        )
    }

    fn build_action_plan(
        analysis: String,
        goals: Vec<String>,
        necessary_checks: Vec<NecessaryCheck>,
        command_plan: Vec<CommandStep>,
        rollback_plan: Vec<RollbackStep>,
        notes_for_user: String,
        template_name: &str,
    ) -> Result<ActionPlan> {
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("network.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some(template_name.to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_network_requests() {
        // Connectivity checks
        assert!(NetworkRecipe::matches_request("check internet connection"));
        assert!(NetworkRecipe::matches_request("am I connected to the network"));
        assert!(NetworkRecipe::matches_request("why is my internet not working"));

        // Interface status
        assert!(NetworkRecipe::matches_request("show network interfaces"));
        assert!(NetworkRecipe::matches_request("check ethernet status"));

        // WiFi
        assert!(NetworkRecipe::matches_request("show available wifi networks"));
        assert!(NetworkRecipe::matches_request("why is my wifi not working"));
        assert!(NetworkRecipe::matches_request("list wifi"));

        // DNS
        assert!(NetworkRecipe::matches_request("check DNS settings"));
        assert!(NetworkRecipe::matches_request("show my DNS configuration"));

        // Static IP
        assert!(NetworkRecipe::matches_request("configure static IP"));
        assert!(NetworkRecipe::matches_request("setup static network"));

        // Should not match
        assert!(!NetworkRecipe::matches_request("what is networking"));
        assert!(!NetworkRecipe::matches_request("install network tools"));
    }

    #[test]
    fn test_query_type_detection() {
        assert_eq!(
            NetworkRecipe::detect_query_type("am I connected to the internet"),
            NetworkQuery::Connectivity
        );
        assert_eq!(
            NetworkRecipe::detect_query_type("show network interfaces"),
            NetworkQuery::InterfaceStatus
        );
        assert_eq!(
            NetworkRecipe::detect_query_type("why is my wifi not working"),
            NetworkQuery::WifiStatus
        );
        assert_eq!(
            NetworkRecipe::detect_query_type("list available wifi networks"),
            NetworkQuery::WifiList
        );
        assert_eq!(
            NetworkRecipe::detect_query_type("check my DNS settings"),
            NetworkQuery::DnsCheck
        );
        assert_eq!(
            NetworkRecipe::detect_query_type("configure static IP address"),
            NetworkQuery::StaticIpGuide
        );
    }

    #[test]
    fn test_connectivity_plan_is_read_only() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check internet".to_string());

        let plan = NetworkRecipe::build_plan(&telemetry).unwrap();

        // All commands should be INFO level (read-only)
        for cmd in &plan.command_plan {
            assert_eq!(cmd.risk_level, RiskLevel::Info);
            assert!(!cmd.requires_confirmation);
        }

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "network_connectivity");
    }

    #[test]
    fn test_static_ip_guide_is_instructional_only() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "configure static IP".to_string());

        let plan = NetworkRecipe::build_plan(&telemetry).unwrap();

        // Should only show current config, not modify anything
        assert_eq!(plan.command_plan.len(), 2);
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert_eq!(plan.command_plan[1].risk_level, RiskLevel::Info);

        // Notes should contain instructions but not automated commands
        assert!(plan.notes_for_user.contains("⚠️ IMPORTANT"));
        assert!(plan.notes_for_user.contains("NOT automated"));
        assert!(plan.notes_for_user.contains("Follow these steps"));
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "network_static_ip_guide");
    }

    #[test]
    fn test_wifi_list_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "show available wifi".to_string());

        let plan = NetworkRecipe::build_plan(&telemetry).unwrap();

        // Should include nmcli wifi list command
        assert!(plan.command_plan.iter().any(|cmd| cmd.command.contains("nmcli device wifi list")));
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "network_wifi_list");
    }
}

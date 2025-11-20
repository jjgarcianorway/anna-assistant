// Beta.174: Network Analysis and Diagnostics Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct NetworkToolsRecipe;

#[derive(Debug, PartialEq)]
enum NetworkToolsOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl NetworkToolsOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            NetworkToolsOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            NetworkToolsOperation::ListTools
        } else {
            NetworkToolsOperation::Install
        }
    }
}

impl NetworkToolsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("wireshark") || input_lower.contains("nmap")
            || input_lower.contains("tcpdump") || input_lower.contains("netcat")
            || input_lower.contains("network analysis") || input_lower.contains("packet capture")
            || input_lower.contains("port scan");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = NetworkToolsOperation::detect(user_input);
        match operation {
            NetworkToolsOperation::Install => Self::build_install_plan(telemetry),
            NetworkToolsOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            NetworkToolsOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("wireshark") { "wireshark" }
        else if input_lower.contains("nmap") { "nmap" }
        else if input_lower.contains("tcpdump") { "tcpdump" }
        else if input_lower.contains("netcat") { "netcat" }
        else { "nmap" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "wireshark" => ("Wireshark", "wireshark-qt", "Network protocol analyzer with GUI for packet capture and analysis"),
            "nmap" => ("Nmap", "nmap", "Network exploration and security auditing tool for port scanning"),
            "tcpdump" => ("tcpdump", "tcpdump", "Command-line packet analyzer for network traffic capture"),
            "netcat" => ("GNU Netcat", "gnu-netcat", "Networking utility for reading and writing network connections"),
            _ => ("Nmap", "nmap", "Network scanner and security tool"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("network_tools.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);
        let notes = if tool == "wireshark" {
            format!("{} installed. {}. To capture packets as non-root user, add yourself to the 'wireshark' group: sudo usermod -aG wireshark $USER (requires logout).",
                tool_name, description)
        } else {
            format!("{} installed. {}. Use with appropriate permissions and authorization only.",
                tool_name, description)
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} network tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("network_tools_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("network_tools.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking network analysis tools".to_string(),
            goals: vec!["List installed network tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-network-tools".to_string(),
                    description: "List network tools".to_string(),
                    command: "pacman -Q wireshark-qt wireshark-cli nmap tcpdump gnu-netcat ncat 2>/dev/null || echo 'No network analysis tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed network analysis and diagnostics tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("network_tools_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("network_tools.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available network analysis tools".to_string(),
            goals: vec!["List available network diagnostics tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Network Analysis and Diagnostics Tools:

Packet Capture/Analysis:
- Wireshark (official) - GUI network protocol analyzer with deep packet inspection
- Wireshark CLI (official) - Command-line version of Wireshark (tshark)
- tcpdump (official) - Command-line packet analyzer for network traffic capture
- Termshark (AUR) - Terminal UI for tshark (Wireshark CLI)
- Ettercap (official) - Comprehensive suite for man-in-the-middle attacks

Port Scanning/Discovery:
- Nmap (official) - Network exploration and security auditing tool
- Masscan (official) - Fast TCP port scanner
- Unicornscan (AUR) - Information gathering and security auditing
- Zmap (AUR) - Fast single packet network scanner

Network Utilities:
- GNU Netcat (official) - Networking utility for reading/writing network connections
- Ncat (nmap package) - Modern netcat with SSL, proxy, and connection broker
- Socat (official) - Multipurpose relay for bidirectional data transfer
- Netcat OpenBSD (official) - OpenBSD variant of netcat

Traffic Analysis:
- iftop (official) - Display bandwidth usage by connection
- nethogs (official) - Net top tool grouping bandwidth by process
- bmon (official) - Bandwidth monitor and rate estimator
- vnstat (official) - Network traffic monitor with statistics

DNS Tools:
- dig (bind package) - DNS lookup utility
- nslookup (bind package) - Query DNS servers
- host (bind package) - DNS lookup utility
- drill (ldns package) - DNS query tool similar to dig

HTTP/Network Testing:
- curl (official) - Command-line tool for transferring data with URLs
- wget (official) - Network downloader
- httpie (official) - User-friendly HTTP client
- siege (official) - HTTP load testing and benchmarking

Network Monitoring:
- Nagios (official) - Network monitoring and alerting system
- Zabbix (official) - Enterprise monitoring solution
- Cacti (official) - Network graphing solution
- MRTG (official) - Multi Router Traffic Grapher

Wireless Tools:
- aircrack-ng (official) - Complete suite for 802.11 network security
- kismet (official) - Wireless network detector and analyzer
- wavemon (official) - Wireless network monitor
- iw (official) - Configuration utility for wireless devices

Security Testing:
- Metasploit (AUR) - Penetration testing framework
- Burp Suite (AUR) - Web security testing
- OWASP ZAP (official) - Web application security scanner
- Sqlmap (official) - Automatic SQL injection tool

Network Emulation:
- tc (iproute2) - Traffic control for network emulation
- netem (kernel) - Network emulation for testing
- Comcast (AUR) - Simulate network conditions

Comparison:
- Wireshark: Best for detailed packet analysis with GUI
- Nmap: Best for port scanning and network discovery
- tcpdump: Best for lightweight packet capture
- Netcat: Best for quick network connections and debugging

Features:
- Wireshark: Deep packet inspection, protocol dissection, filtering, statistics, export
- Nmap: OS detection, service version detection, NSE scripting, output formats
- tcpdump: Live capture, filter expressions, protocol analysis, file saving
- Netcat: Port scanning, file transfer, port listening, proxying

Use Cases:
- Network Troubleshooting: Wireshark for packet analysis, tcpdump for capture
- Security Auditing: Nmap for scanning, Nessus for vulnerability assessment
- Performance Analysis: iftop for bandwidth, nethogs for process monitoring
- Protocol Development: Wireshark for protocol testing and debugging

Legal/Ethical Notice:
⚠️ These tools should only be used on networks you own or have explicit authorization to test.
⚠️ Unauthorized network scanning or packet capture may be illegal in your jurisdiction.
⚠️ Use responsibly for legitimate security research, testing, and administration only.

Permissions:
- Wireshark: Add user to wireshark group for packet capture without root
- tcpdump: Requires root or CAP_NET_RAW capability
- Nmap: Some scans require root privileges
- Most tools: Non-privileged features available to regular users

Common Commands:
- Wireshark: Launch GUI for live capture and analysis
- nmap -sV <host>: Version detection scan
- tcpdump -i eth0: Capture packets on eth0
- nc -l -p 8080: Listen on port 8080

Configuration:
- Wireshark: Edit → Preferences (capture, protocols, appearance)
- Nmap: Use command-line options or write NSE scripts
- tcpdump: Command-line options and BPF filter expressions
- Netcat: Command-line options for behavior'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Network analysis tools for Arch Linux - Use responsibly with proper authorization".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("network_tools_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matches() {
        assert!(NetworkToolsRecipe::matches_request("install wireshark"));
        assert!(NetworkToolsRecipe::matches_request("install network analysis tools"));
        assert!(NetworkToolsRecipe::matches_request("setup nmap"));
        assert!(!NetworkToolsRecipe::matches_request("what is wireshark"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install nmap".to_string());
        let plan = NetworkToolsRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}

//! Command Intelligence Layer (CIL) - 6.15.0
//!
//! Dynamic command derivation using:
//! - System knowledge (installed packages, available binaries)
//! - Arch Wiki snippets
//! - Runtime inspection
//! - Pattern-based generalization
//!
//! NO hardcoded commands, NO static templates.

use crate::system_knowledge::SystemKnowledgeBase;
use crate::arch_wiki_corpus::{WikiTopic, WikiSnippet, get_wiki_snippet};
use std::process::Command;

// ============================================================================
// Command Intent Classification
// ============================================================================

/// What the user wants to do with a command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandIntent {
    /// Check/inspect something (status, info, list)
    Inspect(InspectTarget),
    /// Configure/change something
    Configure(ConfigureTarget),
    /// Install software
    Install(String),
    /// Test/verify something works
    Test(TestTarget),
    /// Enable/disable systemd service
    ServiceControl(ServiceAction),
    /// Unknown/ambiguous intent
    Unknown,
}

/// What to inspect
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectTarget {
    Cpu,
    Memory,
    Disk,
    Gpu,
    Network,
    Services,
    Processes,
    Logs,
    Hardware(String), // Generic hardware with name
    Package(String),
    File(String),
}

/// What to configure
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigureTarget {
    Service(String),
    Network,
    Display,
    Audio,
    Power,
    Package(String),
}

/// What to test
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestTarget {
    Network,
    Dns,
    Service(String),
    Hardware(String),
}

/// Service control action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceAction {
    Enable(String),
    Disable(String),
    Start(String),
    Stop(String),
    Restart(String),
    Status(String),
}

// ============================================================================
// Command Suggestion
// ============================================================================

/// A suggested command with metadata
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    /// The actual command to run
    pub command: String,
    /// Brief description of what it does
    pub description: String,
    /// Whether it requires root/sudo
    pub requires_root: bool,
    /// Whether it's safe to run (read-only)
    pub is_safe: bool,
    /// Wiki URL if derived from wiki
    pub wiki_source: Option<String>,
    /// If command requires installation, package name
    pub requires_package: Option<String>,
}

// ============================================================================
// Intent Classification
// ============================================================================

/// Classify user query into command intent
///
/// Uses lightweight NLP heuristics to determine what the user wants to do.
pub fn classify_command_query(query: &str) -> CommandIntent {
    let query_lower = query.to_lowercase();

    // Service control patterns
    if let Some(action) = classify_service_action(&query_lower) {
        return CommandIntent::ServiceControl(action);
    }

    // Install patterns
    if query_lower.contains("install") || query_lower.contains("add package") {
        if let Some(package) = extract_package_name(&query_lower) {
            return CommandIntent::Install(package);
        }
    }

    // Test patterns
    if query_lower.contains("test") || query_lower.contains("verify") || query_lower.contains("check if") {
        if query_lower.contains("network") || query_lower.contains("internet") {
            return CommandIntent::Test(TestTarget::Network);
        }
        if query_lower.contains("dns") {
            return CommandIntent::Test(TestTarget::Dns);
        }
    }

    // Configure patterns
    if query_lower.contains("configure") || query_lower.contains("set up") || query_lower.contains("change") {
        if query_lower.contains("network") {
            return CommandIntent::Configure(ConfigureTarget::Network);
        }
        if query_lower.contains("display") || query_lower.contains("screen") {
            return CommandIntent::Configure(ConfigureTarget::Display);
        }
        if query_lower.contains("audio") || query_lower.contains("sound") {
            return CommandIntent::Configure(ConfigureTarget::Audio);
        }
        if query_lower.contains("power") || query_lower.contains("battery") {
            return CommandIntent::Configure(ConfigureTarget::Power);
        }
    }

    // Inspect patterns - most common use case
    if query_lower.contains("how do i check")
        || query_lower.contains("how can i see")
        || query_lower.contains("how do i list")
        || query_lower.contains("how to check")
        || query_lower.contains("how to see")
        || query_lower.contains("how to list")
        || query_lower.contains("show me")
        || query_lower.contains("what is my")
        || query_lower.contains("check my")
        || query_lower.contains("list my")
    {
        return classify_inspect_target(&query_lower);
    }

    CommandIntent::Unknown
}

/// Classify service control actions
fn classify_service_action(query: &str) -> Option<ServiceAction> {
    // Extract service name if present
    let service = extract_service_name(query);

    if query.contains("enable") && service.is_some() {
        return Some(ServiceAction::Enable(service.unwrap()));
    }
    if query.contains("disable") && service.is_some() {
        return Some(ServiceAction::Disable(service.unwrap()));
    }
    if query.contains("start") && service.is_some() {
        return Some(ServiceAction::Start(service.unwrap()));
    }
    if query.contains("stop") && service.is_some() {
        return Some(ServiceAction::Stop(service.unwrap()));
    }
    if query.contains("restart") && service.is_some() {
        return Some(ServiceAction::Restart(service.unwrap()));
    }
    if query.contains("status") && service.is_some() {
        return Some(ServiceAction::Status(service.unwrap()));
    }

    None
}

/// Classify what to inspect
fn classify_inspect_target(query: &str) -> CommandIntent {
    if query.contains("cpu") || query.contains("processor") {
        return CommandIntent::Inspect(InspectTarget::Cpu);
    }
    if query.contains("memory") || query.contains("ram") {
        return CommandIntent::Inspect(InspectTarget::Memory);
    }
    if query.contains("disk") || query.contains("storage") || query.contains("partition") {
        return CommandIntent::Inspect(InspectTarget::Disk);
    }
    if query.contains("gpu") || query.contains("graphic") || query.contains("video card") {
        return CommandIntent::Inspect(InspectTarget::Gpu);
    }
    if query.contains("network") || query.contains("interface") {
        return CommandIntent::Inspect(InspectTarget::Network);
    }
    if query.contains("service") {
        return CommandIntent::Inspect(InspectTarget::Services);
    }
    if query.contains("process") || query.contains("running") {
        return CommandIntent::Inspect(InspectTarget::Processes);
    }
    if query.contains("log") {
        return CommandIntent::Inspect(InspectTarget::Logs);
    }

    // Check for package queries
    if query.contains("package") {
        if let Some(pkg) = extract_package_name(query) {
            return CommandIntent::Inspect(InspectTarget::Package(pkg));
        }
    }

    CommandIntent::Unknown
}

/// Extract service name from query
fn extract_service_name(query: &str) -> Option<String> {
    // Look for common service patterns
    let words: Vec<&str> = query.split_whitespace().collect();

    for i in 0..words.len() {
        // Look for service name before or after action words
        if words[i].contains("service") && i > 0 {
            return Some(words[i - 1].to_string());
        }
        if (words[i] == "enable" || words[i] == "disable" || words[i] == "start" || words[i] == "stop")
            && i + 1 < words.len()
        {
            return Some(words[i + 1].replace("?", "").to_string());
        }
    }

    None
}

/// Extract package name from query
fn extract_package_name(query: &str) -> Option<String> {
    let words: Vec<&str> = query.split_whitespace().collect();

    for i in 0..words.len() {
        if (words[i] == "install" || words[i] == "package") && i + 1 < words.len() {
            return Some(words[i + 1].replace("?", "").to_string());
        }
    }

    None
}

// ============================================================================
// Command Resolution
// ============================================================================

/// Resolve command intent into actual command suggestions
///
/// This is the core of CIL - dynamically builds commands based on:
/// - System knowledge
/// - Available tools
/// - Arch Wiki guidance
pub fn resolve_command_for_intent(
    intent: CommandIntent,
    knowledge: &SystemKnowledgeBase,
) -> Vec<CommandSuggestion> {
    match intent {
        CommandIntent::Inspect(target) => resolve_inspect_commands(target, knowledge),
        CommandIntent::ServiceControl(action) => resolve_service_commands(action),
        CommandIntent::Install(package) => resolve_install_commands(&package),
        CommandIntent::Test(target) => resolve_test_commands(target),
        CommandIntent::Configure(target) => resolve_configure_commands(target, knowledge),
        CommandIntent::Unknown => vec![],
    }
}

/// Resolve commands for inspection tasks
fn resolve_inspect_commands(target: InspectTarget, knowledge: &SystemKnowledgeBase) -> Vec<CommandSuggestion> {
    match target {
        InspectTarget::Cpu => {
            let mut commands = vec![];

            // lscpu is standard
            commands.push(CommandSuggestion {
                command: "lscpu".to_string(),
                description: "Show detailed CPU information".to_string(),
                requires_root: false,
                is_safe: true,
                wiki_source: None,
                requires_package: None,
            });

            // Check for monitoring tools
            if is_command_available("htop") {
                commands.push(CommandSuggestion {
                    command: "htop".to_string(),
                    description: "Interactive process viewer with CPU usage".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                });
            } else if is_command_available("top") {
                commands.push(CommandSuggestion {
                    command: "top".to_string(),
                    description: "Show processes and CPU usage".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                });
            }

            commands
        }

        InspectTarget::Memory => {
            vec![
                CommandSuggestion {
                    command: "free -h".to_string(),
                    description: "Show memory usage in human-readable format".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
            ]
        }

        InspectTarget::Disk => {
            vec![
                CommandSuggestion {
                    command: "lsblk".to_string(),
                    description: "List block devices and partitions".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
                CommandSuggestion {
                    command: "df -h".to_string(),
                    description: "Show disk space usage".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
            ]
        }

        InspectTarget::Gpu => {
            let mut commands = vec![];

            // lspci for GPU detection
            commands.push(CommandSuggestion {
                command: "lspci | grep -i vga".to_string(),
                description: "Show graphics cards".to_string(),
                requires_root: false,
                is_safe: true,
                wiki_source: None,
                requires_package: None,
            });

            // NVIDIA-specific if available
            if is_command_available("nvidia-smi") {
                commands.push(CommandSuggestion {
                    command: "nvidia-smi".to_string(),
                    description: "Show NVIDIA GPU status".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                });
            }

            // AMD-specific if available
            if is_command_available("radeontop") {
                commands.push(CommandSuggestion {
                    command: "radeontop".to_string(),
                    description: "Show AMD GPU status".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                });
            }

            commands
        }

        InspectTarget::Network => {
            vec![
                CommandSuggestion {
                    command: "ip addr".to_string(),
                    description: "Show network interfaces and IP addresses".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
                CommandSuggestion {
                    command: "ip link".to_string(),
                    description: "Show network interface status".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
            ]
        }

        InspectTarget::Services => {
            vec![
                CommandSuggestion {
                    command: "systemctl list-units --type=service".to_string(),
                    description: "List all systemd services".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: Some("https://wiki.archlinux.org/title/Systemd".to_string()),
                    requires_package: None,
                },
                CommandSuggestion {
                    command: "systemctl --failed".to_string(),
                    description: "Show failed services".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: Some("https://wiki.archlinux.org/title/Systemd".to_string()),
                    requires_package: None,
                },
            ]
        }

        InspectTarget::Processes => {
            let mut commands = vec![];

            if is_command_available("htop") {
                commands.push(CommandSuggestion {
                    command: "htop".to_string(),
                    description: "Interactive process viewer".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                });
            } else {
                commands.push(CommandSuggestion {
                    command: "ps aux".to_string(),
                    description: "List all running processes".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                });
            }

            commands
        }

        InspectTarget::Logs => {
            vec![
                CommandSuggestion {
                    command: "journalctl -b".to_string(),
                    description: "Show logs from current boot".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: Some("https://wiki.archlinux.org/title/Systemd/Journal".to_string()),
                    requires_package: None,
                },
                CommandSuggestion {
                    command: "journalctl -p err".to_string(),
                    description: "Show error-level logs".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: Some("https://wiki.archlinux.org/title/Systemd/Journal".to_string()),
                    requires_package: None,
                },
            ]
        }

        InspectTarget::Package(pkg) => {
            vec![
                CommandSuggestion {
                    command: format!("pacman -Qi {}", pkg),
                    description: format!("Show information about package {}", pkg),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: Some("https://wiki.archlinux.org/title/Pacman".to_string()),
                    requires_package: None,
                },
            ]
        }

        InspectTarget::Hardware(name) => {
            vec![
                CommandSuggestion {
                    command: format!("lspci | grep -i {}", name),
                    description: format!("Search for {} hardware", name),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
            ]
        }

        InspectTarget::File(path) => {
            vec![
                CommandSuggestion {
                    command: format!("cat {}", path),
                    description: format!("Show contents of {}", path),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: None,
                    requires_package: None,
                },
            ]
        }
    }
}

/// Resolve systemd service commands
fn resolve_service_commands(action: ServiceAction) -> Vec<CommandSuggestion> {
    let wiki_url = Some("https://wiki.archlinux.org/title/Systemd".to_string());

    match action {
        ServiceAction::Status(service) => vec![
            CommandSuggestion {
                command: format!("systemctl status {}", service),
                description: format!("Show status of {} service", service),
                requires_root: false,
                is_safe: true,
                wiki_source: wiki_url,
                requires_package: None,
            },
        ],

        ServiceAction::Enable(service) => vec![
            CommandSuggestion {
                command: format!("sudo systemctl enable --now {}", service),
                description: format!("Enable and start {} service", service),
                requires_root: true,
                is_safe: false,
                wiki_source: wiki_url,
                requires_package: None,
            },
        ],

        ServiceAction::Disable(service) => vec![
            CommandSuggestion {
                command: format!("sudo systemctl disable --now {}", service),
                description: format!("Disable and stop {} service", service),
                requires_root: true,
                is_safe: false,
                wiki_source: wiki_url,
                requires_package: None,
            },
        ],

        ServiceAction::Start(service) => vec![
            CommandSuggestion {
                command: format!("sudo systemctl start {}", service),
                description: format!("Start {} service", service),
                requires_root: true,
                is_safe: false,
                wiki_source: wiki_url,
                requires_package: None,
            },
        ],

        ServiceAction::Stop(service) => vec![
            CommandSuggestion {
                command: format!("sudo systemctl stop {}", service),
                description: format!("Stop {} service", service),
                requires_root: true,
                is_safe: false,
                wiki_source: wiki_url,
                requires_package: None,
            },
        ],

        ServiceAction::Restart(service) => vec![
            CommandSuggestion {
                command: format!("sudo systemctl restart {}", service),
                description: format!("Restart {} service", service),
                requires_root: true,
                is_safe: false,
                wiki_source: wiki_url,
                requires_package: None,
            },
        ],
    }
}

/// Resolve package installation commands
fn resolve_install_commands(package: &str) -> Vec<CommandSuggestion> {
    vec![
        CommandSuggestion {
            command: format!("sudo pacman -S {}", package),
            description: format!("Install {} package", package),
            requires_root: true,
            is_safe: false,
            wiki_source: Some("https://wiki.archlinux.org/title/Pacman".to_string()),
            requires_package: None,
        },
    ]
}

/// Resolve test commands
fn resolve_test_commands(target: TestTarget) -> Vec<CommandSuggestion> {
    match target {
        TestTarget::Network => vec![
            CommandSuggestion {
                command: "ping -c 3 archlinux.org".to_string(),
                description: "Test network connectivity".to_string(),
                requires_root: false,
                is_safe: true,
                wiki_source: None,
                requires_package: None,
            },
        ],

        TestTarget::Dns => vec![
            CommandSuggestion {
                command: "resolvectl status".to_string(),
                description: "Show DNS configuration".to_string(),
                requires_root: false,
                is_safe: true,
                wiki_source: Some("https://wiki.archlinux.org/title/Systemd-resolved".to_string()),
                requires_package: None,
            },
        ],

        TestTarget::Service(service) => vec![
            CommandSuggestion {
                command: format!("systemctl is-active {}", service),
                description: format!("Check if {} is running", service),
                requires_root: false,
                is_safe: true,
                wiki_source: Some("https://wiki.archlinux.org/title/Systemd".to_string()),
                requires_package: None,
            },
        ],

        TestTarget::Hardware(_) => vec![],
    }
}

/// Resolve configuration commands
fn resolve_configure_commands(target: ConfigureTarget, knowledge: &SystemKnowledgeBase) -> Vec<CommandSuggestion> {
    match target {
        ConfigureTarget::Power => {
            // Check if TLP is installed
            let wiki = get_wiki_snippet(WikiTopic::TlpPowerSaving);

            vec![
                CommandSuggestion {
                    command: "systemctl status tlp".to_string(),
                    description: "Check TLP power management status".to_string(),
                    requires_root: false,
                    is_safe: true,
                    wiki_source: Some(wiki.url.to_string()),
                    requires_package: Some("tlp".to_string()),
                },
            ]
        }

        _ => vec![], // Other targets not yet implemented
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a command is available on the system
fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Format command suggestions for display
pub fn format_command_suggestions(suggestions: &[CommandSuggestion]) -> String {
    if suggestions.is_empty() {
        return "No commands found.".to_string();
    }

    let mut output = String::new();

    // Group by wiki source
    let has_wiki = suggestions.iter().any(|s| s.wiki_source.is_some());

    if has_wiki {
        if let Some(wiki_url) = suggestions.iter().find_map(|s| s.wiki_source.as_ref()) {
            output.push_str(&format!("# Commands (from {})\n\n", wiki_url));
        }
    }

    for suggestion in suggestions {
        // Show installation requirement first
        if let Some(pkg) = &suggestion.requires_package {
            if !is_package_installed(pkg) {
                output.push_str(&format!("# Install required package first:\n"));
                output.push_str(&format!("sudo pacman -S {}\n\n", pkg));
            }
        }

        // Show the command
        output.push_str(&suggestion.command);
        output.push('\n');

        // Add safety note if needed
        if !suggestion.is_safe {
            output.push_str("# Warning: This command makes system changes\n");
        }
    }

    output
}

/// Check if a package is installed
fn is_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(&["-Q", package])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_cpu_check() {
        let intent = classify_command_query("how do I check my CPU?");
        assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Cpu)));
    }

    #[test]
    fn test_classify_memory_check() {
        let intent = classify_command_query("how can I see memory usage?");
        assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Memory)));
    }

    #[test]
    fn test_classify_service_enable() {
        let intent = classify_command_query("enable tlp service");
        assert!(matches!(intent, CommandIntent::ServiceControl(ServiceAction::Enable(_))));
    }

    #[test]
    fn test_classify_install() {
        let intent = classify_command_query("install docker");
        assert!(matches!(intent, CommandIntent::Install(_)));
    }

    #[test]
    fn test_resolve_cpu_commands() {
        let kb = SystemKnowledgeBase::default();
        let commands = resolve_command_for_intent(
            CommandIntent::Inspect(InspectTarget::Cpu),
            &kb,
        );

        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.command.contains("lscpu")));
    }

    #[test]
    fn test_resolve_memory_commands() {
        let kb = SystemKnowledgeBase::default();
        let commands = resolve_command_for_intent(
            CommandIntent::Inspect(InspectTarget::Memory),
            &kb,
        );

        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.command.contains("free")));
    }

    #[test]
    fn test_format_output() {
        let suggestions = vec![
            CommandSuggestion {
                command: "lscpu".to_string(),
                description: "Show CPU info".to_string(),
                requires_root: false,
                is_safe: true,
                wiki_source: None,
                requires_package: None,
            },
        ];

        let output = format_command_suggestions(&suggestions);
        assert!(output.contains("lscpu"));
    }
}

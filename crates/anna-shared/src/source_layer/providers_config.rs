//! Local Config Provider and Intent Mapping - v0.0.443.
//!
//! Provides access to local config files and maps intents to canonical commands.

use crate::source_layer::providers_types::IntentCommands;

/// Local config provider.
pub struct LocalConfigProvider;

impl LocalConfigProvider {
    /// Common config paths.
    pub fn common_paths(name: &str) -> Vec<String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

        vec![
            format!("/etc/{}", name),
            format!("{}/.config/{}", home, name),
            format!("{}/.{}", home, name),
            format!("/etc/{}.conf", name),
            format!("{}/.{}rc", home, name),
        ]
    }

    /// Find config file.
    pub fn find(name: &str) -> Option<String> {
        for path in Self::common_paths(name) {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
        None
    }

    /// Read config file.
    pub fn read(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
    }
}

/// Get canonical commands for an intent.
pub fn commands_for_intent(intent: &str) -> Option<IntentCommands> {
    let (commands, wiki) = match intent {
        "packages.update_system" | "system_update" => (
            vec!["pacman", "checkupdates"],
            vec!["Pacman", "System_maintenance"],
        ),
        "packages.install" | "package_install" => (vec!["pacman"], vec!["Pacman"]),
        "services.failed_services" | "services_failed" => (vec!["systemctl"], vec!["Systemd"]),
        "boot.boot_time" | "boot_time" => (
            vec!["systemd-analyze"],
            vec!["Improving_performance/Boot_process"],
        ),
        "network.dns_check" | "dns_check" => (
            vec!["dig", "nslookup", "resolvectl"],
            vec!["Domain_name_resolution"],
        ),
        "security.firewall_status" | "firewall_status" => (
            vec!["firewall-cmd", "ufw", "iptables"],
            vec!["Firewalld", "Uncomplicated_Firewall"],
        ),
        "memory.status" | "memory_free" => (vec!["free"], vec!["Swap"]),
        "disk.usage" | "disk_free" => (vec!["df", "du"], vec!["File_systems"]),
        _ => return None,
    };

    Some(IntentCommands {
        intent: intent.to_string(),
        commands: commands.into_iter().map(String::from).collect(),
        wiki_pages: wiki.into_iter().map(String::from).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands_for_intent() {
        let cmds = commands_for_intent("packages.update_system").unwrap();
        assert!(cmds.commands.contains(&"pacman".to_string()));
        assert!(cmds.wiki_pages.contains(&"Pacman".to_string()));
    }
}

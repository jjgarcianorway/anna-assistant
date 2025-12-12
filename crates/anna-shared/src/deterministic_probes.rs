//! Deterministic Probe Mapping (v0.0.448).
//!
//! CORE PRINCIPLE: Common intents get deterministic probes, not LLM guesses.
//!
//! This module maps specific question intents to exact probe sets.
//! The translator should check this FIRST before falling back to domain-based selection.
//!
//! Why deterministic:
//! - "which service uses most CPU?" → must run top_cpu, NOT cpu_info
//! - "do I have swap?" → must run swap_files, NOT pacman -Q swap
//! - "what is my vim setup?" → must run vimrc_content + nvim_config, NOT memory_info

use std::collections::HashMap;

/// A deterministic probe rule.
#[derive(Debug, Clone)]
pub struct ProbeRule {
    /// Intent ID for matching.
    pub intent_id: &'static str,
    /// Keywords that trigger this rule (all must match).
    pub keywords: &'static [&'static str],
    /// Negative keywords (if any match, rule doesn't apply).
    pub negative_keywords: &'static [&'static str],
    /// Exact probes to run (in order).
    pub probes: &'static [&'static str],
    /// Description for debugging.
    pub description: &'static str,
}

/// Registry of deterministic probe rules.
pub struct DeterministicProbeRegistry {
    rules: Vec<ProbeRule>,
}

impl DeterministicProbeRegistry {
    /// Create registry with all rules.
    pub fn new() -> Self {
        Self {
            rules: vec![
                // === CPU QUERIES ===
                ProbeRule {
                    intent_id: "cpu.top_process",
                    keywords: &["cpu", "most"],
                    negative_keywords: &["info", "what cpu", "which cpu"],
                    probes: &["top_cpu"],
                    description: "Which process uses most CPU",
                },
                ProbeRule {
                    intent_id: "cpu.top_process_alt",
                    keywords: &["using", "cpu"],
                    negative_keywords: &["info"],
                    probes: &["top_cpu"],
                    description: "What is using CPU",
                },
                ProbeRule {
                    intent_id: "cpu.info",
                    keywords: &["what", "cpu"],
                    negative_keywords: &["using", "most"],
                    probes: &["cpu_info"],
                    description: "CPU hardware info",
                },
                // === MEMORY QUERIES ===
                ProbeRule {
                    intent_id: "memory.free",
                    keywords: &["free", "ram"],
                    negative_keywords: &[],
                    probes: &["memory_info"],
                    description: "Free RAM",
                },
                ProbeRule {
                    intent_id: "memory.free_alt",
                    keywords: &["memory", "available"],
                    negative_keywords: &[],
                    probes: &["memory_info"],
                    description: "Available memory",
                },
                ProbeRule {
                    intent_id: "memory.top_process",
                    keywords: &["memory", "most"],
                    negative_keywords: &[],
                    probes: &["top_memory"],
                    description: "Which process uses most memory",
                },
                ProbeRule {
                    intent_id: "memory.top_process_alt",
                    keywords: &["using", "memory"],
                    negative_keywords: &[],
                    probes: &["top_memory"],
                    description: "What is using memory",
                },
                // === SWAP QUERIES (NOT PACKAGES!) ===
                ProbeRule {
                    intent_id: "swap.status",
                    keywords: &["swap"],
                    negative_keywords: &["install", "package"],
                    probes: &["swap_files", "memory_info"],
                    description: "Swap configuration status",
                },
                ProbeRule {
                    intent_id: "swap.have",
                    keywords: &["have", "swap"],
                    negative_keywords: &["install"],
                    probes: &["swap_files", "memory_info"],
                    description: "Do I have swap",
                },
                // === BLUETOOTH QUERIES ===
                ProbeRule {
                    intent_id: "bluetooth.status",
                    keywords: &["bluetooth"],
                    negative_keywords: &["install", "package"],
                    probes: &["bluetooth_service", "bluetooth_devices"],
                    description: "Bluetooth status",
                },
                ProbeRule {
                    intent_id: "bluetooth.enabled",
                    keywords: &["bluetooth", "enabled"],
                    negative_keywords: &[],
                    probes: &["bluetooth_service", "bluetooth_devices"],
                    description: "Is bluetooth enabled",
                },
                ProbeRule {
                    intent_id: "bluetooth.working",
                    keywords: &["bluetooth", "working"],
                    negative_keywords: &[],
                    probes: &["bluetooth_service", "bluetooth_devices"],
                    description: "Is bluetooth working",
                },
                // === EDITOR/CONFIG QUERIES ===
                ProbeRule {
                    intent_id: "vim.setup",
                    keywords: &["vim"],
                    negative_keywords: &["install", "package"],
                    probes: &[
                        "command_v_vim",
                        "command_v_nvim",
                        "vimrc_content",
                        "nvim_config",
                    ],
                    description: "Vim/nvim setup",
                },
                ProbeRule {
                    intent_id: "neovim.setup",
                    keywords: &["neovim"],
                    negative_keywords: &["install"],
                    probes: &["command_v_nvim", "nvim_config"],
                    description: "Neovim setup",
                },
                ProbeRule {
                    intent_id: "nvim.setup",
                    keywords: &["nvim"],
                    negative_keywords: &["install"],
                    probes: &["command_v_nvim", "nvim_config"],
                    description: "Nvim setup",
                },
                ProbeRule {
                    intent_id: "editor.setup",
                    keywords: &["editor", "setup"],
                    negative_keywords: &[],
                    probes: &[
                        "command_v_vim",
                        "command_v_nvim",
                        "command_v_nano",
                        "command_v_emacs",
                        "command_v_code",
                    ],
                    description: "Editor setup",
                },
                ProbeRule {
                    intent_id: "bash.config",
                    keywords: &["bashrc"],
                    negative_keywords: &[],
                    probes: &["bashrc_content"],
                    description: "Bashrc content",
                },
                ProbeRule {
                    intent_id: "zsh.config",
                    keywords: &["zshrc"],
                    negative_keywords: &[],
                    probes: &["zshrc_content"],
                    description: "Zshrc content",
                },
                // === BOOT QUERIES ===
                ProbeRule {
                    intent_id: "boot.slow",
                    keywords: &["boot", "slow"],
                    negative_keywords: &[],
                    probes: &["boot_time", "boot_blame", "failed_services"],
                    description: "Why is boot slow",
                },
                ProbeRule {
                    intent_id: "boot.time",
                    keywords: &["boot", "time"],
                    negative_keywords: &[],
                    probes: &["boot_time", "boot_blame"],
                    description: "Boot time analysis",
                },
                ProbeRule {
                    intent_id: "boot.analyze",
                    keywords: &["boot"],
                    negative_keywords: &["loader"],
                    probes: &["boot_time", "boot_blame"],
                    description: "Boot analysis",
                },
                // === SERVICE QUERIES ===
                ProbeRule {
                    intent_id: "services.failed",
                    keywords: &["failed", "service"],
                    negative_keywords: &[],
                    probes: &["failed_services"],
                    description: "Failed services",
                },
                ProbeRule {
                    intent_id: "services.running",
                    keywords: &["running", "service"],
                    negative_keywords: &[],
                    probes: &["running_services"],
                    description: "Running services",
                },
                ProbeRule {
                    intent_id: "services.status",
                    keywords: &["service", "status"],
                    negative_keywords: &[],
                    probes: &["running_services", "failed_services"],
                    description: "Service status",
                },
                // === DISK/STORAGE QUERIES ===
                ProbeRule {
                    intent_id: "disk.usage",
                    keywords: &["disk", "usage"],
                    negative_keywords: &[],
                    probes: &["disk_usage", "findmnt"],
                    description: "Disk usage",
                },
                ProbeRule {
                    intent_id: "disk.space",
                    keywords: &["disk", "space"],
                    negative_keywords: &[],
                    probes: &["disk_usage", "findmnt"],
                    description: "Disk space",
                },
                ProbeRule {
                    intent_id: "disk.free",
                    keywords: &["free", "disk"],
                    negative_keywords: &[],
                    probes: &["disk_usage"],
                    description: "Free disk space",
                },
                ProbeRule {
                    intent_id: "storage.largest",
                    keywords: &["largest", "folder"],
                    negative_keywords: &[],
                    probes: &["largest_dirs", "largest_home"],
                    description: "Largest folders",
                },
                ProbeRule {
                    intent_id: "storage.biggest",
                    keywords: &["biggest"],
                    negative_keywords: &[],
                    probes: &["largest_dirs", "largest_home"],
                    description: "Biggest folders",
                },
                // === NETWORK QUERIES ===
                ProbeRule {
                    intent_id: "network.ip",
                    keywords: &["ip", "address"],
                    negative_keywords: &[],
                    probes: &["network_addrs"],
                    description: "IP addresses",
                },
                ProbeRule {
                    intent_id: "network.ports",
                    keywords: &["listening", "port"],
                    negative_keywords: &[],
                    probes: &["listening_ports"],
                    description: "Listening ports",
                },
                ProbeRule {
                    intent_id: "network.dns",
                    keywords: &["dns"],
                    negative_keywords: &[],
                    probes: &["dns_servers"],
                    description: "DNS servers",
                },
                ProbeRule {
                    intent_id: "network.connection",
                    keywords: &["internet", "connection"],
                    negative_keywords: &[],
                    probes: &["ping_check", "network_addrs", "dns_servers"],
                    description: "Internet connection status",
                },
                ProbeRule {
                    intent_id: "network.wifi",
                    keywords: &["wifi"],
                    negative_keywords: &[],
                    probes: &["wireless_networks", "network_addrs"],
                    description: "WiFi status",
                },
                // === DESKTOP QUERIES ===
                ProbeRule {
                    intent_id: "desktop.wallpaper",
                    keywords: &["wallpaper"],
                    negative_keywords: &[],
                    probes: &["desktop_wallpaper", "desktop_session"],
                    description: "Wallpaper location",
                },
                ProbeRule {
                    intent_id: "desktop.session",
                    keywords: &["desktop", "environment"],
                    negative_keywords: &[],
                    probes: &["desktop_session", "installed_desktops"],
                    description: "Desktop environment",
                },
                ProbeRule {
                    intent_id: "display.server",
                    keywords: &["wayland"],
                    negative_keywords: &[],
                    probes: &["display_server"],
                    description: "Display server (Wayland/X11)",
                },
                ProbeRule {
                    intent_id: "display.server_alt",
                    keywords: &["x11"],
                    negative_keywords: &[],
                    probes: &["display_server"],
                    description: "Display server (X11)",
                },
                // === GPU/GRAPHICS QUERIES ===
                ProbeRule {
                    intent_id: "gpu.info",
                    keywords: &["gpu"],
                    negative_keywords: &["install"],
                    probes: &["gpu_info", "gpu_drivers", "glxinfo_renderer"],
                    description: "GPU information",
                },
                ProbeRule {
                    intent_id: "gpu.driver",
                    keywords: &["graphics", "driver"],
                    negative_keywords: &[],
                    probes: &["gpu_drivers", "gpu_info"],
                    description: "Graphics drivers",
                },
                ProbeRule {
                    intent_id: "gpu.acceleration",
                    keywords: &["hardware", "acceleration"],
                    negative_keywords: &[],
                    probes: &[
                        "vaapi_status",
                        "vdpau_status",
                        "vulkan_status",
                        "glxinfo_renderer",
                    ],
                    description: "Hardware acceleration",
                },
                // === AUDIO QUERIES ===
                ProbeRule {
                    intent_id: "audio.status",
                    keywords: &["audio"],
                    negative_keywords: &["install"],
                    probes: &["audio_devices", "audio_server"],
                    description: "Audio status",
                },
                ProbeRule {
                    intent_id: "audio.sound",
                    keywords: &["sound"],
                    negative_keywords: &["install"],
                    probes: &["audio_devices", "audio_server"],
                    description: "Sound status",
                },
                // === SECURITY QUERIES ===
                ProbeRule {
                    intent_id: "security.firewall",
                    keywords: &["firewall"],
                    negative_keywords: &[],
                    probes: &["firewall_status", "iptables_rules"],
                    description: "Firewall status",
                },
                ProbeRule {
                    intent_id: "security.logins",
                    keywords: &["login", "failed"],
                    negative_keywords: &[],
                    probes: &["failed_logins", "last_logins"],
                    description: "Failed logins",
                },
                ProbeRule {
                    intent_id: "security.ssh",
                    keywords: &["ssh", "connection"],
                    negative_keywords: &[],
                    probes: &["ssh_connections", "listening_ports"],
                    description: "SSH connections",
                },
                // === SYSTEM INFO QUERIES ===
                ProbeRule {
                    intent_id: "system.uptime",
                    keywords: &["uptime"],
                    negative_keywords: &[],
                    probes: &["uptime"],
                    description: "System uptime",
                },
                ProbeRule {
                    intent_id: "system.kernel",
                    keywords: &["kernel"],
                    negative_keywords: &["install"],
                    probes: &["uname", "installed_kernels"],
                    description: "Kernel info",
                },
                ProbeRule {
                    intent_id: "system.os",
                    keywords: &["os", "version"],
                    negative_keywords: &[],
                    probes: &["os_release", "uname"],
                    description: "OS version",
                },
                ProbeRule {
                    intent_id: "system.distro",
                    keywords: &["distro"],
                    negative_keywords: &[],
                    probes: &["os_release"],
                    description: "Distribution info",
                },
                // === DOCKER QUERIES ===
                ProbeRule {
                    intent_id: "docker.containers",
                    keywords: &["docker", "container"],
                    negative_keywords: &["install"],
                    probes: &["docker_containers"],
                    description: "Docker containers",
                },
                ProbeRule {
                    intent_id: "docker.images",
                    keywords: &["docker", "image"],
                    negative_keywords: &["install"],
                    probes: &["docker_images"],
                    description: "Docker images",
                },
                // === TEMPERATURE/SENSORS QUERIES ===
                ProbeRule {
                    intent_id: "sensors.temp",
                    keywords: &["temperature"],
                    negative_keywords: &[],
                    probes: &["sensors_temp"],
                    description: "System temperature",
                },
                ProbeRule {
                    intent_id: "sensors.cpu_temp",
                    keywords: &["cpu", "temp"],
                    negative_keywords: &[],
                    probes: &["sensors_temp"],
                    description: "CPU temperature",
                },
                // === UPDATES QUERIES ===
                ProbeRule {
                    intent_id: "updates.available",
                    keywords: &["update"],
                    negative_keywords: &["install"],
                    probes: &["package_updates"],
                    description: "Available updates",
                },
                ProbeRule {
                    intent_id: "updates.packages",
                    keywords: &["packages", "update"],
                    negative_keywords: &[],
                    probes: &["package_updates"],
                    description: "Package updates",
                },
                // === JOURNAL/LOGS QUERIES ===
                ProbeRule {
                    intent_id: "logs.errors",
                    keywords: &["error", "log"],
                    negative_keywords: &[],
                    probes: &["journal_errors", "dmesg_errors"],
                    description: "Error logs",
                },
                ProbeRule {
                    intent_id: "logs.warnings",
                    keywords: &["warning", "log"],
                    negative_keywords: &[],
                    probes: &["journal_warnings"],
                    description: "Warning logs",
                },
                ProbeRule {
                    intent_id: "logs.recent",
                    keywords: &["recent", "log"],
                    negative_keywords: &[],
                    probes: &["systemd_journal"],
                    description: "Recent logs",
                },
            ],
        }
    }

    /// Find matching rule for a query.
    /// Returns the first rule where all keywords match and no negative keywords match.
    pub fn find_rule(&self, query: &str) -> Option<&ProbeRule> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        for rule in &self.rules {
            // Check all keywords present
            let all_keywords_match = rule
                .keywords
                .iter()
                .all(|kw| query_words.iter().any(|w| w.contains(kw)));

            if !all_keywords_match {
                continue;
            }

            // Check no negative keywords present
            let no_negative_match = rule
                .negative_keywords
                .iter()
                .all(|nkw| !query_words.iter().any(|w| w.contains(nkw)));

            if no_negative_match {
                return Some(rule);
            }
        }

        None
    }

    /// Get probes for a query. Returns None if no deterministic rule matches.
    pub fn get_probes(&self, query: &str) -> Option<Vec<&'static str>> {
        self.find_rule(query).map(|rule| rule.probes.to_vec())
    }

    /// Check if query should NEVER be treated as a package query.
    /// These are concept queries that happen to contain words that might be package names.
    pub fn is_concept_not_package(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Words that are concepts, not packages
        let concept_words = [
            "swap",
            "games",
            "apps",
            "tools",
            "utils",
            "drivers",
            "audio",
            "sound",
            "video",
            "network",
            "bluetooth",
            "wifi",
            "graphics",
            "display",
            "desktop",
            "fonts",
            "themes",
        ];

        // Package verbs that indicate actual package intent
        let package_verbs = [
            "install",
            "remove",
            "uninstall",
            "update",
            "upgrade",
            "pacman",
            "apt",
            "yum",
        ];

        let has_concept_word = concept_words.iter().any(|w| query_lower.contains(w));
        let has_package_verb = package_verbs.iter().any(|v| query_lower.contains(v));

        // If it has a concept word but no package verb, it's a concept query
        has_concept_word && !has_package_verb
    }
}

impl Default for DeterministicProbeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a query matches a deterministic probe rule.
/// Returns the probes if matched, None otherwise.
pub fn deterministic_probes_for_query(query: &str) -> Option<Vec<&'static str>> {
    DeterministicProbeRegistry::new().get_probes(query)
}

/// Check if query is a concept (not a package query).
pub fn is_concept_query(query: &str) -> bool {
    DeterministicProbeRegistry::new().is_concept_not_package(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_most_maps_to_top_cpu() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("which service is using the most CPU?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(
            probes.contains(&"top_cpu"),
            "Should contain top_cpu, got {:?}",
            probes
        );
        assert!(!probes.contains(&"cpu_info"), "Should NOT contain cpu_info");
    }

    #[test]
    fn test_what_cpu_maps_to_cpu_info() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("what CPU do I have?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"cpu_info"));
    }

    #[test]
    fn test_swap_maps_to_swap_files_not_package() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("do I have swap?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"swap_files"), "Should contain swap_files");
        assert!(!probes.iter().any(|p| p.contains("pacman")));
    }

    #[test]
    fn test_vim_setup_maps_to_config_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("what is my vim setup?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"vimrc_content") || probes.contains(&"nvim_config"));
    }

    #[test]
    fn test_bluetooth_maps_to_bluetooth_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("is bluetooth enabled and working?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"bluetooth_service"));
    }

    #[test]
    fn test_boot_slow_maps_to_boot_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("why is my boot slow?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"boot_time"));
        assert!(probes.contains(&"boot_blame"));
    }

    #[test]
    fn test_free_ram_maps_to_memory_info() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("how much free ram do I have?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"memory_info"));
    }

    #[test]
    fn test_wallpaper_maps_to_desktop_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("where are my wallpapers?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"desktop_wallpaper"));
    }

    #[test]
    fn test_concept_not_package() {
        let registry = DeterministicProbeRegistry::new();

        // Concepts that should NOT be package queries
        assert!(registry.is_concept_not_package("do I have swap?"));
        assert!(registry.is_concept_not_package("is bluetooth working?"));
        assert!(registry.is_concept_not_package("how is my audio?"));

        // Actual package queries
        assert!(!registry.is_concept_not_package("install firefox"));
        assert!(!registry.is_concept_not_package("pacman -S vim"));
    }

    #[test]
    fn test_package_install_not_blocked() {
        let registry = DeterministicProbeRegistry::new();

        // "install vim" should NOT match vim.setup (has negative keyword "install")
        let probes = registry.get_probes("install vim");
        // Either no match (goes to package flow) or different intent
        if let Some(probes) = probes {
            assert!(!probes.contains(&"vimrc_content"));
        }
    }
}

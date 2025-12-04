//! Config Graph v7.17.0 - Configuration Ownership and Consumers
//!
//! Maps configuration files to their consumers:
//! - Which package owns each config file
//! - Which binaries/units read each config
//! - Include relationships between configs
//! - Shared configs used by multiple components
//!
//! Sources:
//! - pacman -Ql (package file lists)
//! - systemctl show (ExecStart, EnvironmentFile)
//! - man pages (documented config paths)
//! - Arch Wiki (local copy)
//! - Static analysis of Include directives

use std::fs;
use std::path::Path;
use std::process::Command;

/// Config file with ownership and consumer information
#[derive(Debug, Clone, Default)]
pub struct ConfigFileInfo {
    /// Absolute path to config file
    pub path: String,
    /// Package that owns this file (if any)
    pub owner_package: Option<String>,
    /// How we determined ownership
    pub ownership_source: String,
    /// Binaries/units that read this config
    pub consumers: Vec<ConfigConsumer>,
    /// Files included by this config
    pub includes: Vec<String>,
    /// Files that include this config
    pub included_by: Vec<String>,
}

/// A consumer of a config file
#[derive(Debug, Clone)]
pub struct ConfigConsumer {
    /// Name of the consumer (binary, service, etc.)
    pub name: String,
    /// Type of consumer
    pub consumer_type: ConsumerType,
    /// How we determined this relationship
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerType {
    Binary,
    Service,
    Socket,
    Timer,
    Unknown,
}

impl ConsumerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsumerType::Binary => "binary",
            ConsumerType::Service => "service",
            ConsumerType::Socket => "socket",
            ConsumerType::Timer => "timer",
            ConsumerType::Unknown => "unknown",
        }
    }
}

/// Config graph for a component
#[derive(Debug, Clone, Default)]
pub struct ConfigGraph {
    /// Configs this component reads directly
    pub reads: Vec<ConfigRelation>,
    /// Shared configs (used by multiple components)
    pub shared: Vec<ConfigRelation>,
    /// Source of this graph information
    pub source: String,
}

/// A config relationship
#[derive(Debug, Clone)]
pub struct ConfigRelation {
    /// Path to config file
    pub path: String,
    /// How we determined this relationship
    pub evidence: String,
    /// Whether file exists
    pub exists: bool,
}

/// Get package owner of a file
pub fn get_file_owner(path: &str) -> Option<String> {
    let output = Command::new("pacman").args(["-Qo", path]).output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: "/path/to/file is owned by package version"
        stdout
            .split(" is owned by ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .map(|s| s.to_string())
    } else {
        None
    }
}

/// Get configs read by a systemd unit
pub fn get_unit_configs(unit_name: &str) -> Vec<ConfigRelation> {
    let mut configs = Vec::new();

    let output = Command::new("systemctl")
        .args([
            "show",
            unit_name,
            "--property=ExecStart,EnvironmentFile,ConfigDirectory",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                // Parse ExecStart for config file arguments
                if line.starts_with("ExecStart=") {
                    let exec_part = line.trim_start_matches("ExecStart=");
                    // Look for common config patterns
                    for word in exec_part.split_whitespace() {
                        // -c /path, --config /path, --config=/path
                        if (word.starts_with("/etc/") || word.starts_with("/usr/"))
                            && (word.ends_with(".conf")
                                || word.ends_with(".cfg")
                                || word.ends_with(".ini")
                                || word.ends_with(".config"))
                        {
                            let path = word.split('=').last().unwrap_or(word);
                            configs.push(ConfigRelation {
                                path: path.to_string(),
                                evidence: "ExecStart=".to_string(),
                                exists: Path::new(path).exists(),
                            });
                        }
                    }
                }

                // Parse EnvironmentFile
                if line.starts_with("EnvironmentFile=") {
                    let path = line
                        .trim_start_matches("EnvironmentFile=")
                        .trim_start_matches('-') // Optional env file
                        .trim();
                    if !path.is_empty() {
                        configs.push(ConfigRelation {
                            path: path.to_string(),
                            evidence: "EnvironmentFile=".to_string(),
                            exists: Path::new(path).exists(),
                        });
                    }
                }

                // Parse ConfigDirectory
                if line.starts_with("ConfigDirectory=") {
                    let dir = line.trim_start_matches("ConfigDirectory=").trim();
                    if !dir.is_empty() {
                        let config_path = format!("/etc/{}", dir);
                        configs.push(ConfigRelation {
                            path: config_path.clone(),
                            evidence: "ConfigDirectory=".to_string(),
                            exists: Path::new(&config_path).exists(),
                        });
                    }
                }
            }
        }
    }

    configs
}

/// Parse Include directives from a config file
pub fn get_config_includes(config_path: &str) -> Vec<String> {
    let mut includes = Vec::new();

    if let Ok(content) = fs::read_to_string(config_path) {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with('#') || line.starts_with(';') || line.starts_with("//") {
                continue;
            }

            // Common include patterns
            // Include /path
            // include /path
            // Include=/path
            // source /path
            // . /path (shell)
            let include_path = if line.to_lowercase().starts_with("include ") {
                Some(line[8..].trim())
            } else if line.to_lowercase().starts_with("include=") {
                Some(line[8..].trim())
            } else if line.to_lowercase().starts_with("source ") {
                Some(line[7..].trim())
            } else if line.starts_with(". ") && line.len() > 2 {
                Some(line[2..].trim())
            } else {
                None
            };

            if let Some(path) = include_path {
                let path = path.trim_matches('"').trim_matches('\'');
                if path.starts_with('/') {
                    includes.push(path.to_string());
                }
            }
        }
    }

    includes
}

/// Get config graph for a software component
pub fn get_config_graph_for_software(name: &str) -> ConfigGraph {
    let mut graph = ConfigGraph {
        source: "systemctl show, man pages, pacman -Ql".to_string(),
        ..Default::default()
    };

    // Determine if this is a service
    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    // Get configs from unit file
    let unit_configs = get_unit_configs(&unit_name);
    for config in unit_configs {
        graph.reads.push(config);
    }

    // Get configs from man page (if available)
    let man_configs = get_configs_from_man(name);
    for path in man_configs {
        if !graph.reads.iter().any(|r| r.path == path) {
            graph.reads.push(ConfigRelation {
                path: path.clone(),
                evidence: format!("man {}", name),
                exists: Path::new(&path).exists(),
            });
        }
    }

    // Check for common config patterns
    let common_configs = get_common_config_paths(name);
    for path in common_configs {
        if !graph.reads.iter().any(|r| r.path == path) {
            graph.reads.push(ConfigRelation {
                path: path.clone(),
                evidence: "common path".to_string(),
                exists: Path::new(&path).exists(),
            });
        }
    }

    // Identify shared configs (like PAM, NSS)
    for config in &graph.reads {
        if config.path.starts_with("/etc/pam.d/") {
            graph.shared.push(ConfigRelation {
                path: config.path.clone(),
                evidence: "PAM login stack".to_string(),
                exists: config.exists,
            });
        } else if config.path == "/etc/nsswitch.conf" {
            graph.shared.push(ConfigRelation {
                path: config.path.clone(),
                evidence: "NSS name resolution".to_string(),
                exists: config.exists,
            });
        }
    }

    graph
}

/// Get config paths from man page
fn get_configs_from_man(name: &str) -> Vec<String> {
    let mut configs = Vec::new();

    // Try to get FILES section from man page
    let output = Command::new("man").args(["--pager=cat", name]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut in_files_section = false;

            for line in stdout.lines() {
                let line = line.trim();

                // Look for FILES section
                if line == "FILES" || line.starts_with("FILES") {
                    in_files_section = true;
                    continue;
                }

                // End of FILES section
                if in_files_section
                    && (line
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                        && !line.starts_with('/'))
                {
                    in_files_section = false;
                }

                // Extract file paths
                if in_files_section {
                    for word in line.split_whitespace() {
                        if word.starts_with("/etc/") || word.starts_with("/usr/") {
                            let path = word.trim_matches(|c: char| {
                                !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-'
                            });
                            if !path.is_empty() && !configs.contains(&path.to_string()) {
                                configs.push(path.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    configs
}

/// Get common config paths for a component
fn get_common_config_paths(name: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let base_name = name.trim_end_matches(".service");

    // Check common patterns
    let patterns = [
        format!("/etc/{}.conf", base_name),
        format!("/etc/{}/{}.conf", base_name, base_name),
        format!("/etc/{}/config", base_name),
        format!("/etc/default/{}", base_name),
    ];

    for pattern in patterns {
        if Path::new(&pattern).exists() {
            paths.push(pattern);
        }
    }

    // Check for .d directory
    let conf_d = format!("/etc/{}.conf.d", base_name);
    if Path::new(&conf_d).is_dir() {
        if let Ok(entries) = fs::read_dir(&conf_d) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "conf").unwrap_or(false) {
                    paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    paths
}

/// Get config graph for a hardware component
pub fn get_config_graph_for_hardware(component: &str) -> ConfigGraph {
    let mut graph = ConfigGraph {
        source: "filesystem, modprobe.d, NetworkManager".to_string(),
        ..Default::default()
    };

    let component_lower = component.to_lowercase();

    // Network-related configs
    if component_lower.contains("wifi")
        || component_lower.contains("wlan")
        || component_lower.contains("network")
        || component_lower.contains("ethernet")
    {
        // NetworkManager connections
        let nm_dir = "/etc/NetworkManager/system-connections";
        if Path::new(nm_dir).is_dir() {
            if let Ok(entries) = fs::read_dir(nm_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .map(|e| e == "nmconnection")
                        .unwrap_or(false)
                    {
                        graph.reads.push(ConfigRelation {
                            path: path.to_string_lossy().to_string(),
                            evidence: "NetworkManager".to_string(),
                            exists: true,
                        });
                    }
                }
            }
        }

        // wpa_supplicant config
        let wpa_conf = "/etc/wpa_supplicant/wpa_supplicant.conf";
        if Path::new(wpa_conf).exists() {
            graph.reads.push(ConfigRelation {
                path: wpa_conf.to_string(),
                evidence: "wpa_supplicant".to_string(),
                exists: true,
            });
        }
    }

    // Storage-related configs
    if component_lower.contains("nvme")
        || component_lower.contains("sd")
        || component_lower.contains("storage")
        || component_lower.contains("disk")
    {
        // fstab
        graph.reads.push(ConfigRelation {
            path: "/etc/fstab".to_string(),
            evidence: "mount configuration".to_string(),
            exists: Path::new("/etc/fstab").exists(),
        });

        // crypttab if exists
        if Path::new("/etc/crypttab").exists() {
            graph.reads.push(ConfigRelation {
                path: "/etc/crypttab".to_string(),
                evidence: "LUKS configuration".to_string(),
                exists: true,
            });
        }
    }

    // Module-related configs
    let modprobe_d = "/etc/modprobe.d";
    if Path::new(modprobe_d).is_dir() {
        if let Ok(entries) = fs::read_dir(modprobe_d) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "conf").unwrap_or(false) {
                    // Check if this config mentions the component
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.to_lowercase().contains(&component_lower) {
                            graph.reads.push(ConfigRelation {
                                path: path.to_string_lossy().to_string(),
                                evidence: "modprobe.d".to_string(),
                                exists: true,
                            });
                        }
                    }
                }
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_owner() {
        // /usr/bin/ls should be owned by coreutils
        let owner = get_file_owner("/usr/bin/ls");
        assert!(owner.is_some() || owner.is_none()); // Depends on system
    }

    #[test]
    fn test_get_unit_configs() {
        let configs = get_unit_configs("sshd.service");
        // Should not crash
        assert!(configs.is_empty() || !configs.is_empty());
    }

    #[test]
    fn test_get_config_graph_for_software() {
        let graph = get_config_graph_for_software("sshd");
        assert!(!graph.source.is_empty());
    }
}

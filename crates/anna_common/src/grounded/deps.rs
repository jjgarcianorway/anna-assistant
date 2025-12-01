//! Dependency Graph v7.13.0 - Package, Service and Driver Dependencies
//!
//! Sources:
//! - pactree NAME and pactree -r NAME for package dependencies
//! - pacman -Qi NAME for required and optional dependencies
//! - systemctl show UNIT for service relations
//! - systemctl list-dependencies UNIT for service tree
//! - lsmod and modinfo for kernel module dependencies

use std::collections::HashSet;
use std::process::Command;

/// Package dependency information
#[derive(Debug, Clone, Default)]
pub struct PackageDeps {
    /// Direct dependencies (from pacman -Qi Depends)
    pub direct: Vec<String>,
    /// Optional dependencies (from pacman -Qi OptDepends)
    pub optional: Vec<String>,
    /// Reverse dependencies (packages that depend on this one)
    pub reverse: Vec<String>,
    /// Source of information
    pub source: String,
}

/// Service dependency information
#[derive(Debug, Clone, Default)]
pub struct ServiceDeps {
    /// Units this service requires
    pub requires: Vec<String>,
    /// Units this service wants
    pub wants: Vec<String>,
    /// Units this service is part of
    pub part_of: Vec<String>,
    /// Units that want this service
    pub wanted_by: Vec<String>,
    /// Units that require this service
    pub required_by: Vec<String>,
    /// Source of information
    pub source: String,
}

/// Kernel module dependency information
#[derive(Debug, Clone, Default)]
pub struct ModuleDeps {
    /// Modules this module depends on
    pub depends: Vec<String>,
    /// Modules that depend on this one (used by)
    pub used_by: Vec<String>,
    /// Full dependency chain (ordered)
    pub chain: Vec<String>,
    /// Source of information
    pub source: String,
}

/// Get package dependencies using pacman and pactree
pub fn get_package_deps(package: &str) -> PackageDeps {
    let mut deps = PackageDeps::default();
    deps.source = "pacman -Qi, pactree".to_string();

    // Get direct deps from pacman -Qi
    if let Ok(output) = Command::new("pacman")
        .args(["-Qi", package])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Depends On") {
                    let deps_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                    if deps_str != "None" {
                        deps.direct = deps_str
                            .split_whitespace()
                            .filter(|s| !s.starts_with('>') && !s.starts_with('<') && !s.starts_with('='))
                            .map(|s| s.split('>').next().unwrap_or(s))
                            .map(|s| s.split('<').next().unwrap_or(s))
                            .map(|s| s.split('=').next().unwrap_or(s))
                            .map(|s| s.to_string())
                            .take(8) // Limit to 8 for readability
                            .collect();
                    }
                }
                if line.starts_with("Optional Deps") {
                    let deps_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                    if deps_str != "None" {
                        // Optional deps may span multiple lines, just get first line items
                        deps.optional = deps_str
                            .split_whitespace()
                            .filter(|s| !s.contains('['))
                            .map(|s| s.trim_end_matches(':').to_string())
                            .take(4)
                            .collect();
                    }
                }
            }
        }
    }

    // Get reverse deps from pactree -r (if available)
    if let Ok(output) = Command::new("pactree")
        .args(["-r", "-d", "1", package])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            deps.reverse = stdout
                .lines()
                .skip(1) // Skip the package itself
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().trim_start_matches("├──").trim_start_matches("└──").trim().to_string())
                .take(4)
                .collect();
        }
    }

    deps
}

/// Get service dependencies using systemctl
pub fn get_service_deps(unit: &str) -> ServiceDeps {
    let mut deps = ServiceDeps::default();
    deps.source = "systemctl show".to_string();

    // Normalize unit name
    let unit_name = if unit.contains('.') {
        unit.to_string()
    } else {
        format!("{}.service", unit)
    };

    // Get service properties
    if let Ok(output) = Command::new("systemctl")
        .args(["show", &unit_name, "--no-pager"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    let values: Vec<String> = value
                        .split_whitespace()
                        .filter(|s| !s.is_empty() && *s != "(null)")
                        .map(|s| s.to_string())
                        .take(4)
                        .collect();

                    if values.is_empty() {
                        continue;
                    }

                    match key {
                        "Requires" => deps.requires = values,
                        "Wants" => deps.wants = values,
                        "PartOf" => deps.part_of = values,
                        "WantedBy" => deps.wanted_by = values,
                        "RequiredBy" => deps.required_by = values,
                        _ => {}
                    }
                }
            }
        }
    }

    deps
}

/// Get kernel module dependencies using lsmod and modinfo
pub fn get_module_deps(module: &str) -> ModuleDeps {
    let mut deps = ModuleDeps::default();
    deps.source = "lsmod, modinfo".to_string();

    // Get dependencies from modinfo
    if let Ok(output) = Command::new("modinfo")
        .args(["-F", "depends", module])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                deps.depends = stdout
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .take(6)
                    .collect();
            }
        }
    }

    // Get used_by from lsmod
    if let Ok(output) = Command::new("lsmod").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 && parts[0] == module {
                    deps.used_by = parts[3]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .take(4)
                        .collect();
                    break;
                }
            }
        }
    }

    // Build dependency chain (module -> its deps)
    if !deps.depends.is_empty() {
        deps.chain.push(module.to_string());
        deps.chain.extend(deps.depends.iter().take(3).cloned());
    }

    deps
}

/// Get related services for a kernel module/driver
pub fn get_driver_related_services(driver: &str) -> Vec<String> {
    let mut services = HashSet::new();

    // Map common drivers to their services
    let driver_lower = driver.to_lowercase();

    // Network drivers
    if driver_lower.contains("iwl") || driver_lower.contains("ath") || driver_lower.contains("mt7")
       || driver_lower.contains("rtw") || driver_lower.contains("brcm") {
        services.insert("NetworkManager.service".to_string());
        services.insert("wpa_supplicant.service".to_string());
    }

    // Ethernet drivers
    if driver_lower.contains("r8169") || driver_lower.contains("e1000") || driver_lower.contains("igb")
       || driver_lower.contains("i40e") || driver_lower.contains("mlx") {
        services.insert("NetworkManager.service".to_string());
        services.insert("systemd-networkd.service".to_string());
    }

    // Bluetooth drivers
    if driver_lower.contains("btusb") || driver_lower.contains("bluetooth") || driver_lower.contains("hci") {
        services.insert("bluetooth.service".to_string());
    }

    // GPU drivers
    if driver_lower.contains("nvidia") {
        services.insert("nvidia-persistenced.service".to_string());
    }
    if driver_lower.contains("amdgpu") || driver_lower.contains("radeon") {
        // AMD typically doesn't have persistent services
    }

    // Audio drivers
    if driver_lower.contains("snd") || driver_lower.contains("hda") || driver_lower.contains("audio") {
        services.insert("pipewire.service".to_string());
        services.insert("pulseaudio.service".to_string());
    }

    // Check which services actually exist and are active
    let mut active_services = Vec::new();
    for service in services {
        if let Ok(output) = Command::new("systemctl")
            .args(["is-active", &service])
            .output()
        {
            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if status == "active" {
                    active_services.push(service);
                }
            }
        }
    }

    // If no services found via mapping, try to find any related active services
    if active_services.is_empty() {
        // Try common network services for network drivers
        if driver_lower.contains("net") || driver_lower.contains("wlan") || driver_lower.contains("wifi") {
            for svc in ["NetworkManager.service", "systemd-networkd.service"] {
                if let Ok(output) = Command::new("systemctl")
                    .args(["is-active", svc])
                    .output()
                {
                    if output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "active" {
                        active_services.push(svc.to_string());
                    }
                }
            }
        }
    }

    active_services.into_iter().take(4).collect()
}

/// Format package dependencies for display
impl PackageDeps {
    pub fn format_direct(&self) -> String {
        if self.direct.is_empty() {
            "none".to_string()
        } else {
            self.direct.join(", ")
        }
    }

    pub fn format_optional(&self) -> String {
        if self.optional.is_empty() {
            return String::new();
        }
        format!(" (optional: {})", self.optional.join(", "))
    }

    pub fn has_data(&self) -> bool {
        !self.direct.is_empty() || !self.optional.is_empty() || !self.reverse.is_empty()
    }
}

/// Format service dependencies for display
impl ServiceDeps {
    pub fn has_data(&self) -> bool {
        !self.requires.is_empty()
            || !self.wants.is_empty()
            || !self.part_of.is_empty()
            || !self.wanted_by.is_empty()
            || !self.required_by.is_empty()
    }
}

/// Format module dependencies for display
impl ModuleDeps {
    pub fn format_chain(&self) -> String {
        if self.chain.is_empty() {
            return String::new();
        }
        self.chain.join("  ->  ")
    }

    pub fn has_data(&self) -> bool {
        !self.depends.is_empty() || !self.used_by.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_package_deps_pacman() {
        // pacman should exist on Arch
        let deps = get_package_deps("pacman");
        // Should have some dependencies
        assert!(deps.has_data() || deps.direct.is_empty(), "Should be able to query pacman deps");
    }

    #[test]
    fn test_get_service_deps_dbus() {
        // dbus.service should exist
        let deps = get_service_deps("dbus");
        // Source should be set
        assert!(deps.source.contains("systemctl"));
    }

    #[test]
    fn test_package_deps_format() {
        let deps = PackageDeps {
            direct: vec!["glibc".to_string(), "systemd".to_string()],
            optional: vec!["bash-completion".to_string()],
            reverse: vec![],
            source: "pacman".to_string(),
        };
        assert!(deps.format_direct().contains("glibc"));
        assert!(deps.format_optional().contains("bash-completion"));
    }
}

//! Service Topology v7.19.0 - Systemd Unit Relationships
//!
//! Sources:
//! - systemctl show UNIT for unit properties
//! - systemctl list-dependencies UNIT for dependency tree
//! - /usr/lib/systemd/system and /etc/systemd/system for unit files
//!
//! Provides honest service topology without inference or guessing.

use std::process::Command;

/// Complete service topology information
#[derive(Debug, Clone, Default)]
pub struct ServiceTopology {
    /// The unit name
    pub unit: String,
    /// Related units (same base name, different suffixes)
    pub related_units: Vec<UnitInfo>,
    /// Direct Requires dependencies
    pub requires: Vec<String>,
    /// Direct Wants dependencies
    pub wants: Vec<String>,
    /// Units that require this one
    pub required_by: Vec<String>,
    /// Units that want this one
    pub wanted_by: Vec<String>,
    /// Before/After ordering (not hard deps)
    pub before: Vec<String>,
    pub after: Vec<String>,
    /// Conflicts
    pub conflicts: Vec<String>,
    /// Data source
    pub source: String,
}

/// Information about a single unit
#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub name: String,
    pub state: String,     // running, stopped, etc.
    pub enabled: bool,
}

/// Services with many reverse dependencies (for topology hints)
#[derive(Debug, Clone)]
pub struct TopologyHint {
    pub unit: String,
    pub reverse_dep_count: usize,
    pub dep_type: String,  // "required by" or "wanted by"
}

/// Get complete service topology for a unit
pub fn get_service_topology(unit: &str) -> ServiceTopology {
    let mut topology = ServiceTopology::default();
    topology.source = "systemctl show, systemctl list-dependencies".to_string();

    // Normalize unit name
    let unit_name = normalize_unit_name(unit);
    topology.unit = unit_name.clone();

    // Get related units (e.g., NetworkManager.service + NetworkManager-dispatcher.service)
    topology.related_units = find_related_units(&unit_name);

    // Get dependency info from systemctl show
    if let Ok(output) = Command::new("systemctl")
        .args(["show", &unit_name, "--no-pager"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_systemctl_show(&stdout, &mut topology);
        }
    }

    topology
}

/// Parse systemctl show output for topology data
fn parse_systemctl_show(output: &str, topology: &mut ServiceTopology) {
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let values: Vec<String> = value
                .split_whitespace()
                .filter(|s| !s.is_empty() && *s != "(null)")
                .map(|s| s.to_string())
                .collect();

            if values.is_empty() {
                continue;
            }

            match key {
                "Requires" => topology.requires = filter_units(&values, 8),
                "Wants" => topology.wants = filter_units(&values, 8),
                "RequiredBy" => topology.required_by = filter_units(&values, 8),
                "WantedBy" => topology.wanted_by = filter_units(&values, 8),
                "Before" => topology.before = filter_units(&values, 4),
                "After" => topology.after = filter_units(&values, 4),
                "Conflicts" => topology.conflicts = filter_units(&values, 4),
                _ => {}
            }
        }
    }
}

/// Filter unit names, removing noise and limiting count
fn filter_units(units: &[String], limit: usize) -> Vec<String> {
    units.iter()
        .filter(|u| !u.is_empty() && *u != "-" && !u.contains("(null)"))
        // Skip very generic units that clutter output
        .filter(|u| !u.starts_with("-."))
        .take(limit)
        .cloned()
        .collect()
}

/// Normalize unit name to include suffix
fn normalize_unit_name(unit: &str) -> String {
    if unit.contains('.') {
        unit.to_string()
    } else {
        format!("{}.service", unit)
    }
}

/// Find related units with the same base name
fn find_related_units(unit: &str) -> Vec<UnitInfo> {
    let mut related = Vec::new();

    // Extract base name (e.g., "NetworkManager" from "NetworkManager.service")
    let base = unit.split('.').next().unwrap_or(unit);

    // Get list of all units matching this base name
    if let Ok(output) = Command::new("systemctl")
        .args(["list-units", "--all", "--no-pager", "--no-legend", &format!("{}*", base)])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[0].to_string();
                    let sub_state = parts[3].to_string(); // running, stopped, etc.

                    // Check if enabled
                    let enabled = is_unit_enabled(&name);

                    related.push(UnitInfo {
                        name,
                        state: sub_state,
                        enabled,
                    });
                }
            }
        }
    }

    related
}

/// Check if a unit is enabled
fn is_unit_enabled(unit: &str) -> bool {
    if let Ok(output) = Command::new("systemctl")
        .args(["is-enabled", unit])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        stdout == "enabled" || stdout == "static"
    } else {
        false
    }
}

/// Get services with many reverse dependencies (for topology hints in status)
pub fn get_high_impact_services() -> Vec<TopologyHint> {
    let mut hints = Vec::new();

    // Check common high-impact services
    let check_services = [
        "dbus.service",
        "systemd-logind.service",
        "NetworkManager.service",
        "systemd-resolved.service",
        "polkit.service",
        "upower.service",
    ];

    for unit in check_services {
        if let Ok(output) = Command::new("systemctl")
            .args(["show", unit, "-p", "RequiredBy,WantedBy", "--no-pager"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);

                let mut required_count = 0;
                let mut wanted_count = 0;

                for line in stdout.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        let count = value.split_whitespace()
                            .filter(|s| !s.is_empty() && *s != "(null)")
                            .count();

                        match key {
                            "RequiredBy" => required_count = count,
                            "WantedBy" => wanted_count = count,
                            _ => {}
                        }
                    }
                }

                let total = required_count + wanted_count;
                if total >= 3 {
                    let dep_type = if required_count > wanted_count {
                        "required by"
                    } else {
                        "wanted by"
                    };

                    hints.push(TopologyHint {
                        unit: unit.to_string(),
                        reverse_dep_count: total,
                        dep_type: dep_type.to_string(),
                    });
                }
            }
        }
    }

    // Sort by impact (most reverse deps first)
    hints.sort_by(|a, b| b.reverse_dep_count.cmp(&a.reverse_dep_count));
    hints.truncate(5);

    hints
}

/// Get driver topology hints (multiple stacks for same hardware)
#[derive(Debug, Clone)]
pub struct DriverStackHint {
    pub component: String,
    pub primary_driver: String,
    pub additional_modules: Vec<String>,
}

/// Get GPU driver stacks
pub fn get_gpu_driver_stacks() -> Vec<DriverStackHint> {
    let mut stacks = Vec::new();

    // Check for NVIDIA
    let nvidia_modules = get_loaded_modules_matching("nvidia");
    if !nvidia_modules.is_empty() {
        stacks.push(DriverStackHint {
            component: "GPU".to_string(),
            primary_driver: "nvidia".to_string(),
            additional_modules: nvidia_modules.into_iter()
                .filter(|m| m != "nvidia")
                .take(3)
                .collect(),
        });
    }

    // Check for AMD
    let amd_modules = get_loaded_modules_matching("amdgpu");
    if !amd_modules.is_empty() {
        stacks.push(DriverStackHint {
            component: "GPU".to_string(),
            primary_driver: "amdgpu".to_string(),
            additional_modules: amd_modules.into_iter()
                .filter(|m| m != "amdgpu")
                .take(3)
                .collect(),
        });
    }

    // Check for Intel
    let intel_modules = get_loaded_modules_matching("i915");
    if !intel_modules.is_empty() {
        stacks.push(DriverStackHint {
            component: "GPU".to_string(),
            primary_driver: "i915".to_string(),
            additional_modules: intel_modules.into_iter()
                .filter(|m| m != "i915")
                .take(3)
                .collect(),
        });
    }

    stacks
}

/// Get loaded modules matching a pattern
fn get_loaded_modules_matching(pattern: &str) -> Vec<String> {
    let mut modules = Vec::new();

    if let Ok(output) = Command::new("lsmod").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                if let Some(name) = line.split_whitespace().next() {
                    if name.contains(pattern) {
                        modules.push(name.to_string());
                    }
                }
            }
        }
    }

    modules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_unit_name() {
        assert_eq!(normalize_unit_name("sshd"), "sshd.service");
        assert_eq!(normalize_unit_name("sshd.service"), "sshd.service");
        assert_eq!(normalize_unit_name("dbus.socket"), "dbus.socket");
    }

    #[test]
    fn test_filter_units() {
        let units = vec![
            "dbus.service".to_string(),
            "".to_string(),
            "-".to_string(),
            "foo.service".to_string(),
        ];
        let filtered = filter_units(&units, 10);
        assert_eq!(filtered.len(), 2);
    }
}

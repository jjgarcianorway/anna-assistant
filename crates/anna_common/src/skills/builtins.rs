//! Built-in Skills v0.40.0
//!
//! Pre-defined generic skills that work out of the box.
//! These serve as patterns for learned skills.

use super::schema::{ParamType, Skill};

/// Get all built-in skills
pub fn builtin_skills() -> Vec<Skill> {
    vec![
        // Log viewing skills
        journalctl_service_window(),
        journalctl_unit_recent(),
        // Disk/storage skills
        disk_free_filesystem(),
        disk_usage_directory(),
        block_devices(),
        // Service management skills
        service_status(),
        service_list_active(),
        // Process skills
        process_list(),
        process_by_name(),
        // System info skills
        system_uptime(),
        memory_info(),
        cpu_info(),
        // Network skills
        network_interfaces(),
        network_connections(),
        // Package skills
        package_info(),
        package_search(),
        package_list_installed(),
    ]
}

// ============================================================
// LOG SKILLS
// ============================================================

/// Show logs for a systemd service in a time window
fn journalctl_service_window() -> Skill {
    Skill::new(
        "journalctl.service_window",
        "logs_for_service",
        "Show journalctl logs for a systemd service within a time window",
        "journalctl -u {{service_name}} --since {{since}} --no-pager -n {{max_lines}}",
    )
    .with_param("service_name", ParamType::String, "Systemd service name (e.g., annad.service)", true)
    .with_param("since", ParamType::Duration, "Time window start (e.g., '6 hours ago', '1 day ago')", true)
    .with_param("max_lines", ParamType::Integer, "Maximum lines to show", false)
    .with_default("max_lines", "200")
    .with_example("show the log of annad service for the last 6 hours")
    .with_example("show sshd logs from last 24 hours")
    .with_example("give me the last 50 lines of NetworkManager logs from the last hour")
    .with_parser("text")
    .builtin()
}

/// Show recent logs for a systemd unit
fn journalctl_unit_recent() -> Skill {
    Skill::new(
        "journalctl.unit_recent",
        "logs_recent",
        "Show recent logs for a systemd unit",
        "journalctl -u {{unit}} -n {{lines}} --no-pager",
    )
    .with_param("unit", ParamType::String, "Systemd unit name", true)
    .with_param("lines", ParamType::Integer, "Number of recent lines", false)
    .with_default("lines", "50")
    .with_example("show recent logs for docker")
    .with_example("last 100 lines of nginx logs")
    .with_parser("text")
    .builtin()
}

// ============================================================
// DISK/STORAGE SKILLS
// ============================================================

/// Check free space on a filesystem
fn disk_free_filesystem() -> Skill {
    Skill::new(
        "disk.free_filesystem",
        "disk_usage",
        "Check free space on a filesystem or mount point",
        "df -h {{path}}",
    )
    .with_param("path", ParamType::Path, "Mount point or filesystem path", false)
    .with_default("path", "/")
    .with_example("how much free space on root")
    .with_example("check disk space on /home")
    .with_example("show filesystem usage")
    .with_parser("df")
    .builtin()
}

/// Check disk usage of a directory
fn disk_usage_directory() -> Skill {
    Skill::new(
        "disk.usage_directory",
        "directory_size",
        "Check disk usage of a directory",
        "du -sh {{path}}",
    )
    .with_param("path", ParamType::Path, "Directory path", true)
    .with_example("how big is /var/log")
    .with_example("size of home directory")
    .with_parser("text")
    .builtin()
}

/// List block devices
fn block_devices() -> Skill {
    Skill::new(
        "disk.block_devices",
        "storage_devices",
        "List block devices and partitions",
        "lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT",
    )
    .with_example("show my disks")
    .with_example("list partitions")
    .with_example("what storage devices do I have")
    .with_parser("lsblk")
    .builtin()
}

// ============================================================
// SERVICE SKILLS
// ============================================================

/// Check status of a systemd service
fn service_status() -> Skill {
    Skill::new(
        "service.status",
        "service_status",
        "Check the status of a systemd service",
        "systemctl status {{service}} --no-pager",
    )
    .with_param("service", ParamType::String, "Service name", true)
    .with_example("is annad running")
    .with_example("status of docker service")
    .with_example("check if nginx is active")
    .with_parser("systemctl")
    .builtin()
}

/// List active services
fn service_list_active() -> Skill {
    Skill::new(
        "service.list_active",
        "list_services",
        "List active systemd services",
        "systemctl list-units --type=service --state={{state}} --no-pager",
    )
    .with_param("state", ParamType::String, "Service state filter", false)
    .with_default("state", "running")
    .with_example("what services are running")
    .with_example("list all active services")
    .with_parser("systemctl")
    .builtin()
}

// ============================================================
// PROCESS SKILLS
// ============================================================

/// List processes
fn process_list() -> Skill {
    Skill::new(
        "process.list",
        "list_processes",
        "List running processes",
        "ps aux --sort=-{{sort}}",
    )
    .with_param("sort", ParamType::String, "Sort by (cpu, mem, time)", false)
    .with_default("sort", "cpu")
    .with_example("show running processes")
    .with_example("list processes by memory usage")
    .with_parser("ps")
    .builtin()
}

/// Find process by name
fn process_by_name() -> Skill {
    Skill::new(
        "process.by_name",
        "find_process",
        "Find processes by name",
        "ps aux | grep -i {{name}} | grep -v grep",
    )
    .with_param("name", ParamType::String, "Process name to search", true)
    .with_example("is firefox running")
    .with_example("find python processes")
    .with_parser("text")
    .builtin()
}

// ============================================================
// SYSTEM INFO SKILLS
// ============================================================

/// System uptime
fn system_uptime() -> Skill {
    Skill::new(
        "system.uptime",
        "uptime",
        "Show system uptime and load average",
        "uptime",
    )
    .with_example("how long has the system been running")
    .with_example("show uptime")
    .with_example("what is the load average")
    .with_parser("uptime")
    .builtin()
}

/// Memory information
fn memory_info() -> Skill {
    Skill::new(
        "system.memory",
        "memory_info",
        "Show memory usage information",
        "free -h",
    )
    .with_example("how much RAM do I have")
    .with_example("show memory usage")
    .with_example("is there enough free memory")
    .with_parser("free")
    .builtin()
}

/// CPU information
fn cpu_info() -> Skill {
    Skill::new(
        "system.cpu",
        "cpu_info",
        "Show CPU information",
        "lscpu",
    )
    .with_example("what CPU do I have")
    .with_example("how many cores")
    .with_example("show processor info")
    .with_parser("lscpu")
    .builtin()
}

// ============================================================
// NETWORK SKILLS
// ============================================================

/// Network interfaces
fn network_interfaces() -> Skill {
    Skill::new(
        "network.interfaces",
        "network_info",
        "Show network interfaces and IP addresses",
        "ip -c addr",
    )
    .with_example("what is my IP address")
    .with_example("show network interfaces")
    .with_example("list network adapters")
    .with_parser("ip")
    .builtin()
}

/// Network connections
fn network_connections() -> Skill {
    Skill::new(
        "network.connections",
        "network_connections",
        "Show active network connections",
        "ss -tuln",
    )
    .with_example("what ports are open")
    .with_example("show listening services")
    .with_example("list network connections")
    .with_parser("ss")
    .builtin()
}

// ============================================================
// PACKAGE SKILLS
// ============================================================

/// Package info
fn package_info() -> Skill {
    Skill::new(
        "package.info",
        "package_info",
        "Show information about an installed package",
        "pacman -Qi {{package}}",
    )
    .with_param("package", ParamType::String, "Package name", true)
    .with_example("info about vim package")
    .with_example("when was firefox installed")
    .with_example("what version of python is installed")
    .with_parser("pacman")
    .builtin()
}

/// Package search
fn package_search() -> Skill {
    Skill::new(
        "package.search",
        "package_search",
        "Search for packages in repositories",
        "pacman -Ss {{query}}",
    )
    .with_param("query", ParamType::String, "Search query", true)
    .with_example("search for docker packages")
    .with_example("find packages related to python")
    .with_parser("pacman")
    .builtin()
}

/// List installed packages
fn package_list_installed() -> Skill {
    Skill::new(
        "package.list",
        "list_packages",
        "List installed packages",
        "pacman -Q | head -n {{limit}}",
    )
    .with_param("limit", ParamType::Integer, "Maximum packages to show", false)
    .with_default("limit", "50")
    .with_example("list installed packages")
    .with_example("what packages are installed")
    .with_parser("text")
    .builtin()
}

/// Initialize the skill store with built-in skills
pub fn init_builtin_skills(store: &super::store::SkillStore) -> anyhow::Result<()> {
    for skill in builtin_skills() {
        // Only add if not already present (preserve learned stats)
        if store.get(&skill.skill_id).is_none() {
            store.upsert(&skill)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_builtin_skills_count() {
        let skills = builtin_skills();
        assert!(skills.len() >= 15, "Should have at least 15 built-in skills");
    }

    #[test]
    fn test_journalctl_service_window() {
        let skill = journalctl_service_window();
        assert_eq!(skill.skill_id, "journalctl.service_window");
        assert!(skill.builtin);

        let mut params = HashMap::new();
        params.insert("service_name".to_string(), "annad.service".to_string());
        params.insert("since".to_string(), "6 hours ago".to_string());

        let cmd = skill.build_command(&params).unwrap();
        assert!(cmd.contains(&"journalctl".to_string()));
        assert!(cmd.contains(&"-u".to_string()));
        assert!(cmd.contains(&"annad.service".to_string()));
    }

    #[test]
    fn test_disk_free_filesystem() {
        let skill = disk_free_filesystem();

        let params = HashMap::new();
        let cmd = skill.build_command(&params).unwrap();
        assert!(cmd.contains(&"df".to_string()));
        assert!(cmd.contains(&"/".to_string())); // Default path
    }

    #[test]
    fn test_service_status() {
        let skill = service_status();

        let mut params = HashMap::new();
        params.insert("service".to_string(), "docker".to_string());

        let cmd = skill.build_command(&params).unwrap();
        assert!(cmd.contains(&"systemctl".to_string()));
        assert!(cmd.contains(&"status".to_string()));
        assert!(cmd.contains(&"docker".to_string()));
    }

    #[test]
    fn test_all_skills_have_examples() {
        for skill in builtin_skills() {
            assert!(
                !skill.question_examples.is_empty(),
                "Skill {} should have examples",
                skill.skill_id
            );
        }
    }

    #[test]
    fn test_all_skills_are_builtin() {
        for skill in builtin_skills() {
            assert!(skill.builtin, "Skill {} should be marked as builtin", skill.skill_id);
        }
    }
}

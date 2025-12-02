//! Anna Relationship Store v7.24.0 - Ground Truth Link Database
//!
//! Links software, services, processes, and hardware based on
//! actual system data only. No guessing.
//!
//! Persisted to /var/lib/anna/state/links.db
//!
//! Link types:
//! - package -> service (from pacman -Ql, unit files)
//! - service -> process (from systemctl status, cgroups)
//! - process -> device (from /proc/net, /sys, sockets)
//! - device -> driver (from lspci, modinfo)
//! - driver -> firmware (from modinfo, /sys/firmware)

use rusqlite::{params, Connection, Result as SqlResult};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Link type between entities
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkType {
    PackageToService,
    ServiceToProcess,
    ProcessToDevice,
    DeviceToDriver,
    DriverToFirmware,
    PackageToPackage, // Same stack relationships
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkType::PackageToService => "pkg_to_svc",
            LinkType::ServiceToProcess => "svc_to_proc",
            LinkType::ProcessToDevice => "proc_to_dev",
            LinkType::DeviceToDriver => "dev_to_drv",
            LinkType::DriverToFirmware => "drv_to_fw",
            LinkType::PackageToPackage => "pkg_to_pkg",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pkg_to_svc" => Some(LinkType::PackageToService),
            "svc_to_proc" => Some(LinkType::ServiceToProcess),
            "proc_to_dev" => Some(LinkType::ProcessToDevice),
            "dev_to_drv" => Some(LinkType::DeviceToDriver),
            "drv_to_fw" => Some(LinkType::DriverToFirmware),
            "pkg_to_pkg" => Some(LinkType::PackageToPackage),
            _ => None,
        }
    }
}

/// A directional link between two entities
#[derive(Debug, Clone)]
pub struct Link {
    pub link_type: LinkType,
    pub source: String,
    pub target: String,
    pub evidence: String, // How we know this link exists
}

/// Relationship store backed by SQLite
pub struct RelationshipStore {
    conn: Connection,
}

impl RelationshipStore {
    /// Open or create the relationship store
    pub fn open() -> SqlResult<Self> {
        let state_dir = "/var/lib/anna/state";
        let db_path = format!("{}/links.db", state_dir);

        // Ensure directory exists
        let _ = std::fs::create_dir_all(state_dir);

        let conn = Connection::open(&db_path)?;
        let store = RelationshipStore { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Open in-memory for testing
    pub fn open_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let store = RelationshipStore { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS links (
                id INTEGER PRIMARY KEY,
                link_type TEXT NOT NULL,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                evidence TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(link_type, source, target)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_source ON links(source)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_target ON links(target)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_type ON links(link_type)",
            [],
        )?;

        Ok(())
    }

    /// Insert or update a link
    pub fn upsert_link(&self, link: &Link) -> SqlResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO links (link_type, source, target, evidence, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                link.link_type.as_str(),
                link.source,
                link.target,
                link.evidence,
                now
            ],
        )?;
        Ok(())
    }

    /// Get links from a source
    pub fn get_links_from(&self, source: &str) -> SqlResult<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT link_type, source, target, evidence FROM links WHERE source = ?1",
        )?;

        let links = stmt
            .query_map(params![source], |row| {
                let link_type_str: String = row.get(0)?;
                let link_type = LinkType::from_str(&link_type_str).unwrap_or(LinkType::PackageToService);
                Ok(Link {
                    link_type,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    evidence: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(links)
    }

    /// Get links to a target
    pub fn get_links_to(&self, target: &str) -> SqlResult<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT link_type, source, target, evidence FROM links WHERE target = ?1",
        )?;

        let links = stmt
            .query_map(params![target], |row| {
                let link_type_str: String = row.get(0)?;
                let link_type = LinkType::from_str(&link_type_str).unwrap_or(LinkType::PackageToService);
                Ok(Link {
                    link_type,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    evidence: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(links)
    }

    /// Get links of a specific type from a source
    pub fn get_links_from_of_type(&self, source: &str, link_type: LinkType) -> SqlResult<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT link_type, source, target, evidence FROM links
             WHERE source = ?1 AND link_type = ?2",
        )?;

        let links = stmt
            .query_map(params![source, link_type.as_str()], |row| {
                let lt_str: String = row.get(0)?;
                let lt = LinkType::from_str(&lt_str).unwrap_or(LinkType::PackageToService);
                Ok(Link {
                    link_type: lt,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    evidence: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(links)
    }

    /// Get links of a specific type to a target
    pub fn get_links_to_of_type(&self, target: &str, link_type: LinkType) -> SqlResult<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT link_type, source, target, evidence FROM links
             WHERE target = ?1 AND link_type = ?2",
        )?;

        let links = stmt
            .query_map(params![target, link_type.as_str()], |row| {
                let lt_str: String = row.get(0)?;
                let lt = LinkType::from_str(&lt_str).unwrap_or(LinkType::PackageToService);
                Ok(Link {
                    link_type: lt,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    evidence: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(links)
    }

    /// Clear old links (older than threshold)
    pub fn clear_old_links(&self, age_secs: i64) -> SqlResult<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let threshold = now - age_secs;
        let count = self.conn.execute(
            "DELETE FROM links WHERE updated_at < ?1",
            params![threshold],
        )?;
        Ok(count)
    }
}

/// Discover package to service links from pacman and systemd
pub fn discover_package_service_links(package: &str) -> Vec<Link> {
    let mut links = Vec::new();
    let mut sources_used = HashSet::new();

    // Method 1: Check pacman -Ql for .service files
    if let Some(services) = get_package_service_files(package) {
        sources_used.insert("pacman -Ql");
        for service in services {
            links.push(Link {
                link_type: LinkType::PackageToService,
                source: format!("package:{}", package),
                target: format!("service:{}", service),
                evidence: "pacman -Ql".to_string(),
            });
        }
    }

    // Method 2: Check unit file ExecStart for package binaries
    if let Some(services) = get_services_using_package_binaries(package) {
        sources_used.insert("systemctl show");
        for service in services {
            if !links.iter().any(|l| l.target == format!("service:{}", service)) {
                links.push(Link {
                    link_type: LinkType::PackageToService,
                    source: format!("package:{}", package),
                    target: format!("service:{}", service),
                    evidence: "systemctl show".to_string(),
                });
            }
        }
    }

    links
}

/// Get service files owned by a package
fn get_package_service_files(package: &str) -> Option<Vec<String>> {
    let output = Command::new("pacman")
        .args(["-Ql", package])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let services: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[1];
                if path.ends_with(".service") && path.contains("/systemd/") {
                    // Extract service name from path
                    Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if services.is_empty() {
        None
    } else {
        Some(services)
    }
}

/// Get services that use binaries from a package
fn get_services_using_package_binaries(package: &str) -> Option<Vec<String>> {
    // Get package binaries
    let output = Command::new("pacman")
        .args(["-Ql", package])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let binaries: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[1];
                if path.contains("/bin/") || path.contains("/sbin/") {
                    Some(path.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if binaries.is_empty() {
        return None;
    }

    // Check running services for these binaries
    let output = Command::new("systemctl")
        .args(["list-units", "--type=service", "--no-pager", "--no-legend"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut matching_services = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(unit_name) = parts.first() {
            // Check ExecStart
            if let Some(exec_start) = get_service_exec_start(unit_name) {
                for bin in &binaries {
                    if exec_start.contains(bin) {
                        matching_services.push(unit_name.to_string());
                        break;
                    }
                }
            }
        }
    }

    if matching_services.is_empty() {
        None
    } else {
        Some(matching_services)
    }
}

/// Get ExecStart from a service unit
fn get_service_exec_start(unit: &str) -> Option<String> {
    let output = Command::new("systemctl")
        .args(["show", unit, "-p", "ExecStart", "--no-pager"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find(|l| l.starts_with("ExecStart="))
        .map(|l| l.strip_prefix("ExecStart=").unwrap_or(l).to_string())
}

/// Discover service to process links from systemctl and cgroups
pub fn discover_service_process_links(service: &str) -> Vec<Link> {
    let mut links = Vec::new();

    // Get MainPID from systemctl
    if let Some(pid) = get_service_main_pid(service) {
        if pid > 0 {
            links.push(Link {
                link_type: LinkType::ServiceToProcess,
                source: format!("service:{}", service),
                target: format!("process:{}", pid),
                evidence: "systemctl show MainPID".to_string(),
            });
        }
    }

    // Get additional PIDs from cgroup
    if let Some(pids) = get_service_cgroup_pids(service) {
        for pid in pids {
            if !links.iter().any(|l| l.target == format!("process:{}", pid)) {
                links.push(Link {
                    link_type: LinkType::ServiceToProcess,
                    source: format!("service:{}", service),
                    target: format!("process:{}", pid),
                    evidence: "/sys/fs/cgroup".to_string(),
                });
            }
        }
    }

    links
}

/// Get main PID of a service
fn get_service_main_pid(service: &str) -> Option<u32> {
    let output = Command::new("systemctl")
        .args(["show", service, "-p", "MainPID", "--no-pager"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find(|l| l.starts_with("MainPID="))
        .and_then(|l| l.strip_prefix("MainPID="))
        .and_then(|s| s.trim().parse().ok())
}

/// Get PIDs from service cgroup
fn get_service_cgroup_pids(service: &str) -> Option<Vec<u32>> {
    // Get cgroup path
    let output = Command::new("systemctl")
        .args(["show", service, "-p", "ControlGroup", "--no-pager"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cgroup_path = stdout
        .lines()
        .find(|l| l.starts_with("ControlGroup="))
        .and_then(|l| l.strip_prefix("ControlGroup="))
        .map(|s| s.trim().to_string())?;

    if cgroup_path.is_empty() {
        return None;
    }

    // Read cgroup.procs
    let procs_path = format!("/sys/fs/cgroup{}/cgroup.procs", cgroup_path);
    let content = std::fs::read_to_string(&procs_path).ok()?;

    let pids: Vec<u32> = content
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();

    if pids.is_empty() {
        None
    } else {
        Some(pids)
    }
}

/// Discover process to device links
pub fn discover_process_device_links(pid: u32) -> Vec<Link> {
    let mut links = Vec::new();

    // Check network sockets
    if let Some(devices) = get_process_network_devices(pid) {
        for device in devices {
            links.push(Link {
                link_type: LinkType::ProcessToDevice,
                source: format!("process:{}", pid),
                target: format!("device:{}", device),
                evidence: "/proc/net".to_string(),
            });
        }
    }

    links
}

/// Get network devices used by a process
fn get_process_network_devices(_pid: u32) -> Option<Vec<String>> {
    // This is a simplified implementation
    // In practice, would parse /proc/net/tcp, /proc/net/udp, etc.
    // and map sockets to interfaces

    // For now, return None - full implementation would be complex
    None
}

/// Discover device to driver links
pub fn discover_device_driver_links(device: &str) -> Vec<Link> {
    let mut links = Vec::new();

    // For network devices
    if Path::new(&format!("/sys/class/net/{}", device)).exists() {
        if let Some(driver) = get_network_device_driver(device) {
            links.push(Link {
                link_type: LinkType::DeviceToDriver,
                source: format!("device:{}", device),
                target: format!("driver:{}", driver),
                evidence: "/sys/class/net".to_string(),
            });
        }
    }

    // For PCI devices (simplified)
    if device.starts_with("0000:") || device.contains("pci") {
        if let Some(driver) = get_pci_device_driver(device) {
            links.push(Link {
                link_type: LinkType::DeviceToDriver,
                source: format!("device:{}", device),
                target: format!("driver:{}", driver),
                evidence: "lspci".to_string(),
            });
        }
    }

    links
}

/// Get driver for a network device
fn get_network_device_driver(device: &str) -> Option<String> {
    let driver_link = format!("/sys/class/net/{}/device/driver", device);
    std::fs::read_link(&driver_link)
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
}

/// Get driver for a PCI device
fn get_pci_device_driver(device: &str) -> Option<String> {
    let output = Command::new("lspci")
        .args(["-k", "-s", device])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("Kernel driver in use:") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }

    None
}

/// Discover driver to firmware links
pub fn discover_driver_firmware_links(driver: &str) -> Vec<Link> {
    let mut links = Vec::new();

    // Use modinfo to get firmware
    if let Some(firmwares) = get_driver_firmware(driver) {
        for fw in firmwares {
            links.push(Link {
                link_type: LinkType::DriverToFirmware,
                source: format!("driver:{}", driver),
                target: format!("firmware:{}", fw),
                evidence: "modinfo".to_string(),
            });
        }
    }

    links
}

/// Get firmware files for a driver
fn get_driver_firmware(driver: &str) -> Option<Vec<String>> {
    let output = Command::new("modinfo")
        .args(["-F", "firmware", driver])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let firmwares: Vec<String> = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if firmwares.is_empty() {
        None
    } else {
        Some(firmwares)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_type_roundtrip() {
        let types = vec![
            LinkType::PackageToService,
            LinkType::ServiceToProcess,
            LinkType::ProcessToDevice,
            LinkType::DeviceToDriver,
            LinkType::DriverToFirmware,
            LinkType::PackageToPackage,
        ];

        for lt in types {
            let s = lt.as_str();
            let parsed = LinkType::from_str(s);
            assert!(parsed.is_some());
            assert_eq!(parsed.unwrap(), lt);
        }
    }

    #[test]
    fn test_relationship_store_memory() {
        let store = RelationshipStore::open_memory().unwrap();

        let link = Link {
            link_type: LinkType::PackageToService,
            source: "package:NetworkManager".to_string(),
            target: "service:NetworkManager.service".to_string(),
            evidence: "pacman -Ql".to_string(),
        };

        store.upsert_link(&link).unwrap();

        let links = store.get_links_from("package:NetworkManager").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "service:NetworkManager.service");
    }

    #[test]
    fn test_get_links_to() {
        let store = RelationshipStore::open_memory().unwrap();

        let link = Link {
            link_type: LinkType::DeviceToDriver,
            source: "device:wlan0".to_string(),
            target: "driver:iwlwifi".to_string(),
            evidence: "/sys/class/net".to_string(),
        };

        store.upsert_link(&link).unwrap();

        let links = store.get_links_to("driver:iwlwifi").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].source, "device:wlan0");
    }
}

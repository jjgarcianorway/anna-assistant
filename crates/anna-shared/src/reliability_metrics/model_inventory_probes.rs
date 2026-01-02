//! Probe inventory and default probes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A probe in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEntry {
    /// Probe ID (e.g., "df", "free", "systemctl_status")
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Command to run
    pub command: String,
    /// Is this probe available (command exists)?
    pub available: bool,
}

/// Probe inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeInventory {
    /// All probes by ID.
    pub probes: HashMap<String, ProbeEntry>,
}

impl ProbeInventory {
    /// Create empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a probe.
    pub fn add(&mut self, id: &str, description: &str, command: &str, available: bool) {
        self.probes.insert(
            id.to_string(),
            ProbeEntry {
                id: id.to_string(),
                description: description.to_string(),
                command: command.to_string(),
                available,
            },
        );
    }

    /// Get probe by ID.
    pub fn get(&self, id: &str) -> Option<&ProbeEntry> {
        self.probes.get(id)
    }

    /// Get available probes.
    pub fn available(&self) -> Vec<&ProbeEntry> {
        self.probes.values().filter(|p| p.available).collect()
    }

    /// Get unavailable probes.
    pub fn unavailable(&self) -> Vec<&ProbeEntry> {
        self.probes.values().filter(|p| !p.available).collect()
    }

    /// Format for display in annactl debug probes.
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str("[probes]\n");

        let mut probes: Vec<_> = self.probes.values().collect();
        probes.sort_by(|a, b| a.id.cmp(&b.id));

        for p in probes {
            let status = if p.available { "✓" } else { "✗" };
            out.push_str(&format!(
                "  {} {} - {} ({})\n",
                status, p.id, p.description, p.command
            ));
        }

        let available = self.available().len();
        let total = self.probes.len();
        out.push_str(&format!("\n  {}/{} probes available\n", available, total));

        out
    }
}

/// Build default probe inventory with common probes.
pub fn default_probe_inventory() -> ProbeInventory {
    let mut inv = ProbeInventory::new();

    // System info probes
    inv.add("uname", "System information", "uname -a", true);
    inv.add("hostname", "Hostname", "hostname", true);
    inv.add("uptime", "System uptime", "uptime", true);

    // Disk probes
    inv.add("df", "Disk space usage", "df -h", true);
    inv.add("du_home", "Home directory size", "du -sh ~", true);
    inv.add("lsblk", "Block devices", "lsblk", true);

    // Memory probes
    inv.add("free", "Memory usage", "free -h", true);
    inv.add("meminfo", "Memory info", "cat /proc/meminfo", true);

    // Process probes
    inv.add("ps_aux", "Running processes", "ps aux", true);
    inv.add("top_snapshot", "Top processes", "top -bn1 | head -20", true);

    // Network probes
    inv.add("ip_addr", "IP addresses", "ip addr", true);
    inv.add("ss_listen", "Listening ports", "ss -tlnp", true);

    // Service probes
    inv.add(
        "systemctl_failed",
        "Failed services",
        "systemctl --failed --no-pager",
        true,
    );
    inv.add(
        "systemctl_list",
        "All services",
        "systemctl list-units --type=service --no-pager",
        true,
    );

    // Package probes
    inv.add("pacman_q", "Installed packages", "pacman -Q", true);
    inv.add("which", "Find command", "which", true);

    inv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_inventory() {
        let inv = default_probe_inventory();
        assert!(inv.probes.len() > 10);
        assert!(inv.get("df").is_some());
        assert!(inv.get("free").is_some());
    }

    #[test]
    fn test_probe_available() {
        let mut inv = ProbeInventory::new();
        inv.add("test1", "Test 1", "cmd1", true);
        inv.add("test2", "Test 2", "cmd2", false);
        inv.add("test3", "Test 3", "cmd3", true);

        assert_eq!(inv.available().len(), 2);
        assert_eq!(inv.unavailable().len(), 1);
    }

    #[test]
    fn test_probe_display() {
        let inv = default_probe_inventory();
        let display = inv.display();
        assert!(display.contains("[probes]"));
        assert!(display.contains("probes available"));
    }
}

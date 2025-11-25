//! Tool Inventory - v6.42.0
//!
//! Detects available system tools for dynamic command planning.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Tool inventory - detects what's actually installed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolInventory {
    // Package managers
    pub pacman: bool,
    pub yay: bool,
    pub paru: bool,
    pub flatpak: bool,
    pub snap: bool,

    // System tools
    pub grep: bool,
    pub awk: bool,
    pub sed: bool,
    pub du: bool,
    pub df: bool,
    pub find: bool,
    pub ps: bool,
    pub systemctl: bool,
    pub lscpu: bool,
    pub lspci: bool,
    pub lsusb: bool,
    pub free: bool,

    // Gaming tools
    pub steam: bool,
    pub lutris: bool,
    pub heroic: bool,
    pub wine: bool,
    pub proton: bool,

    // File managers
    pub thunar: bool,
    pub dolphin: bool,
    pub nautilus: bool,
    pub nemo: bool,
    pub pcmanfm: bool,
    pub ranger: bool,
    pub mc: bool,
}

impl ToolInventory {
    /// Detect all available tools
    pub fn detect() -> Self {
        let mut inventory = Self::default();

        // Check each tool
        inventory.pacman = check_tool("pacman");
        inventory.yay = check_tool("yay");
        inventory.paru = check_tool("paru");
        inventory.flatpak = check_tool("flatpak");
        inventory.snap = check_tool("snap");

        inventory.grep = check_tool("grep");
        inventory.awk = check_tool("awk");
        inventory.sed = check_tool("sed");
        inventory.du = check_tool("du");
        inventory.df = check_tool("df");
        inventory.find = check_tool("find");
        inventory.ps = check_tool("ps");
        inventory.systemctl = check_tool("systemctl");
        inventory.lscpu = check_tool("lscpu");
        inventory.lspci = check_tool("lspci");
        inventory.lsusb = check_tool("lsusb");
        inventory.free = check_tool("free");

        inventory.steam = check_tool("steam");
        inventory.lutris = check_tool("lutris");
        inventory.heroic = check_tool("heroic");
        inventory.wine = check_tool("wine");
        inventory.proton = check_tool("proton");

        inventory.thunar = check_tool("thunar");
        inventory.dolphin = check_tool("dolphin");
        inventory.nautilus = check_tool("nautilus");
        inventory.nemo = check_tool("nemo");
        inventory.pcmanfm = check_tool("pcmanfm");
        inventory.ranger = check_tool("ranger");
        inventory.mc = check_tool("mc");

        inventory
    }

    /// Get a compact JSON representation for LLM context
    pub fn to_json_context(&self) -> serde_json::Value {
        serde_json::json!({
            "package_managers": {
                "pacman": self.pacman,
                "yay": self.yay,
                "paru": self.paru,
                "flatpak": self.flatpak,
                "snap": self.snap,
            },
            "system_tools": {
                "grep": self.grep,
                "awk": self.awk,
                "sed": self.sed,
                "systemctl": self.systemctl,
                "lscpu": self.lscpu,
                "lspci": self.lspci,
                "free": self.free,
            },
            "gaming": {
                "steam": self.steam,
                "lutris": self.lutris,
                "heroic": self.heroic,
                "wine": self.wine,
            },
            "file_managers": {
                "thunar": self.thunar,
                "dolphin": self.dolphin,
                "nautilus": self.nautilus,
                "nemo": self.nemo,
                "ranger": self.ranger,
            }
        })
    }

    /// Get list of available package managers
    pub fn available_package_managers(&self) -> Vec<&str> {
        let mut managers = Vec::new();
        if self.pacman {
            managers.push("pacman");
        }
        if self.yay {
            managers.push("yay");
        }
        if self.paru {
            managers.push("paru");
        }
        if self.flatpak {
            managers.push("flatpak");
        }
        if self.snap {
            managers.push("snap");
        }
        managers
    }
}

/// Check if a tool is available in PATH
fn check_tool(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_inventory_detect() {
        let inventory = ToolInventory::detect();

        // Basic system tools should exist on most systems
        // We can't assert specific values since it depends on the system
        // but we can verify the structure works
        let _ = inventory.pacman;
        let _ = inventory.grep;
    }

    #[test]
    fn test_tool_inventory_json_context() {
        let inventory = ToolInventory {
            pacman: true,
            yay: false,
            grep: true,
            steam: false,
            ..Default::default()
        };

        let json = inventory.to_json_context();
        assert_eq!(json["package_managers"]["pacman"], true);
        assert_eq!(json["package_managers"]["yay"], false);
        assert_eq!(json["system_tools"]["grep"], true);
        assert_eq!(json["gaming"]["steam"], false);
    }

    #[test]
    fn test_available_package_managers() {
        let inventory = ToolInventory {
            pacman: true,
            yay: true,
            paru: false,
            flatpak: true,
            ..Default::default()
        };

        let managers = inventory.available_package_managers();
        assert_eq!(managers.len(), 3);
        assert!(managers.contains(&"pacman"));
        assert!(managers.contains(&"yay"));
        assert!(managers.contains(&"flatpak"));
        assert!(!managers.contains(&"paru"));
    }
}

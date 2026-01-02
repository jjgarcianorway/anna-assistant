//! Status response types for system and package queries.

use super::types::SwapKind;
use serde::{Deserialize, Serialize};

/// Swap status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStatus {
    /// Does system have swap?
    pub has_swap: bool,
    /// Total swap in GiB.
    pub total_swap_gib: f64,
    /// Kind of swap.
    pub kind: SwapKind,
    /// Evidence (probe outputs).
    pub evidence: Vec<String>,
}

impl SwapStatus {
    /// Parse from /proc/swaps output.
    pub fn from_proc_swaps(output: &str) -> Self {
        let lines: Vec<&str> = output.lines().collect();

        // Skip header, check for swap entries
        if lines.len() <= 1 || output.trim().is_empty() {
            return Self {
                has_swap: false,
                total_swap_gib: 0.0,
                kind: SwapKind::None,
                evidence: vec!["/proc/swaps is empty".to_string()],
            };
        }

        let mut total_kb: u64 = 0;
        let mut kind = SwapKind::None;

        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0];
                let swap_type = parts[1];

                // Parse size (in KB)
                if let Ok(size) = parts[2].parse::<u64>() {
                    total_kb += size;
                }

                // Determine kind
                kind = if name.contains("zram") {
                    SwapKind::Zram
                } else if swap_type == "file" {
                    SwapKind::File
                } else {
                    SwapKind::Partition
                };
            }
        }

        let total_gib = total_kb as f64 / 1024.0 / 1024.0;

        Self {
            has_swap: total_kb > 0,
            total_swap_gib: total_gib,
            kind,
            evidence: vec![format!("From /proc/swaps: {} KB total", total_kb)],
        }
    }

    /// Format for user display.
    pub fn display(&self) -> String {
        if !self.has_swap {
            return "You have NO swap configured on this system.\n• evidence: /proc/swaps is empty"
                .to_string();
        }

        let kind_str = match self.kind {
            SwapKind::File => "swapfile",
            SwapKind::Partition => "partition",
            SwapKind::Zram => "zram",
            SwapKind::None => "unknown",
        };

        format!(
            "Yes, you have swap: {:.1} GiB ({}).",
            self.total_swap_gib, kind_str
        )
    }
}

/// Package status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStatus {
    /// Package name.
    pub package: String,
    /// Is installed?
    pub installed: bool,
    /// Version if installed.
    pub version: Option<String>,
    /// Evidence (probe output).
    pub evidence: Vec<String>,
}

impl PackageStatus {
    /// Parse from pacman -Q output.
    pub fn from_pacman_output(package: &str, output: &str) -> Self {
        let trimmed = output.trim();

        // Check for NOT_INSTALLED marker or error
        if trimmed.contains("NOT_INSTALLED")
            || trimmed.contains("was not found")
            || trimmed.is_empty()
        {
            return Self {
                package: package.to_string(),
                installed: false,
                version: None,
                evidence: vec![format!("pacman -Q {} → not found", package)],
            };
        }

        // Parse "package version" format
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == package {
            return Self {
                package: package.to_string(),
                installed: true,
                version: Some(parts[1].to_string()),
                evidence: vec![format!("pacman -Q {} → {}", package, trimmed)],
            };
        }

        // Unexpected format
        Self {
            package: package.to_string(),
            installed: false,
            version: None,
            evidence: vec![format!("Unexpected pacman output: {}", trimmed)],
        }
    }

    /// Format for user display.
    pub fn display(&self) -> String {
        if !self.installed {
            return format!(
                "{} is NOT installed.\n• evidence: {}",
                self.package,
                self.evidence.join(", ")
            );
        }

        let version = self.version.as_deref().unwrap_or("unknown");
        format!(
            "{} is installed, version {}.\n• evidence: {}",
            self.package,
            version,
            self.evidence.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_status_parsing() {
        let empty = "";
        let status = SwapStatus::from_proc_swaps(empty);
        assert!(!status.has_swap);
        assert_eq!(status.kind, SwapKind::None);

        let with_swap = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/swapfile\tfile\t\t8388604\t\t0\t\t-2";
        let status = SwapStatus::from_proc_swaps(with_swap);
        assert!(status.has_swap);
        assert!(status.total_swap_gib > 7.0);
        assert_eq!(status.kind, SwapKind::File);
    }

    #[test]
    fn test_package_status_parsing() {
        let installed = "nano 7.2-1";
        let status = PackageStatus::from_pacman_output("nano", installed);
        assert!(status.installed);
        assert_eq!(status.version, Some("7.2-1".to_string()));

        let not_installed = "NOT_INSTALLED";
        let status = PackageStatus::from_pacman_output("nano", not_installed);
        assert!(!status.installed);
    }

    #[test]
    fn test_swap_display() {
        let no_swap = SwapStatus {
            has_swap: false,
            total_swap_gib: 0.0,
            kind: SwapKind::None,
            evidence: vec![],
        };
        assert!(no_swap.display().contains("NO swap"));

        let with_swap = SwapStatus {
            has_swap: true,
            total_swap_gib: 8.0,
            kind: SwapKind::File,
            evidence: vec![],
        };
        assert!(with_swap.display().contains("8.0 GiB"));
    }
}

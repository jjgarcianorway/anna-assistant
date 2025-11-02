// Anna v0.12.3-btrfs - Btrfs Storage Intelligence
// Detects Btrfs layout, health, snapshots, and tools

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::warn;

/// Btrfs storage profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsProfile {
    pub version: String,
    pub generated_at: String,
    pub detected: bool,
    pub layout: BtrfsLayout,
    pub mount_opts: MountOptions,
    pub tools: BtrfsTools,
    pub health: BtrfsHealth,
    pub bootloader: BootloaderInfo,
    pub notes: Vec<String>,
}

/// Btrfs layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsLayout {
    pub subvolumes: Vec<Subvolume>,
    pub default_subvol: Option<String>,
    pub snapshots_dir: Option<String>,
    pub has_separate_home: bool,
    pub has_separate_var: bool,
    pub root_fs_type: String,
    pub boot_fs_type: Option<String>,
    pub esp_mount: Option<String>,
}

/// Subvolume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subvolume {
    pub id: String,
    pub path: String,
    pub mount_point: Option<String>,
    pub is_snapshot: bool,
    pub readonly: bool,
}

/// Mount options for Btrfs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountOptions {
    pub compression: Option<String>,
    pub compression_level: Option<u8>,
    pub autodefrag: bool,
    pub noatime: bool,
    pub space_cache: Option<String>,
    pub ssd: bool,
}

/// Btrfs tools detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsTools {
    pub snapper: bool,
    pub timeshift: bool,
    pub btrfs_assistant: bool,
    pub grub_btrfs: bool,
    pub pacman_hook: bool,
}

/// Btrfs health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsHealth {
    pub devices: Vec<DeviceInfo>,
    pub free_percent: f32,
    pub last_scrub_days: Option<u32>,
    pub needs_balance: bool,
    pub balance_in_progress: bool,
    pub qgroups_enabled: bool,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device: String,
    pub size_gb: f32,
    pub used_gb: f32,
    pub free_percent: f32,
}

/// Bootloader detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootloaderInfo {
    pub detected: String, // "grub", "systemd-boot", "other", "unknown"
    pub grub_btrfs_installed: bool,
    pub snapshot_entries: Vec<String>,
}

/// Btrfs collector
pub struct BtrfsCollector {
    timeout_per_tool: Duration,
    overall_timeout: Duration,
}

impl Default for BtrfsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BtrfsCollector {
    pub fn new() -> Self {
        Self {
            timeout_per_tool: Duration::from_secs(2),
            overall_timeout: Duration::from_secs(6),
        }
    }

    /// Collect Btrfs profile
    pub async fn collect(&self) -> Result<BtrfsProfile> {
        let start = std::time::Instant::now();
        let mut notes = Vec::new();

        // Check if root is btrfs
        let root_fs = self.detect_root_filesystem().await?;
        let detected = root_fs == "btrfs";

        if !detected {
            return Ok(BtrfsProfile {
                version: "1".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                detected: false,
                layout: Self::empty_layout(),
                mount_opts: Self::empty_mount_opts(),
                tools: Self::empty_tools(),
                health: Self::empty_health(),
                bootloader: Self::empty_bootloader(),
                notes: vec!["Root filesystem is not Btrfs".to_string()],
            });
        }

        // Collect data with timeout budget
        let layout = match timeout(self.timeout_per_tool, self.collect_layout()).await {
            Ok(Ok(l)) => l,
            Ok(Err(e)) => {
                warn!("Failed to collect Btrfs layout: {}", e);
                notes.push(format!("layout collection failed: {}", e));
                Self::empty_layout()
            }
            Err(_) => {
                notes.push("layout collection timed out".to_string());
                Self::empty_layout()
            }
        };

        let mount_opts = match timeout(self.timeout_per_tool, self.collect_mount_opts()).await {
            Ok(Ok(m)) => m,
            Ok(Err(e)) => {
                warn!("Failed to collect mount options: {}", e);
                notes.push(format!("mount options collection failed: {}", e));
                Self::empty_mount_opts()
            }
            Err(_) => {
                notes.push("mount options collection timed out".to_string());
                Self::empty_mount_opts()
            }
        };

        let tools = self.detect_tools().await;

        let health = match timeout(self.timeout_per_tool, self.collect_health()).await {
            Ok(Ok(h)) => h,
            Ok(Err(e)) => {
                warn!("Failed to collect health: {}", e);
                notes.push(format!("health collection failed: {}", e));
                Self::empty_health()
            }
            Err(_) => {
                notes.push("health collection timed out".to_string());
                Self::empty_health()
            }
        };

        let bootloader = self.detect_bootloader().await;

        let elapsed = start.elapsed();
        if elapsed > self.overall_timeout {
            notes.push(format!("collection took {}ms (exceeded budget)", elapsed.as_millis()));
        }

        Ok(BtrfsProfile {
            version: "1".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            detected,
            layout,
            mount_opts,
            tools,
            health,
            bootloader,
            notes,
        })
    }

    /// Detect root filesystem type
    async fn detect_root_filesystem(&self) -> Result<String> {
        let output = Command::new("findmnt")
            .args(&["-no", "FSTYPE", "/"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .context("Failed to run findmnt")?;

        if output.status.success() {
            let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(fstype)
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Collect Btrfs layout
    async fn collect_layout(&self) -> Result<BtrfsLayout> {
        // Get subvolumes
        let subvolumes = self.parse_subvolumes().await?;

        // Detect default subvolume
        let default_subvol = self.detect_default_subvol().await?;

        // Detect snapshots directory
        let snapshots_dir = subvolumes
            .iter()
            .find(|s| s.path.contains("snapshot") || s.path.contains(".snapshot"))
            .map(|s| s.path.clone());

        // Check for separate home and var
        let has_separate_home = subvolumes.iter().any(|s| {
            s.mount_point.as_ref().map(|m| m == "/home").unwrap_or(false)
        });

        let has_separate_var = subvolumes.iter().any(|s| {
            s.mount_point.as_ref().map(|m| m.starts_with("/var")).unwrap_or(false)
        });

        // Detect boot and ESP
        let boot_fs_type = self.detect_filesystem("/boot").await.ok();
        let esp_mount = self.detect_esp_mount().await.ok();

        Ok(BtrfsLayout {
            subvolumes,
            default_subvol,
            snapshots_dir,
            has_separate_home,
            has_separate_var,
            root_fs_type: "btrfs".to_string(),
            boot_fs_type,
            esp_mount,
        })
    }

    /// Parse btrfs subvolume list
    async fn parse_subvolumes(&self) -> Result<Vec<Subvolume>> {
        let output = Command::new("btrfs")
            .args(&["subvolume", "list", "-o", "/"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut subvolumes = Vec::new();

        for line in stdout.lines() {
            // Format: ID 256 gen 12345 top level 5 path @
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let id = parts[1].to_string();
                let path = parts[8..].join(" ");

                subvolumes.push(Subvolume {
                    id,
                    path: path.clone(),
                    mount_point: self.find_mount_point(&path).await,
                    is_snapshot: path.contains("snapshot") || path.contains(".snapshot"),
                    readonly: false, // TODO: detect from subvolume show
                });
            }
        }

        Ok(subvolumes)
    }

    /// Find mount point for subvolume path
    async fn find_mount_point(&self, _subvol_path: &str) -> Option<String> {
        // TODO: Parse findmnt -J to match subvolume to mount point
        None
    }

    /// Detect default subvolume
    async fn detect_default_subvol(&self) -> Result<Option<String>> {
        let output = Command::new("btrfs")
            .args(&["subvolume", "get-default", "/"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Format: ID 256 gen 12345 top level 5 path @
                if let Some(path) = stdout.split("path ").nth(1) {
                    Ok(Some(path.trim().to_string()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Detect filesystem type for mount point
    async fn detect_filesystem(&self, mount_point: &str) -> Result<String> {
        let output = Command::new("findmnt")
            .args(&["-no", "FSTYPE", mount_point])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Detect ESP mount point
    async fn detect_esp_mount(&self) -> Result<String> {
        let output = Command::new("findmnt")
            .args(&["-no", "TARGET", "-t", "vfat"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await?;

        if output.status.success() {
            let mounts = String::from_utf8_lossy(&output.stdout);
            for mount in mounts.lines() {
                if mount.contains("/boot") || mount.contains("/efi") {
                    return Ok(mount.trim().to_string());
                }
            }
        }

        Ok("/boot/efi".to_string()) // Default
    }

    /// Collect mount options
    async fn collect_mount_opts(&self) -> Result<MountOptions> {
        let output = Command::new("findmnt")
            .args(&["-no", "OPTIONS", "/"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await?;

        let opts = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::new()
        };

        let compression = if opts.contains("compress=") {
            opts.split("compress=")
                .nth(1)
                .and_then(|s| s.split(',').next())
                .map(|s| s.to_string())
        } else if opts.contains("compress-force=") {
            opts.split("compress-force=")
                .nth(1)
                .and_then(|s| s.split(',').next())
                .map(|s| s.to_string())
        } else {
            None
        };

        let compression_level = compression.as_ref().and_then(|c| {
            if c.starts_with("zstd:") {
                c.strip_prefix("zstd:")
                    .and_then(|l| l.parse::<u8>().ok())
            } else {
                None
            }
        });

        Ok(MountOptions {
            compression,
            compression_level,
            autodefrag: opts.contains("autodefrag"),
            noatime: opts.contains("noatime"),
            space_cache: if opts.contains("space_cache=v2") {
                Some("v2".to_string())
            } else if opts.contains("space_cache") {
                Some("v1".to_string())
            } else {
                None
            },
            ssd: opts.contains("ssd"),
        })
    }

    /// Detect Btrfs tools
    async fn detect_tools(&self) -> BtrfsTools {
        BtrfsTools {
            snapper: self.command_exists("snapper").await,
            timeshift: self.command_exists("timeshift").await,
            btrfs_assistant: self.command_exists("btrfs-assistant").await,
            grub_btrfs: self.command_exists("grub-btrfs").await || std::path::Path::new("/etc/grub.d/41_snapshots-btrfs").exists(),
            pacman_hook: std::path::Path::new("/etc/pacman.d/hooks/90-btrfs-autosnap.hook").exists(),
        }
    }

    /// Check if command exists
    async fn command_exists(&self, cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Collect health metrics
    async fn collect_health(&self) -> Result<BtrfsHealth> {
        let devices = self.parse_devices().await?;
        let free_percent = if !devices.is_empty() {
            devices[0].free_percent
        } else {
            0.0
        };

        let last_scrub_days = self.detect_last_scrub().await.ok();
        let (needs_balance, balance_in_progress) = self.check_balance_status().await;
        let qgroups_enabled = self.check_qgroups().await;

        Ok(BtrfsHealth {
            devices,
            free_percent,
            last_scrub_days,
            needs_balance,
            balance_in_progress,
            qgroups_enabled,
        })
    }

    /// Parse btrfs filesystem show
    async fn parse_devices(&self) -> Result<Vec<DeviceInfo>> {
        let output = Command::new("btrfs")
            .args(&["filesystem", "show", "/"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // TODO: Parse output properly
        // For now, return empty or mock data
        if !stdout.is_empty() {
            devices.push(DeviceInfo {
                device: "/dev/sda".to_string(),
                size_gb: 1000.0,
                used_gb: 500.0,
                free_percent: 50.0,
            });
        }

        Ok(devices)
    }

    /// Detect last scrub time
    async fn detect_last_scrub(&self) -> Result<u32> {
        // TODO: Parse btrfs scrub status
        Ok(999) // Mock: over 30 days
    }

    /// Check balance status
    async fn check_balance_status(&self) -> (bool, bool) {
        // TODO: Parse btrfs balance status
        (false, false)
    }

    /// Check if qgroups enabled
    async fn check_qgroups(&self) -> bool {
        let output = Command::new("btrfs")
            .args(&["qgroup", "show", "/"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        output.map(|s| s.success()).unwrap_or(false)
    }

    /// Detect bootloader
    async fn detect_bootloader(&self) -> BootloaderInfo {
        let grub = std::path::Path::new("/boot/grub/grub.cfg").exists();
        let systemd_boot = std::path::Path::new("/boot/loader/loader.conf").exists();

        let detected = if grub {
            "grub".to_string()
        } else if systemd_boot {
            "systemd-boot".to_string()
        } else {
            "unknown".to_string()
        };

        let grub_btrfs_installed = grub && (
            self.command_exists("grub-btrfs").await ||
            std::path::Path::new("/etc/grub.d/41_snapshots-btrfs").exists()
        );

        let snapshot_entries = self.detect_snapshot_entries(&detected).await;

        BootloaderInfo {
            detected,
            grub_btrfs_installed,
            snapshot_entries,
        }
    }

    /// Detect snapshot boot entries
    async fn detect_snapshot_entries(&self, bootloader: &str) -> Vec<String> {
        if bootloader == "systemd-boot" {
            // TODO: Parse /boot/loader/entries for snapshot entries
        }
        Vec::new()
    }

    // Empty/default structures
    fn empty_layout() -> BtrfsLayout {
        BtrfsLayout {
            subvolumes: Vec::new(),
            default_subvol: None,
            snapshots_dir: None,
            has_separate_home: false,
            has_separate_var: false,
            root_fs_type: "unknown".to_string(),
            boot_fs_type: None,
            esp_mount: None,
        }
    }

    fn empty_mount_opts() -> MountOptions {
        MountOptions {
            compression: None,
            compression_level: None,
            autodefrag: false,
            noatime: false,
            space_cache: None,
            ssd: false,
        }
    }

    fn empty_tools() -> BtrfsTools {
        BtrfsTools {
            snapper: false,
            timeshift: false,
            btrfs_assistant: false,
            grub_btrfs: false,
            pacman_hook: false,
        }
    }

    fn empty_health() -> BtrfsHealth {
        BtrfsHealth {
            devices: Vec::new(),
            free_percent: 0.0,
            last_scrub_days: None,
            needs_balance: false,
            balance_in_progress: false,
            qgroups_enabled: false,
        }
    }

    fn empty_bootloader() -> BootloaderInfo {
        BootloaderInfo {
            detected: "unknown".to_string(),
            grub_btrfs_installed: false,
            snapshot_entries: Vec::new(),
        }
    }
}

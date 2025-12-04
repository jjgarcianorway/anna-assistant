//! SD Card Discovery v7.25.0
//!
//! Discovers SD/MMC card readers from /sys/block, lsblk, and lspci.

use super::types::{SdCardReader, SdCardSummary};
use std::process::Command;

/// Get SD card reader summary
pub fn get_sdcard_summary() -> SdCardSummary {
    let mut summary = SdCardSummary {
        reader_count: 0,
        readers: Vec::new(),
        source: "lsblk, lspci, /sys/block".to_string(),
    };

    // Look for MMC block devices
    let block_path = std::path::Path::new("/sys/block");
    if !block_path.exists() {
        check_sd_controllers(&mut summary);
        summary.reader_count = summary.readers.len() as u32;
        return summary;
    }

    if let Ok(entries) = std::fs::read_dir(block_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // MMC devices are mmcblk*
            if !name.starts_with("mmcblk") {
                continue;
            }

            // Skip partitions (mmcblk0p1)
            if name.contains('p') && name.chars().last().map(|c| c.is_numeric()).unwrap_or(false) {
                continue;
            }

            let device_path = entry.path();
            let mut reader = SdCardReader {
                name: name.clone(),
                driver: String::new(),
                bus: String::new(),
                device_path: Some(format!("/dev/{}", name)),
                media_present: false,
                media_size: None,
                media_fs: None,
            };

            // Get driver
            let driver_path = device_path.join("device/driver");
            if let Ok(link) = std::fs::read_link(&driver_path) {
                reader.driver = link
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
            }

            // Check if media is present by looking at size
            if let Ok(size_str) = std::fs::read_to_string(device_path.join("size")) {
                if let Ok(sectors) = size_str.trim().parse::<u64>() {
                    if sectors > 0 {
                        reader.media_present = true;
                        reader.media_size = Some(sectors * 512);
                        reader.media_fs = get_partition_fs(&format!("{}p1", name));
                    }
                }
            }

            // Determine bus type
            let modalias_path = device_path.join("device/modalias");
            if let Ok(modalias) = std::fs::read_to_string(&modalias_path) {
                if modalias.contains("pci:") {
                    reader.bus = "PCIe".to_string();
                } else if modalias.contains("usb:") {
                    reader.bus = "USB".to_string();
                }
            }

            summary.readers.push(reader);
        }
    }

    // Also check for rtsx_pci or sdhci controllers in lspci
    check_sd_controllers(&mut summary);

    summary.reader_count = summary.readers.len() as u32;
    summary
}

fn check_sd_controllers(summary: &mut SdCardSummary) {
    let output = Command::new("lspci").args(["-k"]).output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if line.contains("SD Host controller")
                    || line.contains("Card reader")
                    || line.contains("RTS5")
                    || line.contains("SDHCI")
                {
                    if summary.readers.is_empty() {
                        summary.readers.push(SdCardReader {
                            name: "SD Card Reader".to_string(),
                            driver: if line.contains("rtsx") {
                                "rtsx_pci".to_string()
                            } else {
                                "sdhci".to_string()
                            },
                            bus: "PCIe".to_string(),
                            device_path: None,
                            media_present: false,
                            media_size: None,
                            media_fs: None,
                        });
                    }
                    break;
                }
            }
        }
    }
}

fn get_partition_fs(partition: &str) -> Option<String> {
    let output = Command::new("blkid")
        .args(["-o", "value", "-s", "TYPE", &format!("/dev/{}", partition)])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let fs = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !fs.is_empty() {
                return Some(fs);
            }
        }
    }
    None
}

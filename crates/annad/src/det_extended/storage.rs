//! Storage answer functions (v0.0.806).
//!
//! Block devices, ZFS, LVM, RAID, fstab, swap, mounted filesystems.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer block devices query
pub fn answer_block_devices(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "block_devices")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No block devices found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Block devices ({}):\n```\n{}\n```", device_count, output),
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

/// Answer installed kernels query
pub fn answer_installed_kernels(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "installed_kernels")?;
    if probe.exit_code != 0 && probe.stdout.is_empty() {
        return Some(DeterministicResult {
            answer: "Could not determine installed kernels.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No kernels found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let kernel_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Installed kernels ({}):\n{}", kernel_count, output),
        grounded: true,
        parsed_data_count: kernel_count,
        route_class: route_class.to_string(),
    })
}

/// Answer ZFS status query
pub fn answer_zfs_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "zfs_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "ZFS is not installed on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    if output.contains("no pools available") {
        return Some(DeterministicResult {
            answer: "ZFS is installed but no pools are configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("ZFS pool status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer boot loader query
pub fn answer_boot_loader(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "boot_loader")?;

    let output = probe.stdout.trim();
    if output.contains("not detected") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Could not detect boot loader configuration.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let loader_type =
        if output.contains("systemd-boot") || output.contains("Boot Loader Specification") {
            "systemd-boot"
        } else if output.contains("GRUB") || output.contains("grub") {
            "GRUB"
        } else {
            "Unknown"
        };

    Some(DeterministicResult {
        answer: format!("Boot loader: {}\n```\n{}\n```", loader_type, output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer mounted filesystems query using findmnt
pub fn answer_mounted_filesystems(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "findmnt")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No mounted filesystems found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let mount_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Mounted filesystems ({}):\n{}", mount_count, output),
        grounded: true,
        parsed_data_count: mount_count,
        route_class: route_class.to_string(),
    })
}

/// Answer LVM status query
pub fn answer_lvm_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "lvm_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.contains("no volumes") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "LVM is not installed or no logical volumes configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("LVM status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer RAID status query
pub fn answer_raid_status(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "raid_status")?;

    let output = probe.stdout.trim();
    if output.contains("No RAID") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No software RAID (mdadm) detected on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("RAID status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer fstab entries query
pub fn answer_fstab_entries(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "fstab_entries")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No entries found in /etc/fstab.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (
            format!("/etc/fstab ({} entries):\n```\n{}\n```", count, output),
            count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer swap files query using /proc/swaps
pub fn answer_swap_files(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "swap_files")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() || output.lines().count() <= 1 {
        ("No swap files or partitions configured.".to_string(), 0)
    } else {
        let swap_count = output.lines().count() - 1;
        (
            format!(
                "Swap configuration ({} entries):\n```\n{}\n```",
                swap_count, output
            ),
            swap_count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd mounts query using systemctl
pub fn answer_systemd_mounts(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_mounts")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No systemd mount units found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (
            format!("Systemd mount units ({}):\n```\n{}\n```", count, output),
            count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// v0.0.806: Answer largest folders query using du output
pub fn answer_largest_folders(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    // Try largest_dirs first, then largest_home
    let dirs_probe = find_probe(probes, "largest_dirs");
    let home_probe = find_probe(probes, "largest_home");

    let mut results: Vec<String> = Vec::new();

    // Parse largest_dirs output
    if let Some(probe) = dirs_probe {
        let output = probe.stdout.trim();
        if !output.is_empty() && !output.contains("timed out") {
            results.push("**System directories:**".to_string());
            for line in output.lines().take(10) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let size = parts[0];
                    let path = parts[1..].join(" ");
                    results.push(format!("  {} - {}", size, path));
                }
            }
        }
    }

    // Parse largest_home output
    if let Some(probe) = home_probe {
        let output = probe.stdout.trim();
        if !output.is_empty() && !output.contains("timed out") {
            if !results.is_empty() {
                results.push(String::new()); // blank line separator
            }
            results.push("**Home directory:**".to_string());
            for line in output.lines().take(10) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let size = parts[0];
                    let path = parts[1..].join(" ");
                    results.push(format!("  {} - {}", size, path));
                }
            }
        }
    }

    if results.is_empty() {
        return Some(DeterministicResult {
            answer: "Could not scan directories. Try checking specific paths manually with `du -sh /path/*`".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let folder_count = results.iter().filter(|l| l.starts_with("  ")).count();
    Some(DeterministicResult {
        answer: format!("**Largest folders ({}):**\n{}", folder_count, results.join("\n")),
        grounded: true,
        parsed_data_count: folder_count,
        route_class: route_class.to_string(),
    })
}

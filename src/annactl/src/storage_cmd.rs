// Anna v0.12.5 - Storage Command Module (Btrfs Intelligence)
// Displays Btrfs profile with TUI formatting

use anyhow::{Context, Result};
use anna_common::tui::{self, Level, TermCaps};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::rpc_call;

/// Btrfs storage profile (mirrors daemon types)
#[derive(Debug, Serialize, Deserialize)]
struct BtrfsProfile {
    version: String,
    generated_at: String,
    detected: bool,
    layout: BtrfsLayout,
    mount_opts: MountOptions,
    tools: BtrfsTools,
    health: BtrfsHealth,
    bootloader: BootloaderInfo,
    notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BtrfsLayout {
    subvolumes: Vec<Subvolume>,
    default_subvol: Option<String>,
    snapshots_dir: Option<String>,
    has_separate_home: bool,
    has_separate_var: bool,
    root_fs_type: String,
    boot_fs_type: Option<String>,
    esp_mount: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Subvolume {
    id: String,
    path: String,
    mount_point: Option<String>,
    is_snapshot: bool,
    readonly: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MountOptions {
    compression: Option<String>,
    compression_level: Option<u8>,
    autodefrag: bool,
    noatime: bool,
    space_cache: Option<String>,
    ssd: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BtrfsTools {
    snapper: bool,
    timeshift: bool,
    btrfs_assistant: bool,
    grub_btrfs: bool,
    pacman_hook: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BtrfsHealth {
    devices: Vec<DeviceInfo>,
    free_percent: f32,
    last_scrub_days: Option<u32>,
    needs_balance: bool,
    balance_in_progress: bool,
    qgroups_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceInfo {
    device: String,
    size_gb: f32,
    used_gb: f32,
    free_percent: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct BootloaderInfo {
    detected: String,
    grub_btrfs_installed: bool,
    snapshot_entries: Vec<String>,
}

/// Show Btrfs storage profile
pub async fn show_btrfs(json: bool, wide: bool, explain: Option<String>) -> Result<()> {
    // Call RPC
    let response: JsonValue = rpc_call("storage_profile", None)
        .await
        .context("Failed to get storage profile from daemon")?;

    let profile: BtrfsProfile =
        serde_json::from_value(response).context("Invalid storage profile response")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&profile)?);
        return Ok(());
    }

    // If explain flag provided, show detailed explanation for a specific aspect
    if let Some(topic) = explain {
        return show_explanation(&topic, &profile);
    }

    // TUI rendering
    let caps = TermCaps::detect();

    // Header
    println!("{}", tui::header(&caps, "Anna Storage Profile (Btrfs)"));
    println!();

    // Detection status
    if !profile.detected {
        println!(
            "{}",
            tui::status(
                &caps,
                Level::Info,
                "Btrfs not detected - root filesystem is not Btrfs"
            )
        );
        if !profile.notes.is_empty() {
            for note in &profile.notes {
                println!("  {}", tui::dim(&caps, note));
            }
        }
        return Ok(());
    }

    println!(
        "{}",
        tui::status(&caps, Level::Ok, "Btrfs filesystem detected")
    );
    println!();

    // Layout section
    println!("{}", tui::section(&caps, "Layout"));
    println!(
        "{}",
        tui::kv(&caps, "Root FS:", &profile.layout.root_fs_type)
    );

    if let Some(ref default_subvol) = profile.layout.default_subvol {
        println!("{}", tui::kv(&caps, "Default:", default_subvol));
    }

    if let Some(ref snapshots_dir) = profile.layout.snapshots_dir {
        println!("{}", tui::kv(&caps, "Snapshots:", snapshots_dir));
    } else {
        println!(
            "{}",
            tui::kv(
                &caps,
                "Snapshots:",
                &tui::dim(&caps, "not configured")
            )
        );
    }

    println!(
        "{}",
        tui::kv(
            &caps,
            "Separate /home:",
            if profile.layout.has_separate_home {
                "yes"
            } else {
                "no"
            }
        )
    );
    println!(
        "{}",
        tui::kv(
            &caps,
            "Separate /var:",
            if profile.layout.has_separate_var {
                "yes"
            } else {
                "no"
            }
        )
    );
    println!();

    // Subvolumes (in wide mode or if < 10)
    if wide || profile.layout.subvolumes.len() <= 10 {
        println!("{}", tui::section(&caps, "Subvolumes"));
        if profile.layout.subvolumes.is_empty() {
            println!("  {}", tui::dim(&caps, "No subvolumes detected"));
        } else {
            let headers = vec!["ID", "Path", "Mount", "Snapshot", "RO"];
            let rows: Vec<Vec<String>> = profile
                .layout
                .subvolumes
                .iter()
                .map(|s| {
                    vec![
                        s.id.clone(),
                        s.path.clone(),
                        s.mount_point.as_ref().unwrap_or(&"-".to_string()).clone(),
                        if s.is_snapshot { "yes" } else { "no" }.to_string(),
                        if s.readonly { "yes" } else { "no" }.to_string(),
                    ]
                })
                .collect();
            print!("{}", tui::table(&caps, &headers, &rows));
        }
        println!();
    } else {
        let snapshot_count = profile
            .layout
            .subvolumes
            .iter()
            .filter(|s| s.is_snapshot)
            .count();
        println!(
            "{}",
            tui::kv(
                &caps,
                "Subvolumes:",
                &format!(
                    "{} total ({} snapshots)",
                    profile.layout.subvolumes.len(),
                    snapshot_count
                )
            )
        );
        println!(
            "  {}",
            tui::hint(&caps, "Use --wide to see all subvolumes")
        );
        println!();
    }

    // Mount options
    println!("{}", tui::section(&caps, "Mount Options"));
    if let Some(ref comp) = profile.mount_opts.compression {
        println!("{}", tui::kv(&caps, "Compression:", comp));
    } else {
        println!(
            "{}",
            tui::kv(&caps, "Compression:", &tui::warn(&caps, "disabled"))
        );
    }

    println!(
        "{}",
        tui::kv(
            &caps,
            "Autodefrag:",
            if profile.mount_opts.autodefrag {
                "enabled"
            } else {
                "disabled"
            }
        )
    );
    println!(
        "{}",
        tui::kv(
            &caps,
            "NoAtime:",
            if profile.mount_opts.noatime {
                "yes"
            } else {
                "no"
            }
        )
    );
    println!(
        "{}",
        tui::kv(
            &caps,
            "SSD:",
            if profile.mount_opts.ssd {
                "yes"
            } else {
                "no"
            }
        )
    );

    if let Some(ref sc) = profile.mount_opts.space_cache {
        println!("{}", tui::kv(&caps, "Space cache:", sc));
    }
    println!();

    // Tools
    println!("{}", tui::section(&caps, "Tools"));
    render_tool_status(&caps, "Snapper:", profile.tools.snapper);
    render_tool_status(&caps, "Timeshift:", profile.tools.timeshift);
    render_tool_status(&caps, "Btrfs Assist:", profile.tools.btrfs_assistant);
    render_tool_status(&caps, "GRUB-btrfs:", profile.tools.grub_btrfs);
    render_tool_status(&caps, "Pacman hook:", profile.tools.pacman_hook);
    println!();

    // Bootloader
    println!("{}", tui::section(&caps, "Bootloader"));
    println!("{}", tui::kv(&caps, "Detected:", &profile.bootloader.detected));

    if profile.bootloader.detected == "grub" {
        let grub_status = if profile.bootloader.grub_btrfs_installed {
            tui::ok(&caps, "installed")
        } else {
            tui::warn(&caps, "not installed")
        };
        println!("{}", tui::kv(&caps, "GRUB-btrfs:", &grub_status));
    }

    if !profile.bootloader.snapshot_entries.is_empty() {
        println!(
            "{}",
            tui::kv(
                &caps,
                "Boot entries:",
                &format!("{} snapshots", profile.bootloader.snapshot_entries.len())
            )
        );
    }
    println!();

    // Health
    println!("{}", tui::section(&caps, "Health"));

    // Devices
    if !profile.health.devices.is_empty() {
        for dev in &profile.health.devices {
            println!(
                "{}",
                tui::kv(
                    &caps,
                    "Device:",
                    &format!(
                        "{} ({:.1} GB total, {:.1}% free)",
                        dev.device, dev.size_gb, dev.free_percent
                    )
                )
            );
        }
    }

    // Free space
    let free_color = if profile.health.free_percent < 10.0 {
        tui::err(&caps, &format!("{:.1}%", profile.health.free_percent))
    } else if profile.health.free_percent < 20.0 {
        tui::warn(&caps, &format!("{:.1}%", profile.health.free_percent))
    } else {
        tui::ok(&caps, &format!("{:.1}%", profile.health.free_percent))
    };
    println!("{}", tui::kv(&caps, "Free space:", &free_color));

    // Scrub
    if let Some(days) = profile.health.last_scrub_days {
        let scrub_str = if days > 30 {
            tui::warn(&caps, &format!("{} days ago (overdue)", days))
        } else {
            format!("{} days ago", days)
        };
        println!("{}", tui::kv(&caps, "Last scrub:", &scrub_str));
    } else {
        println!(
            "{}",
            tui::kv(&caps, "Last scrub:", &tui::dim(&caps, "unknown"))
        );
    }

    // Balance
    if profile.health.balance_in_progress {
        println!(
            "{}",
            tui::kv(&caps, "Balance:", &tui::primary(&caps, "in progress"))
        );
    } else if profile.health.needs_balance {
        println!(
            "{}",
            tui::kv(&caps, "Balance:", &tui::warn(&caps, "recommended"))
        );
    } else {
        println!("{}", tui::kv(&caps, "Balance:", "not needed"));
    }

    // Qgroups
    println!(
        "{}",
        tui::kv(
            &caps,
            "Qgroups:",
            if profile.health.qgroups_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    );
    println!();

    // Notes
    if !profile.notes.is_empty() {
        println!("{}", tui::section(&caps, "Notes"));
        for note in &profile.notes {
            println!("{}", tui::bullet(&caps, note));
        }
        println!();
    }

    // Footer hints
    println!("{}", tui::hint(&caps, "Tips:"));
    println!("{}", tui::bullet(&caps, "Use --json for machine-readable output"));
    println!(
        "{}",
        tui::bullet(&caps, "Use --wide to see all subvolumes")
    );
    println!(
        "{}",
        tui::bullet(
            &caps,
            "Use --explain <topic> for detailed info on: snapshots, compression, scrub, balance"
        )
    );
    println!();
    println!(
        "{}",
        tui::hint(&caps, "Run 'annactl advisor arch' to see storage recommendations")
    );

    Ok(())
}

/// Show detailed explanation for a topic
fn show_explanation(topic: &str, profile: &BtrfsProfile) -> Result<()> {
    let caps = TermCaps::detect();

    match topic {
        "snapshots" => {
            println!("{}", tui::header(&caps, "Btrfs Snapshots Explained"));
            println!();
            println!("Btrfs snapshots are instant, zero-copy backups of subvolumes.");
            println!("They allow you to recover from failed updates or accidental deletions.");
            println!();
            println!("{}", tui::section(&caps, "Current Status"));
            if let Some(ref dir) = profile.layout.snapshots_dir {
                println!("{}", tui::status(&caps, Level::Ok, &format!("Snapshots configured at: {}", dir)));
            } else {
                println!("{}", tui::status(&caps, Level::Warn, "No snapshots directory configured"));
            }
            if profile.tools.snapper || profile.tools.timeshift {
                println!("{}", tui::status(&caps, Level::Ok, "Snapshot tool installed"));
            } else {
                println!("{}", tui::status(&caps, Level::Warn, "No snapshot tool installed (install snapper or timeshift)"));
            }
            println!();
            println!("{}", tui::section(&caps, "Recommended Actions"));
            if !profile.tools.snapper && !profile.tools.timeshift {
                println!("{}", tui::code(&caps, "sudo pacman -S snapper"));
                println!("{}", tui::code(&caps, "sudo snapper -c root create-config /"));
            }
            if !profile.tools.pacman_hook {
                println!("{}", tui::code(&caps, "sudo pacman -S snap-pac  # Auto-snapshot before pacman operations"));
            }
        }
        "compression" => {
            println!("{}", tui::header(&caps, "Btrfs Compression Explained"));
            println!();
            println!("Zstd compression (level 3) provides the best balance of compression ratio,");
            println!("speed, and CPU usage. It typically reduces disk usage by 30-50%.");
            println!();
            println!("{}", tui::section(&caps, "Current Status"));
            if let Some(ref comp) = profile.mount_opts.compression {
                if comp.starts_with("zstd") {
                    println!("{}", tui::status(&caps, Level::Ok, &format!("Optimal: {}", comp)));
                } else {
                    println!("{}", tui::status(&caps, Level::Warn, &format!("Suboptimal: {} (zstd recommended)", comp)));
                }
            } else {
                println!("{}", tui::status(&caps, Level::Warn, "Compression disabled"));
            }
            println!();
            println!("{}", tui::section(&caps, "Recommended Action"));
            println!("{}", tui::hint(&caps, "Update /etc/fstab to include compress=zstd:3 in mount options"));
            println!("{}", tui::code(&caps, "sudo mount -o remount,compress=zstd:3 /"));
        }
        "scrub" => {
            println!("{}", tui::header(&caps, "Btrfs Scrub Explained"));
            println!();
            println!("Btrfs scrub reads all data and metadata, verifying checksums and");
            println!("repairing corruption. Monthly scrubs catch bit rot and disk errors.");
            println!();
            println!("{}", tui::section(&caps, "Current Status"));
            if let Some(days) = profile.health.last_scrub_days {
                if days > 30 {
                    println!("{}", tui::status(&caps, Level::Warn, &format!("Last scrub: {} days ago (overdue)", days)));
                } else {
                    println!("{}", tui::status(&caps, Level::Ok, &format!("Last scrub: {} days ago", days)));
                }
            } else {
                println!("{}", tui::status(&caps, Level::Info, "Last scrub: unknown"));
            }
            println!();
            println!("{}", tui::section(&caps, "Recommended Actions"));
            println!("{}", tui::code(&caps, "sudo btrfs scrub start -Bd /  # Manual scrub"));
            println!("{}", tui::code(&caps, "sudo systemctl enable --now btrfs-scrub@-.timer  # Monthly auto-scrub"));
        }
        "balance" => {
            println!("{}", tui::header(&caps, "Btrfs Balance Explained"));
            println!();
            println!("Btrfs allocates space in chunks. Over time, partially-used chunks");
            println!("accumulate. Balance rewrites data to fewer chunks, reclaiming space.");
            println!();
            println!("{}", tui::section(&caps, "Current Status"));
            if profile.health.balance_in_progress {
                println!("{}", tui::status(&caps, Level::Info, "Balance currently in progress"));
            } else if profile.health.needs_balance {
                println!("{}", tui::status(&caps, Level::Warn, "Balance recommended"));
            } else {
                println!("{}", tui::status(&caps, Level::Ok, "Balance not needed"));
            }
            println!();
            println!("{}", tui::section(&caps, "Recommended Actions"));
            println!("{}", tui::code(&caps, "sudo btrfs balance start -dusage=50 -musage=50 /"));
            println!("{}", tui::hint(&caps, "This is safe and only rewrites highly fragmented chunks"));
        }
        _ => {
            println!("{}", tui::status(&caps, Level::Err, &format!("Unknown topic: {}", topic)));
            println!();
            println!("{}", tui::hint(&caps, "Available topics: snapshots, compression, scrub, balance"));
            return Ok(());
        }
    }

    Ok(())
}

/// Render tool installation status
fn render_tool_status(caps: &TermCaps, label: &str, installed: bool) {
    let value = if installed {
        tui::ok(caps, "installed")
    } else {
        tui::dim(caps, "not installed")
    };
    println!("{}", tui::kv(caps, label, &value));
}

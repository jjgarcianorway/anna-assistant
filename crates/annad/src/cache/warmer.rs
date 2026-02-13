//! Pre-warmer task that populates cache with common system queries.

use super::types::{InvalidationTag, SystemCache};
use std::process::Command;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, warn};

/// Commands to pre-warm with their cache metadata.
struct WarmCommand {
    key: &'static str,
    cmd: &'static str,
    args: &'static [&'static str],
    ttl_secs: u64,
    tags: &'static [InvalidationTag],
}

const WARM_COMMANDS: &[WarmCommand] = &[
    // Block devices (invalidated by udev block events)
    WarmCommand {
        key: "lsblk_devices",
        cmd: "lsblk",
        args: &["-d", "-o", "NAME,SIZE,TYPE,ROTA"],
        ttl_secs: 300,
        tags: &[InvalidationTag::BlockDevice],
    },
    // Disk usage (invalidated by fstab changes, but also short TTL)
    WarmCommand {
        key: "df_usage",
        cmd: "df",
        args: &["-h", "--output=source,size,used,avail,pcent,target"],
        ttl_secs: 60,
        tags: &[InvalidationTag::Fstab],
    },
    // Memory (short TTL, no event tracking for memory usage changes)
    WarmCommand {
        key: "free_memory",
        cmd: "free",
        args: &["-h"],
        ttl_secs: 15,
        tags: &[InvalidationTag::Memory],
    },
    // IP addresses
    WarmCommand {
        key: "ip_addr",
        cmd: "ip",
        args: &["addr", "show"],
        ttl_secs: 30,
        tags: &[InvalidationTag::Network],
    },
    // Routes
    WarmCommand {
        key: "ip_route",
        cmd: "ip",
        args: &["route", "show"],
        ttl_secs: 30,
        tags: &[InvalidationTag::Network],
    },
    // PCI devices (hardware changes are rare)
    WarmCommand {
        key: "lspci",
        cmd: "lspci",
        args: &[],
        ttl_secs: 300,
        tags: &[InvalidationTag::Hardware],
    },
    // Failed services
    WarmCommand {
        key: "systemctl_failed",
        cmd: "systemctl",
        args: &["--failed", "--no-pager"],
        ttl_secs: 30,
        tags: &[InvalidationTag::Services],
    },
    // Top CPU consumers
    WarmCommand {
        key: "ps_cpu",
        cmd: "/bin/sh",
        args: &["-c", "ps aux --sort=-%cpu | head -15"],
        ttl_secs: 15,
        tags: &[InvalidationTag::Process],
    },
    // Top memory consumers
    WarmCommand {
        key: "ps_mem",
        cmd: "/bin/sh",
        args: &["-c", "ps aux --sort=-%mem | head -15"],
        ttl_secs: 15,
        tags: &[InvalidationTag::Process],
    },
    // Open ports
    WarmCommand {
        key: "ss_ports",
        cmd: "ss",
        args: &["-tulpn"],
        ttl_secs: 30,
        tags: &[InvalidationTag::Network],
    },
    // Recent reboots
    WarmCommand {
        key: "last_reboot",
        cmd: "/bin/sh",
        args: &["-c", "last reboot | head -5"],
        ttl_secs: 300,
        tags: &[InvalidationTag::Bootloader],
    },
    // User groups
    WarmCommand {
        key: "groups",
        cmd: "groups",
        args: &[],
        ttl_secs: 300,
        tags: &[],
    },
    // Kernel version (very stable)
    WarmCommand {
        key: "uname_kernel",
        cmd: "uname",
        args: &["-r"],
        ttl_secs: 3600,
        tags: &[InvalidationTag::Bootloader],
    },
    // CPU info (very stable)
    WarmCommand {
        key: "cpuinfo",
        cmd: "cat",
        args: &["/proc/cpuinfo"],
        ttl_secs: 3600,
        tags: &[InvalidationTag::Hardware],
    },
    // Root filesystem
    WarmCommand {
        key: "findmnt_root",
        cmd: "findmnt",
        args: &["/", "-o", "SOURCE,TARGET,FSTYPE,SIZE,USED"],
        ttl_secs: 300,
        tags: &[InvalidationTag::Fstab],
    },
    // Available updates (longer TTL)
    WarmCommand {
        key: "checkupdates",
        cmd: "/bin/sh",
        args: &["-c", "checkupdates 2>/dev/null | head -20"],
        ttl_secs: 600,
        tags: &[InvalidationTag::Packages],
    },
];

/// Background task that pre-warms cache every 30 seconds.
pub async fn warmer_loop(cache: SystemCache) {
    debug!("Cache warmer starting (30s interval)");
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;
        warm_cache(&cache);
    }
}

fn warm_cache(cache: &SystemCache) {
    for cmd in WARM_COMMANDS {
        // Only warm if not already cached
        if cache.get(cmd.key).is_some() {
            continue;
        }

        match run_command(cmd.cmd, cmd.args) {
            Ok(output) => {
                if !output.is_empty() {
                    cache.set(
                        cmd.key.to_string(),
                        output,
                        Duration::from_secs(cmd.ttl_secs),
                        cmd.tags.to_vec(),
                    );
                    debug!("Warmed cache: {}", cmd.key);
                }
            }
            Err(e) => {
                // Silently skip failed commands (e.g., checkupdates on non-Arch)
                if cmd.key != "checkupdates" {
                    warn!("Failed to warm {}: {}", cmd.key, e);
                }
            }
        }
    }
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, std::io::Error> {
    let output = Command::new(cmd).args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

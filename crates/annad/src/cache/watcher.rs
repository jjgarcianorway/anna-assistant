//! Event-driven cache invalidation through filesystem/proc polling.

use super::types::{InvalidationTag, SystemCache};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{debug, warn};

/// State tracker for detecting changes.
struct WatcherState {
    pci_count: usize,
    block_count: usize,
    net_count: usize,
    meminfo_hash: u64,
    partitions_hash: u64,
    grub_mtime: Option<SystemTime>,
    fstab_mtime: Option<SystemTime>,
    pacman_count: usize,
    systemd_mtime: Option<SystemTime>,
    resolv_mtime: Option<SystemTime>,
}

impl WatcherState {
    fn new() -> Self {
        Self {
            pci_count: 0,
            block_count: 0,
            net_count: 0,
            meminfo_hash: 0,
            partitions_hash: 0,
            grub_mtime: None,
            fstab_mtime: None,
            pacman_count: 0,
            systemd_mtime: None,
            resolv_mtime: None,
        }
    }
}

/// Background task that polls for system changes and invalidates cache.
pub async fn watcher_loop(cache: SystemCache) {
    debug!("Cache watcher starting (5s poll interval)");
    let mut state = WatcherState::new();
    let mut interval = interval(Duration::from_secs(5));

    // Initial population
    update_state(&mut state);

    loop {
        interval.tick().await;

        // Check for changes and invalidate
        check_and_invalidate(&cache, &mut state);

        // Periodic cleanup of expired entries
        cache.cleanup_expired();
    }
}

fn check_and_invalidate(cache: &SystemCache, state: &mut WatcherState) {
    // Hardware: PCI device count
    if let Ok(count) = count_entries("/sys/bus/pci/devices") {
        if count != state.pci_count && state.pci_count > 0 {
            debug!("PCI device count changed: {} → {}", state.pci_count, count);
            cache.invalidate_tag(InvalidationTag::Hardware);
        }
        state.pci_count = count;
    }

    // Block devices
    if let Ok(count) = count_entries("/sys/block") {
        if count != state.block_count && state.block_count > 0 {
            debug!("Block device count changed: {} → {}", state.block_count, count);
            cache.invalidate_tag(InvalidationTag::BlockDevice);
            cache.invalidate_tag(InvalidationTag::Partitions);
        }
        state.block_count = count;
    }

    // Network interfaces
    if let Ok(count) = count_entries("/sys/class/net") {
        if count != state.net_count && state.net_count > 0 {
            debug!("Network interface count changed: {} → {}", state.net_count, count);
            cache.invalidate_tag(InvalidationTag::Network);
        }
        state.net_count = count;
    }

    // Memory: hash first line of /proc/meminfo (MemTotal)
    if let Ok(hash) = hash_file_first_line("/proc/meminfo") {
        if hash != state.meminfo_hash && state.meminfo_hash > 0 {
            debug!("Memory configuration changed");
            cache.invalidate_tag(InvalidationTag::Memory);
        }
        state.meminfo_hash = hash;
    }

    // Partitions: hash /proc/partitions
    if let Ok(hash) = hash_file("/proc/partitions") {
        if hash != state.partitions_hash && state.partitions_hash > 0 {
            debug!("Partition table changed");
            cache.invalidate_tag(InvalidationTag::Partitions);
            cache.invalidate_tag(InvalidationTag::BlockDevice);
        }
        state.partitions_hash = hash;
    }

    // Bootloader: grub.cfg or loader entries mtime
    for grub_path in &["/boot/grub/grub.cfg", "/boot/loader/entries"] {
        if let Ok(mtime) = get_mtime(grub_path) {
            if let Some(prev) = state.grub_mtime {
                if mtime > prev {
                    debug!("Bootloader config changed: {}", grub_path);
                    cache.invalidate_tag(InvalidationTag::Bootloader);
                }
            }
            state.grub_mtime = Some(mtime);
            break;
        }
    }

    // Fstab
    if let Ok(mtime) = get_mtime("/etc/fstab") {
        if let Some(prev) = state.fstab_mtime {
            if mtime > prev {
                debug!("fstab changed");
                cache.invalidate_tag(InvalidationTag::Fstab);
            }
        }
        state.fstab_mtime = Some(mtime);
    }

    // Packages: /var/lib/pacman/local entry count
    if let Ok(count) = count_entries("/var/lib/pacman/local") {
        if count != state.pacman_count && state.pacman_count > 0 {
            debug!("Package count changed: {} → {}", state.pacman_count, count);
            cache.invalidate_tag(InvalidationTag::Packages);
            cache.invalidate_tag(InvalidationTag::Services); // services may be added/removed
        }
        state.pacman_count = count;
    }

    // Systemd services
    if let Ok(mtime) = get_mtime("/etc/systemd/system") {
        if let Some(prev) = state.systemd_mtime {
            if mtime > prev {
                debug!("Systemd services changed");
                cache.invalidate_tag(InvalidationTag::Services);
            }
        }
        state.systemd_mtime = Some(mtime);
    }

    // DNS config
    if let Ok(mtime) = get_mtime("/etc/resolv.conf") {
        if let Some(prev) = state.resolv_mtime {
            if mtime > prev {
                debug!("DNS config changed");
                cache.invalidate_tag(InvalidationTag::DnsConfig);
            }
        }
        state.resolv_mtime = Some(mtime);
    }
}

fn update_state(state: &mut WatcherState) {
    state.pci_count = count_entries("/sys/bus/pci/devices").unwrap_or(0);
    state.block_count = count_entries("/sys/block").unwrap_or(0);
    state.net_count = count_entries("/sys/class/net").unwrap_or(0);
    state.meminfo_hash = hash_file_first_line("/proc/meminfo").unwrap_or(0);
    state.partitions_hash = hash_file("/proc/partitions").unwrap_or(0);
    state.grub_mtime = get_mtime("/boot/grub/grub.cfg")
        .or_else(|_| get_mtime("/boot/loader/entries"))
        .ok();
    state.fstab_mtime = get_mtime("/etc/fstab").ok();
    state.pacman_count = count_entries("/var/lib/pacman/local").unwrap_or(0);
    state.systemd_mtime = get_mtime("/etc/systemd/system").ok();
    state.resolv_mtime = get_mtime("/etc/resolv.conf").ok();
}

// Helper functions

fn count_entries<P: AsRef<Path>>(path: P) -> Result<usize, std::io::Error> {
    Ok(fs::read_dir(path)?.count())
}

fn get_mtime<P: AsRef<Path>>(path: P) -> Result<SystemTime, std::io::Error> {
    Ok(fs::metadata(path)?.modified()?)
}

fn hash_file<P: AsRef<Path>>(path: P) -> Result<u64, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(hasher.finish())
}

fn hash_file_first_line<P: AsRef<Path>>(path: P) -> Result<u64, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let first_line = content.lines().next().unwrap_or("");
    let mut hasher = DefaultHasher::new();
    first_line.hash(&mut hasher);
    Ok(hasher.finish())
}

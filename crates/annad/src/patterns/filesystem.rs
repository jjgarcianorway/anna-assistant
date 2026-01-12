//! Filesystem patterns for mounts, permissions, LVM, RAID, btrfs.
//! v0.0.962: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a filesystem-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type FsPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match filesystem-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_mounts(q)
        .or_else(|| match_permissions(q))
        .or_else(|| match_lvm(q))
        .or_else(|| match_raid(q))
        .or_else(|| match_btrfs(q))
        .or_else(|| match_general_fs(q))
}

/// Mount patterns
fn match_mounts(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FsPattern] = &[
        // List mounts
        (&["mounted", "filesystems"], "list mounted filesystems", "filesystem",
         &["mount | column -t", "findmnt"]),
        (&["list", "mounts"], "list mounts", "filesystem",
         &["findmnt", "mount"]),
        (&["show", "mounts"], "show mounts", "filesystem",
         &["findmnt --real"]),
        // Fstab
        (&["fstab"], "show fstab", "filesystem",
         &["cat /etc/fstab"]),
        (&["auto", "mount"], "show automount config", "filesystem",
         &["cat /etc/fstab", "systemctl list-units --type=automount"]),
        // Mount options
        (&["mount", "options"], "show mount options", "filesystem",
         &["findmnt -o TARGET,SOURCE,FSTYPE,OPTIONS"]),
        // Failed mounts
        (&["failed", "mount"], "check failed mounts", "filesystem",
         &["systemctl list-units --type=mount --state=failed", "dmesg | grep -i 'mount\\|filesystem' | tail -10"]),
        // UUID
        (&["disk", "uuid"], "show disk UUIDs", "filesystem",
         &["blkid"]),
        (&["partition", "uuid"], "show partition UUIDs", "filesystem",
         &["blkid", "ls -l /dev/disk/by-uuid/"]),
        // Labels
        (&["disk", "labels"], "show disk labels", "filesystem",
         &["blkid -o list", "ls -l /dev/disk/by-label/"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Permission patterns
fn match_permissions(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FsPattern] = &[
        // File permissions
        (&["file", "permissions"], "check file permissions", "filesystem",
         &["echo 'Use: ls -la <file>'", "stat <file>"]),
        (&["directory", "permissions"], "check directory permissions", "filesystem",
         &["echo 'Use: ls -la <directory>'"]),
        // Ownership
        (&["file", "ownership"], "check file ownership", "filesystem",
         &["echo 'Use: ls -la <file> or stat <file>'"]),
        (&["who", "owns"], "check file owner", "filesystem",
         &["echo 'Use: ls -la <file>'"]),
        // Special permissions
        (&["suid", "files"], "find SUID files", "filesystem",
         &["find /usr -perm -4000 2>/dev/null | head -20"]),
        (&["sgid", "files"], "find SGID files", "filesystem",
         &["find /usr -perm -2000 2>/dev/null | head -20"]),
        (&["world", "writable"], "find world-writable files", "filesystem",
         &["find /tmp -perm -002 -type f 2>/dev/null | head -20"]),
        // ACLs
        (&["acl", "permissions"], "show ACL permissions", "filesystem",
         &["echo 'Use: getfacl <file>'"]),
        (&["extended", "attributes"], "show extended attributes", "filesystem",
         &["echo 'Use: lsattr <file> or getfattr <file>'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// LVM patterns
fn match_lvm(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FsPattern] = &[
        // Physical volumes
        (&["lvm", "pv"], "list LVM physical volumes", "filesystem",
         &["pvs", "pvdisplay"]),
        (&["physical", "volumes"], "show physical volumes", "filesystem",
         &["pvs", "pvdisplay"]),
        // Volume groups
        (&["lvm", "vg"], "list LVM volume groups", "filesystem",
         &["vgs", "vgdisplay"]),
        (&["volume", "groups"], "show volume groups", "filesystem",
         &["vgs", "vgdisplay"]),
        // Logical volumes
        (&["lvm", "lv"], "list LVM logical volumes", "filesystem",
         &["lvs", "lvdisplay"]),
        (&["logical", "volumes"], "show logical volumes", "filesystem",
         &["lvs", "lvdisplay"]),
        // LVM overview
        (&["lvm", "status"], "show LVM status", "filesystem",
         &["pvs", "vgs", "lvs"]),
        (&["lvm", "info"], "show LVM info", "filesystem",
         &["pvs", "vgs", "lvs"]),
        // LVM space
        (&["lvm", "free"], "show LVM free space", "filesystem",
         &["vgs -o +vg_free", "pvs -o +pv_free"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// RAID patterns
fn match_raid(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FsPattern] = &[
        // RAID status
        (&["raid", "status"], "show RAID status", "filesystem",
         &["cat /proc/mdstat", "mdadm --detail --scan"]),
        (&["mdadm", "status"], "show mdadm RAID status", "filesystem",
         &["cat /proc/mdstat"]),
        // RAID arrays
        (&["raid", "arrays"], "list RAID arrays", "filesystem",
         &["cat /proc/mdstat", "ls /dev/md*"]),
        // RAID health
        (&["raid", "health"], "check RAID health", "filesystem",
         &["cat /proc/mdstat", "mdadm --detail /dev/md* 2>/dev/null | grep -E 'State|Active|Working|Failed'"]),
        // RAID rebuild
        (&["raid", "rebuild"], "check RAID rebuild status", "filesystem",
         &["cat /proc/mdstat"]),
        // RAID details
        (&["raid", "details"], "show RAID details", "filesystem",
         &["mdadm --detail --scan", "cat /proc/mdstat"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Btrfs patterns
fn match_btrfs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FsPattern] = &[
        // Btrfs status
        (&["btrfs", "status"], "show btrfs filesystem status", "filesystem",
         &["btrfs filesystem show", "btrfs device stats /"]),
        // Subvolumes
        (&["btrfs", "subvolumes"], "list btrfs subvolumes", "filesystem",
         &["btrfs subvolume list /"]),
        (&["btrfs", "subvol"], "show btrfs subvolumes", "filesystem",
         &["btrfs subvolume list /"]),
        // Snapshots
        (&["btrfs", "snapshots"], "list btrfs snapshots", "filesystem",
         &["btrfs subvolume list -s /"]),
        // Usage
        (&["btrfs", "usage"], "show btrfs usage", "filesystem",
         &["btrfs filesystem usage /", "btrfs filesystem df /"]),
        (&["btrfs", "space"], "show btrfs space", "filesystem",
         &["btrfs filesystem df /", "btrfs filesystem usage /"]),
        // Scrub
        (&["btrfs", "scrub"], "show btrfs scrub status", "filesystem",
         &["btrfs scrub status /"]),
        // Device stats
        (&["btrfs", "errors"], "show btrfs errors", "filesystem",
         &["btrfs device stats /", "dmesg | grep -i btrfs | tail -10"]),
        // Balance
        (&["btrfs", "balance"], "show btrfs balance status", "filesystem",
         &["btrfs balance status /"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// General filesystem patterns
fn match_general_fs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FsPattern] = &[
        // Filesystem types
        (&["filesystem", "types"], "show filesystem types", "filesystem",
         &["df -T", "findmnt -o TARGET,FSTYPE"]),
        (&["fs", "type"], "check filesystem type", "filesystem",
         &["df -T", "blkid"]),
        // Inodes
        (&["inode", "usage"], "show inode usage", "filesystem",
         &["df -i"]),
        (&["inodes"], "check inodes", "filesystem",
         &["df -i"]),
        // Filesystem check
        (&["fsck", "status"], "check filesystem status", "filesystem",
         &["echo 'Filesystems are checked at boot. Use: sudo fsck -n <device> for read-only check'"]),
        // Large files
        (&["large", "files"], "find large files", "filesystem",
         &["find / -xdev -type f -size +100M 2>/dev/null | head -20"]),
        (&["biggest", "files"], "find biggest files", "filesystem",
         &["du -ah / 2>/dev/null | sort -rh | head -20"]),
        // Directory sizes - v0.1.0: Show actual content, not just top-level
        (&["directory", "sizes"], "show directory sizes with actual content", "filesystem",
         &["du -xhd3 ~/ 2>/dev/null | sort -rh | head -25"]),
        (&["folder", "sizes"], "show folder sizes with actual content", "filesystem",
         &["du -xhd3 ~/ 2>/dev/null | sort -rh | head -25"]),
        // v0.1.0: Improved biggest folders - depth 6 to find actual content (games, projects)
        (&["biggest", "folders"], "show biggest folders (actual content)", "filesystem",
         &["du -xhd6 ~/ 2>/dev/null | sort -rh | head -30"]),
        (&["largest", "folders"], "show largest folders (actual content)", "filesystem",
         &["du -xhd6 ~/ 2>/dev/null | sort -rh | head -30"]),
        (&["top", "folders"], "show top folders by size (actual content)", "filesystem",
         &["du -xhd6 ~/ 2>/dev/null | sort -rh | head -30"]),
        // v0.1.0: Catch "folders by size" pattern
        (&["folders", "by", "size"], "show folders by size (actual content)", "filesystem",
         &["du -xhd6 ~/ 2>/dev/null | sort -rh | head -30"]),
        (&["biggest", "directories"], "show biggest directories (actual content)", "filesystem",
         &["du -xhd6 ~/ 2>/dev/null | sort -rh | head -30"]),
        // v0.1.0: Storage hogs - what's eating disk space
        (&["eating", "space"], "find what's eating disk space", "filesystem",
         &["du -xhd4 ~/ 2>/dev/null | sort -rh | head -30 | grep -E '^[0-9.]+[KMGT]'"]),
        (&["eating", "storage"], "find what's eating storage", "filesystem",
         &["du -xhd4 ~/ 2>/dev/null | sort -rh | head -30 | grep -E '^[0-9.]+[KMGT]'"]),
        (&["using", "disk"], "find what's using disk space", "filesystem",
         &["du -xhd4 ~/ 2>/dev/null | sort -rh | head -30 | grep -E '^[0-9.]+[KMGT]'"]),
        // Disk I/O
        (&["disk", "io"], "show disk I/O", "filesystem",
         &["iostat -x 1 3 2>/dev/null || cat /proc/diskstats"]),
        (&["io", "stats"], "show I/O statistics", "filesystem",
         &["iostat 2>/dev/null || iotop -b -n 1 2>/dev/null || cat /proc/diskstats"]),
        // Open files
        (&["open", "files"], "show open files", "filesystem",
         &["lsof 2>/dev/null | head -30 || ls /proc/*/fd 2>/dev/null | wc -l"]),
        // Deleted but open
        (&["deleted", "files", "space"], "find deleted files using space", "filesystem",
         &["lsof 2>/dev/null | grep deleted | head -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mounts() {
        assert!(match_patterns("mounted filesystems").is_some());
        assert!(match_patterns("list mounts").is_some());
        assert!(match_patterns("fstab").is_some());
        assert!(match_patterns("disk uuid").is_some());
    }

    #[test]
    fn test_permissions() {
        assert!(match_patterns("file permissions").is_some());
        assert!(match_patterns("suid files").is_some());
        assert!(match_patterns("world writable").is_some());
    }

    #[test]
    fn test_lvm() {
        assert!(match_patterns("lvm status").is_some());
        assert!(match_patterns("logical volumes").is_some());
        assert!(match_patterns("volume groups").is_some());
    }

    #[test]
    fn test_raid() {
        assert!(match_patterns("raid status").is_some());
        assert!(match_patterns("raid health").is_some());
    }

    #[test]
    fn test_btrfs() {
        assert!(match_patterns("btrfs status").is_some());
        assert!(match_patterns("btrfs subvolumes").is_some());
        assert!(match_patterns("btrfs snapshots").is_some());
    }

    #[test]
    fn test_general_fs() {
        assert!(match_patterns("filesystem types").is_some());
        assert!(match_patterns("inode usage").is_some());
        assert!(match_patterns("large files").is_some());
        assert!(match_patterns("directory sizes").is_some());
    }
}

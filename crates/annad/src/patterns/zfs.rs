//! ZFS patterns for OpenZFS on Linux.
//! v0.0.985: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a ZFS-related DeepUnderstanding
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

type ZfsPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match ZFS patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_zpool(q)
        .or_else(|| match_zfs_dataset(q))
        .or_else(|| match_zfs_snapshot(q))
        .or_else(|| match_zfs_health(q))
        .or_else(|| match_zfs_general(q))
}

/// Zpool patterns
fn match_zpool(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ZfsPattern] = &[
        // Pool status
        (&["zpool", "status"], "show zpool status", "zfs",
         &["zpool status"]),
        (&["zpool", "health"], "show zpool health", "zfs",
         &["zpool status -x"]),
        // Pool list
        (&["zpool", "list"], "list zpools", "zfs",
         &["zpool list"]),
        (&["list", "pool"], "list storage pools", "zfs",
         &["zpool list"]),
        // Pool info
        (&["zpool", "info"], "show zpool information", "zfs",
         &["zpool list -v", "zpool status"]),
        (&["pool", "info"], "show pool information", "zfs",
         &["zpool list -v"]),
        // Pool space
        (&["zpool", "space"], "show zpool space usage", "zfs",
         &["zpool list -o name,size,alloc,free,fragmentation,capacity"]),
        (&["pool", "usage"], "show pool usage", "zfs",
         &["zpool list -o name,size,alloc,free,capacity"]),
        // Pool history
        (&["zpool", "history"], "show zpool history", "zfs",
         &["zpool history | tail -30"]),
        // Pool IO stats
        (&["zpool", "iostat"], "show zpool IO stats", "zfs",
         &["zpool iostat -v"]),
        (&["pool", "io"], "show pool IO", "zfs",
         &["zpool iostat"]),
        // Pool version
        (&["zpool", "version"], "show zpool version", "zfs",
         &["zpool version", "zpool upgrade -v"]),
        // Pool features
        (&["zpool", "feature"], "show zpool features", "zfs",
         &["zpool get all | grep feature"]),
        // Pool properties
        (&["zpool", "propert"], "show zpool properties", "zfs",
         &["zpool get all"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// ZFS dataset patterns
fn match_zfs_dataset(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ZfsPattern] = &[
        // List datasets
        (&["zfs", "list"], "list ZFS datasets", "zfs",
         &["zfs list"]),
        (&["zfs", "dataset"], "list ZFS datasets", "zfs",
         &["zfs list -t all"]),
        (&["list", "dataset"], "list datasets", "zfs",
         &["zfs list"]),
        // Dataset properties
        (&["zfs", "propert"], "show ZFS properties", "zfs",
         &["zfs get all <dataset> 2>/dev/null | head -30 || zfs get all | head -30"]),
        (&["dataset", "propert"], "show dataset properties", "zfs",
         &["zfs get all | head -30"]),
        // Compression
        (&["zfs", "compress"], "show ZFS compression", "zfs",
         &["zfs get compression,compressratio"]),
        (&["compress", "ratio"], "show compression ratio", "zfs",
         &["zfs get compressratio"]),
        // Quota
        (&["zfs", "quota"], "show ZFS quotas", "zfs",
         &["zfs get quota,reservation,refquota,refreservation"]),
        // Space usage
        (&["zfs", "space"], "show ZFS space usage", "zfs",
         &["zfs list -o name,used,avail,refer,mountpoint"]),
        (&["zfs", "usage"], "show ZFS usage", "zfs",
         &["zfs list -o name,used,avail,refer"]),
        // Mountpoints
        (&["zfs", "mount"], "show ZFS mounts", "zfs",
         &["zfs mount", "zfs list -o name,mountpoint"]),
        // Encryption
        (&["zfs", "encrypt"], "show ZFS encryption", "zfs",
         &["zfs get encryption,keystatus,keyformat"]),
        // Dedup
        (&["zfs", "dedup"], "show ZFS deduplication", "zfs",
         &["zfs get dedup", "zpool get dedupratio"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// ZFS snapshot patterns
fn match_zfs_snapshot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ZfsPattern] = &[
        // List snapshots
        (&["zfs", "snapshot"], "list ZFS snapshots", "zfs",
         &["zfs list -t snapshot"]),
        (&["list", "snapshot"], "list snapshots", "zfs",
         &["zfs list -t snapshot"]),
        (&["zfs", "snap"], "list ZFS snapshots", "zfs",
         &["zfs list -t snapshot | head -30"]),
        // Snapshot size
        (&["snapshot", "size"], "show snapshot sizes", "zfs",
         &["zfs list -t snapshot -o name,used,refer"]),
        // Snapshot space
        (&["snapshot", "space"], "show snapshot space", "zfs",
         &["zfs list -t snapshot -o name,used,refer,written"]),
        // Recent snapshots
        (&["recent", "snapshot"], "show recent snapshots", "zfs",
         &["zfs list -t snapshot -s creation | tail -20"]),
        (&["latest", "snapshot"], "show latest snapshots", "zfs",
         &["zfs list -t snapshot -s creation | tail -10"]),
        // Snapshot diff
        (&["snapshot", "diff"], "show snapshot differences", "zfs",
         &["zfs diff <snapshot1> <snapshot2>"]),
        (&["snapshot", "change"], "show snapshot changes", "zfs",
         &["zfs diff <snapshot>"]),
        // Bookmarks
        (&["zfs", "bookmark"], "list ZFS bookmarks", "zfs",
         &["zfs list -t bookmark"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// ZFS health patterns
fn match_zfs_health(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ZfsPattern] = &[
        // Scrub status
        (&["zfs", "scrub"], "show ZFS scrub status", "zfs",
         &["zpool status | grep -A5 scan"]),
        (&["scrub", "status"], "show scrub status", "zfs",
         &["zpool status | grep -A5 scan"]),
        // Errors
        (&["zfs", "error"], "show ZFS errors", "zfs",
         &["zpool status -v | grep -A10 errors"]),
        (&["pool", "error"], "show pool errors", "zfs",
         &["zpool status -x"]),
        // Resilver
        (&["zfs", "resilver"], "show resilver status", "zfs",
         &["zpool status | grep -E 'resilver|scan'"]),
        (&["resilver", "status"], "show resilver progress", "zfs",
         &["zpool status | grep -A5 scan"]),
        // Degraded
        (&["zfs", "degrad"], "check for degraded pools", "zfs",
         &["zpool status -x"]),
        (&["pool", "degrad"], "check degraded pools", "zfs",
         &["zpool status | grep -i degraded"]),
        // Faulted
        (&["zfs", "fault"], "check for faulted devices", "zfs",
         &["zpool status | grep -i faulted"]),
        // ARC stats
        (&["zfs", "arc"], "show ZFS ARC stats", "zfs",
         &["cat /proc/spl/kstat/zfs/arcstats | head -30", "arc_summary 2>/dev/null"]),
        (&["arc", "stat"], "show ARC statistics", "zfs",
         &["cat /proc/spl/kstat/zfs/arcstats"]),
        // L2ARC
        (&["l2arc"], "show L2ARC stats", "zfs",
         &["cat /proc/spl/kstat/zfs/arcstats | grep l2"]),
        // SLOG
        (&["slog"], "show SLOG info", "zfs",
         &["zpool status | grep log"]),
        // Events
        (&["zfs", "event"], "show ZFS events", "zfs",
         &["zpool events | tail -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// General ZFS patterns
fn match_zfs_general(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ZfsPattern] = &[
        // ZFS version
        (&["zfs", "version"], "show ZFS version", "zfs",
         &["zfs version", "modinfo zfs | grep version"]),
        (&["openzfs", "version"], "show OpenZFS version", "zfs",
         &["zfs version", "dmesg | grep -i zfs"]),
        // ZFS installed
        (&["zfs", "install"], "check ZFS installation", "zfs",
         &["pacman -Qi zfs-dkms 2>/dev/null || pacman -Qi zfs-linux 2>/dev/null", "modinfo zfs"]),
        // ZFS module
        (&["zfs", "module"], "check ZFS module", "zfs",
         &["lsmod | grep zfs", "modinfo zfs | head -10"]),
        // ZFS services
        (&["zfs", "service"], "check ZFS services", "zfs",
         &["systemctl status zfs-import-cache zfs-mount zfs.target"]),
        // Send/receive
        (&["zfs", "send"], "ZFS send information", "zfs",
         &["echo 'zfs send <snapshot> | zfs receive <destination>'"]),
        (&["zfs", "receive"], "ZFS receive information", "zfs",
         &["echo 'zfs receive <dataset> < <file>'"]),
        // Cache
        (&["zfs", "cache"], "show ZFS cache info", "zfs",
         &["cat /proc/spl/kstat/zfs/arcstats | grep -E 'hits|misses|size'"]),
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
    fn test_zpool() {
        assert!(match_patterns("zpool status").is_some());
        assert!(match_patterns("zpool list").is_some());
        assert!(match_patterns("zpool iostat").is_some());
    }

    #[test]
    fn test_zfs_dataset() {
        assert!(match_patterns("zfs list").is_some());
        assert!(match_patterns("zfs compression").is_some());
        assert!(match_patterns("zfs mount").is_some());
    }

    #[test]
    fn test_zfs_snapshot() {
        assert!(match_patterns("zfs snapshots").is_some());
        assert!(match_patterns("list snapshots").is_some());
        assert!(match_patterns("recent snapshots").is_some());
    }

    #[test]
    fn test_zfs_health() {
        assert!(match_patterns("zfs scrub").is_some());
        assert!(match_patterns("zfs errors").is_some());
        assert!(match_patterns("zfs arc").is_some());
    }

    #[test]
    fn test_zfs_general() {
        assert!(match_patterns("zfs version").is_some());
        assert!(match_patterns("zfs module").is_some());
        assert!(match_patterns("zfs services").is_some());
    }
}

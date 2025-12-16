//! Storage query classification patterns (v0.0.804).
//!
//! Block devices, LVM, RAID, ZFS, mounts, fstab, swap.

use crate::router::QueryClass;

/// Classify storage queries.
/// Returns Some if matched, None otherwise.
pub fn classify_storage(q: &str) -> Option<QueryClass> {
    // v0.0.390: Largest folders/directories queries
    // "top 10 folders taking storage", "what's using my disk space"
    if (q.contains("folder") || q.contains("director"))
        && (q.contains("largest")
            || q.contains("biggest")
            || q.contains("top")
            || q.contains("taking")
            || q.contains("using"))
    {
        return Some(QueryClass::LargestFolders);
    }
    // Also catch "what is taking space", "what's using storage"
    if (q.contains("what") || q.contains("which"))
        && (q.contains("taking") || q.contains("using"))
        && (q.contains("space") || q.contains("storage") || q.contains("disk"))
    {
        return Some(QueryClass::LargestFolders);
    }

    // v0.0.124: Mounted filesystems
    if q.contains("mounted")
        || q.contains("mount points")
        || q.contains("show mounts")
        || q.contains("list mounts")
        || q.contains("filesystems")
        || q.trim() == "mounts"
        || q.trim() == "findmnt"
    {
        return Some(QueryClass::MountedFilesystems);
    }

    // v0.0.127: Block devices
    if q.trim() == "lsblk"
        || q.contains("block device")
        || q.contains("partition")
        || q.contains("show disk")
        || q.contains("list disk")
        || (q.contains("disk") && q.contains("layout"))
    {
        return Some(QueryClass::BlockDevices);
    }

    // v0.0.127: ZFS status
    if q.contains("zfs")
        || q.contains("zpool")
        || (q.contains("storage pool") && (q.contains("status") || q.contains("health")))
    {
        return Some(QueryClass::ZfsStatus);
    }

    // v0.0.134: LVM status
    if q.contains("lvm")
        || q.contains("logical volume")
        || q.contains("volume group")
        || q.trim() == "lvs"
        || q.trim() == "vgs"
        || q.trim() == "pvs"
    {
        return Some(QueryClass::LvmStatus);
    }

    // v0.0.134: RAID status
    if q.contains("raid")
        || q.contains("mdadm")
        || q.contains("software raid")
        || q.contains("md status")
    {
        return Some(QueryClass::RaidStatus);
    }

    // v0.0.136: Fstab entries
    if q.contains("fstab")
        || q.contains("/etc/fstab")
        || q.contains("mount table")
        || q.contains("mount entry")
        || (q.contains("show") && q.contains("fstab"))
    {
        return Some(QueryClass::FstabEntries);
    }

    // v0.0.122: Swap info
    // v0.0.796: Added "have swap", "is swap", "any swap" patterns
    if q.contains("swap usage")
        || q.contains("swap space")
        || q.contains("show swap")
        || q.contains("how much swap")
        || q.contains("swap status")
        || q.contains("have swap")
        || q.contains("is swap")
        || q.contains("any swap")
        || q.contains("swap enabled")
        || q.contains("swap active")
        || q.trim() == "swap"
    {
        return Some(QueryClass::SwapInfo);
    }

    // v0.0.141: Swap files
    if q.contains("swap file")
        || q.contains("swapfile")
        || q.contains("/proc/swaps")
        || (q.contains("swap") && q.contains("partition"))
        || (q.contains("list") && q.contains("swap"))
    {
        return Some(QueryClass::SwapFiles);
    }

    None
}

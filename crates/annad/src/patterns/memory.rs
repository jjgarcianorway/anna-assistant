//! Memory and swap patterns.
//! v0.0.971: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a memory-related DeepUnderstanding
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

type MemPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match memory-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_ram_usage(q)
        .or_else(|| match_swap(q))
        .or_else(|| match_cache_buffers(q))
        .or_else(|| match_oom(q))
        .or_else(|| match_memory_info(q))
}

/// RAM usage patterns
fn match_ram_usage(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[MemPattern] = &[
        // Memory usage
        (&["memory", "usage"], "show memory usage", "memory",
         &["free -h", "cat /proc/meminfo | head -10"]),
        (&["ram", "usage"], "show RAM usage", "memory",
         &["free -h"]),
        (&["memory", "available"], "show available memory", "memory",
         &["free -h | grep Mem", "cat /proc/meminfo | grep -E 'MemTotal|MemAvailable|MemFree'"]),
        // Free memory
        (&["free", "memory"], "show free memory", "memory",
         &["free -h", "cat /proc/meminfo | grep -E 'MemFree|MemAvailable'"]),
        (&["free", "ram"], "show free RAM", "memory",
         &["free -h | grep Mem"]),
        // Used memory
        (&["used", "memory"], "show used memory", "memory",
         &["free -h | grep Mem"]),
        (&["used", "ram"], "show used RAM", "memory",
         &["free -h | grep Mem"]),
        // Memory percentage
        (&["memory", "percent"], "show memory usage percentage", "memory",
         &["free | awk '/Mem:/ {printf \"%.1f%% used\\n\", $3/$2*100}'"]),
        // Low memory
        (&["low", "memory"], "check for low memory", "memory",
         &["free -h", "cat /proc/meminfo | grep -E 'MemAvailable|LowFree'"]),
        (&["running", "out", "memory"], "check memory status", "memory",
         &["free -h", "dmesg | grep -i 'out of memory' | tail -5"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Swap patterns
fn match_swap(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[MemPattern] = &[
        // Swap usage
        (&["swap", "usage"], "show swap usage", "memory",
         &["free -h | grep Swap", "swapon --show"]),
        (&["swap", "status"], "show swap status", "memory",
         &["swapon --show", "cat /proc/swaps"]),
        // Swap enabled
        (&["swap", "enabled"], "check if swap is enabled", "memory",
         &["swapon --show", "cat /proc/swaps"]),
        (&["swap", "active"], "show active swap", "memory",
         &["swapon --show"]),
        // Swap file/partition
        (&["swap", "file"], "show swap file info", "memory",
         &["swapon --show", "ls -la /swapfile 2>/dev/null"]),
        (&["swap", "partition"], "show swap partition", "memory",
         &["cat /proc/swaps", "lsblk | grep -i swap"]),
        // Swappiness
        (&["swappiness"], "show swappiness value", "memory",
         &["cat /proc/sys/vm/swappiness", "sysctl vm.swappiness"]),
        (&["swap", "priority"], "show swap priority", "memory",
         &["swapon --show", "cat /proc/swaps"]),
        // Swap full
        (&["swap", "full"], "check if swap is full", "memory",
         &["free -h | grep Swap"]),
        // Zram/zswap
        (&["zram", "status"], "show zram status", "memory",
         &["zramctl 2>/dev/null || echo 'zram not in use'", "cat /sys/block/zram*/comp_algorithm 2>/dev/null"]),
        (&["zswap", "status"], "show zswap status", "memory",
         &["cat /sys/module/zswap/parameters/enabled 2>/dev/null", "grep -r . /sys/kernel/debug/zswap/ 2>/dev/null | head -10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Cache and buffers patterns
fn match_cache_buffers(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[MemPattern] = &[
        // Cache
        (&["memory", "cache"], "show memory cache", "memory",
         &["free -h | head -2", "cat /proc/meminfo | grep -E 'Cached|Buffers'"]),
        (&["page", "cache"], "show page cache", "memory",
         &["cat /proc/meminfo | grep -E 'Cached|Active|Inactive'"]),
        // Buffers
        (&["memory", "buffers"], "show memory buffers", "memory",
         &["cat /proc/meminfo | grep Buffers"]),
        // Cached memory
        (&["cached", "memory"], "show cached memory", "memory",
         &["free -h", "cat /proc/meminfo | grep Cached"]),
        // Slab
        (&["slab", "memory"], "show slab memory", "memory",
         &["cat /proc/meminfo | grep Slab", "slabtop -o 2>/dev/null | head -20"]),
        (&["slab", "info"], "show slab info", "memory",
         &["cat /proc/slabinfo | head -20"]),
        // Dirty pages
        (&["dirty", "pages"], "show dirty pages", "memory",
         &["cat /proc/meminfo | grep Dirty"]),
        // Writeback
        (&["writeback"], "show writeback status", "memory",
         &["cat /proc/meminfo | grep -E 'Dirty|Writeback'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// OOM (Out of Memory) patterns
fn match_oom(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[MemPattern] = &[
        // OOM killer
        (&["oom", "killer"], "show OOM killer status", "memory",
         &["dmesg | grep -i 'out of memory' | tail -10", "dmesg | grep -i 'oom' | tail -10"]),
        (&["oom", "kills"], "show OOM kills", "memory",
         &["dmesg | grep -i 'killed process' | tail -10"]),
        // OOM logs
        (&["oom", "logs"], "show OOM logs", "memory",
         &["journalctl -k | grep -i oom | tail -20", "dmesg | grep -i oom | tail -20"]),
        // OOM score
        (&["oom", "score"], "show OOM scores", "memory",
         &["for p in /proc/[0-9]*/oom_score; do echo \"$(cat $p 2>/dev/null) $(cat ${p%/*}/comm 2>/dev/null)\"; done | sort -rn | head -20"]),
        // Memory pressure
        (&["memory", "pressure"], "show memory pressure", "memory",
         &["cat /proc/pressure/memory 2>/dev/null || echo 'PSI not available'"]),
        // Memory overcommit
        (&["memory", "overcommit"], "show memory overcommit settings", "memory",
         &["cat /proc/sys/vm/overcommit_memory", "cat /proc/sys/vm/overcommit_ratio"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Memory info patterns
fn match_memory_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[MemPattern] = &[
        // Total memory
        (&["total", "memory"], "show total memory", "memory",
         &["free -h | grep Mem | awk '{print $2}'", "cat /proc/meminfo | grep MemTotal"]),
        (&["total", "ram"], "show total RAM", "memory",
         &["free -h | grep Mem | awk '{print $2}'"]),
        // Memory info
        (&["meminfo"], "show /proc/meminfo", "memory",
         &["cat /proc/meminfo | head -30"]),
        // Memory type
        (&["memory", "type"], "show memory type", "memory",
         &["dmidecode -t memory 2>/dev/null | grep -E 'Type:|Speed:|Size:' | head -20 || echo 'Need sudo for dmidecode'"]),
        (&["ram", "type"], "show RAM type", "memory",
         &["dmidecode -t memory 2>/dev/null | grep -E 'Type:|Speed:' | head -10"]),
        // Memory slots
        (&["memory", "slots"], "show memory slots", "memory",
         &["dmidecode -t memory 2>/dev/null | grep -E 'Locator:|Size:' | head -20"]),
        (&["ram", "slots"], "show RAM slots", "memory",
         &["dmidecode -t memory 2>/dev/null | grep -E 'Locator:|Size:' | head -20"]),
        // Memory speed
        (&["memory", "speed"], "show memory speed", "memory",
         &["dmidecode -t memory 2>/dev/null | grep Speed | head -10"]),
        // Huge pages
        (&["huge", "pages"], "show huge pages info", "memory",
         &["cat /proc/meminfo | grep -i huge"]),
        (&["hugepages"], "show hugepages status", "memory",
         &["cat /proc/meminfo | grep -i huge"]),
        // Transparent huge pages
        (&["transparent", "huge"], "show transparent hugepages", "memory",
         &["cat /sys/kernel/mm/transparent_hugepage/enabled"]),
        (&["thp", "status"], "show THP status", "memory",
         &["cat /sys/kernel/mm/transparent_hugepage/enabled", "cat /proc/meminfo | grep AnonHugePages"]),
        // Numa
        (&["numa", "memory"], "show NUMA memory", "memory",
         &["numactl --hardware 2>/dev/null || echo 'numactl not installed'"]),
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
    fn test_ram_usage() {
        assert!(match_patterns("memory usage").is_some());
        assert!(match_patterns("ram usage").is_some());
        assert!(match_patterns("free memory").is_some());
        assert!(match_patterns("used memory").is_some());
    }

    #[test]
    fn test_swap() {
        assert!(match_patterns("swap usage").is_some());
        assert!(match_patterns("swap status").is_some());
        assert!(match_patterns("swappiness").is_some());
        assert!(match_patterns("zram status").is_some());
    }

    #[test]
    fn test_cache_buffers() {
        assert!(match_patterns("memory cache").is_some());
        assert!(match_patterns("cached memory").is_some());
        assert!(match_patterns("slab memory").is_some());
    }

    #[test]
    fn test_oom() {
        assert!(match_patterns("oom killer").is_some());
        assert!(match_patterns("oom logs").is_some());
        assert!(match_patterns("memory pressure").is_some());
    }

    #[test]
    fn test_memory_info() {
        assert!(match_patterns("total memory").is_some());
        assert!(match_patterns("memory type").is_some());
        assert!(match_patterns("memory slots").is_some());
        assert!(match_patterns("huge pages").is_some());
    }
}

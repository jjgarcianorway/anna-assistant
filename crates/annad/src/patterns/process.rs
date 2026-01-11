//! Process management patterns for ps, top, kill, nice.
//! v0.0.963: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a process-related DeepUnderstanding
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

type ProcessPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match process-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_list_processes(q)
        .or_else(|| match_resource_usage(q))
        .or_else(|| match_process_info(q))
        .or_else(|| match_zombie_orphan(q))
}

/// List processes patterns
fn match_list_processes(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ProcessPattern] = &[
        // All processes
        (&["all", "processes"], "list all processes", "process",
         &["ps aux"]),
        (&["list", "processes"], "list processes", "process",
         &["ps aux | head -30"]),
        (&["running", "processes"], "list running processes", "process",
         &["ps aux --sort=-%cpu | head -20"]),
        // Process tree
        (&["process", "tree"], "show process tree", "process",
         &["pstree -p | head -50"]),
        (&["process", "hierarchy"], "show process hierarchy", "process",
         &["pstree"]),
        // By user
        (&["my", "processes"], "show my processes", "process",
         &["ps aux | grep $USER | head -30"]),
        (&["user", "processes"], "show user processes", "process",
         &["ps aux | grep -v root | head -30"]),
        (&["root", "processes"], "show root processes", "process",
         &["ps aux | grep root | head -30"]),
        // Count
        (&["process", "count"], "count processes", "process",
         &["ps aux | wc -l"]),
        (&["how", "many", "processes"], "count total processes", "process",
         &["ps aux | wc -l"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Resource usage patterns
fn match_resource_usage(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ProcessPattern] = &[
        // CPU usage
        (&["cpu", "hogs"], "find CPU-hogging processes", "process",
         &["ps aux --sort=-%cpu | head -10"]),
        (&["high", "cpu"], "find high CPU processes", "process",
         &["ps aux --sort=-%cpu | head -10"]),
        (&["top", "cpu"], "show top CPU processes", "process",
         &["ps aux --sort=-%cpu | head -10"]),
        (&["cpu", "usage", "processes"], "show CPU usage by process", "process",
         &["ps aux --sort=-%cpu | head -15"]),
        // Memory usage
        (&["memory", "hogs"], "find memory-hogging processes", "process",
         &["ps aux --sort=-%mem | head -10"]),
        (&["high", "memory"], "find high memory processes", "process",
         &["ps aux --sort=-%mem | head -10"]),
        (&["top", "memory"], "show top memory processes", "process",
         &["ps aux --sort=-%mem | head -10"]),
        (&["ram", "usage", "processes"], "show RAM usage by process", "process",
         &["ps aux --sort=-%mem | head -15"]),
        // What's using resources
        (&["what", "using", "cpu"], "find what's using CPU", "process",
         &["ps aux --sort=-%cpu | head -10"]),
        (&["what", "using", "memory"], "find what's using memory", "process",
         &["ps aux --sort=-%mem | head -10"]),
        (&["what", "using", "ram"], "find what's using RAM", "process",
         &["ps aux --sort=-%mem | head -10"]),
        // Load
        (&["system", "load"], "show system load", "process",
         &["uptime", "cat /proc/loadavg"]),
        (&["load", "average"], "show load average", "process",
         &["uptime", "cat /proc/loadavg"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Process info patterns
fn match_process_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ProcessPattern] = &[
        // Find process
        (&["find", "process"], "find a process", "process",
         &["echo 'Use: pgrep -a <name> or ps aux | grep <name>'"]),
        (&["is", "running"], "check if process is running", "process",
         &["echo 'Use: pgrep <name> or ps aux | grep <name>'"]),
        // Process details
        (&["process", "details"], "show process details", "process",
         &["echo 'Use: ps -p <PID> -o pid,ppid,user,%cpu,%mem,stat,start,time,command'"]),
        (&["process", "info"], "show process info", "process",
         &["echo 'Use: ps -p <PID> -f or cat /proc/<PID>/status'"]),
        // Open files
        (&["process", "files"], "show files opened by process", "process",
         &["echo 'Use: lsof -p <PID>'"]),
        (&["what", "files", "open"], "find what files are open", "process",
         &["lsof 2>/dev/null | head -30"]),
        // Network connections
        (&["process", "connections"], "show process network connections", "process",
         &["echo 'Use: lsof -i -p <PID> or ss -p'"]),
        (&["process", "ports"], "show process ports", "process",
         &["ss -tulnp | head -20"]),
        // Parent/child
        (&["parent", "process"], "find parent process", "process",
         &["echo 'Use: ps -o ppid= -p <PID>'"]),
        (&["child", "processes"], "find child processes", "process",
         &["echo 'Use: pgrep -P <PID>'"]),
        // Environment
        (&["process", "environment"], "show process environment", "process",
         &["echo 'Use: cat /proc/<PID>/environ | tr \"\\0\" \"\\n\"'"]),
        // Limits
        (&["process", "limits"], "show process limits", "process",
         &["echo 'Use: cat /proc/<PID>/limits'"]),
        (&["ulimits"], "show user limits", "process",
         &["ulimit -a"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Zombie and orphan process patterns
fn match_zombie_orphan(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ProcessPattern] = &[
        // Zombie processes
        (&["zombie", "processes"], "find zombie processes", "process",
         &["ps aux | awk '$8 ~ /Z/ {print}'", "ps aux | grep -w Z"]),
        (&["zombies"], "find zombies", "process",
         &["ps aux | awk '$8 ~ /Z/ {print}'"]),
        (&["defunct", "processes"], "find defunct processes", "process",
         &["ps aux | grep defunct"]),
        // Orphan processes
        (&["orphan", "processes"], "find orphan processes", "process",
         &["ps -eo pid,ppid,stat,cmd | awk '$2 == 1 && $3 !~ /S/ {print}' | head -20"]),
        // Stuck processes
        (&["stuck", "processes"], "find stuck processes", "process",
         &["ps aux | awk '$8 ~ /D/ {print}'", "ps aux | grep -E '^.{15}D'"]),
        (&["uninterruptible"], "find uninterruptible processes", "process",
         &["ps aux | awk '$8 ~ /D/ {print}'"]),
        // Sleeping
        (&["sleeping", "processes"], "find sleeping processes", "process",
         &["ps aux | awk '$8 ~ /S/ {print}' | head -20"]),
        // Background jobs
        (&["background", "jobs"], "list background jobs", "process",
         &["jobs -l"]),
        (&["bg", "jobs"], "show background jobs", "process",
         &["jobs -l"]),
        // Nice values
        (&["nice", "values"], "show process nice values", "process",
         &["ps -eo pid,ni,cmd --sort=-ni | head -20"]),
        (&["process", "priority"], "show process priorities", "process",
         &["ps -eo pid,ni,pri,cmd | head -20"]),
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
    fn test_list_processes() {
        assert!(match_patterns("all processes").is_some());
        assert!(match_patterns("list processes").is_some());
        assert!(match_patterns("process tree").is_some());
        assert!(match_patterns("my processes").is_some());
    }

    #[test]
    fn test_resource_usage() {
        assert!(match_patterns("cpu hogs").is_some());
        assert!(match_patterns("high cpu").is_some());
        assert!(match_patterns("memory hogs").is_some());
        assert!(match_patterns("system load").is_some());
    }

    #[test]
    fn test_process_info() {
        assert!(match_patterns("find process").is_some());
        assert!(match_patterns("process files").is_some());
        assert!(match_patterns("ulimits").is_some());
    }

    #[test]
    fn test_zombie_orphan() {
        assert!(match_patterns("zombie processes").is_some());
        assert!(match_patterns("stuck processes").is_some());
        assert!(match_patterns("background jobs").is_some());
        assert!(match_patterns("nice values").is_some());
    }
}

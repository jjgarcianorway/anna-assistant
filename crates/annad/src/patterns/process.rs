//! Process management patterns for ps, top, kill, nice.
//! v0.0.963: Initial implementation.
//! v0.0.989: Added threads, I/O, memory map, signals patterns

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
        .or_else(|| match_process_inspection(q))
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

/// Advanced process inspection patterns
fn match_process_inspection(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ProcessPattern] = &[
        // Process threads
        (&["process", "threads"], "show process threads", "process",
         &["echo 'Use: ps -T -p <PID>'", "echo 'Or: cat /proc/<PID>/status | grep Threads'"]),
        (&["thread", "count"], "count threads for process", "process",
         &["echo 'Use: ps -T -p <PID> | wc -l'", "echo 'Or: ls /proc/<PID>/task | wc -l'"]),
        (&["list", "threads"], "list all threads", "process",
         &["ps -eLf | head -30"]),
        // Process I/O
        (&["process", "io"], "show process I/O", "process",
         &["echo 'Use: cat /proc/<PID>/io'", "echo 'Or: iotop -p <PID>'"]),
        (&["io", "stats"], "show I/O statistics", "process",
         &["iotop -o 2>/dev/null | head -20 || echo 'Install: sudo pacman -S iotop'"]),
        (&["disk", "io", "process"], "show disk I/O by process", "process",
         &["iotop -o 2>/dev/null || pidstat -d 1 3"]),
        // Process open files (expanded)
        (&["process", "open", "files"], "show open files for process", "process",
         &["echo 'Use: lsof -p <PID>'", "echo 'Or: ls -l /proc/<PID>/fd'"]),
        (&["open", "files"], "list open files", "process",
         &["lsof 2>/dev/null | head -30", "echo 'For specific process: lsof -p <PID>'"]),
        (&["file", "descriptors"], "show file descriptors", "process",
         &["echo 'Use: ls -l /proc/<PID>/fd'", "lsof | head -30"]),
        // Process CPU time
        (&["process", "cpu", "time"], "show process CPU time", "process",
         &["echo 'Use: ps -o pid,etime,cputime -p <PID>'",
           "echo 'etime=elapsed time, cputime=CPU time used'"]),
        (&["cpu", "time"], "show CPU time statistics", "process",
         &["ps -eo pid,etime,cputime,cmd --sort=-cputime | head -15"]),
        // Process memory map
        (&["process", "memory", "map"], "show process memory map", "process",
         &["echo 'Use: pmap -x <PID>'", "echo 'Or: cat /proc/<PID>/maps'"]),
        (&["memory", "map"], "show memory mappings", "process",
         &["echo 'Use: pmap <PID> or cat /proc/<PID>/smaps'"]),
        (&["pmap"], "pmap usage", "process",
         &["echo 'pmap -x <PID> for extended info'", "echo 'pmap -X <PID> for extra details'"]),
        // Process signals
        (&["process", "signals"], "show process signal handling", "process",
         &["echo 'Use: cat /proc/<PID>/status | grep -i sig'",
           "echo 'Send signals with: kill -<signal> <PID>'"]),
        (&["signal", "list"], "list available signals", "process",
         &["kill -l"]),
        (&["pending", "signals"], "show pending signals", "process",
         &["echo 'Use: cat /proc/<PID>/status | grep SigPnd'"]),
        // Process user
        (&["process", "user"], "show process owner", "process",
         &["echo 'Use: ps -o pid,user,cmd -p <PID>'",
           "ps aux | head -20"]),
        (&["process", "owner"], "find process owner", "process",
         &["echo 'Use: ps -o user= -p <PID>'", "ps aux | grep <process>"]),
        // Strace
        (&["strace", "process"], "trace process system calls", "process",
         &["echo 'Use: strace -p <PID>'", "echo 'Or for new: strace <command>'"]),
        (&["system", "calls"], "trace system calls", "process",
         &["echo 'Use: strace -c -p <PID> (summary)'",
           "echo 'Or: strace -f <command> (follow forks)'"]),
        // ltrace
        (&["ltrace"], "trace library calls", "process",
         &["echo 'Use: ltrace -p <PID>'", "echo 'Or: ltrace <command>'"]),
        // /proc info
        (&["proc", "info"], "show /proc process info", "process",
         &["echo 'Status: cat /proc/<PID>/status'",
           "echo 'Command: cat /proc/<PID>/cmdline'",
           "echo 'CWD: ls -l /proc/<PID>/cwd'"]),
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

    #[test]
    fn test_process_inspection() {
        assert!(match_patterns("process threads").is_some());
        assert!(match_patterns("process io").is_some());
        assert!(match_patterns("process open files").is_some());
        assert!(match_patterns("process cpu time").is_some());
        assert!(match_patterns("process memory map").is_some());
        assert!(match_patterns("process signals").is_some());
        assert!(match_patterns("process user").is_some());
    }
}

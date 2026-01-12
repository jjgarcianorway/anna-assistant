//! Performance and resource usage patterns
//! v0.0.914: Added suggested_commands for diagnostics
//! v0.0.989: Added optimization patterns (SSD, gaming, battery, profiling)

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, and diagnostic commands
type PerfPattern = (&'static [&'static str], &'static str, &'static [&'static str]);

/// Match performance-related queries
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Thermal/fan issues
    if let Some(u) = match_thermal(q) {
        return Some(u);
    }
    // Memory issues
    if let Some(u) = match_memory(q) {
        return Some(u);
    }
    // CPU/process issues
    if let Some(u) = match_cpu(q) {
        return Some(u);
    }
    // Service/shutdown issues
    if let Some(u) = match_services(q) {
        return Some(u);
    }
    // Optimization patterns
    if let Some(u) = match_optimization(q) {
        return Some(u);
    }
    // General slowness
    if let Some(u) = match_slowness(q) {
        return Some(u);
    }
    None
}

fn match_thermal(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PerfPattern] = &[
        (&["fan", "spin", "idle"], "fan running when idle",
            &["sensors", "cat /sys/class/thermal/thermal_zone*/temp",
              "ps aux --sort=-%cpu | head -5"]),
        (&["fan", "loud"], "loud fan noise",
            &["sensors", "ps aux --sort=-%cpu | head -5"]),
        (&["overheating"], "system overheating",
            &["sensors", "cat /sys/class/thermal/thermal_zone*/temp"]),
        (&["thermal", "throttl"], "thermal throttling",
            &["dmesg | grep -i thermal | tail -10", "sensors"]),
        (&["cpu", "temp", "high"], "high CPU temperature",
            &["sensors", "cat /sys/class/thermal/thermal_zone*/temp"]),
        (&["hot", "laptop"], "laptop overheating",
            &["sensors", "cat /proc/acpi/thermal_zone/*/temperature 2>/dev/null"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("hardware".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_memory(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PerfPattern] = &[
        (&["memory", "leak"], "memory leak detection",
            &["ps aux --sort=-%mem | head -10", "free -h"]),
        (&["ram", "usage", "high"], "high RAM usage",
            &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["ram", "full"], "RAM full",
            &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["using", "all", "ram"], "high RAM usage",
            &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["firefox", "memory"], "Firefox memory usage",
            &["ps aux | grep -i firefox | head -5", "about:memory in Firefox"]),
        (&["chrome", "memory"], "Chrome memory usage",
            &["ps aux | grep -i chrom | head -5"]),
        (&["browser", "memory"], "browser memory usage",
            &["ps aux | grep -E 'firefox|chrom' | head -5"]),
        (&["oom", "killer"], "OOM killer triggered",
            &["dmesg | grep -i 'out of memory' | tail -10", "free -h"]),
        (&["out of memory"], "out of memory error",
            &["dmesg | grep -i 'out of memory' | tail -5", "free -h"]),
        (&["swap", "full"], "swap space full",
            &["swapon --show", "free -h", "ps aux --sort=-%mem | head -5"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("performance".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_cpu(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PerfPattern] = &[
        (&["cpu", "usage", "high"], "high CPU usage",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["cpu", "100"], "CPU at 100%",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["what", "using", "cpu"], "CPU usage query",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["process", "cpu"], "process CPU usage",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["zombie", "process"], "zombie processes",
            &["ps aux | grep 'Z' | head -10"]),
        (&["process", "still", "running"], "orphan process",
            &["ps aux | head -10"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("performance".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Service pattern with category
type ServicePattern = (&'static [&'static str], &'static str, &'static str, IntentCategory, &'static [&'static str]);

fn match_services(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ServicePattern] = &[
        (&["failed", "service"], "failed systemd services", "services", IntentCategory::Troubleshoot,
            &["systemctl --failed", "journalctl -p err -b | tail -20"]),
        (&["service", "fail"], "service failure", "services", IntentCategory::Troubleshoot,
            &["systemctl --failed", "journalctl -xe | tail -30"]),
        (&["what", "using", "port"], "port usage query", "network", IntentCategory::Factual,
            &["ss -tulpn | head -20"]),
        (&["port", "in", "use"], "port in use query", "network", IntentCategory::Factual,
            &["ss -tulpn | head -20"]),
        (&["won't", "shut", "down"], "shutdown hanging", "services", IntentCategory::Troubleshoot,
            &["systemctl list-jobs", "systemctl --state=running"]),
        (&["shutdown", "stuck"], "shutdown stuck", "services", IntentCategory::Troubleshoot,
            &["systemctl list-jobs", "echo 'Try: sudo systemctl --force poweroff'"]),
        (&["prevent", "shutdown"], "process preventing shutdown", "services", IntentCategory::Troubleshoot,
            &["systemctl list-jobs"]),
    ];

    for (keywords, interpreted, topic, category, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: category.clone(),
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Optimization pattern with topic
type OptPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_optimization(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[OptPattern] = &[
        // Boot time optimization
        (&["reduce", "boot", "time"], "reduce boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -15",
              "echo 'Disable slow services: sudo systemctl disable <service>'"]),
        (&["faster", "boot"], "faster boot optimization", "boot",
            &["systemd-analyze", "systemd-analyze critical-chain",
              "echo 'Tips: Disable unused services, use faster storage'"]),
        (&["optimize", "boot"], "optimize boot time", "boot",
            &["systemd-analyze blame | head -15",
              "echo 'Check /etc/mkinitcpio.conf for unused hooks'"]),
        // Gaming optimization
        (&["optimize", "gaming"], "optimize for gaming", "gaming",
            &["echo 'Install: gamemode, mangohud, corectrl'",
              "echo 'Set CPU governor: sudo cpupower frequency-set -g performance'",
              "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"]),
        (&["gaming", "performance"], "gaming performance", "gaming",
            &["echo 'Enable gamemode: gamemoderun ./game'",
              "echo 'Check GPU: nvidia-smi or radeontop'",
              "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"]),
        (&["game", "lag"], "game lag/stutter", "gaming",
            &["echo 'Try: gamemode, disable compositor'",
              "nvidia-smi 2>/dev/null || echo 'Check GPU with: radeontop'"]),
        // Battery optimization
        (&["improve", "battery"], "improve battery life", "power",
            &["echo 'Install: tlp (power management)'",
              "echo 'sudo pacman -S tlp && sudo systemctl enable tlp'",
              "cat /sys/class/power_supply/BAT*/capacity 2>/dev/null"]),
        (&["battery", "life"], "battery life optimization", "power",
            &["echo 'Install tlp: sudo pacman -S tlp tlp-rdw'",
              "echo 'Enable: sudo systemctl enable --now tlp'",
              "tlp-stat -b 2>/dev/null || echo 'tlp not installed'"]),
        (&["optimize", "battery"], "optimize battery life", "power",
            &["echo 'TLP: sudo pacman -S tlp && sudo systemctl enable --now tlp'",
              "echo 'Check power draw: powertop'"]),
        (&["save", "battery"], "save battery power", "power",
            &["echo 'Enable power saving: sudo tlp start'",
              "echo 'Or: sudo powertop --auto-tune'"]),
        // Swap optimization
        (&["reduce", "swap"], "reduce swap usage", "memory",
            &["cat /proc/sys/vm/swappiness",
              "echo 'Lower swappiness: sudo sysctl vm.swappiness=10'",
              "echo 'Make permanent: add vm.swappiness=10 to /etc/sysctl.d/99-swappiness.conf'"]),
        (&["swap", "usage"], "check swap usage", "memory",
            &["swapon --show", "free -h", "cat /proc/sys/vm/swappiness"]),
        (&["disable", "swap"], "disable swap", "memory",
            &["echo 'Temporarily: sudo swapoff -a'",
              "echo 'Permanently: comment out swap in /etc/fstab'"]),
        // SSD optimization
        (&["tune", "ssd"], "SSD tuning", "storage",
            &["cat /sys/block/*/queue/rotational | head -1",
              "echo 'Enable TRIM: sudo systemctl enable fstrim.timer'",
              "echo 'Check scheduler: cat /sys/block/sda/queue/scheduler'"]),
        (&["optimize", "ssd"], "optimize SSD", "storage",
            &["echo 'Enable weekly TRIM: sudo systemctl enable --now fstrim.timer'",
              "echo 'Use noatime in fstab'",
              "cat /etc/fstab | grep -v '^#'"]),
        (&["ssd", "performance"], "SSD performance settings", "storage",
            &["systemctl status fstrim.timer",
              "echo 'Best practices: noatime in fstab, fstrim.timer enabled'"]),
        // Application profiling
        (&["profile", "application"], "profile application performance", "performance",
            &["echo 'CPU profiling: perf record -g ./program && perf report'",
              "echo 'Or: valgrind --tool=callgrind ./program'",
              "which perf valgrind 2>/dev/null || echo 'Install: pacman -S perf valgrind'"]),
        (&["profile", "performance"], "profile performance", "performance",
            &["echo 'System: perf top'",
              "echo 'Memory: valgrind --tool=memcheck ./program'",
              "echo 'Install tools: pacman -S perf valgrind'"]),
        (&["application", "slow"], "slow application diagnosis", "performance",
            &["echo 'Profile with: strace -c ./program'",
              "echo 'Or: ltrace -c ./program'"]),
        // Memory leak detection
        (&["find", "memory", "leak"], "find memory leaks", "memory",
            &["echo 'Use valgrind: valgrind --leak-check=full ./program'",
              "echo 'Or AddressSanitizer: compile with -fsanitize=address'",
              "which valgrind 2>/dev/null || echo 'Install: pacman -S valgrind'"]),
        (&["memory", "leak", "detection"], "memory leak detection", "memory",
            &["echo 'valgrind --leak-check=full --show-leak-kinds=all ./program'",
              "ps aux --sort=-%mem | head -10"]),
        (&["detect", "memory", "leak"], "detect memory leaks", "memory",
            &["echo 'Run: valgrind --leak-check=full ./program'",
              "echo 'Watch growth: watch -n1 \"ps -o rss= -p PID\"'"]),
        // CPU optimization
        (&["cpu", "governor"], "CPU governor settings", "performance",
            &["cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
              "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors"]),
        (&["cpu", "performance", "mode"], "CPU performance mode", "performance",
            &["echo 'Set performance: sudo cpupower frequency-set -g performance'",
              "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"]),
        // I/O optimization
        (&["io", "performance"], "I/O performance", "storage",
            &["cat /sys/block/*/queue/scheduler",
              "echo 'For SSD use: mq-deadline or none'",
              "echo 'For HDD use: bfq'"]),
        (&["disk", "performance"], "disk performance check", "storage",
            &["iostat -x 1 2 2>/dev/null || echo 'Install: pacman -S sysstat'",
              "cat /sys/block/*/queue/scheduler"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::HowTo,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Slowness pattern with topic
type SlownessPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_slowness(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SlownessPattern] = &[
        // v0.0.991: Post-update slowdown troubleshooting
        (&["slow", "after", "update"], "system slow after update", "performance",
            &["echo '=== Recent Updates ==='",
              "tail -50 /var/log/pacman.log | grep -E '\\[ALPM\\] (installed|upgraded)'",
              "echo '=== Check for issues ==='",
              "systemctl --failed",
              "journalctl -p err -b | tail -20",
              "echo '=== Resource Usage ==='",
              "ps aux --sort=-%cpu | head -5",
              "free -h"]),
        (&["slow", "since", "update"], "system slow since last update", "performance",
            &["tail -50 /var/log/pacman.log | grep -E '\\[ALPM\\]'",
              "systemctl --failed", "journalctl -p err -b | tail -15",
              "ps aux --sort=-%cpu | head -5"]),
        (&["after", "update", "slow"], "performance degraded after update", "performance",
            &["tail -30 /var/log/pacman.log | grep -E 'upgraded|installed'",
              "echo 'Downgrade: sudo pacman -U /var/cache/pacman/pkg/package-OLD.pkg.tar.zst'",
              "systemctl --failed"]),
        (&["update", "broke"], "update broke performance", "performance",
            &["tail -50 /var/log/pacman.log | grep -E '\\[ALPM\\]'",
              "echo 'Recent kernels:' && pacman -Q linux linux-lts 2>/dev/null",
              "echo 'Downgrade if needed: sudo pacman -U /var/cache/pacman/pkg/<package>'"]),
        (&["last", "update", "slow"], "slow after last update", "performance",
            &["tail -30 /var/log/pacman.log | grep -E '\\[ALPM\\]'",
              "journalctl -p err -b | tail -10",
              "ps aux --sort=-%cpu | head -5"]),
        // Boot time
        (&["boot", "time", "slow"], "slow boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -10"]),
        (&["boot", "takes", "long"], "slow boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -10"]),
        (&["slow", "boot"], "slow boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -10"]),
        // System slow
        (&["system", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h", "df -h"]),
        (&["computer", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h"]),
        (&["it's", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h"]),
        (&["everything", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h", "iostat 1 2"]),
        // Desktop/UI slow
        (&["workspace", "stutter"], "workspace switching stutter", "display",
            &["echo 'Check compositor: try picom -b or disable effects'"]),
        (&["compositor", "lag"], "compositor lag", "display",
            &["echo 'Try: picom --vsync or disable compositor'"]),
        (&["animation", "stutter"], "animation stuttering", "display",
            &["nvidia-smi 2>/dev/null || echo 'Check GPU drivers'"]),
        // Network slow
        (&["bandwidth", "using"], "bandwidth usage query", "network",
            &["ss -s", "ip -s link"]),
        (&["what", "using", "network"], "network usage query", "network",
            &["ss -tulpn | head -10"]),
        (&["internet", "slow"], "slow internet connection", "network",
            &["ping -c 3 8.8.8.8", "curl -s https://fast.com/api 2>/dev/null | head -1"]),
        (&["download", "slow"], "slow download speed", "network",
            &["ping -c 3 8.8.8.8", "cat /sys/class/net/*/operstate"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

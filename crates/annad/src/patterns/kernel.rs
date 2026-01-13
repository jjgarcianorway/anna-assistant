//! Kernel and module patterns for Linux kernel management.
//! v0.0.984: Initial implementation.
//! v0.3.30: Added generic kernel version comparison (R3)

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a kernel-related DeepUnderstanding
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

type KernelPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match kernel patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_kernel_version_compare(q)
        .or_else(|| match_kernel_info(q))
        .or_else(|| match_modules(q))
        .or_else(|| match_kernel_params(q))
        .or_else(|| match_dkms(q))
        .or_else(|| match_kernel_debug(q))
}

/// v0.3.30: Generic kernel version comparison (R3)
/// Detects installed kernel package, compares against repo using vercmp
/// Works for linux, linux-lts, linux-zen, linux-cachyos, etc.
fn match_kernel_version_compare(q: &str) -> Option<DeepUnderstanding> {
    // v0.3.30: Generic kernel update check using proper package version comparison
    // This script:
    // 1. Detects which kernel package is installed (linux, linux-lts, linux-cachyos, etc.)
    // 2. Gets installed version from pacman -Qi
    // 3. Gets repo version from pacman -Si
    // 4. Compares using vercmp (proper version comparison)
    const KERNEL_UPDATE_CHECK: &str = r#"KERNEL=$(pacman -Qq | grep -E '^linux$|^linux-lts$|^linux-zen$|^linux-hardened$|^linux-cachyos' | head -1); \
if [ -z "$KERNEL" ]; then echo "No standard kernel package found"; exit 1; fi; \
INSTALLED=$(pacman -Qi "$KERNEL" 2>/dev/null | grep '^Version' | awk '{print $3}'); \
REPO=$(pacman -Si "$KERNEL" 2>/dev/null | grep '^Version' | awk '{print $3}'); \
if [ -z "$REPO" ]; then REPO="$INSTALLED (repo unavailable)"; fi; \
RUNNING=$(uname -r); \
echo "Kernel package: $KERNEL"; \
echo "Installed version: $INSTALLED"; \
echo "Repo version: $REPO"; \
echo "Running kernel: $RUNNING"; \
if [ "$INSTALLED" != "$REPO" ]; then \
  CMP=$(vercmp "$INSTALLED" "$REPO" 2>/dev/null || echo "0"); \
  if [ "$CMP" -lt 0 ]; then echo "Status: UPDATE AVAILABLE"; \
  elif [ "$CMP" -gt 0 ]; then echo "Status: Installed is NEWER than repo"; \
  else echo "Status: Up to date"; fi; \
else echo "Status: Up to date"; fi"#;

    const KERNEL_NEEDS_REBOOT: &str = r#"KERNEL=$(pacman -Qq | grep -E '^linux$|^linux-lts$|^linux-zen$|^linux-hardened$|^linux-cachyos' | head -1); \
INSTALLED=$(pacman -Qi "$KERNEL" 2>/dev/null | grep '^Version' | awk '{print $3}'); \
RUNNING=$(uname -r); \
echo "Installed: $INSTALLED"; \
echo "Running: $RUNNING"; \
if echo "$RUNNING" | grep -qF "${INSTALLED%%-*}"; then \
  echo "Status: Running matches installed (no reboot needed)"; \
else \
  echo "Status: REBOOT RECOMMENDED - running kernel differs from installed"; \
fi"#;

    let patterns: &[KernelPattern] = &[
        // Kernel update check - uses proper package version comparison
        (&["kernel", "update"], "check kernel update status", "kernel",
         &[KERNEL_UPDATE_CHECK]),
        (&["kernel", "outdated"], "check if kernel is outdated", "kernel",
         &[KERNEL_UPDATE_CHECK]),
        (&["kernel", "latest"], "check latest kernel version", "kernel",
         &[KERNEL_UPDATE_CHECK]),
        // Compare running vs installed
        (&["kernel", "reboot"], "check if kernel reboot needed", "kernel",
         &[KERNEL_NEEDS_REBOOT]),
        (&["need", "reboot"], "check if reboot is needed", "kernel",
         &[KERNEL_NEEDS_REBOOT, "cat /var/run/reboot-required 2>/dev/null || echo 'No reboot-required file'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Kernel information patterns
fn match_kernel_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[KernelPattern] = &[
        // Kernel version
        (&["kernel", "version"], "show kernel version", "kernel",
         &["uname -r", "cat /proc/version"]),
        (&["uname"], "show system information", "kernel",
         &["uname -a"]),
        // Installed kernels
        (&["installed", "kernel"], "list installed kernels", "kernel",
         &["pacman -Q | grep -E '^linux[0-9]|^linux-'", "ls /boot/vmlinuz-*"]),
        (&["available", "kernel"], "show available kernels", "kernel",
         &["pacman -Ss '^linux$|^linux-lts|^linux-zen|^linux-hardened'"]),
        (&["kernel", "packages"], "list kernel packages", "kernel",
         &["pacman -Q | grep -E '^linux'"]),
        // Kernel release info
        (&["kernel", "release"], "show kernel release", "kernel",
         &["uname -r", "hostnamectl | grep Kernel"]),
        // Running kernel
        (&["running", "kernel"], "show running kernel", "kernel",
         &["uname -r"]),
        (&["current", "kernel"], "show current kernel", "kernel",
         &["uname -r", "cat /proc/version"]),
        // Kernel config
        (&["kernel", "config"], "show kernel configuration", "kernel",
         &["zcat /proc/config.gz 2>/dev/null | head -50 || cat /boot/config-$(uname -r) 2>/dev/null | head -50"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Module patterns
fn match_modules(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[KernelPattern] = &[
        // List modules
        (&["loaded", "module"], "list loaded modules", "kernel",
         &["lsmod"]),
        (&["list", "module"], "list kernel modules", "kernel",
         &["lsmod"]),
        (&["lsmod"], "show loaded modules", "kernel",
         &["lsmod | head -30"]),
        // Module info
        (&["module", "info"], "show module information", "kernel",
         &["modinfo <module_name>"]),
        (&["modinfo"], "show module details", "kernel",
         &["modinfo <module_name>"]),
        // Module dependencies
        (&["module", "depend"], "show module dependencies", "kernel",
         &["modprobe --show-depends <module_name>"]),
        // Blacklisted modules
        (&["blacklist", "module"], "show blacklisted modules", "kernel",
         &["cat /etc/modprobe.d/*.conf 2>/dev/null | grep blacklist"]),
        (&["blocked", "module"], "show blocked modules", "kernel",
         &["cat /etc/modprobe.d/*.conf 2>/dev/null | grep -E 'blacklist|install.*false'"]),
        // Module parameters
        (&["module", "param"], "show module parameters", "kernel",
         &["systool -v -m <module_name> 2>/dev/null | head -30"]),
        (&["module", "option"], "show module options", "kernel",
         &["modinfo <module_name> | grep parm"]),
        // Specific modules
        (&["nvidia", "module"], "check NVIDIA module", "kernel",
         &["lsmod | grep nvidia", "modinfo nvidia 2>/dev/null | head -10"]),
        (&["amdgpu", "module"], "check AMD GPU module", "kernel",
         &["lsmod | grep amdgpu", "modinfo amdgpu 2>/dev/null | head -10"]),
        (&["bluetooth", "module"], "check Bluetooth modules", "kernel",
         &["lsmod | grep -i bluetooth", "lsmod | grep btusb"]),
        (&["wifi", "module"], "check WiFi modules", "kernel",
         &["lsmod | grep -E 'cfg80211|mac80211|iwl|ath|rtl|brcm'"]),
        (&["wireless", "module"], "check wireless modules", "kernel",
         &["lsmod | grep -E 'cfg80211|mac80211'"]),
        // USB modules
        (&["usb", "module"], "check USB modules", "kernel",
         &["lsmod | grep usb"]),
        // Sound modules
        (&["sound", "module"], "check sound modules", "kernel",
         &["lsmod | grep snd"]),
        (&["audio", "module"], "check audio modules", "kernel",
         &["lsmod | grep -E 'snd|hda|hdmi'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Kernel parameters patterns
fn match_kernel_params(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[KernelPattern] = &[
        // Boot parameters
        (&["kernel", "param"], "show kernel parameters", "kernel",
         &["cat /proc/cmdline"]),
        (&["boot", "param"], "show boot parameters", "kernel",
         &["cat /proc/cmdline"]),
        (&["cmdline"], "show kernel command line", "kernel",
         &["cat /proc/cmdline"]),
        // Sysctl
        (&["sysctl"], "show sysctl values", "kernel",
         &["sysctl -a 2>/dev/null | head -50"]),
        (&["kernel", "sysctl"], "show kernel sysctl", "kernel",
         &["sysctl kernel"]),
        // Specific sysctls
        (&["swappiness"], "show swappiness value", "kernel",
         &["sysctl vm.swappiness", "cat /proc/sys/vm/swappiness"]),
        (&["dirty", "ratio"], "show dirty ratio", "kernel",
         &["sysctl vm.dirty_ratio", "sysctl vm.dirty_background_ratio"]),
        (&["inotify", "watch"], "show inotify watches", "kernel",
         &["sysctl fs.inotify.max_user_watches"]),
        (&["file", "handles"], "show file handle limits", "kernel",
         &["sysctl fs.file-max", "cat /proc/sys/fs/file-max"]),
        (&["max", "files"], "show max open files", "kernel",
         &["ulimit -n", "sysctl fs.file-max"]),
        // Kernel features
        (&["kernel", "feature"], "check kernel features", "kernel",
         &["zcat /proc/config.gz 2>/dev/null | grep -E '^CONFIG_' | head -30"]),
        (&["kernel", "capabil"], "show kernel capabilities", "kernel",
         &["cat /proc/sys/kernel/cap_last_cap", "capsh --print"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// DKMS patterns
fn match_dkms(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[KernelPattern] = &[
        // DKMS status
        (&["dkms", "status"], "show DKMS status", "kernel",
         &["dkms status"]),
        (&["dkms", "list"], "list DKMS modules", "kernel",
         &["dkms status"]),
        // DKMS modules
        (&["dkms", "module"], "show DKMS modules", "kernel",
         &["dkms status", "ls /var/lib/dkms/"]),
        // DKMS version
        (&["dkms", "version"], "show DKMS version", "kernel",
         &["dkms --version"]),
        // DKMS issues
        (&["dkms", "fail"], "check DKMS failures", "kernel",
         &["dkms status | grep -i fail", "journalctl -b | grep -i dkms"]),
        (&["dkms", "error"], "check DKMS errors", "kernel",
         &["dkms status", "cat /var/lib/dkms/*/kernel-*/log/make.log 2>/dev/null | tail -30"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Kernel debugging patterns
fn match_kernel_debug(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[KernelPattern] = &[
        // Kernel messages
        (&["kernel", "message"], "show kernel messages", "kernel",
         &["dmesg | tail -50"]),
        (&["kernel", "log"], "show kernel log", "kernel",
         &["dmesg -T | tail -50", "journalctl -k -n 50"]),
        // Kernel errors
        (&["kernel", "error"], "show kernel errors", "kernel",
         &["dmesg -l err,warn | tail -30"]),
        (&["kernel", "warn"], "show kernel warnings", "kernel",
         &["dmesg -l warn | tail -30"]),
        // Kernel panic
        (&["kernel", "panic"], "check for kernel panics", "kernel",
         &["journalctl -k | grep -i panic", "dmesg | grep -i panic"]),
        // Kernel oops
        (&["kernel", "oops"], "check for kernel oops", "kernel",
         &["dmesg | grep -iE 'oops|bug|rip|call trace' | head -30"]),
        // Tainted kernel
        (&["tainted", "kernel"], "check if kernel is tainted", "kernel",
         &["cat /proc/sys/kernel/tainted", "dmesg | grep -i tainted"]),
        (&["kernel", "taint"], "show kernel taint status", "kernel",
         &["cat /proc/sys/kernel/tainted"]),
        // Kernel stack traces
        (&["kernel", "trace"], "show kernel traces", "kernel",
         &["dmesg | grep -A20 'Call Trace' | head -40"]),
        (&["call", "trace"], "show call traces", "kernel",
         &["dmesg | grep -A20 'Call Trace' | head -40"]),
        // Kernel lockups
        (&["kernel", "lockup"], "check for kernel lockups", "kernel",
         &["dmesg | grep -i lockup", "journalctl -k | grep -i 'hard lockup\\|soft lockup'"]),
        // IRQ
        (&["irq", "info"], "show IRQ information", "kernel",
         &["cat /proc/interrupts | head -30"]),
        (&["interrupt"], "show interrupts", "kernel",
         &["cat /proc/interrupts | head -30"]),
        // I/O schedulers
        (&["io", "scheduler"], "show I/O schedulers", "kernel",
         &["cat /sys/block/*/queue/scheduler"]),
        (&["elevator"], "show disk elevator", "kernel",
         &["cat /sys/block/*/queue/scheduler"]),
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
    fn test_kernel_info() {
        assert!(match_patterns("kernel version").is_some());
        assert!(match_patterns("installed kernels").is_some());
        assert!(match_patterns("running kernel").is_some());
    }

    #[test]
    fn test_modules() {
        assert!(match_patterns("loaded modules").is_some());
        assert!(match_patterns("lsmod").is_some());
        assert!(match_patterns("blacklisted modules").is_some());
    }

    #[test]
    fn test_kernel_params() {
        assert!(match_patterns("kernel parameters").is_some());
        assert!(match_patterns("sysctl").is_some());
        assert!(match_patterns("swappiness").is_some());
    }

    #[test]
    fn test_dkms() {
        assert!(match_patterns("dkms status").is_some());
        assert!(match_patterns("dkms modules").is_some());
    }

    #[test]
    fn test_kernel_debug() {
        assert!(match_patterns("kernel errors").is_some());
        assert!(match_patterns("kernel panic").is_some());
        assert!(match_patterns("tainted kernel").is_some());
    }

    // v0.3.30: Test kernel version comparison (R3)
    #[test]
    fn test_kernel_version_compare() {
        // Test that kernel update patterns exist and use vercmp
        let update = match_patterns("kernel update");
        assert!(update.is_some(), "kernel update pattern should match");
        let cmds = update.unwrap().suggested_commands;
        assert!(!cmds.is_empty(), "should have commands");
        // Verify commands use proper package comparison, not just uname
        let cmd = &cmds[0];
        assert!(cmd.contains("pacman -Qi"), "should query installed version");
        assert!(cmd.contains("pacman -Si") || cmd.contains("repo"), "should query repo version");
        assert!(cmd.contains("vercmp"), "should use vercmp for comparison");

        // Test kernel outdated pattern
        let outdated = match_patterns("kernel outdated");
        assert!(outdated.is_some(), "kernel outdated pattern should match");

        // Test need reboot pattern
        let reboot = match_patterns("need reboot");
        assert!(reboot.is_some(), "need reboot pattern should match");
    }

    #[test]
    fn test_kernel_version_compare_uses_package_not_uname() {
        // v0.3.30: Verify we use package version comparison, not uname output
        let update = match_patterns("kernel update").unwrap();
        let cmd = &update.suggested_commands[0];

        // Should NOT be comparing uname -r output to package names
        // Should be comparing package versions from pacman
        assert!(!cmd.contains("uname -r | grep"), "should not grep uname output");
        assert!(cmd.contains("pacman -Qq"), "should detect kernel package name");
        assert!(cmd.contains("linux-cachyos"), "should support CachyOS kernel");
    }
}

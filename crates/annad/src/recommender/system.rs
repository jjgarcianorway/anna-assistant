//! System recommendations

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_trim_timer(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if any SSD is present
    let has_ssd = facts
        .storage_devices
        .iter()
        .any(|d| d.name.starts_with("/dev/sd") || d.name.starts_with("/dev/nvme"));

    if has_ssd {
        // Check if fstrim.timer is enabled
        let timer_status = Command::new("systemctl")
            .args(&["is-enabled", "fstrim.timer"])
            .output();

        let is_enabled = timer_status.map(|o| o.status.success()).unwrap_or(false);

        if !is_enabled {
            result.push(Advice {
                id: "fstrim-timer".to_string(),
                title: "Keep your SSD healthy with TRIM".to_string(),
                reason: "I noticed you have a solid-state drive. SSDs need regular 'TRIM' operations to stay fast and last longer. Think of it like taking out the trash - it tells the SSD which data blocks are no longer in use.".to_string(),
                action: "Enable automatic weekly TRIM to keep your SSD running smoothly".to_string(),
                command: Some("systemctl enable --now fstrim.timer".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Solid_state_drive#TRIM".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_sysctl_parameters() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if sysctl parameters file exists
    if let Ok(sysctl) = std::fs::read_to_string("/etc/sysctl.conf") {
        // Check for important security parameters
        if !sysctl.contains("kernel.dmesg_restrict") {
            result.push(Advice {
                id: "kernel-dmesg-restrict".to_string(),
                title: "Restrict kernel dmesg to root only".to_string(),
                reason: "kernel.dmesg_restrict=1 prevents non-root users from reading kernel ring buffer messages, which can contain sensitive system information. This is a basic security hardening step.".to_string(),
                action: "Add kernel.dmesg_restrict=1 to sysctl.conf".to_string(),
                command: Some("echo 'kernel.dmesg_restrict = 1' | sudo tee -a /etc/sysctl.conf && sudo sysctl -p".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Security#Kernel_hardening".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("security-hardening".to_string()),
            satisfies: Vec::new(),
                popularity: 60,
            requires: Vec::new(),
            });
        }

        if !sysctl.contains("kernel.kptr_restrict") {
            result.push(Advice {
                id: "kernel-kptr-restrict".to_string(),
                title: "Hide kernel pointers from unprivileged users".to_string(),
                reason: "kernel.kptr_restrict prevents exposing kernel addresses which could be used in exploits. Setting this to 2 hides kernel pointers even from root when using sudo (only visible to direct root login).".to_string(),
                action: "Add kernel.kptr_restrict=2 to sysctl.conf".to_string(),
                command: Some("echo 'kernel.kptr_restrict = 2' | sudo tee -a /etc/sysctl.conf && sudo sysctl -p".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Security#Kernel_hardening".to_string()],
                depends_on: Vec::new(),
                related_to: vec!["kernel-dmesg-restrict".to_string()],
                bundle: Some("security-hardening".to_string()),
            satisfies: Vec::new(),
                popularity: 60,
            requires: Vec::new(),
            });
        }

        if !sysctl.contains("net.ipv4.tcp_syncookies") {
            result.push(Advice {
                id: "tcp-syncookies".to_string(),
                title: "Enable TCP SYN cookie protection".to_string(),
                reason: "TCP SYN cookies protect against SYN flood attacks (a type of DDoS). This is essential protection for any system exposed to the internet.".to_string(),
                action: "Enable TCP SYN cookies".to_string(),
                command: Some("echo 'net.ipv4.tcp_syncookies = 1' | sudo tee -a /etc/sysctl.conf && sudo sysctl -p".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Security#TCP/IP_stack_hardening".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("security-hardening".to_string()),
            satisfies: Vec::new(),
                popularity: 70,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_firmware_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for fwupd (firmware updates)
    let has_fwupd = Command::new("pacman")
        .args(&["-Q", "fwupd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fwupd {
        result.push(Advice {
            id: "fwupd".to_string(),
            title: "Install fwupd for firmware updates".to_string(),
            reason: "Firmware updates are important for security and hardware stability! fwupd lets you update device firmware (BIOS, SSD, USB devices, etc.) right from Linux. Many vendors now support it officially. Keep your hardware up to date!".to_string(),
            action: "Install fwupd for firmware management".to_string(),
            command: Some("pacman -S --noconfirm fwupd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fwupd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_firmware_updates() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if fwupd is installed
    let has_fwupd = Command::new("pacman")
        .args(&["-Q", "fwupd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fwupd {
        result.push(Advice {
            id: "firmware-fwupd".to_string(),
            title: "Install fwupd for automatic firmware updates".to_string(),
            reason: "Your system hardware probably has firmware that needs updates! fwupd provides firmware updates for your motherboard, SSD, GPU, USB devices, and more - all from within Linux. It's like Windows Update but for your hardware firmware. Keeping firmware updated fixes bugs, improves performance, and patches security vulnerabilities.".to_string(),
            action: "Install fwupd for firmware management".to_string(),
            command: Some("pacman -S --noconfirm fwupd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fwupd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    } else {
        // Suggest running firmware check
        result.push(Advice {
            id: "firmware-check-updates".to_string(),
            title: "Check for available firmware updates".to_string(),
            reason: "You have fwupd installed - run 'fwupdmgr get-updates' to check if your hardware has firmware updates available! This checks your motherboard, SSD, peripherals, and more for security patches and improvements.".to_string(),
            action: "Check for firmware updates".to_string(),
            command: Some("fwupdmgr refresh && fwupdmgr get-updates".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fwupd#Usage".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_journal_size() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check journal size
    let journal_size = Command::new("journalctl")
        .args(&["--disk-usage"])
        .output()
        .ok()
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            // Parse "Archived and active journals take up 512.0M in the file system."
            output
                .split_whitespace()
                .find(|s| s.ends_with("M") || s.ends_with("G"))
                .and_then(|s| {
                    let num: String = s
                        .chars()
                        .take_while(|c| c.is_numeric() || *c == '.')
                        .collect();
                    num.parse::<f64>()
                        .ok()
                        .map(|n| if s.ends_with("G") { n * 1024.0 } else { n })
                })
        });

    if let Some(size_mb) = journal_size {
        if size_mb > 500.0 {
            result.push(Advice {
                id: "journal-large-size".to_string(),
                title: format!("Journal logs are using {:.0}MB of disk space", size_mb),
                reason: format!("Your systemd journal has grown to {:.0}MB! Journal logs accumulate over time and can waste significant disk space. You can safely clean old logs - they're mainly useful for debugging recent issues. systemd can automatically limit journal size.", size_mb),
                action: "Clean old journal logs and set size limit".to_string(),
                command: Some("journalctl --vacuum-size=100M && echo 'SystemMaxUse=100M' >> /etc/systemd/journald.conf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal#Journal_size_limit".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check if journal size limit is configured
    let journald_conf = std::fs::read_to_string("/etc/systemd/journald.conf").unwrap_or_default();
    let has_size_limit = journald_conf
        .lines()
        .any(|line| !line.trim().starts_with('#') && line.contains("SystemMaxUse"));

    if !has_size_limit {
        result.push(Advice {
            id: "journal-set-limit".to_string(),
            title: "Set systemd journal size limit".to_string(),
            reason: "No journal size limit is configured! Without a limit, logs can grow indefinitely and fill your disk over time. Setting 'SystemMaxUse=100M' in journald.conf keeps logs under control while still keeping enough history for troubleshooting. Set it and forget it!".to_string(),
            action: "Configure journal size limit".to_string(),
            command: Some("echo 'SystemMaxUse=100M' >> /etc/systemd/journald.conf && systemctl restart systemd-journald".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal#Journal_size_limit".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_locale_timezone() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if locale is configured
    let locale_conf = std::fs::read_to_string("/etc/locale.conf").unwrap_or_default();
    if locale_conf.is_empty() || !locale_conf.contains("LANG=") {
        result.push(Advice {
            id: "locale-not-set".to_string(),
            title: "System locale is not configured".to_string(),
            reason: "Your system locale isn't properly set! This affects how dates, times, numbers, and currency are displayed. It can cause weird formatting in applications and sometimes break programs that expect a specific locale. Set it to match your language/region (e.g., en_US.UTF-8 for US English).".to_string(),
            action: "Configure system locale".to_string(),
            command: Some("echo 'LANG=en_US.UTF-8' > /etc/locale.conf && locale-gen".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Locale".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check timezone
    let tz_link = std::fs::read_link("/etc/localtime").ok();
    if tz_link.is_none() {
        result.push(Advice {
            id: "timezone-not-set".to_string(),
            title: "System timezone is not configured".to_string(),
            reason: "Your timezone isn't set! This means your system clock shows UTC time instead of your local time. It affects file timestamps, logs, scheduled tasks, and anything time-related. Set it to your actual timezone so times make sense!".to_string(),
            action: "Set system timezone".to_string(),
            command: Some("timedatectl set-timezone America/New_York".to_string()), // Example
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/System_time#Time_zone".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check if NTP is enabled for time sync
    let ntp_enabled = Command::new("timedatectl")
        .args(&["show", "--property=NTP", "--value"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "yes")
        .unwrap_or(false);

    if !ntp_enabled {
        result.push(Advice {
            id: "ntp-not-enabled".to_string(),
            title: "Automatic time synchronization is disabled".to_string(),
            reason: "NTP (Network Time Protocol) isn't enabled! Your system clock will drift over time, leading to incorrect timestamps. This can break SSL certificates, cause authentication failures, and mess up logs. Enable NTP to keep your clock accurate automatically - systemd-timesyncd does this perfectly!".to_string(),
            action: "Enable NTP time synchronization".to_string(),
            command: Some("timedatectl set-ntp true".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd-timesyncd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_kernel_parameters() -> Vec<Advice> {
    let mut result = Vec::new();

    // Read current kernel parameters
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();

    // Check for quiet parameter (reduces boot messages)
    if !cmdline.contains("quiet") {
        result.push(Advice {
            id: "kernel-quiet-boot".to_string(),
            title: "Add 'quiet' kernel parameter for cleaner boot".to_string(),
            reason: "Your boot shows all kernel messages! Adding 'quiet' to kernel parameters makes boot look cleaner by hiding verbose kernel output. You'll still see important errors, but not the flood of driver messages. Makes your system look more polished!".to_string(),
            action: "Add 'quiet' to GRUB_CMDLINE_LINUX in /etc/default/grub".to_string(),
            command: Some("cat /proc/cmdline".to_string()), // Show current kernel parameters
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Desktop Customization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Kernel_parameters".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for splash (boot splash screen)
    if !cmdline.contains("splash") {
        result.push(Advice {
            id: "kernel-splash-screen".to_string(),
            title: "Add 'splash' for graphical boot screen".to_string(),
            reason: "Want a pretty boot screen instead of text? Add 'splash' kernel parameter and install plymouth for a graphical boot animation. Makes your system boot look professional and modern instead of showing raw text. Purely cosmetic but nice!".to_string(),
            action: "Add 'splash' parameter and install plymouth".to_string(),
            command: Some("pacman -Qi plymouth 2>/dev/null | grep -E '^Name|^Installed' || echo 'Plymouth not installed'".to_string()), // Check if plymouth is installed
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Desktop Customization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Plymouth".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_bootloader_optimization() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check GRUB timeout
    let grub_config = std::fs::read_to_string("/etc/default/grub").unwrap_or_default();

    if grub_config.contains("GRUB_TIMEOUT=5") || grub_config.contains("GRUB_TIMEOUT=10") {
        result.push(Advice {
            id: "bootloader-reduce-timeout".to_string(),
            title: "Reduce GRUB timeout for faster boot".to_string(),
            reason: "Your GRUB waits 5-10 seconds before booting! If you have one OS and don't need to pick kernels, reduce GRUB_TIMEOUT to 1 or 2 seconds. Saves time on every boot! You can still access the menu by holding Shift during boot if needed.".to_string(),
            action: "Set GRUB_TIMEOUT=1 in /etc/default/grub".to_string(),
            command: Some("sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=1/' /etc/default/grub && grub-mkconfig -o /boot/grub/grub.cfg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GRUB/Tips_and_tricks#Speeding_up_GRUB".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for GRUB background
    if !grub_config.contains("GRUB_BACKGROUND") {
        result.push(Advice {
            id: "bootloader-custom-background".to_string(),
            title: "Consider adding custom GRUB background".to_string(),
            reason: "Your GRUB menu is plain! You can set a custom background image with GRUB_BACKGROUND in /etc/default/grub. Makes your bootloader look personalized and professional. Any PNG/JPG image works!".to_string(),
            action: "Set GRUB_BACKGROUND=/path/to/image.png".to_string(),
            command: Some("grep -E '^GRUB_' /etc/default/grub 2>/dev/null | head -5".to_string()), // Show current GRUB settings
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Desktop Customization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GRUB/Tips_and_tricks#Background_image_and_bitmap_fonts".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_systemd_timers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has cron jobs but not using systemd timers
    let has_crontab = Command::new("crontab")
        .arg("-l")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_crontab {
        result.push(Advice {
            id: "timers-systemd".to_string(),
            title: "Consider using systemd timers instead of cron".to_string(),
            reason: "You have cron jobs! Systemd timers are the modern alternative - better logging, easier debugging, dependency management, and integrated with systemctl. Plus they can run on boot, handle missed runs, and have calendar-based scheduling. Arch recommends timers over cron!".to_string(),
            action: "Learn about systemd timers".to_string(),
            command: Some("crontab -l 2>/dev/null | grep -v '^#' | head -5 || echo 'No user crontab entries'".to_string()), // Show current crontab
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Timers".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_locale_configuration(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if facts.locale_info.timezone == "UTC" || facts.locale_info.timezone == "Unknown" {
        result.push(Advice::new(
            "configure-timezone".to_string(),
            "Configure System Timezone".to_string(),
            "System timezone is UTC or unconfigured. Setting your local timezone ensures accurate timestamps in applications, logs, and files.".to_string(),
            "Set your local timezone using timedatectl".to_string(),
            Some("timedatectl list-timezones | grep -i <region> && sudo timedatectl set-timezone <timezone>".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/System_time#Time_zone".to_string()],
            "system".to_string(),
        ).with_popularity(40));
    }

    result
}


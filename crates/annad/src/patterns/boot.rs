//! Boot patterns - bootloader, GRUB, EFI, kernel, and boot diagnostics
//! v0.0.951: Initial boot patterns for boot troubleshooting

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, and command templates
type BootPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

/// Match common boot-related questions
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // GRUB
    if let Some(u) = match_grub(q) {
        return Some(u);
    }
    // EFI/UEFI
    if let Some(u) = match_efi(q) {
        return Some(u);
    }
    // Kernel
    if let Some(u) = match_kernel(q) {
        return Some(u);
    }
    // Boot issues
    if let Some(u) = match_boot_issues(q) {
        return Some(u);
    }
    // Initramfs
    if let Some(u) = match_initramfs(q) {
        return Some(u);
    }
    None
}

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

/// GRUB queries
fn match_grub(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BootPattern] = &[
        // GRUB configuration
        (&["grub", "config"], "show GRUB configuration", "boot",
            &["cat /etc/default/grub", "cat /boot/grub/grub.cfg | head -50"]),
        (&["grub", "menu"], "show GRUB menu entries", "boot",
            &["grep -E '^menuentry|^submenu' /boot/grub/grub.cfg",
              "cat /boot/grub/grub.cfg | grep menuentry | head -20"]),
        (&["grub", "timeout"], "check GRUB timeout", "boot",
            &["grep GRUB_TIMEOUT /etc/default/grub"]),
        // GRUB regeneration
        (&["regenerate", "grub"], "regenerate GRUB config", "boot",
            &["sudo grub-mkconfig -o /boot/grub/grub.cfg"]),
        (&["update", "grub"], "update GRUB configuration", "boot",
            &["sudo grub-mkconfig -o /boot/grub/grub.cfg"]),
        (&["reinstall", "grub"], "reinstall GRUB bootloader", "boot",
            &["sudo grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB",
              "sudo grub-install /dev/sda"]),
        // GRUB theme
        (&["grub", "theme"], "check GRUB theme", "boot",
            &["grep GRUB_THEME /etc/default/grub", "ls /boot/grub/themes 2>/dev/null"]),
        // GRUB rescue
        (&["grub", "rescue"], "GRUB rescue mode help", "boot",
            &["echo 'In GRUB rescue: ls, set root=(hdX,Y), insmod normal, normal'",
              "echo 'Or boot live USB to reinstall GRUB'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// EFI/UEFI queries
fn match_efi(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BootPattern] = &[
        // EFI variables
        (&["efi", "variable"], "list EFI variables", "boot",
            &["efivar -l 2>/dev/null | head -20 || ls /sys/firmware/efi/efivars | head -20"]),
        (&["efi", "boot", "entry"], "list EFI boot entries", "boot",
            &["efibootmgr -v", "efibootmgr"]),
        (&["uefi", "boot"], "show UEFI boot entries", "boot",
            &["efibootmgr -v"]),
        // Boot order
        (&["boot", "order"], "show boot order", "boot",
            &["efibootmgr", "efibootmgr -v | grep -E 'BootOrder|Boot[0-9]+'"]),
        (&["change", "boot", "order"], "change boot order", "boot",
            &["echo 'sudo efibootmgr -o XXXX,YYYY'", "efibootmgr"]),
        // EFI partition
        (&["efi", "partition"], "check EFI partition", "boot",
            &["lsblk -o NAME,SIZE,TYPE,MOUNTPOINT | grep -i efi",
              "df -Th | grep -i efi", "ls /boot/efi 2>/dev/null || ls /boot 2>/dev/null"]),
        (&["esp", "partition"], "check ESP partition", "boot",
            &["lsblk -f | grep -i efi", "findmnt /boot/efi 2>/dev/null || findmnt /boot 2>/dev/null"]),
        // UEFI mode
        (&["uefi", "mode"], "check if booted in UEFI mode", "boot",
            &["ls /sys/firmware/efi && echo 'UEFI mode' || echo 'Legacy/BIOS mode'",
              "efibootmgr 2>/dev/null && echo 'UEFI mode'"]),
        (&["bios", "uefi"], "check BIOS or UEFI", "boot",
            &["[ -d /sys/firmware/efi ] && echo 'UEFI mode' || echo 'Legacy BIOS mode'"]),
        // Secure Boot
        (&["secure", "boot"], "check Secure Boot status", "boot",
            &["mokutil --sb-state 2>/dev/null || echo 'mokutil not available'",
              "bootctl status 2>/dev/null | grep -i secure"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Kernel queries
fn match_kernel(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BootPattern] = &[
        // Kernel version
        (&["kernel", "version"], "show kernel version", "boot",
            &["uname -r", "uname -a"]),
        (&["kernel", "info"], "show kernel information", "boot",
            &["uname -a", "cat /proc/version"]),
        // Installed kernels
        (&["installed", "kernel"], "list installed kernels", "boot",
            &["pacman -Q | grep -E '^linux'", "ls /boot/vmlinuz-*",
              "ls /lib/modules"]),
        (&["list", "kernel"], "list available kernels", "boot",
            &["pacman -Q | grep linux | head -10", "ls /boot/vmlinuz-* 2>/dev/null"]),
        // Kernel parameters
        (&["kernel", "param"], "show kernel parameters", "boot",
            &["cat /proc/cmdline"]),
        (&["boot", "param"], "show boot parameters", "boot",
            &["cat /proc/cmdline", "grep GRUB_CMDLINE_LINUX /etc/default/grub"]),
        // Kernel modules
        (&["kernel", "module"], "list kernel modules", "boot",
            &["lsmod | head -30", "cat /proc/modules | head -30"]),
        (&["loaded", "module"], "list loaded modules", "boot",
            &["lsmod", "lsmod | wc -l"]),
        // DKMS
        (&["dkms", "status"], "check DKMS status", "boot",
            &["dkms status", "ls /var/lib/dkms"]),
        (&["dkms", "module"], "list DKMS modules", "boot",
            &["dkms status", "ls /var/lib/dkms 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Boot issue queries
fn match_boot_issues(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BootPattern] = &[
        // Boot time
        (&["boot", "time"], "check boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -15"]),
        (&["slow", "boot"], "diagnose slow boot", "boot",
            &["systemd-analyze blame | head -20", "systemd-analyze critical-chain"]),
        (&["boot", "blame"], "show boot time blame", "boot",
            &["systemd-analyze blame | head -20"]),
        // Boot logs
        (&["boot", "log"], "show boot logs", "boot",
            &["journalctl -b | head -100", "journalctl -b -p err | head -30"]),
        (&["last", "boot"], "show last boot log", "boot",
            &["journalctl -b -1 | head -50 2>/dev/null || journalctl -b | head -50"]),
        (&["previous", "boot"], "show previous boot log", "boot",
            &["journalctl -b -1 | head -100"]),
        // Boot errors
        (&["boot", "error"], "check boot errors", "boot",
            &["journalctl -b -p err | head -30", "dmesg | grep -i error | head -20"]),
        (&["boot", "fail"], "check boot failures", "boot",
            &["systemctl --failed", "journalctl -b -p crit | head -20"]),
        // Boot stages
        (&["boot", "stage"], "show boot stages", "boot",
            &["systemd-analyze critical-chain", "systemd-analyze"]),
        (&["boot", "target"], "show boot target", "boot",
            &["systemctl get-default", "systemctl list-units --type=target --state=active"]),
        // Recovery
        (&["boot", "recovery"], "boot recovery options", "boot",
            &["echo 'Add init=/bin/bash to kernel parameters for rescue shell'",
              "echo 'Or select recovery option from GRUB menu'"]),
        (&["emergency", "mode"], "emergency mode info", "boot",
            &["systemctl status emergency.target", "echo 'Use: systemctl default to return to normal'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Initramfs queries
fn match_initramfs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BootPattern] = &[
        // Initramfs info
        (&["initramfs", "info"], "show initramfs information", "boot",
            &["ls -la /boot/initramfs-*.img 2>/dev/null || ls -la /boot/initrd*",
              "lsinitcpio /boot/initramfs-linux.img 2>/dev/null | head -30"]),
        (&["initrd"], "show initrd information", "boot",
            &["ls -la /boot/init*", "file /boot/initramfs-*.img 2>/dev/null"]),
        // Regenerate initramfs
        (&["regenerate", "initramfs"], "regenerate initramfs", "boot",
            &["sudo mkinitcpio -P", "sudo mkinitcpio -p linux"]),
        (&["rebuild", "initramfs"], "rebuild initramfs", "boot",
            &["sudo mkinitcpio -P"]),
        (&["mkinitcpio"], "mkinitcpio status", "boot",
            &["cat /etc/mkinitcpio.conf | grep -v '^#' | grep -v '^$'",
              "ls /etc/mkinitcpio.d/"]),
        // Initramfs contents
        (&["initramfs", "content"], "show initramfs contents", "boot",
            &["lsinitcpio /boot/initramfs-linux.img 2>/dev/null | head -50",
              "lsinitrd 2>/dev/null | head -50"]),
        // Dracut (alternative)
        (&["dracut"], "dracut status", "boot",
            &["dracut --list-modules 2>/dev/null || echo 'dracut not installed (Arch uses mkinitcpio)'",
              "pacman -Qs dracut"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grub() {
        assert!(match_patterns("grub config").is_some());
        assert!(match_patterns("update grub").is_some());
        assert!(match_patterns("grub menu").is_some());
    }

    #[test]
    fn test_efi() {
        assert!(match_patterns("efi boot entry").is_some());
        assert!(match_patterns("boot order").is_some());
        assert!(match_patterns("secure boot").is_some());
    }

    #[test]
    fn test_kernel() {
        assert!(match_patterns("kernel version").is_some());
        assert!(match_patterns("installed kernels").is_some());
        assert!(match_patterns("kernel parameters").is_some());
    }

    #[test]
    fn test_boot_issues() {
        assert!(match_patterns("boot time").is_some());
        assert!(match_patterns("slow boot").is_some());
        assert!(match_patterns("boot errors").is_some());
    }

    #[test]
    fn test_initramfs() {
        assert!(match_patterns("initramfs info").is_some());
        assert!(match_patterns("regenerate initramfs").is_some());
    }
}

//! Recovery scenario patterns - user clearly needs urgent help
//! v0.0.913: Added suggested_commands with immediate recovery steps
//! v0.0.989: Expanded with rescue mode, chroot, fstab, bootloader patterns

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, and solution commands
type RecoveryPattern = (&'static [&'static str], &'static str, &'static [&'static str]);

/// Match recovery scenarios that need immediate help
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Accidental deletion
    if let Some(u) = match_deletion(q) {
        return Some(u);
    }
    // Boot failures
    if let Some(u) = match_boot_failure(q) {
        return Some(u);
    }
    // Rescue mode and chroot
    if let Some(u) = match_rescue_mode(q) {
        return Some(u);
    }
    // Permission disasters
    if let Some(u) = match_permission_disaster(q) {
        return Some(u);
    }
    // Other emergencies
    if let Some(u) = match_emergency(q) {
        return Some(u);
    }
    None
}

fn match_deletion(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[RecoveryPattern] = &[
        (&["deleted", "/usr/bin"], "accidentally deleted /usr/bin",
            &["echo 'EMERGENCY: Boot from Arch ISO, mount your root partition'",
              "echo 'Then run: pacman -Qk | grep missing | pacman -S $(awk \"{print $1}\")'",
              "echo 'Or reinstall all packages: pacman -Qqn | pacman -S -'"]),
        (&["deleted", "/usr"], "accidentally deleted /usr directory",
            &["echo 'EMERGENCY: Boot from Arch ISO, mount partitions'",
              "echo 'Reinstall base system: pacstrap /mnt base linux linux-firmware'"]),
        (&["removed", "/usr"], "accidentally removed /usr directory",
            &["echo 'EMERGENCY: Boot from Arch ISO and reinstall'"]),
        (&["deleted", "/etc"], "accidentally deleted /etc",
            &["echo 'EMERGENCY: Boot from Arch ISO'",
              "echo 'Reinstall: pacstrap /mnt base linux linux-firmware'"]),
        (&["deleted", "/boot"], "accidentally deleted /boot",
            &["echo 'EMERGENCY: Boot from Arch ISO, mount partitions'",
              "echo 'Reinstall kernel: pacstrap /mnt linux linux-firmware'",
              "echo 'Regenerate initramfs: arch-chroot /mnt mkinitcpio -P'"]),
        (&["accidentally", "deleted"], "accidental file deletion recovery",
            &["echo 'For btrfs: check snapshots with: sudo btrfs subvolume list /'",
              "echo 'For ext4: consider testdisk or photorec for recovery'"]),
        (&["accidentally", "removed"], "accidental file removal recovery",
            &["echo 'Check snapshots or backups first'",
              "echo 'For specific package files: pacman -Qkk <package>'"]),
        (&["accidentally", "rm", "-rf"], "accidental recursive deletion",
            &["echo 'Stop immediately! More writes = less recovery chance'",
              "echo 'Boot live USB, use testdisk/photorec for data recovery'"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("recovery".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_boot_failure(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[RecoveryPattern] = &[
        (&["won't", "boot"], "system boot failure",
            &["echo 'Boot from Arch ISO, mount partitions, chroot'",
              "echo 'Check: journalctl -xb --root=/mnt'",
              "echo 'Try: arch-chroot /mnt mkinitcpio -P'"]),
        (&["can't", "boot"], "system boot failure",
            &["echo 'Boot from Arch ISO, mount root to /mnt'",
              "echo 'arch-chroot /mnt && mkinitcpio -P'"]),
        (&["not", "boot"], "system not booting",
            &["echo 'Boot from Arch ISO, check: lsblk && mount /dev/sdX /mnt'"]),
        (&["boot", "stuck"], "boot process stuck",
            &["echo 'Add kernel param: systemd.unit=multi-user.target'",
              "echo 'Or boot to rescue: systemd.unit=rescue.target'"]),
        (&["boot", "hang"], "boot process hanging",
            &["echo 'Add kernel param: nosplash debug'",
              "echo 'Check: journalctl -b after booting'"]),
        (&["grub", "rescue"], "GRUB rescue mode",
            &["echo 'Boot Arch ISO, mount, then: grub-install /dev/sdX'",
              "echo 'Then: grub-mkconfig -o /boot/grub/grub.cfg'"]),
        (&["grub", "error"], "GRUB error",
            &["echo 'Boot Arch ISO, chroot, reinstall GRUB'",
              "echo 'grub-install --target=x86_64-efi --efi-directory=/boot'"]),
        (&["kernel", "panic"], "kernel panic",
            &["echo 'Boot older kernel from bootloader menu'",
              "echo 'Then: sudo pacman -S linux linux-headers'"]),
        (&["initramfs", "error"], "initramfs/mkinitcpio error",
            &["echo 'Boot Arch ISO, chroot: arch-chroot /mnt'",
              "echo 'Regenerate: mkinitcpio -P'"]),
        (&["mkinitcpio", "error"], "mkinitcpio error",
            &["echo 'Check hooks in /etc/mkinitcpio.conf'",
              "echo 'Reinstall: pacman -S linux'"]),
        (&["starting version"], "boot stuck at systemd version",
            &["echo 'Boot to rescue: systemd.unit=rescue.target'",
              "echo 'Check logs: journalctl -xb'"]),
        (&["black", "screen"], "black screen issue",
            &["echo 'Try Ctrl+Alt+F2 for TTY'",
              "echo 'Add kernel params: nomodeset or nvidia-drm.modeset=1'"]),
        (&["display", "manager", "won't"], "display manager failure",
            &["echo 'Switch to TTY: Ctrl+Alt+F2'",
              "echo 'Check: sudo systemctl status gdm'"]),
        (&["gdm", "not", "start"], "GDM not starting",
            &["systemctl status gdm", "journalctl -u gdm -b",
              "echo 'Try: sudo systemctl restart gdm'"]),
        (&["sddm", "not", "start"], "SDDM not starting",
            &["systemctl status sddm", "journalctl -u sddm -b",
              "echo 'Try: sudo systemctl restart sddm'"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("boot".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_rescue_mode(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[RecoveryPattern] = &[
        // Single user mode
        (&["single", "user", "mode"], "enter single user mode",
            &["echo 'At boot menu, add to kernel line: single'",
              "echo 'Or: systemd.unit=rescue.target'",
              "echo 'Or: init=/bin/bash (emergency)'"]),
        (&["enter", "single", "user"], "how to enter single user mode",
            &["echo 'Edit GRUB entry, add: single (or: 1)'",
              "echo 'Or: systemd.unit=rescue.target'"]),
        // Rescue mode
        (&["rescue", "mode"], "boot into rescue mode",
            &["echo 'At boot menu, add kernel param: systemd.unit=rescue.target'",
              "echo 'Or press e in GRUB, add to linux line, then Ctrl+X'"]),
        (&["boot", "rescue"], "boot to rescue mode",
            &["echo 'Add kernel param: systemd.unit=rescue.target'",
              "echo 'Or: systemd.unit=emergency.target for minimal boot'"]),
        (&["emergency", "mode"], "boot into emergency mode",
            &["echo 'Add kernel param: systemd.unit=emergency.target'",
              "echo 'Root filesystem mounted read-only'"]),
        // Failed update recovery
        (&["failed", "update"], "recover from failed update",
            &["echo 'Boot Arch ISO, mount root: mount /dev/sdX /mnt'",
              "echo 'Mount boot: mount /dev/sdY /mnt/boot'",
              "echo 'Chroot: arch-chroot /mnt'",
              "echo 'Fix: pacman -Syu or pacman -S <broken-packages>'"]),
        (&["recover", "failed", "update"], "recover from failed system update",
            &["echo '1. Boot from Arch ISO'",
              "echo '2. Mount: mount /dev/sdX /mnt && arch-chroot /mnt'",
              "echo '3. Fix: pacman -Syu --overwrite \"*\"'"]),
        (&["update", "broke"], "system update broke something",
            &["echo 'Try downgrade: pacman -U /var/cache/pacman/pkg/package-OLD.pkg.tar.zst'",
              "echo 'Or boot Arch ISO and chroot to fix'"]),
        // Chroot
        (&["chroot", "broken"], "chroot into broken system",
            &["echo 'Boot Arch ISO, then:'",
              "echo 'mount /dev/sdX /mnt   # your root partition'",
              "echo 'mount /dev/sdY /mnt/boot   # if separate boot'",
              "echo 'arch-chroot /mnt'"]),
        (&["arch-chroot"], "how to use arch-chroot",
            &["echo 'mount /dev/sdX /mnt'",
              "echo 'For UEFI: mount /dev/sdY /mnt/boot'",
              "echo 'arch-chroot /mnt'",
              "echo 'Now run commands in broken system'"]),
        (&["how", "chroot"], "how to chroot into system",
            &["echo '1. Boot from live USB/ISO'",
              "echo '2. mount /dev/root_partition /mnt'",
              "echo '3. mount /dev/boot_partition /mnt/boot (if separate)'",
              "echo '4. arch-chroot /mnt (or: mount --bind /dev /mnt/dev && chroot /mnt)'"]),
        // Bootloader reinstall
        (&["reinstall", "bootloader"], "reinstall bootloader",
            &["echo 'Boot Arch ISO, chroot, then:'",
              "echo 'For UEFI: grub-install --target=x86_64-efi --efi-directory=/boot'",
              "echo 'For BIOS: grub-install /dev/sdX'",
              "echo 'Then: grub-mkconfig -o /boot/grub/grub.cfg'"]),
        (&["reinstall", "grub"], "reinstall GRUB bootloader",
            &["echo 'UEFI: grub-install --target=x86_64-efi --efi-directory=/boot'",
              "echo 'BIOS: grub-install /dev/sdX'",
              "echo 'Regen config: grub-mkconfig -o /boot/grub/grub.cfg'"]),
        (&["fix", "grub"], "fix GRUB bootloader",
            &["echo 'Boot Arch ISO, mount partitions, chroot'",
              "echo 'grub-install [--target=x86_64-efi --efi-directory=/boot] /dev/sdX'",
              "echo 'grub-mkconfig -o /boot/grub/grub.cfg'"]),
        // Fstab issues
        (&["fstab", "mistake"], "fix fstab mistake",
            &["echo 'Boot with: systemd.unit=emergency.target'",
              "echo 'mount -o remount,rw /'",
              "echo 'nano /etc/fstab'",
              "echo 'Or boot Arch ISO and edit /mnt/etc/fstab'"]),
        (&["fix", "fstab"], "fix broken fstab",
            &["echo 'If boot fails: add systemd.unit=emergency.target'",
              "echo 'Remount: mount -o remount,rw /'",
              "echo 'Edit: nano /etc/fstab'"]),
        (&["broken", "fstab"], "broken fstab preventing boot",
            &["echo 'Boot Arch ISO, mount root: mount /dev/sdX /mnt'",
              "echo 'Edit: nano /mnt/etc/fstab'",
              "echo 'Use lsblk -f to find correct UUIDs'"]),
        (&["wrong", "uuid", "fstab"], "wrong UUID in fstab",
            &["echo 'Boot emergency: systemd.unit=emergency.target'",
              "echo 'Find correct UUID: blkid'",
              "echo 'Edit fstab: nano /etc/fstab'"]),
        // File recovery
        (&["recover", "deleted"], "recover deleted files",
            &["echo 'For btrfs: btrfs subvolume list / (check snapshots)'",
              "echo 'For ext4: sudo testdisk /dev/sdX'",
              "echo 'Or: sudo photorec /dev/sdX'"]),
        (&["undelete", "file"], "undelete files",
            &["echo 'Stop using the disk immediately!'",
              "echo 'Use testdisk or photorec for recovery'",
              "echo 'sudo testdisk /dev/sdX'"]),
        (&["file", "recovery"], "file recovery tools",
            &["echo 'testdisk - partition/file recovery'",
              "echo 'photorec - file carving recovery'",
              "echo 'extundelete - ext3/ext4 recovery'",
              "which testdisk photorec extundelete 2>/dev/null || echo 'Install: pacman -S testdisk'"]),
        // Initramfs
        (&["broken", "initramfs"], "fix broken initramfs",
            &["echo 'Boot Arch ISO, mount and chroot'",
              "echo 'Regenerate: mkinitcpio -P'",
              "echo 'Or for specific preset: mkinitcpio -p linux'"]),
        (&["fix", "initramfs"], "fix initramfs issues",
            &["echo 'arch-chroot /mnt'",
              "echo 'mkinitcpio -P'",
              "echo 'Check /etc/mkinitcpio.conf for issues'"]),
        (&["regenerate", "initramfs"], "regenerate initramfs",
            &["echo 'Run: sudo mkinitcpio -P (all presets)'",
              "echo 'Or: sudo mkinitcpio -p linux (specific kernel)'"]),
        // Package manager recovery
        (&["pacman", "broken"], "pacman database broken",
            &["echo 'Try: sudo rm /var/lib/pacman/db.lck'",
              "echo 'Rebuild: sudo pacman -Syy'",
              "echo 'Fix corrupted: sudo pacman -Qk | grep warning'"]),
        (&["corrupt", "package"], "corrupted package recovery",
            &["echo 'Force reinstall: sudo pacman -S package --overwrite \"*\"'",
              "echo 'Verify all: pacman -Qkk | grep warning'"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("recovery".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_permission_disaster(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[RecoveryPattern] = &[
        (&["chmod", "777", "-r"], "recursive chmod 777 recovery",
            &["echo 'DANGER: Files are now world-writable'",
              "echo 'Restore from backup or reinstall affected packages'",
              "echo 'For /usr: sudo pacman -S $(pacman -Qqn)'"]),
        (&["chmod", "777", "recursive"], "recursive chmod 777 recovery",
            &["echo 'Reset to sane defaults: find /path -type d -exec chmod 755 {} \\;'",
              "echo 'Then: find /path -type f -exec chmod 644 {} \\;'"]),
        (&["chmod", "-r", "/"], "recursive chmod on root",
            &["echo 'EMERGENCY: Boot Arch ISO, reinstall all packages'",
              "echo 'pacstrap /mnt base linux linux-firmware'"]),
        (&["chown", "-r", "/"], "recursive chown on root",
            &["echo 'EMERGENCY: System ownership corrupted'",
              "echo 'Boot Arch ISO, reinstall packages to restore ownership'"]),
        (&["permission", "denied", "everywhere"], "widespread permission issues",
            &["echo 'Check: ls -la /usr/bin/sudo'",
              "echo 'May need: chmod 4755 /usr/bin/sudo'"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("recovery".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Emergency patterns with topic information
type EmergencyPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_emergency(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[EmergencyPattern] = &[
        // Password issues
        (&["forgot", "password"], "forgotten password recovery", "security",
            &["echo 'Boot Arch ISO, mount root, chroot'",
              "echo 'Then: passwd <username>'"]),
        (&["forgot", "root", "password"], "forgotten root password", "security",
            &["echo 'Add init=/bin/bash to kernel params'",
              "echo 'Then: mount -o remount,rw / && passwd'"]),
        (&["reset", "password"], "password reset", "security",
            &["echo 'For current user: passwd'",
              "echo 'For other user: sudo passwd <username>'"]),
        // Disk full
        (&["disk", "full", "can't", "login"], "disk full preventing login", "storage",
            &["echo 'Boot single user: add single to kernel params'",
              "echo 'Clear logs: journalctl --vacuum-size=100M'"]),
        (&["filled", "disk"], "disk completely full", "storage",
            &["df -h", "du -sh /* 2>/dev/null | sort -hr | head -10",
              "echo 'Clear: paccache -rk1'"]),
        (&["no", "space", "left"], "no disk space left", "storage",
            &["df -h", "sudo journalctl --vacuum-size=100M",
              "du -sh /var/cache/pacman/pkg"]),
        // System freeze
        (&["freeze", "complete"], "complete system freeze", "hardware",
            &["echo 'Try SysRq: Alt+SysRq+R+E+I+S+U+B'",
              "echo 'Enable: echo 1 | sudo tee /proc/sys/kernel/sysrq'"]),
        (&["sysrq", "not", "work"], "system unresponsive to SysRq", "hardware",
            &["echo 'Hard power off may be only option'",
              "echo 'After: check journalctl -b -1'"]),
        (&["system", "frozen"], "frozen system", "hardware",
            &["echo 'Try SysRq REISUB: Alt+SysRq+R,E,I,S,U,B'",
              "echo 'Check after: journalctl -b -1 for cause'"]),
        // Can't login
        (&["can't", "login"], "unable to login", "security",
            &["echo 'Try TTY: Ctrl+Alt+F2'",
              "echo 'Check: faillock --user <username>'"]),
        (&["login", "loop"], "login loop", "display",
            &["echo 'Switch to TTY: Ctrl+Alt+F2'",
              "echo 'Check: ~/.Xauthority permissions'",
              "ls -la ~/.Xauthority"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

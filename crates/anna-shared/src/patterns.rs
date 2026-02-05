//! Common Error Pattern Recognition - Instant answers for well-known problems.
//!
//! v0.3.123: Instead of going through the full LLM loop for problems with known
//! solutions, recognize patterns and provide instant, high-confidence answers.

use serde::{Deserialize, Serialize};

/// A pattern match result with suggested fix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Pattern that matched
    pub pattern_id: String,
    /// Confidence in the match (0.0-1.0)
    pub confidence: f32,
    /// Recommended answer/solution
    pub answer: String,
    /// Commands that would fix this
    pub commands: Vec<String>,
    /// Whether this is auto-fixable
    pub auto_fixable: bool,
}

/// Check if a question matches a well-known error pattern.
pub fn match_error_pattern(question: &str) -> Option<PatternMatch> {
    let q = question.to_lowercase();

    // Pacman database locked
    if (q.contains("pacman") || q.contains("yay")) && q.contains("database") && q.contains("lock") {
        return Some(PatternMatch {
            pattern_id: "pacman-db-lock".to_string(),
            confidence: 0.95,
            answer: "The pacman database is locked. This usually happens when another package \
                     operation was interrupted or is still running.\n\n\
                     First, check if pacman is running: `pgrep -a pacman`\n\
                     If nothing is running, remove the lock file:\n\
                     `sudo rm /var/lib/pacman/db.lck`".to_string(),
            commands: vec![
                "pgrep -a pacman".to_string(),
                "rm /var/lib/pacman/db.lck".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // No space left on device
    if q.contains("no space left") || (q.contains("disk") && q.contains("full")) {
        return Some(PatternMatch {
            pattern_id: "disk-full".to_string(),
            confidence: 0.9,
            answer: "Your disk is full. Here's how to free space:\n\n\
                     1. Clean package cache: `sudo paccache -rk1`\n\
                     2. Clean old journal logs: `sudo journalctl --vacuum-time=7d`\n\
                     3. Find large files: `du -sh /* 2>/dev/null | sort -hr | head -10`\n\
                     4. Remove orphan packages: `sudo pacman -Rns $(pacman -Qdtq)`".to_string(),
            commands: vec![
                "paccache -rk1".to_string(),
                "journalctl --vacuum-time=7d".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // Permission denied
    if q.contains("permission denied") && !q.contains("ssh") {
        return Some(PatternMatch {
            pattern_id: "permission-denied".to_string(),
            confidence: 0.8,
            answer: "Permission denied errors usually mean:\n\n\
                     1. **File ownership**: Check with `ls -la <file>`\n\
                     2. **Need sudo**: Try with `sudo`\n\
                     3. **SELinux/AppArmor**: Check `dmesg | tail`\n\n\
                     What file or operation is failing?".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // Service failed
    if q.contains("failed") && (q.contains("service") || q.contains("systemd") || q.contains("unit")) {
        return Some(PatternMatch {
            pattern_id: "service-failed".to_string(),
            confidence: 0.85,
            answer: "To diagnose a failed service:\n\n\
                     1. Check status: `systemctl status <service>`\n\
                     2. View logs: `journalctl -xeu <service>`\n\
                     3. Restart: `sudo systemctl restart <service>`\n\n\
                     What service is failing?".to_string(),
            commands: vec![
                "systemctl --failed".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // WiFi not working
    if q.contains("wifi") && (q.contains("not work") || q.contains("can't connect") || q.contains("no network")) {
        return Some(PatternMatch {
            pattern_id: "wifi-not-working".to_string(),
            confidence: 0.85,
            answer: "WiFi troubleshooting steps:\n\n\
                     1. Check if interface is up: `ip link show`\n\
                     2. Scan for networks: `nmcli device wifi list`\n\
                     3. Check NetworkManager: `systemctl status NetworkManager`\n\
                     4. Check rfkill: `rfkill list`\n\n\
                     If rfkill shows blocked, run: `sudo rfkill unblock wifi`".to_string(),
            commands: vec![
                "ip link show".to_string(),
                "nmcli device wifi list".to_string(),
                "rfkill list".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // No sound / audio not working
    if (q.contains("audio") || q.contains("sound")) && (q.contains("not work") || q.contains("no sound") || q.contains("can't hear")) {
        return Some(PatternMatch {
            pattern_id: "audio-not-working".to_string(),
            confidence: 0.85,
            answer: "Audio troubleshooting:\n\n\
                     1. Check if muted: `wpctl status` or `pactl info`\n\
                     2. Set default sink: `wpctl set-default <sink-id>`\n\
                     3. Check PipeWire: `systemctl --user status pipewire`\n\
                     4. Check ALSA: `aplay -l`\n\n\
                     Often unmuting fixes it: `wpctl set-mute @DEFAULT_SINK@ 0`".to_string(),
            commands: vec![
                "wpctl status".to_string(),
                "wpctl set-mute @DEFAULT_SINK@ 0".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // Kernel panic or system freeze
    if q.contains("freeze") || q.contains("kernel panic") || q.contains("system hang") {
        return Some(PatternMatch {
            pattern_id: "system-freeze".to_string(),
            confidence: 0.8,
            answer: "System freezes can be caused by:\n\n\
                     1. **Check logs**: `journalctl -b -1` (previous boot)\n\
                     2. **Hardware**: `dmesg | grep -i error`\n\
                     3. **GPU drivers**: `lspci -k | grep -A3 VGA`\n\
                     4. **Memory**: `free -h` and check for OOM in logs\n\n\
                     Common fixes:\n\
                     - Update kernel: `sudo pacman -Syu linux`\n\
                     - Update GPU drivers".to_string(),
            commands: vec![
                "journalctl -b -1 -p err".to_string(),
                "dmesg | grep -i error".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // Boot issues / GRUB
    if q.contains("won't boot") || q.contains("grub") && (q.contains("error") || q.contains("not found")) {
        return Some(PatternMatch {
            pattern_id: "boot-grub".to_string(),
            confidence: 0.8,
            answer: "Boot/GRUB issues:\n\n\
                     1. Boot from USB and chroot\n\
                     2. Reinstall GRUB: `grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB`\n\
                     3. Regenerate config: `grub-mkconfig -o /boot/grub/grub.cfg`\n\n\
                     If you can boot, just run:\n\
                     `sudo grub-mkconfig -o /boot/grub/grub.cfg`".to_string(),
            commands: vec![
                "grub-mkconfig -o /boot/grub/grub.cfg".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // Nvidia driver issues
    if q.contains("nvidia") && (q.contains("not work") || q.contains("black screen") || q.contains("driver")) {
        return Some(PatternMatch {
            pattern_id: "nvidia-driver".to_string(),
            confidence: 0.85,
            answer: "NVIDIA driver troubleshooting:\n\n\
                     1. Check loaded modules: `lsmod | grep nvidia`\n\
                     2. Check Xorg logs: `cat /var/log/Xorg.0.log | grep -i nvidia`\n\
                     3. Reinstall drivers: `sudo pacman -S nvidia nvidia-utils`\n\
                     4. Rebuild initramfs: `sudo mkinitcpio -P`\n\n\
                     After updates, always reboot.".to_string(),
            commands: vec![
                "lsmod | grep nvidia".to_string(),
                "nvidia-smi".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // SSH connection refused
    if q.contains("ssh") && (q.contains("refused") || q.contains("can't connect") || q.contains("connection timed out")) {
        return Some(PatternMatch {
            pattern_id: "ssh-refused".to_string(),
            confidence: 0.85,
            answer: "SSH connection troubleshooting:\n\n\
                     1. Check if sshd is running: `systemctl status sshd`\n\
                     2. Start if not: `sudo systemctl start sshd`\n\
                     3. Check firewall: `sudo iptables -L -n | grep 22`\n\
                     4. Check port: `ss -tlnp | grep 22`\n\n\
                     For remote host, verify IP and that port 22 is open.".to_string(),
            commands: vec![
                "systemctl status sshd".to_string(),
                "ss -tlnp | grep 22".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // Docker issues
    if q.contains("docker") && (q.contains("permission") || q.contains("socket") || q.contains("daemon")) {
        return Some(PatternMatch {
            pattern_id: "docker-permission".to_string(),
            confidence: 0.85,
            answer: "Docker permission/socket issues:\n\n\
                     1. Add user to docker group: `sudo usermod -aG docker $USER`\n\
                     2. Log out and back in (or `newgrp docker`)\n\
                     3. Start daemon: `sudo systemctl start docker`\n\
                     4. Enable on boot: `sudo systemctl enable docker`".to_string(),
            commands: vec![
                "usermod -aG docker $USER".to_string(),
                "systemctl start docker".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // DNS issues
    if q.contains("dns") || (q.contains("can't resolve") || q.contains("name resolution")) {
        return Some(PatternMatch {
            pattern_id: "dns-resolution".to_string(),
            confidence: 0.85,
            answer: "DNS resolution troubleshooting:\n\n\
                     1. Test DNS: `nslookup google.com`\n\
                     2. Check resolv.conf: `cat /etc/resolv.conf`\n\
                     3. Try different DNS: `echo 'nameserver 8.8.8.8' | sudo tee /etc/resolv.conf`\n\
                     4. Restart NetworkManager: `sudo systemctl restart NetworkManager`".to_string(),
            commands: vec![
                "cat /etc/resolv.conf".to_string(),
                "nslookup google.com 8.8.8.8".to_string(),
            ],
            auto_fixable: true,
        });
    }

    None
}

/// Format a pattern match as a complete answer.
pub fn format_pattern_answer(pm: &PatternMatch) -> String {
    let mut answer = pm.answer.clone();

    if pm.auto_fixable {
        answer.push_str("\n\n(I can help fix this automatically if you'd like.)");
    }

    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_lock_detection() {
        let result = match_error_pattern("pacman says database is locked");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "pacman-db-lock");
    }

    #[test]
    fn test_disk_full_detection() {
        let result = match_error_pattern("I'm getting no space left on device errors");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "disk-full");
    }

    #[test]
    fn test_wifi_detection() {
        let result = match_error_pattern("my wifi is not working");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "wifi-not-working");
    }

    #[test]
    fn test_no_match() {
        let result = match_error_pattern("what is the capital of France");
        assert!(result.is_none());
    }
}

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

    // v0.3.153: Common configuration patterns - vim syntax highlighting
    if (q.contains("vim") || q.contains("vi")) && (q.contains("syntax") || q.contains("highlight")) {
        return Some(PatternMatch {
            pattern_id: "vim-syntax-highlighting".to_string(),
            confidence: 0.95,
            answer: "To enable syntax highlighting in Vim, add this to your ~/.vimrc:\n\n\
                     `echo 'syntax on' >> ~/.vimrc`\n\n\
                     This will be enabled for all future Vim sessions.\n\
                     For the current session only, type `:syntax on` in Vim.".to_string(),
            commands: vec![
                "echo 'syntax on' >> ~/.vimrc".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // v0.3.153: Common configuration - vim line numbers
    if (q.contains("vim") || q.contains("vi")) && q.contains("line number") {
        return Some(PatternMatch {
            pattern_id: "vim-line-numbers".to_string(),
            confidence: 0.95,
            answer: "To enable line numbers in Vim, add this to your ~/.vimrc:\n\n\
                     `echo 'set number' >> ~/.vimrc`\n\n\
                     For relative line numbers (useful for motions), use:\n\
                     `echo 'set relativenumber' >> ~/.vimrc`".to_string(),
            commands: vec![
                "echo 'set number' >> ~/.vimrc".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // v0.3.153: Common configuration - bashrc aliases
    if q.contains("bash") && (q.contains("alias") || q.contains("shortcut")) {
        return Some(PatternMatch {
            pattern_id: "bash-alias".to_string(),
            confidence: 0.85,
            answer: "To add a bash alias, add it to your ~/.bashrc:\n\n\
                     Example: `echo \"alias ll='ls -lah'\" >> ~/.bashrc`\n\n\
                     Then reload: `source ~/.bashrc`\n\n\
                     What command do you want to create an alias for?".to_string(),
            commands: vec![
                "source ~/.bashrc".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // v0.3.154: Git initialization
    if q.contains("git") && (q.contains("init") || q.contains("initialize") || q.contains("start") || q.contains("setup"))
        && !q.contains("config") {
        return Some(PatternMatch {
            pattern_id: "git-init".to_string(),
            confidence: 0.9,
            answer: "To initialize a new Git repository:\n\n\
                     `git init`\n\n\
                     This creates a .git directory in the current folder.\n\
                     After init, you typically:\n\
                     1. `git add .` - Stage files\n\
                     2. `git commit -m \"Initial commit\"` - Create first commit\n\
                     3. `git remote add origin <url>` - Add remote (if needed)\n\
                     4. `git push -u origin main` - Push to remote".to_string(),
            commands: vec![
                "git init".to_string(),
            ],
            auto_fixable: true,
        });
    }

    // v0.3.154: SSH key generation
    if q.contains("ssh") && (q.contains("key") || q.contains("keygen")) && (q.contains("generate") || q.contains("create") || q.contains("setup")) {
        return Some(PatternMatch {
            pattern_id: "ssh-keygen".to_string(),
            confidence: 0.95,
            answer: "To generate an SSH key pair:\n\n\
                     `ssh-keygen -t ed25519 -C \"your_email@example.com\"`\n\n\
                     Press Enter to accept default location (~/.ssh/id_ed25519)\n\
                     Optionally set a passphrase for extra security.\n\n\
                     To copy public key to clipboard:\n\
                     `cat ~/.ssh/id_ed25519.pub`\n\n\
                     Add this to GitHub/GitLab under Settings > SSH Keys.".to_string(),
            commands: vec![
                "ssh-keygen -t ed25519".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // v0.3.154: File permissions - chmod
    if q.contains("chmod") || (q.contains("permission") && (q.contains("755") || q.contains("644") || q.contains("executable"))) {
        return Some(PatternMatch {
            pattern_id: "chmod-permissions".to_string(),
            confidence: 0.85,
            answer: "Common file permission patterns:\n\n\
                     **Executable script**: `chmod +x script.sh` or `chmod 755 script.sh`\n\
                     **Regular file**: `chmod 644 file.txt` (owner read/write, others read)\n\
                     **Private file**: `chmod 600 file.txt` (owner only)\n\
                     **Directory**: `chmod 755 directory/` (owner all, others read/execute)\n\n\
                     Format: chmod [user][group][others] where:\n\
                     - 4 = read, 2 = write, 1 = execute\n\
                     - Example: 755 = owner(7=4+2+1), group(5=4+1), others(5=4+1)".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // v0.3.154: Systemd service control
    if q.contains("systemctl") || (q.contains("service") && (q.contains("start") || q.contains("stop") || q.contains("enable") || q.contains("restart"))) {
        return Some(PatternMatch {
            pattern_id: "systemctl-service".to_string(),
            confidence: 0.9,
            answer: "Common systemd service commands:\n\n\
                     **Start service**: `sudo systemctl start <service>`\n\
                     **Stop service**: `sudo systemctl stop <service>`\n\
                     **Restart service**: `sudo systemctl restart <service>`\n\
                     **Enable on boot**: `sudo systemctl enable <service>`\n\
                     **Disable on boot**: `sudo systemctl disable <service>`\n\
                     **Check status**: `systemctl status <service>`\n\
                     **View logs**: `journalctl -xeu <service>`\n\n\
                     What service are you working with?".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // v0.3.154: Finding files
    if (q.contains("find") && q.contains("file")) || q.contains("locate") {
        return Some(PatternMatch {
            pattern_id: "find-files".to_string(),
            confidence: 0.85,
            answer: "To find files:\n\n\
                     **By name**: `find /path -name \"filename\"`\n\
                     **By pattern**: `find /path -name \"*.txt\"`\n\
                     **By type**: `find /path -type f` (files) or `-type d` (directories)\n\
                     **Modified recently**: `find /path -mtime -7` (last 7 days)\n\
                     **By size**: `find /path -size +100M` (larger than 100MB)\n\n\
                     **Fast alternative** (if locate DB exists):\n\
                     `locate filename`\n\
                     Update DB with: `sudo updatedb`".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // v0.3.154: Grep/search in files
    if (q.contains("grep") || q.contains("search")) && (q.contains("file") || q.contains("text") || q.contains("content")) {
        return Some(PatternMatch {
            pattern_id: "grep-search".to_string(),
            confidence: 0.85,
            answer: "To search text in files:\n\n\
                     **Basic search**: `grep \"pattern\" file.txt`\n\
                     **Recursive search**: `grep -r \"pattern\" /path`\n\
                     **Case insensitive**: `grep -i \"pattern\" file.txt`\n\
                     **Show line numbers**: `grep -n \"pattern\" file.txt`\n\
                     **Show context**: `grep -C 3 \"pattern\" file.txt` (3 lines before/after)\n\n\
                     **Fast alternative** (ripgrep):\n\
                     `rg \"pattern\"` - searches current directory recursively".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // v0.3.154: Tar archive operations
    if q.contains("tar") && (q.contains("extract") || q.contains("compress") || q.contains("archive")) {
        return Some(PatternMatch {
            pattern_id: "tar-operations".to_string(),
            confidence: 0.9,
            answer: "Common tar operations:\n\n\
                     **Extract .tar.gz**: `tar -xzf file.tar.gz`\n\
                     **Extract .tar.xz**: `tar -xJf file.tar.xz`\n\
                     **Create .tar.gz**: `tar -czf archive.tar.gz /path/to/files`\n\
                     **List contents**: `tar -tzf file.tar.gz`\n\n\
                     Flags: -x (extract), -c (create), -z (gzip), -J (xz), -f (file), -v (verbose)".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // v0.3.155: Network troubleshooting
    if (q.contains("network") || q.contains("internet") || q.contains("connection"))
        && (q.contains("troubleshoot") || q.contains("debug") || q.contains("test") || q.contains("check")) {
        return Some(PatternMatch {
            pattern_id: "network-troubleshooting".to_string(),
            confidence: 0.85,
            answer: "Network troubleshooting steps:\n\n\
                     **Test connectivity**: `ping 8.8.8.8` (Google DNS)\n\
                     **Test DNS**: `ping google.com`\n\
                     **Trace route**: `traceroute google.com`\n\
                     **Check interfaces**: `ip addr show` or `ip link show`\n\
                     **Check routes**: `ip route show`\n\
                     **Test ports**: `nc -zv hostname port` or `telnet hostname port`\n\
                     **Check listening ports**: `ss -tulpn` or `netstat -tulpn`\n\n\
                     **DNS troubleshooting**: `nslookup google.com` or `dig google.com`".to_string(),
            commands: vec![
                "ping -c 4 8.8.8.8".to_string(),
                "ip addr show".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // v0.3.155: Disk usage analysis
    if (q.contains("disk") && (q.contains("usage") || q.contains("space") || q.contains("full")))
        || q.contains("du ") || q.contains("df ") {
        return Some(PatternMatch {
            pattern_id: "disk-usage".to_string(),
            confidence: 0.9,
            answer: "Disk usage analysis:\n\n\
                     **Overall disk usage**: `df -h` (human-readable)\n\
                     **Directory sizes**: `du -sh *` (current directory)\n\
                     **Largest directories**: `du -h --max-depth=1 / 2>/dev/null | sort -hr | head -10`\n\
                     **Largest files**: `find / -type f -size +100M 2>/dev/null`\n\
                     **Disk usage by directory**: `ncdu /` (interactive, install if needed)\n\n\
                     **Free up space**:\n\
                     - Clean package cache: `sudo paccache -rk1`\n\
                     - Clean journal logs: `sudo journalctl --vacuum-time=7d`\n\
                     - Remove orphan packages: `sudo pacman -Rns $(pacman -Qdtq)`".to_string(),
            commands: vec![
                "df -h".to_string(),
                "du -sh /* 2>/dev/null | sort -hr | head -10".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // v0.3.155: Process management
    if (q.contains("process") || q.contains("kill") || q.contains("htop") || q.contains("ps "))
        && !q.contains("processor") {
        return Some(PatternMatch {
            pattern_id: "process-management".to_string(),
            confidence: 0.85,
            answer: "Process management commands:\n\n\
                     **List processes**: `ps aux` or `ps -ef`\n\
                     **Interactive monitor**: `htop` or `top`\n\
                     **Find process**: `pgrep -a firefox` or `ps aux | grep firefox`\n\
                     **Kill process**: `kill <PID>` or `killall firefox`\n\
                     **Force kill**: `kill -9 <PID>` or `killall -9 firefox`\n\
                     **Process tree**: `pstree -p`\n\n\
                     **Resource usage**:\n\
                     - CPU usage: `top -o %CPU`\n\
                     - Memory usage: `top -o %MEM`\n\
                     - By user: `ps -u username`".to_string(),
            commands: vec![
                "ps aux".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // v0.3.155: Log viewing
    if q.contains("log") && (q.contains("view") || q.contains("read") || q.contains("check") || q.contains("journalctl")) {
        return Some(PatternMatch {
            pattern_id: "log-viewing".to_string(),
            confidence: 0.9,
            answer: "Viewing system logs:\n\n\
                     **System logs** (systemd/journalctl):\n\
                     - Recent logs: `journalctl -xe`\n\
                     - Follow logs: `journalctl -f`\n\
                     - Service logs: `journalctl -xeu <service>`\n\
                     - Boot logs: `journalctl -b` (current boot) or `journalctl -b -1` (previous)\n\
                     - Time range: `journalctl --since \"1 hour ago\"`\n\
                     - Priority: `journalctl -p err` (errors only)\n\n\
                     **Traditional logs** (/var/log):\n\
                     - View file: `less /var/log/syslog`\n\
                     - Follow file: `tail -f /var/log/syslog`\n\
                     - Last N lines: `tail -n 50 /var/log/syslog`".to_string(),
            commands: vec![
                "journalctl -xe".to_string(),
            ],
            auto_fixable: false,
        });
    }

    // v0.3.155: User/group management
    if (q.contains("user") || q.contains("group"))
        && (q.contains("add") || q.contains("create") || q.contains("delete") || q.contains("modify")) {
        return Some(PatternMatch {
            pattern_id: "user-management".to_string(),
            confidence: 0.85,
            answer: "User and group management:\n\n\
                     **User operations**:\n\
                     - Add user: `sudo useradd -m username`\n\
                     - Set password: `sudo passwd username`\n\
                     - Delete user: `sudo userdel username` or `sudo userdel -r username` (with home)\n\
                     - Modify user: `sudo usermod -aG groupname username` (add to group)\n\
                     - List users: `cat /etc/passwd` or `getent passwd`\n\n\
                     **Group operations**:\n\
                     - Add group: `sudo groupadd groupname`\n\
                     - Delete group: `sudo groupdel groupname`\n\
                     - List groups: `cat /etc/group` or `getent group`\n\
                     - User groups: `groups username`".to_string(),
            commands: vec![],
            auto_fixable: false,
        });
    }

    // v0.3.155: Package management (Arch/pacman)
    if (q.contains("package") || q.contains("pacman") || q.contains("yay"))
        && (q.contains("install") || q.contains("remove") || q.contains("search") || q.contains("update")) {
        return Some(PatternMatch {
            pattern_id: "pacman-usage".to_string(),
            confidence: 0.9,
            answer: "Pacman package management:\n\n\
                     **Install packages**: `sudo pacman -S package`\n\
                     **Remove package**: `sudo pacman -R package` or `sudo pacman -Rns package` (with deps)\n\
                     **Update system**: `sudo pacman -Syu`\n\
                     **Search packages**: `pacman -Ss keyword` or `yay -Ss keyword` (AUR)\n\
                     **Query installed**: `pacman -Q` or `pacman -Qi package` (info)\n\
                     **Clean cache**: `sudo paccache -rk1` (keep 1 version)\n\
                     **List files**: `pacman -Ql package`\n\n\
                     **AUR helper** (yay):\n\
                     - Install AUR: `yay -S package`\n\
                     - Update all: `yay -Syu`".to_string(),
            commands: vec![],
            auto_fixable: false,
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

    #[test]
    fn test_vim_syntax_detection() {
        let result = match_error_pattern("enable syntax highlighting on vim");
        assert!(result.is_some());
        let pm = result.unwrap();
        assert_eq!(pm.pattern_id, "vim-syntax-highlighting");
        assert!(pm.confidence > 0.9);
        assert!(pm.auto_fixable);
    }

    #[test]
    fn test_vim_line_numbers() {
        let result = match_error_pattern("how do I show line numbers in vim");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "vim-line-numbers");
    }

    #[test]
    fn test_bash_alias() {
        let result = match_error_pattern("how to create a bash alias");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "bash-alias");
    }

    #[test]
    fn test_git_init() {
        let result = match_error_pattern("how do I initialize a git repository");
        assert!(result.is_some());
        let pm = result.unwrap();
        assert_eq!(pm.pattern_id, "git-init");
        assert!(pm.confidence >= 0.9);
        assert!(pm.auto_fixable);
    }

    #[test]
    fn test_ssh_keygen() {
        let result = match_error_pattern("how to generate SSH keys");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "ssh-keygen");
    }

    #[test]
    fn test_chmod_permissions() {
        let result = match_error_pattern("chmod 755 permissions");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "chmod-permissions");
    }

    #[test]
    fn test_systemctl_service() {
        let result = match_error_pattern("how to start a service with systemctl");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "systemctl-service");
    }

    #[test]
    fn test_find_files() {
        let result = match_error_pattern("how to find files in Linux");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "find-files");
    }

    #[test]
    fn test_grep_search() {
        let result = match_error_pattern("how to search text in files using grep");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "grep-search");
    }

    #[test]
    fn test_tar_operations() {
        let result = match_error_pattern("how to extract tar.gz archive");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "tar-operations");
    }

    #[test]
    fn test_network_troubleshooting() {
        let result = match_error_pattern("how to troubleshoot network connection");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "network-troubleshooting");
    }

    #[test]
    fn test_disk_usage() {
        let result = match_error_pattern("check disk usage and space");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "disk-usage");
    }

    #[test]
    fn test_process_management() {
        let result = match_error_pattern("how to kill a process");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "process-management");
    }

    #[test]
    fn test_log_viewing() {
        let result = match_error_pattern("how to view system logs with journalctl");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "log-viewing");
    }

    #[test]
    fn test_user_management() {
        let result = match_error_pattern("how to add a user");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "user-management");
    }

    #[test]
    fn test_pacman_usage() {
        let result = match_error_pattern("how to install package with pacman");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern_id, "pacman-usage");
    }
}

//! HowTo patterns - common task instructions with known commands
//! v0.0.947: Initial howto patterns for common Linux tasks
//! v0.0.989: Added network, storage, security setup patterns
//! v0.1.0: Use word boundary matching to prevent false positives

mod packages;

use anna_shared::rpc::DeepUnderstanding;
use super::contains_word;

// Re-export helper for submodules
pub(crate) use packages::{HowToPattern, make_understanding};

// Re-export match_patterns for parent module
pub use packages::{match_package_tasks, match_service_tasks, match_user_tasks};

/// Match common "how to" questions
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Package management
    if let Some(u) = match_package_tasks(q) {
        return Some(u);
    }
    // Service management
    if let Some(u) = match_service_tasks(q) {
        return Some(u);
    }
    // User management
    if let Some(u) = match_user_tasks(q) {
        return Some(u);
    }
    // File/permission tasks
    if let Some(u) = match_file_tasks(q) {
        return Some(u);
    }
    // System configuration
    if let Some(u) = match_system_tasks(q) {
        return Some(u);
    }
    // Network setup
    if let Some(u) = match_network_tasks(q) {
        return Some(u);
    }
    // Storage tasks
    if let Some(u) = match_storage_tasks(q) {
        return Some(u);
    }
    None
}

/// File and permission tasks
fn match_file_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Change permissions (both singular and plural)
        (&["how", "change", "permissions"], "change file permissions", "files",
            &["chmod 755 <file>", "chmod +x <file>", "chmod -R 755 <directory>"]),
        (&["how", "change", "permission"], "change file permissions", "files",
            &["chmod 755 <file>", "chmod +x <file>", "chmod -R 755 <directory>"]),
        (&["make", "file", "executable"], "make file executable", "files",
            &["chmod +x <file>"]),
        (&["make", "script", "executable"], "make script executable", "files",
            &["chmod +x <script>"]),
        // Change owner
        (&["how", "change", "owner"], "change file owner", "files",
            &["sudo chown <user>:<group> <file>", "sudo chown -R <user>:<group> <directory>"]),
        (&["change", "file", "owner"], "change file ownership", "files",
            &["sudo chown <user> <file>"]),
        // Find files
        (&["how", "find", "file"], "find a file", "files",
            &["find / -name '<filename>' 2>/dev/null", "locate <filename>"]),
        (&["search", "for", "file"], "search for a file", "files",
            &["find . -name '<pattern>'", "locate <filename>"]),
        // Create directory
        (&["how", "create", "directory"], "create a directory", "files",
            &["mkdir <dirname>", "mkdir -p <path/to/dir>"]),
        (&["how", "make", "folder"], "make a folder", "files",
            &["mkdir <dirname>", "mkdir -p <path>"]),
        // Copy/Move
        (&["how", "copy", "file"], "copy a file", "files",
            &["cp <source> <dest>", "cp -r <source> <dest>"]),
        (&["how", "move", "file"], "move a file", "files",
            &["mv <source> <dest>"]),
        (&["how", "rename", "file"], "rename a file", "files",
            &["mv <oldname> <newname>"]),
        // Create symlink
        (&["how", "create", "symlink"], "create a symlink", "files",
            &["ln -s <target> <linkname>"]),
        (&["create", "symbolic", "link"], "create symbolic link", "files",
            &["ln -s <target> <linkname>"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// System configuration tasks
fn match_system_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Hostname
        (&["how", "change", "hostname"], "change hostname", "system",
            &["sudo hostnamectl set-hostname <newhostname>"]),
        (&["set", "hostname"], "set hostname", "system",
            &["sudo hostnamectl set-hostname <newhostname>"]),
        // Timezone
        (&["how", "set", "timezone"], "set timezone", "system",
            &["sudo timedatectl set-timezone <Zone/City>", "timedatectl list-timezones | grep <region>"]),
        (&["change", "timezone"], "change timezone", "system",
            &["sudo timedatectl set-timezone <Zone/City>"]),
        // Locale
        (&["how", "set", "locale"], "set locale", "system",
            &["sudo localectl set-locale LANG=<locale>", "localectl list-locales | grep <lang>"]),
        (&["change", "language"], "change system language", "system",
            &["sudo localectl set-locale LANG=<locale>"]),
        // Keyboard
        (&["how", "change", "keyboard"], "change keyboard layout", "system",
            &["sudo localectl set-keymap <layout>", "localectl list-keymaps | grep <layout>"]),
        (&["set", "keyboard", "layout"], "set keyboard layout", "system",
            &["sudo localectl set-keymap <layout>"]),
        // Reboot/Shutdown
        (&["how", "reboot"], "reboot the system", "system",
            &["sudo reboot", "systemctl reboot"]),
        (&["how", "shutdown"], "shutdown the system", "system",
            &["sudo shutdown now", "systemctl poweroff"]),
        (&["schedule", "shutdown"], "schedule shutdown", "system",
            &["sudo shutdown +30", "sudo shutdown 22:00"]),
        // Mount
        (&["how", "mount", "drive"], "mount a drive", "storage",
            &["sudo mount /dev/sdX /mnt", "lsblk -f"]),
        (&["mount", "usb"], "mount USB drive", "storage",
            &["lsblk", "sudo mount /dev/sdX1 /mnt/usb"]),
        (&["unmount", "drive"], "unmount a drive", "storage",
            &["sudo umount /mnt", "sudo umount /dev/sdX1"]),
        // Firewall
        (&["how", "open", "port"], "open a firewall port", "network",
            &["sudo ufw allow <port>", "sudo firewall-cmd --add-port=<port>/tcp --permanent"]),
        (&["allow", "port", "firewall"], "allow port in firewall", "network",
            &["sudo ufw allow <port>/tcp"]),
        (&["how", "enable", "firewall"], "enable firewall", "network",
            &["sudo ufw enable", "sudo systemctl enable --now firewalld"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Network setup tasks
fn match_network_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // SSH setup
        (&["set", "up", "ssh"], "set up SSH server", "network",
            &["sudo pacman -S openssh", "sudo systemctl enable --now sshd"]),
        (&["how", "ssh"], "how to configure SSH", "network",
            &["sudo systemctl status sshd", "cat /etc/ssh/sshd_config | grep -v '^#' | grep -v '^$'"]),
        // Network configuration
        (&["configure", "network"], "configure network", "network",
            &["nmcli device status", "nmcli connection show", "ip addr"]),
        (&["how", "network"], "network configuration how-to", "network",
            &["nmcli device wifi list", "nmcli connection add type ethernet con-name <name>"]),
        // Static IP
        (&["set", "static", "ip"], "set static IP address", "network",
            &["echo 'nmcli con mod <connection> ipv4.addresses <ip>/24'",
              "echo 'nmcli con mod <connection> ipv4.gateway <gateway>'",
              "echo 'nmcli con mod <connection> ipv4.method manual'"]),
        (&["static", "ip"], "configure static IP", "network",
            &["nmcli connection show", "echo 'nmcli con mod <name> ipv4.addresses x.x.x.x/24 ipv4.method manual'"]),
        // DNS configuration
        (&["configure", "dns"], "configure DNS", "network",
            &["cat /etc/resolv.conf", "echo 'nmcli con mod <conn> ipv4.dns \"8.8.8.8 8.8.4.4\"'"]),
        (&["how", "dns"], "how to set up DNS", "network",
            &["resolvectl status", "echo 'Add nameserver to /etc/resolv.conf or use nmcli'"]),
        // VNC setup
        (&["set", "up", "vnc"], "set up VNC server", "network",
            &["echo 'Install: sudo pacman -S tigervnc'",
              "echo 'Start: vncserver :1'",
              "echo 'Connect: vncviewer hostname:1'"]),
        // Samba setup
        (&["configure", "samba"], "configure Samba share", "network",
            &["echo 'Install: sudo pacman -S samba'",
              "echo 'Config: /etc/samba/smb.conf'",
              "echo 'Add user: sudo smbpasswd -a <user>'"]),
        (&["how", "samba"], "how to set up Samba", "network",
            &["pacman -Qs samba", "testparm 2>/dev/null || echo 'samba not configured'"]),
        // NFS setup
        (&["set", "up", "nfs"], "set up NFS share", "network",
            &["echo 'Install: sudo pacman -S nfs-utils'",
              "echo 'Export: add to /etc/exports'",
              "echo 'Start: sudo systemctl enable --now nfs-server'"]),
        (&["how", "nfs"], "how to configure NFS", "network",
            &["cat /etc/exports 2>/dev/null", "showmount -e localhost 2>/dev/null"]),
        // Auto login disable root
        (&["enable", "auto", "login"], "enable auto login", "system",
            &["echo 'For GDM: edit /etc/gdm/custom.conf'",
              "echo '[daemon]'",
              "echo 'AutomaticLogin=<username>'",
              "echo 'AutomaticLoginEnable=True'"]),
        (&["disable", "root", "login"], "disable root login via SSH", "security",
            &["echo 'Edit /etc/ssh/sshd_config:'",
              "echo 'PermitRootLogin no'",
              "echo 'Then: sudo systemctl restart sshd'"]),
        // Firewall configuration
        (&["configure", "firewall"], "configure firewall", "network",
            &["sudo ufw status", "echo 'Enable: sudo ufw enable'",
              "echo 'Allow: sudo ufw allow <port>/tcp'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Storage and disk tasks
fn match_storage_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Create partition
        (&["create", "partition"], "create a partition", "storage",
            &["echo 'Use: sudo fdisk /dev/sdX or sudo parted /dev/sdX'",
              "echo 'Or GUI: sudo gparted'",
              "lsblk"]),
        (&["how", "partition"], "how to partition a disk", "storage",
            &["lsblk", "echo 'fdisk /dev/sdX (MBR) or gdisk /dev/sdX (GPT)'"]),
        // Resize partition
        (&["resize", "partition"], "resize a partition", "storage",
            &["echo 'GUI: gparted (unmount first)'",
              "echo 'CLI: parted /dev/sdX resizepart <num> <size>'",
              "echo 'Then resize fs: resize2fs /dev/sdX1'"]),
        (&["how", "resize"], "how to resize partition", "storage",
            &["echo '1. Unmount partition'",
              "echo '2. Use parted or gparted to resize'",
              "echo '3. Resize filesystem: resize2fs /dev/sdX1'"]),
        // Encrypt disk
        (&["encrypt", "disk"], "encrypt a disk", "storage",
            &["echo 'LUKS: cryptsetup luksFormat /dev/sdX'",
              "echo 'Open: cryptsetup open /dev/sdX cryptdisk'",
              "echo 'Format: mkfs.ext4 /dev/mapper/cryptdisk'"]),
        (&["how", "encrypt"], "how to encrypt disk", "storage",
            &["echo 'Use LUKS: sudo cryptsetup luksFormat /dev/sdX'",
              "pacman -Qs cryptsetup"]),
        // Backup system
        (&["backup", "system"], "backup system", "storage",
            &["echo 'rsync: sudo rsync -aAXv --exclude={\"/dev/*\",\"/proc/*\",\"/sys/*\"} / /backup/'",
              "echo 'Or use: borg create /backup::name /'",
              "echo 'Or: timeshift --create'"]),
        (&["how", "backup"], "how to backup system", "storage",
            &["echo 'Tools: rsync, borg, restic, timeshift'",
              "which rsync borg restic timeshift 2>/dev/null"]),
        // Create systemd service
        (&["create", "systemd", "service"], "create systemd service", "system",
            &["echo 'Create /etc/systemd/system/myservice.service:'",
              "echo '[Unit]'",
              "echo 'Description=My Service'",
              "echo '[Service]'",
              "echo 'ExecStart=/path/to/command'",
              "echo '[Install]'",
              "echo 'WantedBy=multi-user.target'"]),
        (&["how", "systemd", "service"], "how to create systemd service", "system",
            &["ls /etc/systemd/system/*.service | head -5",
              "echo 'Create unit file in /etc/systemd/system/<name>.service'"]),
        // Cron job setup
        (&["set", "up", "cron"], "set up cron job", "system",
            &["echo 'Edit crontab: crontab -e'",
              "echo 'Format: MIN HOUR DAY MONTH DOW command'",
              "echo 'Example: 0 * * * * /path/to/script'"]),
        (&["how", "cron"], "how to set up cron job", "system",
            &["crontab -l 2>/dev/null || echo 'No crontab'",
              "echo 'Create: crontab -e'"]),
        // Install nvidia
        (&["install", "nvidia"], "install NVIDIA drivers", "hardware",
            &["echo 'For newer cards: sudo pacman -S nvidia nvidia-utils'",
              "echo 'For older: sudo pacman -S nvidia-390xx'",
              "echo 'Then reboot'"]),
        (&["how", "nvidia"], "how to install NVIDIA drivers", "hardware",
            &["lspci | grep -i nvidia", "pacman -Qs nvidia"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_tasks() {
        assert!(match_patterns("how to install").is_some());
        assert!(match_patterns("how to update system").is_some());
    }

    #[test]
    fn test_service_tasks() {
        assert!(match_patterns("how to enable a service").is_some());
    }

    #[test]
    fn test_user_tasks() {
        assert!(match_patterns("how to add a user").is_some());
    }

    #[test]
    fn test_file_tasks() {
        assert!(match_patterns("how to change permission").is_some());
    }

    #[test]
    fn test_network_tasks() {
        assert!(match_patterns("set up ssh").is_some());
        assert!(match_patterns("configure network").is_some());
        assert!(match_patterns("set static ip").is_some());
        assert!(match_patterns("configure dns").is_some());
        assert!(match_patterns("configure samba").is_some());
    }

    #[test]
    fn test_storage_tasks() {
        assert!(match_patterns("create partition").is_some());
        assert!(match_patterns("encrypt disk").is_some());
        assert!(match_patterns("backup system").is_some());
        assert!(match_patterns("create systemd service").is_some());
    }
}

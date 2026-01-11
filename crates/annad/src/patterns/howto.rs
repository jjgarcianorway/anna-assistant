//! HowTo patterns - common task instructions with known commands
//! v0.0.947: Initial howto patterns for common Linux tasks

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, and command templates
type HowToPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

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
    None
}

fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::HowTo,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

/// Package management tasks
fn match_package_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Install
        (&["how", "install"], "install a package", "packages",
            &["sudo pacman -S <package>", "pacman -Ss <package>"]),
        (&["install", "package"], "install a package", "packages",
            &["sudo pacman -S <package>", "pacman -Ss <package>"]),
        // Update
        (&["how", "update", "system"], "update the system", "packages",
            &["sudo pacman -Syu"]),
        (&["update", "all", "package"], "update all packages", "packages",
            &["sudo pacman -Syu"]),
        (&["upgrade", "system"], "upgrade the system", "packages",
            &["sudo pacman -Syu"]),
        // Remove
        (&["how", "remove", "package"], "remove a package", "packages",
            &["sudo pacman -R <package>", "sudo pacman -Rs <package>"]),
        (&["how", "uninstall"], "uninstall a package", "packages",
            &["sudo pacman -Rs <package>"]),
        (&["remove", "package", "depend"], "remove package with dependencies", "packages",
            &["sudo pacman -Rns <package>"]),
        // Search
        (&["how", "search", "package"], "search for a package", "packages",
            &["pacman -Ss <keyword>", "yay -Ss <keyword>"]),
        (&["find", "package"], "find a package", "packages",
            &["pacman -Ss <keyword>"]),
        // List
        (&["list", "installed", "package"], "list installed packages", "packages",
            &["pacman -Q", "pacman -Q | wc -l"]),
        (&["what", "package", "installed"], "list installed packages", "packages",
            &["pacman -Q | head -30"]),
        // Clean
        (&["clean", "package", "cache"], "clean package cache", "packages",
            &["sudo pacman -Sc", "sudo pacman -Scc"]),
        (&["clear", "pacman", "cache"], "clear pacman cache", "packages",
            &["sudo pacman -Sc"]),
        // Orphans
        (&["remove", "orphan"], "remove orphan packages", "packages",
            &["pacman -Qdt", "sudo pacman -Rns $(pacman -Qdtq)"]),
        (&["clean", "orphan"], "clean orphan packages", "packages",
            &["sudo pacman -Rns $(pacman -Qdtq)"]),
        // AUR
        (&["install", "aur"], "install from AUR", "packages",
            &["yay -S <package>", "paru -S <package>"]),
        (&["how", "use", "aur"], "use AUR packages", "packages",
            &["yay -S <package>", "echo 'Install yay: git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Service management tasks
fn match_service_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Enable
        (&["how", "enable", "service"], "enable a service", "services",
            &["sudo systemctl enable <service>", "sudo systemctl enable --now <service>"]),
        (&["enable", "service", "boot"], "enable service at boot", "services",
            &["sudo systemctl enable <service>"]),
        // Disable
        (&["how", "disable", "service"], "disable a service", "services",
            &["sudo systemctl disable <service>"]),
        (&["stop", "service", "boot"], "disable service at boot", "services",
            &["sudo systemctl disable <service>"]),
        // Start/Stop
        (&["how", "start", "service"], "start a service", "services",
            &["sudo systemctl start <service>"]),
        (&["how", "stop", "service"], "stop a service", "services",
            &["sudo systemctl stop <service>"]),
        (&["how", "restart", "service"], "restart a service", "services",
            &["sudo systemctl restart <service>"]),
        // Status
        (&["check", "service", "status"], "check service status", "services",
            &["systemctl status <service>"]),
        (&["service", "running"], "check if service is running", "services",
            &["systemctl is-active <service>", "systemctl status <service>"]),
        // List
        (&["list", "running", "service"], "list running services", "services",
            &["systemctl list-units --type=service --state=running"]),
        (&["list", "enabled", "service"], "list enabled services", "services",
            &["systemctl list-unit-files --state=enabled"]),
        (&["list", "failed", "service"], "list failed services", "services",
            &["systemctl --failed"]),
        // Logs
        (&["view", "service", "log"], "view service logs", "services",
            &["journalctl -u <service> -f", "journalctl -u <service> -n 50"]),
        (&["check", "service", "log"], "check service logs", "services",
            &["journalctl -u <service> --no-pager | tail -30"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// User management tasks
fn match_user_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Add user
        (&["how", "add", "user"], "add a new user", "users",
            &["sudo useradd -m <username>", "sudo useradd -m -G wheel <username>"]),
        (&["create", "new", "user"], "create a new user", "users",
            &["sudo useradd -m -s /bin/bash <username>"]),
        // Delete user
        (&["how", "delete", "user"], "delete a user", "users",
            &["sudo userdel <username>", "sudo userdel -r <username>"]),
        (&["remove", "user", "account"], "remove user account", "users",
            &["sudo userdel -r <username>"]),
        // Change password
        (&["how", "change", "password"], "change password", "users",
            &["passwd", "sudo passwd <username>"]),
        (&["reset", "password"], "reset password", "users",
            &["sudo passwd <username>"]),
        // Add to group
        (&["add", "user", "group"], "add user to group", "users",
            &["sudo usermod -aG <group> <username>"]),
        (&["how", "add", "sudo"], "add user to sudo", "users",
            &["sudo usermod -aG wheel <username>"]),
        (&["give", "sudo", "access"], "give sudo access", "users",
            &["sudo usermod -aG wheel <username>", "sudo visudo"]),
        // Switch user
        (&["how", "switch", "user"], "switch user", "users",
            &["su - <username>", "sudo -u <username> -i"]),
        (&["login", "as", "root"], "login as root", "users",
            &["sudo -i", "su -"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// File and permission tasks
fn match_file_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Change permissions
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
        if keywords.iter().all(|kw| q.contains(kw)) {
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
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

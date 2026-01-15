//! Package, service, and user management patterns.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use super::contains_word;

/// Pattern with keywords, description, topic, and command templates
pub type HowToPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

pub fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
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
pub fn match_package_tasks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HowToPattern] = &[
        // Install
        (&["how", "install"], "install a package", "packages",
            &["sudo pacman -S <package>", "pacman -Ss <package>"]),
        (&["install", "package"], "install a package", "packages",
            &["sudo pacman -S <package>", "pacman -Ss <package>"]),
        // Update - v0.1.0: Added more update patterns
        (&["how", "update", "system"], "update the system", "packages",
            &["sudo pacman -Syu"]),
        (&["update", "all", "package"], "update all packages", "packages",
            &["sudo pacman -Syu"]),
        (&["upgrade", "system"], "upgrade the system", "packages",
            &["sudo pacman -Syu"]),
        (&["update", "system"], "update the system", "packages",
            &["echo 'Run: sudo pacman -Syu'", "echo 'Or with AUR: paru -Syu / yay -Syu'"]),
        (&["install", "update"], "install pending updates", "packages",
            &["echo 'Run: sudo pacman -Syu'", "echo 'Or with AUR: paru -Syu / yay -Syu'"]),
        (&["run", "update"], "run system updates", "packages",
            &["echo 'Run: sudo pacman -Syu'", "echo 'Or with AUR: paru -Syu / yay -Syu'"]),
        (&["apply", "update"], "apply pending updates", "packages",
            &["echo 'Run: sudo pacman -Syu'", "echo 'Or with AUR: paru -Syu / yay -Syu'"]),
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
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Service management tasks
pub fn match_service_tasks(q: &str) -> Option<DeepUnderstanding> {
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
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// User management tasks
pub fn match_user_tasks(q: &str) -> Option<DeepUnderstanding> {
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
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

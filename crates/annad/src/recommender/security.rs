//! Security recommendations

use super::{check_command_usage, is_package_installed};

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_password_manager() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if we're running a GUI
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        return result;
    }

    // KeePassXC - password manager
    if !Command::new("which")
        .arg("keepassxc")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-keepassxc".to_string(),
            title: "Install KeePassXC - secure password manager".to_string(),
            reason: "KeePassXC is a secure, open-source password manager with browser integration. Store all your passwords in an encrypted database instead of reusing the same password everywhere!".to_string(),
            action: "Install KeePassXC".to_string(),
            command: Some("sudo pacman -S --noconfirm keepassxc".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Security & Privacy".to_string(),
            alternatives: vec![
                Alternative {
                    name: "Bitwarden".to_string(),
                    description: "Cloud-based password manager".to_string(),
                    install_command: "flatpak install flathub com.bitwarden.desktop".to_string(),
                },
                Alternative {
                    name: "pass".to_string(),
                    description: "CLI password manager using GPG".to_string(),
                    install_command: "sudo pacman -S --noconfirm pass".to_string(),
                },
            ],
            wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications/Security#Password_managers".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: Some("security-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 80,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_security_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for rkhunter (rootkit detection)
    let has_rkhunter = Command::new("pacman")
        .args(&["-Q", "rkhunter"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rkhunter {
        result.push(Advice {
            id: "security-rkhunter".to_string(),
            title: "Consider rkhunter for rootkit detection".to_string(),
            reason: "rkhunter scans your system for rootkits, backdoors, and security issues! It checks for suspicious files, hidden processes, and system modifications. Run it monthly to catch compromises early. Think of it as a security health check!".to_string(),
            action: "Install rkhunter".to_string(),
            command: Some("pacman -S --noconfirm rkhunter".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rkhunter".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for ClamAV (antivirus)
    let has_clamav = Command::new("pacman")
        .args(&["-Q", "clamav"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_clamav {
        result.push(Advice {
            id: "security-clamav".to_string(),
            title: "Consider ClamAV for antivirus scanning".to_string(),
            reason: "While Linux malware is rare, ClamAV is useful for scanning files from Windows users or downloaded files! It catches Windows viruses in shared files, email attachments, and USB drives. Protects your Windows-using friends when you share files!".to_string(),
            action: "Install ClamAV".to_string(),
            command: Some("pacman -S --noconfirm clamav".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/ClamAV".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for LUKS encrypted partitions
    let has_encrypted = Command::new("lsblk")
        .args(&["-o", "TYPE"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("crypt"))
        .unwrap_or(false);

    if has_encrypted {
        result.push(Advice {
            id: "security-luks-reminder".to_string(),
            title: "LUKS encryption detected - Remember your backup!".to_string(),
            reason: "You're using LUKS encryption - great for security! Remember: if you lose your encryption password, your data is GONE FOREVER. Make sure you have a secure backup of your passphrase. Consider using a password manager or writing it down in a safe place!".to_string(),
            action: "Verify you have passphrase backup".to_string(),
            command: Some("lsblk -o NAME,FSTYPE,MOUNTPOINT | grep -E 'crypt|LUKS'".to_string()), // Show encrypted devices
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Dm-crypt".to_string()],
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

pub(crate) fn check_security_hardening() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for AppArmor
    if !is_package_installed("apparmor") {
        result.push(
            Advice::new(
                "security-apparmor".to_string(),
                "Install AppArmor for mandatory access control".to_string(),
                "AppArmor is a Mandatory Access Control (MAC) system that confines programs to limited resources. It prevents applications from accessing files they shouldn't, adding a security layer beyond standard permissions. Think of it as a sandbox for every program!".to_string(),
                "Install AppArmor and enable it".to_string(),
                Some("sudo pacman -S --noconfirm apparmor && sudo systemctl enable apparmor".to_string()),
                RiskLevel::Medium,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/AppArmor".to_string(),
                    "https://wiki.archlinux.org/title/Security#Mandatory_access_control".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(55)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check for fail2ban (SSH protection)
    let has_ssh_server = Command::new("systemctl")
        .args(&["is-enabled", "sshd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_ssh_server && !is_package_installed("fail2ban") {
        result.push(
            Advice::new(
                "security-fail2ban".to_string(),
                "Install fail2ban to protect SSH from brute force attacks".to_string(),
                "You're running an SSH server! fail2ban monitors log files and bans IPs with malicious behavior (repeated failed login attempts). It's essential protection against brute force attacks - automatically blocks attackers trying to guess your password.".to_string(),
                "Install and configure fail2ban".to_string(),
                Some("sudo pacman -S --noconfirm fail2ban && sudo systemctl enable --now fail2ban".to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec![
                    "https://wiki.archlinux.org/title/Fail2ban".to_string(),
                    "https://wiki.archlinux.org/title/SSH#Protection_from_brute_force_attacks".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(70)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check for auditd (audit framework)
    if !is_package_installed("audit") {
        result.push(
            Advice::new(
                "security-auditd".to_string(),
                "Install audit framework for system activity monitoring".to_string(),
                "The Linux Audit Framework monitors and logs security-relevant system events - file access, system calls, user actions. Essential for security compliance, forensics, and detecting suspicious activity. Required for many security standards (PCI-DSS, HIPAA).".to_string(),
                "Install audit daemon".to_string(),
                Some("sudo pacman -S --noconfirm audit && sudo systemctl enable --now auditd".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Audit_framework".to_string(),
                    "https://wiki.archlinux.org/title/Security#Auditing".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(50)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check for USBGuard
    if !is_package_installed("usbguard") {
        result.push(
            Advice::new(
                "security-usbguard".to_string(),
                "Install USBGuard to protect against malicious USB devices".to_string(),
                "USBGuard implements USB device authorization - it can block unauthorized USB devices like BadUSB attacks, USB Rubber Ducky, or unknown storage devices. Protects against physical attacks through USB ports. Essential for high-security environments!".to_string(),
                "Install USBGuard".to_string(),
                Some("sudo pacman -S --noconfirm usbguard".to_string()),
                RiskLevel::Medium,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/USBGuard".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(45)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check for Firejail (application sandboxing)
    if !is_package_installed("firejail") {
        result.push(
            Advice::new(
                "security-firejail".to_string(),
                "Install Firejail for application sandboxing".to_string(),
                "Firejail runs programs in isolated sandboxes with restricted filesystem, network, and system access. Run untrusted programs safely: 'firejail firefox' or 'firejail --private transmission-gtk'. Protects your system from compromised or malicious applications!".to_string(),
                "Install Firejail sandbox".to_string(),
                Some("sudo pacman -S --noconfirm firejail".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Firejail".to_string(),
                    "https://wiki.archlinux.org/title/Security#Sandboxing_applications".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(60)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check for AIDE (intrusion detection)
    if !is_package_installed("aide") {
        result.push(
            Advice::new(
                "security-aide".to_string(),
                "Install AIDE for file integrity monitoring".to_string(),
                "AIDE (Advanced Intrusion Detection Environment) creates a database of file checksums and monitors for unauthorized changes to system files. Detects rootkits, backdoors, and unauthorized system modifications. Run 'aide --check' regularly to verify system integrity!".to_string(),
                "Install AIDE".to_string(),
                Some("sudo pacman -S --noconfirm aide".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/AIDE".to_string(),
                    "https://wiki.archlinux.org/title/Security#File_integrity_checking".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(45)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check for secure DNS
    if !is_package_installed("dnscrypt-proxy") {
        result.push(
            Advice::new(
                "security-dnscrypt".to_string(),
                "Install dnscrypt-proxy for encrypted DNS queries".to_string(),
                "dnscrypt-proxy encrypts your DNS queries, preventing ISPs and network attackers from seeing what websites you visit. Supports DNS-over-HTTPS (DoH) and DNS-over-TLS, DNSSEC validation, and can block malicious/ad domains. Privacy and security in one package!".to_string(),
                "Install dnscrypt-proxy".to_string(),
                Some("sudo pacman -S --noconfirm dnscrypt-proxy".to_string()),
                RiskLevel::Medium,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Dnscrypt-proxy".to_string(),
                    "https://wiki.archlinux.org/title/Domain_name_resolution#Privacy_and_security".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(65)
            .with_bundle("security-hardening".to_string())
        );
    }

    // Check kernel hardening parameters
    let has_hardened_kernel = std::fs::read_to_string("/etc/sysctl.d/99-sysctl.conf")
        .or_else(|_| std::fs::read_to_string("/etc/sysctl.conf"))
        .map(|content| {
            content.contains("kernel.yama.ptrace_scope")
                && content.contains("kernel.kptr_restrict")
                && content.contains("net.ipv4.conf.all.rp_filter")
        })
        .unwrap_or(false);

    if !has_hardened_kernel {
        result.push(
            Advice::new(
                "security-kernel-hardening".to_string(),
                "Apply kernel hardening parameters".to_string(),
                "Kernel hardening sysctl parameters improve security: restrict ptrace (prevents debugger attacks), hide kernel pointers (prevents info leaks), enable reverse path filtering (prevents IP spoofing), and more. Industry-standard security hardening that costs nothing!".to_string(),
                "Review and apply kernel hardening parameters".to_string(),
                None, // Manual: requires creating/editing sysctl config
                RiskLevel::Medium,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Security#Kernel_hardening".to_string(),
                    "https://wiki.archlinux.org/title/Sysctl#Security".to_string(),
                ],
                "security".to_string(),
            )
            .with_popularity(70)
            .with_bundle("security-hardening".to_string())
        );
    }

    result
}

pub(crate) fn check_backup_solutions() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for rsync (basic backup)
    let has_rsync = Command::new("pacman")
        .args(&["-Q", "rsync"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rsync {
        result.push(Advice {
            id: "backup-rsync".to_string(),
            title: "Install rsync for file synchronization and backups".to_string(),
            reason: "You don't have rsync! It's THE tool for backups and file syncing - efficient, incremental, and powerful. Perfect for backing up to external drives, NAS, or remote servers. Everyone should have this! 'rsync -av source/ destination/' is all you need for basic backups.".to_string(),
            action: "Install rsync".to_string(),
            command: Some("pacman -S --noconfirm rsync".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rsync".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for borg backup
    let has_borg = Command::new("pacman")
        .args(&["-Q", "borg"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_borg && has_rsync {
        result.push(Advice {
            id: "backup-borg".to_string(),
            title: "Consider BorgBackup for encrypted backups".to_string(),
            reason: "Borg is an AMAZING backup tool! It does deduplication (saves tons of space), encryption, compression, and makes backups super fast. You can keep dozens of snapshots without using much disk space. Way better than rsync for regular backups!".to_string(),
            action: "Install BorgBackup".to_string(),
            command: Some("pacman -S --noconfirm borg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Borg_backup".to_string()],
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

pub(crate) fn check_password_managers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for KeePassXC
    let has_keepass = Command::new("pacman")
        .args(&["-Q", "keepassxc"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for Bitwarden
    let has_bitwarden = Command::new("pacman")
        .args(&["-Q", "bitwarden"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_keepass && !has_bitwarden {
        result.push(Advice {
            id: "password-manager-keepass".to_string(),
            title: "Install a password manager (KeePassXC recommended)".to_string(),
            reason: "You don't have a password manager! In 2025, this is ESSENTIAL for security! KeePassXC stores all passwords in an encrypted database - you only need to remember one master password. Generate strong unique passwords for every site, sync across devices, auto-fill forms. Stop reusing passwords!".to_string(),
            action: "Install KeePassXC".to_string(),
            command: Some("pacman -S --noconfirm keepassxc".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KeePass".to_string()],
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

pub(crate) fn check_backup_system_presence(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if facts.backup_systems.is_empty() {
        result.push(Advice::new(
            "no-backup-system".to_string(),
            "No Backup System Detected".to_string(),
            "No backup tools installed. You risk losing all data from hardware failure, accidental deletion, or system corruption. Regular backups are essential for data protection.".to_string(),
            "Install Timeshift for system snapshots or rsync/borg/restic for file backups".to_string(),
            Some("sudo pacman -S --noconfirm timeshift  # For system snapshots".to_string()),
            RiskLevel::Medium,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/System_backup".to_string()],
            "maintenance".to_string(),
        ).with_bundle("System Safety Essentials".to_string())
         .with_popularity(70));
    } else {
        result.push(Advice::new(
            "automate-backups".to_string(),
            format!("Automate Your Backups (Detected: {})", facts.backup_systems.join(", ")),
            format!(
                "You have {} installed. Manual backups are often forgotten - automate them with systemd timers or cron to run daily/weekly.",
                facts.backup_systems.join(" and ")
            ),
            "Set up systemd timers or cron jobs for automatic backup scheduling".to_string(),
            Some("sudo systemctl enable --now cronie  # Enable cron for scheduled backups".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Systemd/Timers".to_string()],
            "maintenance".to_string(),
        ).with_popularity(50));
    }

    result
}


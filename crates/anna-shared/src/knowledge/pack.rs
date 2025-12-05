//! Built-in Arch Linux knowledge pack (v0.0.39).
//!
//! Static, high-confidence knowledge for common Arch tasks.
//! Never expires, loaded on first use.

use super::sources::{KnowledgeDoc, KnowledgeSource, Provenance};

/// A static knowledge entry
pub struct PackEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub tags: &'static [&'static str],
}

/// Built-in Arch Linux knowledge entries
pub const ARCH_PACK: &[PackEntry] = &[
    // Package Management
    PackEntry {
        id: "arch-update",
        title: "Update Arch Linux system",
        body: "Run `sudo pacman -Syu` to update the package database and upgrade all packages. \
               Use `-Syyu` to force refresh if mirrors were changed. \
               For partial updates, use `sudo pacman -Sy <package>` but full system updates are recommended.",
        tags: &["update", "upgrade", "pacman", "packages"],
    },
    PackEntry {
        id: "arch-install",
        title: "Install a package on Arch",
        body: "Run `sudo pacman -S <package>` to install a package. \
               Use `-S --needed` to skip reinstalling already installed packages. \
               For multiple packages: `sudo pacman -S pkg1 pkg2 pkg3`.",
        tags: &["install", "pacman", "package"],
    },
    PackEntry {
        id: "arch-remove",
        title: "Remove a package from Arch",
        body: "Run `sudo pacman -R <package>` to remove a package. \
               Use `-Rs` to also remove unused dependencies. \
               Use `-Rns` to also remove config files (clean removal).",
        tags: &["remove", "uninstall", "pacman", "package"],
    },
    PackEntry {
        id: "arch-search",
        title: "Search for packages",
        body: "Run `pacman -Ss <query>` to search remote packages. \
               Run `pacman -Qs <query>` to search installed packages. \
               Run `pacman -Qi <package>` for detailed package info.",
        tags: &["search", "find", "pacman", "package"],
    },
    PackEntry {
        id: "arch-aur",
        title: "Using AUR helpers",
        body: "Use `yay` or `paru` to install AUR packages. Example: `yay -S <package>`. \
               These helpers also handle regular pacman operations. \
               Install yay: `pacman -S --needed git base-devel && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si`.",
        tags: &["aur", "yay", "paru", "helper"],
    },
    // System Services
    PackEntry {
        id: "arch-service-status",
        title: "Check service status",
        body: "Run `systemctl status <service>` to check if a service is running. \
               Use `systemctl is-active <service>` for a quick check. \
               Use `systemctl is-enabled <service>` to check if it starts at boot.",
        tags: &["service", "status", "systemctl", "running"],
    },
    PackEntry {
        id: "arch-service-enable",
        title: "Enable a service at boot",
        body: "Run `sudo systemctl enable <service>` to start at boot. \
               Use `enable --now` to also start it immediately. \
               Common services: sshd, docker, NetworkManager, bluetooth.",
        tags: &["service", "enable", "start", "boot", "autostart"],
    },
    PackEntry {
        id: "arch-failed-services",
        title: "Find failed services",
        body: "Run `systemctl --failed` to list all failed units. \
               Use `journalctl -u <service>` to see logs for a specific failed service. \
               Fix common issues: check config files, dependencies, permissions.",
        tags: &["failed", "services", "broken", "errors"],
    },
    PackEntry {
        id: "arch-logs",
        title: "View system logs",
        body: "Run `journalctl -xe` to see recent logs with explanations. \
               Use `journalctl -b` for current boot, `-b -1` for previous boot. \
               Filter by service: `journalctl -u <service>`. \
               Follow live: `journalctl -f`.",
        tags: &["logs", "journal", "journalctl", "debug"],
    },
    // Disk & Storage
    PackEntry {
        id: "arch-disk-usage",
        title: "Check disk usage",
        body: "Run `df -h` to see disk usage by filesystem. \
               Use `du -sh <dir>` to check directory size. \
               Install `ncdu` for interactive disk usage analysis: `sudo pacman -S ncdu`.",
        tags: &["disk", "space", "usage", "full", "df"],
    },
    PackEntry {
        id: "arch-clean-cache",
        title: "Clean package cache",
        body: "Run `sudo pacman -Sc` to remove old package versions from cache. \
               Use `paccache -rk1` to keep only the most recent version. \
               Install paccache: `sudo pacman -S pacman-contrib`. \
               WARNING: `sudo pacman -Scc` removes ALL cached packages.",
        tags: &["clean", "cache", "pacman", "free", "space"],
    },
    PackEntry {
        id: "arch-mount",
        title: "Mount an external drive",
        body: "First identify with `lsblk`. Then: `sudo mount /dev/sdX1 /mnt`. \
               For NTFS: `sudo mount -t ntfs3 /dev/sdX1 /mnt`. \
               Create mount point if needed: `sudo mkdir /mnt/usb`. \
               Unmount: `sudo umount /mnt`.",
        tags: &["mount", "drive", "usb", "external"],
    },
    // Network
    PackEntry {
        id: "arch-network-status",
        title: "Check network status",
        body: "Run `ip addr` to see all network interfaces and IPs. \
               Use `ip link` for interface status. \
               Test connectivity: `ping -c 3 archlinux.org`. \
               Check routes: `ip route`.",
        tags: &["network", "ip", "address", "interface", "connection"],
    },
    PackEntry {
        id: "arch-wifi",
        title: "Connect to WiFi",
        body: "With NetworkManager: `nmcli device wifi connect <SSID> password <pass>`. \
               List networks: `nmcli device wifi list`. \
               With iwd: `iwctl station wlan0 connect <SSID>`. \
               Check status: `nmcli connection show`.",
        tags: &["wifi", "wireless", "connect", "nmcli", "iwctl"],
    },
    PackEntry {
        id: "arch-dns",
        title: "DNS resolution issues",
        body: "Check `/etc/resolv.conf` for DNS servers. \
               Test with `dig <domain>` or `nslookup <domain>`. \
               Common fix: add `nameserver 1.1.1.1` to resolv.conf. \
               For systemd-resolved: `resolvectl status`.",
        tags: &["dns", "resolve", "hostname", "domain"],
    },
    // Troubleshooting
    PackEntry {
        id: "arch-boot-failure",
        title: "Boot failure troubleshooting",
        body: "Boot from live USB, mount root partition, and chroot: \
               `mount /dev/sdX1 /mnt && arch-chroot /mnt`. \
               Check logs: `journalctl -b -1`. \
               Regenerate initramfs: `mkinitcpio -P`. \
               Reinstall bootloader if needed.",
        tags: &["boot", "fail", "grub", "stuck", "chroot"],
    },
    PackEntry {
        id: "arch-pacman-lock",
        title: "Pacman database locked",
        body: "If pacman says database is locked and no other pacman is running: \
               `sudo rm /var/lib/pacman/db.lck`. \
               Only do this if you're SURE no other pacman process is active. \
               Check: `ps aux | grep pacman`.",
        tags: &["lock", "pacman", "database", "locked"],
    },
    PackEntry {
        id: "arch-keyring",
        title: "Pacman keyring issues",
        body: "Run `sudo pacman-key --init && sudo pacman-key --populate archlinux`. \
               If still failing: `sudo pacman -Sy archlinux-keyring && sudo pacman -Su`. \
               For corrupted keyring: `sudo rm -rf /etc/pacman.d/gnupg` then reinit.",
        tags: &["keyring", "signature", "key", "gpg", "trust"],
    },
    // Security
    PackEntry {
        id: "arch-firewall",
        title: "Check firewall status",
        body: "For UFW: `sudo ufw status`. \
               For nftables: `sudo nft list ruleset`. \
               For iptables: `sudo iptables -L -n`. \
               Enable UFW: `sudo ufw enable`.",
        tags: &["firewall", "ufw", "iptables", "nftables"],
    },
    PackEntry {
        id: "arch-sudo",
        title: "Sudo password issues",
        body: "If sudo asks for password: enter YOUR user password, not root's. \
               Add user to sudoers: `sudo usermod -aG wheel <username>`. \
               Edit sudoers safely: `sudo EDITOR=nano visudo`. \
               Ensure `%wheel ALL=(ALL:ALL) ALL` is uncommented.",
        tags: &["sudo", "password", "root", "permission", "wheel"],
    },
];

/// Convert pack entry to KnowledgeDoc
pub fn entry_to_doc(entry: &PackEntry) -> KnowledgeDoc {
    KnowledgeDoc::with_id(
        format!("builtin:{}", entry.id),
        KnowledgeSource::BuiltIn,
        entry.title.to_string(),
        entry.body.to_string(),
        entry.tags.iter().map(|s| s.to_string()).collect(),
        Provenance {
            collected_by: "anna-builtin".to_string(),
            command: None,
            path: None,
            confidence: 95, // High confidence for curated content
        },
    )
}

/// Get all built-in knowledge documents
pub fn get_builtin_docs() -> Vec<KnowledgeDoc> {
    ARCH_PACK.iter().map(entry_to_doc).collect()
}

/// Search built-in pack by keywords (lightweight, no store needed)
pub fn search_builtin_pack(query: &str, limit: usize) -> Vec<(u32, &'static PackEntry)> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut matches: Vec<(u32, &'static PackEntry)> = Vec::new();

    for entry in ARCH_PACK {
        let mut score: u32 = 0;

        // Score tags
        for tag in entry.tags {
            if query_lower.contains(tag) {
                score += 15;
            }
            for word in &query_words {
                if *word == *tag {
                    score += 10;
                } else if word.len() > 3 && tag.contains(word) {
                    score += 5;
                }
            }
        }

        // Score title
        let title_lower = entry.title.to_lowercase();
        for word in &query_words {
            if word.len() > 3 && title_lower.contains(word) {
                score += 8;
            }
        }

        // Score body (lighter weight)
        let body_lower = entry.body.to_lowercase();
        for word in &query_words {
            if word.len() > 4 && body_lower.contains(word) {
                score += 3;
            }
        }

        if score > 0 {
            matches.push((score, entry));
        }
    }

    // Sort by score descending, then by id for determinism
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(b.1.id)));
    matches.truncate(limit);
    matches
}

/// Try to get a high-confidence built-in answer
pub fn try_builtin_answer(query: &str, min_score: u32) -> Option<&'static PackEntry> {
    search_builtin_pack(query, 1)
        .first()
        .filter(|(score, _)| *score >= min_score)
        .map(|(_, entry)| *entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_update() {
        let matches = search_builtin_pack("update arch linux", 5);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].1.id, "arch-update");
    }

    #[test]
    fn test_search_disk() {
        let matches = search_builtin_pack("disk space full", 5);
        assert!(!matches.is_empty());
        assert!(matches[0].1.id.contains("disk"));
    }

    #[test]
    fn test_search_failed() {
        let matches = search_builtin_pack("systemctl failed services", 5);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].1.id, "arch-failed-services");
    }

    #[test]
    fn test_builtin_docs() {
        let docs = get_builtin_docs();
        assert!(!docs.is_empty());
        assert!(docs.iter().all(|d| d.id.starts_with("builtin:")));
        assert!(docs.iter().all(|d| d.provenance.confidence >= 90));
    }

    #[test]
    fn test_try_builtin_answer() {
        let entry = try_builtin_answer("update arch linux system", 20);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().id, "arch-update");
    }
}

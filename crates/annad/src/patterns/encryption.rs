//! Encryption patterns for LUKS, dm-crypt, GPG, SSH keys.
//! v0.0.976: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an encryption-related DeepUnderstanding
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

type EncryptionPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match encryption-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_luks(q)
        .or_else(|| match_dm_crypt(q))
        .or_else(|| match_gpg(q))
        .or_else(|| match_disk_encryption(q))
        .or_else(|| match_key_management(q))
}

/// LUKS patterns
fn match_luks(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[EncryptionPattern] = &[
        // LUKS status
        (&["luks", "status"], "show LUKS status", "encryption",
         &["lsblk -f | grep -i crypt", "dmsetup status"]),
        (&["luks", "info"], "show LUKS info", "encryption",
         &["lsblk -f | grep -i luks", "cryptsetup luksDump /dev/sdX 2>/dev/null || echo 'Specify device'"]),
        // LUKS devices
        (&["luks", "devices"], "list LUKS devices", "encryption",
         &["lsblk -f | grep -E 'crypto|luks'", "blkid | grep -i luks"]),
        (&["encrypted", "partitions"], "show encrypted partitions", "encryption",
         &["lsblk -f | grep -i crypt", "blkid | grep -i crypto"]),
        (&["encrypted", "drives"], "show encrypted drives", "encryption",
         &["lsblk -f | grep -i crypt"]),
        // LUKS version
        (&["luks", "version"], "show LUKS version", "encryption",
         &["cryptsetup --version"]),
        // Open LUKS
        (&["luks", "open"], "show how to open LUKS", "encryption",
         &["echo 'cryptsetup luksOpen /dev/sdX name'"]),
        (&["unlock", "luks"], "show how to unlock LUKS", "encryption",
         &["echo 'cryptsetup luksOpen /dev/sdX name'"]),
        // LUKS keyslots
        (&["luks", "keyslots"], "show LUKS keyslots", "encryption",
         &["echo 'cryptsetup luksDump /dev/sdX | grep -i slot'"]),
        (&["luks", "keys"], "show LUKS key info", "encryption",
         &["echo 'cryptsetup luksDump /dev/sdX'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// dm-crypt patterns
fn match_dm_crypt(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[EncryptionPattern] = &[
        // dm-crypt status
        (&["dmcrypt", "status"], "show dm-crypt status", "encryption",
         &["dmsetup status", "lsblk -f | grep crypt"]),
        (&["dm-crypt", "status"], "show dm-crypt status", "encryption",
         &["dmsetup status", "lsblk -f | grep crypt"]),
        // Mapped devices
        (&["mapped", "devices"], "list mapped devices", "encryption",
         &["dmsetup ls", "ls /dev/mapper/"]),
        (&["device", "mapper"], "show device mapper info", "encryption",
         &["dmsetup ls", "ls -la /dev/mapper/"]),
        // Cryptsetup
        (&["cryptsetup", "status"], "show cryptsetup status", "encryption",
         &["cryptsetup status /dev/mapper/* 2>/dev/null | head -20"]),
        (&["cryptsetup", "version"], "show cryptsetup version", "encryption",
         &["cryptsetup --version"]),
        // Crypt target
        (&["crypt", "target"], "show crypt targets", "encryption",
         &["dmsetup table --target crypt"]),
        // Active encryptions
        (&["active", "encryption"], "show active encrypted volumes", "encryption",
         &["dmsetup ls --target crypt", "lsblk -f | grep crypt"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// GPG patterns
fn match_gpg(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[EncryptionPattern] = &[
        // GPG keys
        (&["gpg", "keys"], "list GPG keys", "encryption",
         &["gpg --list-keys", "gpg --list-secret-keys"]),
        (&["gpg", "list"], "list GPG keys", "encryption",
         &["gpg --list-keys"]),
        (&["my", "gpg"], "show my GPG keys", "encryption",
         &["gpg --list-secret-keys --keyid-format LONG"]),
        // GPG secret keys
        (&["gpg", "secret"], "list GPG secret keys", "encryption",
         &["gpg --list-secret-keys"]),
        (&["gpg", "private"], "list GPG private keys", "encryption",
         &["gpg --list-secret-keys"]),
        // GPG version
        (&["gpg", "version"], "show GPG version", "encryption",
         &["gpg --version"]),
        // GPG agent
        (&["gpg", "agent"], "show GPG agent status", "encryption",
         &["gpg-agent --version", "pgrep -a gpg-agent"]),
        // GPG config
        (&["gpg", "config"], "show GPG config", "encryption",
         &["cat ~/.gnupg/gpg.conf 2>/dev/null | head -20"]),
        // GPG keyserver
        (&["gpg", "keyserver"], "show GPG keyserver", "encryption",
         &["grep -i keyserver ~/.gnupg/gpg.conf 2>/dev/null"]),
        // GPG trust
        (&["gpg", "trust"], "show GPG trust levels", "encryption",
         &["gpg --list-keys --with-colons | grep -E '^pub|^uid'"]),
        // Pacman keys
        (&["pacman", "keys"], "show pacman GPG keys", "encryption",
         &["pacman-key --list-keys | head -50"]),
        (&["pacman", "keyring"], "show pacman keyring", "encryption",
         &["pacman-key --list-keys | head -30", "ls /etc/pacman.d/gnupg/"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Disk encryption patterns
fn match_disk_encryption(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[EncryptionPattern] = &[
        // Encrypted disks
        (&["disk", "encryption"], "show disk encryption status", "encryption",
         &["lsblk -f | grep -E 'crypt|luks'", "dmsetup status"]),
        (&["full", "disk", "encryption"], "check full disk encryption", "encryption",
         &["lsblk -f | grep -E 'crypt|luks'", "cat /etc/crypttab"]),
        (&["fde", "status"], "check FDE status", "encryption",
         &["lsblk -f | grep -E 'crypt|luks'"]),
        // Crypttab
        (&["crypttab"], "show crypttab", "encryption",
         &["cat /etc/crypttab"]),
        (&["crypt", "tab"], "show crypttab entries", "encryption",
         &["cat /etc/crypttab"]),
        // Home encryption
        (&["home", "encrypted"], "check if home is encrypted", "encryption",
         &["lsblk -f | grep -E 'home.*crypt|crypt.*home'", "mount | grep home"]),
        (&["encrypted", "home"], "check home encryption", "encryption",
         &["lsblk -f | grep -E 'home.*crypt'", "df -h /home"]),
        // Root encryption
        (&["root", "encrypted"], "check if root is encrypted", "encryption",
         &["lsblk -f | grep -E '/.*crypt'", "mount | grep ' / '"]),
        // Encryption at rest
        (&["encryption", "at", "rest"], "check encryption at rest", "encryption",
         &["lsblk -f | grep crypt", "dmsetup ls"]),
        // VeraCrypt
        (&["veracrypt", "volumes"], "list VeraCrypt volumes", "encryption",
         &["veracrypt -l 2>/dev/null || echo 'VeraCrypt not installed'"]),
        (&["veracrypt", "status"], "show VeraCrypt status", "encryption",
         &["veracrypt --version 2>/dev/null", "veracrypt -l 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Key management patterns
fn match_key_management(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[EncryptionPattern] = &[
        // Keychain
        (&["keychain", "status"], "show keychain status", "encryption",
         &["keychain --version 2>/dev/null", "pgrep -a keychain"]),
        // GNOME Keyring
        (&["gnome", "keyring"], "show GNOME keyring status", "encryption",
         &["pgrep -a gnome-keyring", "secret-tool --version 2>/dev/null"]),
        (&["secret", "service"], "show secret service status", "encryption",
         &["pgrep -a gnome-keyring", "pgrep -a kwalletd"]),
        // KDE Wallet
        (&["kwallet", "status"], "show KWallet status", "encryption",
         &["pgrep -a kwalletd", "kwallet-query 2>/dev/null || echo 'KWallet tools not found'"]),
        (&["kde", "wallet"], "show KDE wallet status", "encryption",
         &["pgrep -a kwalletd5 kwalletd"]),
        // Pass
        (&["pass", "list"], "list pass entries", "encryption",
         &["pass ls 2>/dev/null || echo 'pass not configured'"]),
        (&["password", "store"], "show password store", "encryption",
         &["ls ~/.password-store/ 2>/dev/null", "pass ls 2>/dev/null"]),
        // TPM
        (&["tpm", "status"], "show TPM status", "encryption",
         &["cat /sys/class/tpm/tpm0/device/description 2>/dev/null", "dmesg | grep -i tpm | tail -5"]),
        (&["tpm", "version"], "show TPM version", "encryption",
         &["cat /sys/class/tpm/tpm0/tpm_version_major 2>/dev/null", "tpm2_getcap properties-fixed 2>/dev/null | head -10"]),
        // Secure boot keys
        (&["secure", "boot", "keys"], "show secure boot keys", "encryption",
         &["mokutil --list-enrolled 2>/dev/null | head -20", "ls /sys/firmware/efi/efivars/ | grep -i key | head -10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luks() {
        assert!(match_patterns("luks status").is_some());
        assert!(match_patterns("luks devices").is_some());
        assert!(match_patterns("encrypted partitions").is_some());
    }

    #[test]
    fn test_dm_crypt() {
        assert!(match_patterns("dmcrypt status").is_some());
        assert!(match_patterns("mapped devices").is_some());
        assert!(match_patterns("device mapper").is_some());
    }

    #[test]
    fn test_gpg() {
        assert!(match_patterns("gpg keys").is_some());
        assert!(match_patterns("gpg version").is_some());
        assert!(match_patterns("pacman keys").is_some());
    }

    #[test]
    fn test_disk_encryption() {
        assert!(match_patterns("disk encryption").is_some());
        assert!(match_patterns("crypttab").is_some());
        assert!(match_patterns("home encrypted").is_some());
    }

    #[test]
    fn test_key_management() {
        assert!(match_patterns("gnome keyring").is_some());
        assert!(match_patterns("tpm status").is_some());
        assert!(match_patterns("password store").is_some());
    }
}

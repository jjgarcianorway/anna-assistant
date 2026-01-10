//! Pacman/package manager error patterns
//! v0.0.913: Added suggested_commands for instant solutions

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, entities, and solution commands
type TroubleshootPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str], &'static [&'static str]);

/// Match pacman and package manager errors
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[TroubleshootPattern] = &[
        // Database locked - MOST COMMON pacman issue
        (&["database", "locked"], "pacman database is locked", "packages",
            &["pacman", "database"],
            &["ls -la /var/lib/pacman/db.lck 2>/dev/null || echo 'Lock file not found'",
              "echo 'FIX: sudo rm /var/lib/pacman/db.lck'"]),
        (&["db.lck"], "pacman database lock file", "packages",
            &["pacman", "lock"],
            &["ls -la /var/lib/pacman/db.lck", "echo 'FIX: sudo rm /var/lib/pacman/db.lck'"]),

        // GPG/keyring errors
        (&["gpgme", "error"], "GPG keyring error", "packages",
            &["gpg", "keyring"],
            &["echo 'FIX: sudo pacman-key --init && sudo pacman-key --populate archlinux'"]),
        (&["key", "could not be", "verified"], "package signature verification failed", "packages",
            &["gpg"],
            &["echo 'FIX: sudo pacman-key --refresh-keys'"]),
        (&["keyring", "pacman"], "pacman keyring issue", "packages",
            &["pacman-key"],
            &["echo 'FIX: sudo pacman-key --init && sudo pacman-key --populate archlinux'"]),
        (&["invalid or corrupted", "package"], "corrupted package", "packages",
            &["pacman"],
            &["echo 'FIX: sudo pacman -Syy && sudo pacman -S archlinux-keyring'"]),

        // Package conflicts
        (&["conflicting", "files"], "package file conflicts", "packages",
            &["pacman", "conflict"],
            &["echo 'FIX: sudo pacman -S --overwrite \"*\" <package>'"]),
        (&["exists in filesystem"], "file exists in filesystem error", "packages",
            &["pacman"],
            &["echo 'FIX: sudo pacman -S --overwrite \"*\" <package> OR find orphaned file owner'"]),

        // Partial updates
        (&["partial", "update"], "partial system update", "packages",
            &["pacman", "update"],
            &["echo 'FIX: sudo pacman -Syu (full system upgrade is required)'"]),
        (&["locale", "error"], "locale errors after update", "packages",
            &["locale", "glibc"],
            &["locale -a 2>&1 | head -5", "echo 'FIX: sudo locale-gen'"]),

        // Orphan packages
        (&["orphan", "package"], "orphan packages query", "packages",
            &["pacman", "orphans"],
            &["pacman -Qtdq", "echo 'FIX: sudo pacman -Rns $(pacman -Qtdq)'"]),

        // Cache cleaning
        (&["clean", "cache", "pacman"], "clean pacman cache", "packages",
            &["paccache"],
            &["du -sh /var/cache/pacman/pkg", "echo 'FIX: sudo paccache -r (keeps 3 versions)'"]),
        (&["clear", "cache", "pacman"], "clean pacman cache", "packages",
            &["paccache"],
            &["du -sh /var/cache/pacman/pkg", "echo 'FIX: sudo paccache -rk1 (keeps 1 version)'"]),

        // yay/AUR issues
        (&["yay", "permission", "denied"], "yay permission denied", "packages",
            &["yay"],
            &["ls -la ~/.cache/yay", "echo 'FIX: Never run yay with sudo. Use: yay -S <package>'"]),
        (&["yay", "git", "clone"], "yay git clone error", "packages",
            &["yay", "git"],
            &["echo 'FIX: rm -rf ~/.cache/yay/<package> && yay -S <package>'"]),
        (&["paru", "error"], "paru AUR helper error", "packages",
            &["paru"],
            &["echo 'FIX: rm -rf ~/.cache/paru/clone/<package> && paru -S <package>'"]),

        // Mirrors
        (&["mirror", "slow"], "slow pacman mirrors", "packages",
            &["reflector"],
            &["echo 'FIX: sudo reflector --latest 10 --sort rate --save /etc/pacman.d/mirrorlist'"]),
        (&["mirror", "fail"], "mirror failure", "packages",
            &["reflector"],
            &["echo 'FIX: sudo reflector --country US --age 12 --protocol https --sort rate --save /etc/pacman.d/mirrorlist'"]),
    ];

    for (keywords, interpreted, topic, entities, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                entities: entities.iter().map(|s| s.to_string()).collect(),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }

    None
}

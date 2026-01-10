//! Recovery scenario patterns - user clearly needs urgent help

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

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
    let patterns: &[(&[&str], &str)] = &[
        (&["deleted", "/usr/bin"], "accidentally deleted /usr/bin"),
        (&["deleted", "/usr"], "accidentally deleted /usr directory"),
        (&["removed", "/usr"], "accidentally removed /usr directory"),
        (&["deleted", "/etc"], "accidentally deleted /etc"),
        (&["deleted", "/boot"], "accidentally deleted /boot"),
        (&["accidentally", "deleted"], "accidental file deletion recovery"),
        (&["accidentally", "removed"], "accidental file removal recovery"),
        (&["accidentally", "rm", "-rf"], "accidental recursive deletion"),
    ];

    for (keywords, interpreted) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("recovery".to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_boot_failure(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str)] = &[
        (&["won't", "boot"], "system boot failure"),
        (&["can't", "boot"], "system boot failure"),
        (&["not", "boot"], "system not booting"),
        (&["boot", "stuck"], "boot process stuck"),
        (&["boot", "hang"], "boot process hanging"),
        (&["grub", "rescue"], "GRUB rescue mode"),
        (&["grub", "error"], "GRUB error"),
        (&["kernel", "panic"], "kernel panic"),
        (&["initramfs", "error"], "initramfs/mkinitcpio error"),
        (&["mkinitcpio", "error"], "mkinitcpio error"),
        (&["starting version"], "boot stuck at systemd version"),
        (&["black", "screen"], "black screen issue"),
        (&["display", "manager", "won't"], "display manager failure"),
        (&["gdm", "not", "start"], "GDM not starting"),
        (&["sddm", "not", "start"], "SDDM not starting"),
    ];

    for (keywords, interpreted) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("boot".to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_permission_disaster(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str)] = &[
        (&["chmod", "777", "-r"], "recursive chmod 777 recovery"),
        (&["chmod", "777", "recursive"], "recursive chmod 777 recovery"),
        (&["chmod", "-r", "/"], "recursive chmod on root"),
        (&["chown", "-r", "/"], "recursive chown on root"),
        (&["permission", "denied", "everywhere"], "widespread permission issues"),
    ];

    for (keywords, interpreted) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some("recovery".to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_emergency(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str)] = &[
        // Password issues
        (&["forgot", "password"], "forgotten password recovery", "security"),
        (&["forgot", "root", "password"], "forgotten root password", "security"),
        (&["reset", "password"], "password reset", "security"),
        // Disk full
        (&["disk", "full", "can't", "login"], "disk full preventing login", "storage"),
        (&["filled", "disk"], "disk completely full", "storage"),
        (&["no", "space", "left"], "no disk space left", "storage"),
        // System freeze
        (&["freeze", "complete"], "complete system freeze", "hardware"),
        (&["sysrq", "not", "work"], "system unresponsive to SysRq", "hardware"),
        (&["system", "frozen"], "frozen system", "hardware"),
        // Can't login
        (&["can't", "login"], "unable to login", "security"),
        (&["login", "loop"], "login loop", "display"),
    ];

    for (keywords, interpreted, topic) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

//! Tip collections for various topics.

use super::types::ContextualTip;

/// Get tips for editor-related queries
pub(super) fn editor_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "editor-config",
            message: "You can configure any editor setting by asking naturally, \
                     like \"enable line numbers in vim\".",
            related_action: None,
        },
        ContextualTip {
            id: "editor-backup",
            message: "I always backup config files before changes. \
                     Say \"undo last change\" if something goes wrong.",
            related_action: Some("undo last change"),
        },
    ]
}

/// Get tips for container-related queries
pub(super) fn container_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "docker-compose",
            message: "For Docker Compose projects, try \"start my docker compose\" \
                     or \"view docker logs\".",
            related_action: None,
        },
        ContextualTip {
            id: "docker-cleanup",
            message: "Docker using too much space? Ask \"clean up docker\" \
                     to remove unused images and containers.",
            related_action: Some("clean up docker"),
        },
    ]
}

/// Get tips for git-related queries
pub(super) fn git_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "git-config",
            message: "Need to set up git? Try \"configure git with my name and email\".",
            related_action: Some("configure git"),
        },
        ContextualTip {
            id: "git-ssh",
            message: "For GitHub SSH access, ask \"set up SSH key for GitHub\".",
            related_action: Some("set up ssh for github"),
        },
    ]
}

/// Get tips for service-related queries
pub(super) fn service_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "service-logs",
            message: "Having service issues? Try \"show logs for [service]\" \
                     or \"why did [service] fail\".",
            related_action: None,
        },
        ContextualTip {
            id: "service-enable",
            message: "Want a service to start on boot? Ask \"enable [service] on startup\".",
            related_action: None,
        },
    ]
}

/// Get tips for network-related queries
pub(super) fn network_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "network-diag",
            message: "Network issues? Try \"diagnose network\" or \"why is my connection slow\".",
            related_action: Some("diagnose network"),
        },
        ContextualTip {
            id: "network-ports",
            message: "Check what's using a port with \"what's using port 8080\".",
            related_action: None,
        },
    ]
}

/// Get tips for storage-related queries
pub(super) fn storage_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "storage-large",
            message: "Looking for space? Ask \"find large files\" or \"what's using disk space\".",
            related_action: Some("find large files"),
        },
        ContextualTip {
            id: "storage-mount",
            message: "For persistent mounts, I can update /etc/fstab. \
                     Just confirm when I ask.",
            related_action: None,
        },
    ]
}

/// Get tips for package-related queries
pub(super) fn package_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "package-search",
            message: "Not sure of the package name? Try \"find package for [tool]\" \
                     or \"what provides [command]\".",
            related_action: None,
        },
        ContextualTip {
            id: "package-update",
            message: "Keep your system updated with \"update my system\" \
                     or \"check for updates\".",
            related_action: Some("update system"),
        },
    ]
}

/// Get tips for scheduling-related queries
pub(super) fn scheduling_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "cron-syntax",
            message: "Cron confusing? Just say \"run [command] every day at 3am\" \
                     and I'll figure out the syntax.",
            related_action: None,
        },
        ContextualTip {
            id: "timer-vs-cron",
            message: "For systemd systems, timers are often better than cron. \
                     I'll suggest the best option for your case.",
            related_action: None,
        },
    ]
}

/// Get tips for security-related queries
pub(super) fn security_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "security-perms",
            message: "Permission denied? Ask \"fix permissions for [path]\" \
                     or \"who owns [file]\".",
            related_action: None,
        },
        ContextualTip {
            id: "security-firewall",
            message: "Need to open a port? Try \"allow port 443\" or \"check firewall rules\".",
            related_action: None,
        },
    ]
}

/// Get learning mode tips
pub(super) fn learning_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "learn-explain",
            message: "With learning mode on, I explain every command. \
                     Ask \"why did you run that\" anytime.",
            related_action: Some("why did you run that"),
        },
        ContextualTip {
            id: "learn-reference",
            message: "Want to know more? I cite from man pages and Arch Wiki. \
                     Ask \"show me the docs for [command]\".",
            related_action: None,
        },
    ]
}

/// Get general tips (no specific context)
pub(super) fn general_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "general-undo",
            message: "Made a mistake? Say \"undo last change\" to restore from backup.",
            related_action: Some("undo last change"),
        },
        ContextualTip {
            id: "general-history",
            message: "Check what we've done with \"show history\" or \"what did you change\".",
            related_action: Some("show history"),
        },
        ContextualTip {
            id: "general-stats",
            message: "Curious about your usage? Try \"show my stats\" or \"fun facts\".",
            related_action: Some("show my stats"),
        },
    ]
}

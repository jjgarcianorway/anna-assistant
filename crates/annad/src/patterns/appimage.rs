//! Flatpak, Snap, and AppImage patterns.
//! v0.0.979: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an app container-related DeepUnderstanding
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

type AppPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match app container-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_flatpak(q)
        .or_else(|| match_snap(q))
        .or_else(|| match_appimage(q))
        .or_else(|| match_app_troubleshoot(q))
}

/// Flatpak patterns
fn match_flatpak(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AppPattern] = &[
        // Flatpak list
        (&["flatpak", "list"], "list installed flatpaks", "flatpak",
         &["flatpak list"]),
        (&["installed", "flatpaks"], "list installed flatpaks", "flatpak",
         &["flatpak list"]),
        (&["flatpak", "installed"], "list installed flatpaks", "flatpak",
         &["flatpak list"]),
        // Flatpak search
        (&["flatpak", "search"], "search flatpak", "flatpak",
         &["echo 'Use: flatpak search <app>'"]),
        // Flatpak remotes
        (&["flatpak", "remotes"], "list flatpak remotes", "flatpak",
         &["flatpak remotes"]),
        (&["flatpak", "repos"], "list flatpak repositories", "flatpak",
         &["flatpak remotes -d"]),
        // Flatpak update
        (&["flatpak", "update"], "update flatpaks", "flatpak",
         &["echo 'Use: flatpak update'"]),
        (&["flatpak", "updates"], "check flatpak updates", "flatpak",
         &["flatpak remote-ls --updates"]),
        // Flatpak version
        (&["flatpak", "version"], "show flatpak version", "flatpak",
         &["flatpak --version"]),
        // Flatpak info
        (&["flatpak", "info"], "show flatpak info", "flatpak",
         &["echo 'Use: flatpak info <app-id>'"]),
        // Flatpak size
        (&["flatpak", "size"], "show flatpak disk usage", "flatpak",
         &["du -sh ~/.var/app/ 2>/dev/null", "du -sh /var/lib/flatpak/ 2>/dev/null"]),
        // Flatpak runtimes
        (&["flatpak", "runtimes"], "list flatpak runtimes", "flatpak",
         &["flatpak list --runtime"]),
        // Flatpak permissions
        (&["flatpak", "permissions"], "show flatpak permissions", "flatpak",
         &["echo 'Use: flatpak info --show-permissions <app-id>'", "flatpak permission-list 2>/dev/null"]),
        // Flathub
        (&["flathub", "add"], "add flathub remote", "flatpak",
         &["echo 'Use: flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo'"]),
        (&["flathub", "status"], "check flathub status", "flatpak",
         &["flatpak remotes | grep flathub"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Snap patterns
fn match_snap(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AppPattern] = &[
        // Snap list
        (&["snap", "list"], "list installed snaps", "snap",
         &["snap list 2>/dev/null || echo 'snapd not installed'"]),
        (&["installed", "snaps"], "list installed snaps", "snap",
         &["snap list"]),
        (&["snap", "installed"], "list installed snaps", "snap",
         &["snap list"]),
        // Snap search
        (&["snap", "search"], "search snap store", "snap",
         &["echo 'Use: snap find <app>'"]),
        // Snap info
        (&["snap", "info"], "show snap info", "snap",
         &["echo 'Use: snap info <snap>'"]),
        // Snap version
        (&["snap", "version"], "show snap version", "snap",
         &["snap version 2>/dev/null || echo 'snapd not installed'"]),
        // Snapd status
        (&["snapd", "status"], "show snapd status", "snap",
         &["systemctl status snapd"]),
        (&["snapd", "running"], "check if snapd is running", "snap",
         &["systemctl is-active snapd"]),
        // Snap refresh
        (&["snap", "refresh"], "refresh snaps", "snap",
         &["echo 'Use: snap refresh'"]),
        (&["snap", "updates"], "check snap updates", "snap",
         &["snap refresh --list 2>/dev/null"]),
        // Snap connections
        (&["snap", "connections"], "show snap connections", "snap",
         &["snap connections --all 2>/dev/null | head -30"]),
        // Snap services
        (&["snap", "services"], "list snap services", "snap",
         &["snap services"]),
        // Snap size
        (&["snap", "size"], "show snap disk usage", "snap",
         &["du -sh /snap/ 2>/dev/null", "du -sh ~/snap/ 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// AppImage patterns
fn match_appimage(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AppPattern] = &[
        // AppImage list
        (&["appimage", "list"], "list AppImages", "appimage",
         &["find ~ -name '*.AppImage' -type f 2>/dev/null | head -20", "ls ~/Applications/*.AppImage 2>/dev/null"]),
        (&["appimages"], "list AppImages", "appimage",
         &["find ~ -name '*.AppImage' -type f 2>/dev/null | head -20"]),
        // AppImage run
        (&["appimage", "run"], "run AppImage", "appimage",
         &["echo 'Make executable: chmod +x file.AppImage && ./file.AppImage'"]),
        // AppImage extract
        (&["appimage", "extract"], "extract AppImage", "appimage",
         &["echo 'Use: ./file.AppImage --appimage-extract'"]),
        // AppImage integrate
        (&["appimage", "integrate"], "integrate AppImage", "appimage",
         &["echo 'Install appimaged for desktop integration'", "which appimaged 2>/dev/null"]),
        // AppImageLauncher
        (&["appimagelauncher"], "check AppImageLauncher", "appimage",
         &["which AppImageLauncher 2>/dev/null", "pacman -Q appimagelauncher 2>/dev/null"]),
        // FUSE for AppImage
        (&["appimage", "fuse"], "check FUSE for AppImage", "appimage",
         &["which fusermount 2>/dev/null", "lsmod | grep fuse"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// App container troubleshooting
fn match_app_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AppPattern] = &[
        // Flatpak repair
        (&["flatpak", "repair"], "repair flatpak", "flatpak",
         &["echo 'Use: flatpak repair'"]),
        // Flatpak clean
        (&["flatpak", "clean"], "clean unused flatpak data", "flatpak",
         &["flatpak uninstall --unused"]),
        (&["flatpak", "unused"], "remove unused flatpak runtimes", "flatpak",
         &["flatpak uninstall --unused"]),
        // Snap disable
        (&["disable", "snap"], "disable snapd", "snap",
         &["echo 'Use: systemctl disable --now snapd'"]),
        (&["remove", "snapd"], "remove snapd", "snap",
         &["echo 'Use: pacman -R snapd'"]),
        // Flatpak vs native
        (&["flatpak", "native"], "compare flatpak vs native", "flatpak",
         &["echo 'Flatpak: sandboxed, larger. Native: smaller, faster.'"]),
        // App permissions
        (&["app", "permissions"], "check app permissions", "flatpak",
         &["flatpak permission-list 2>/dev/null"]),
        // Flatpak overrides
        (&["flatpak", "overrides"], "show flatpak overrides", "flatpak",
         &["ls ~/.local/share/flatpak/overrides/ 2>/dev/null", "cat ~/.local/share/flatpak/overrides/* 2>/dev/null"]),
        // XDG portals
        (&["xdg", "portal"], "check XDG desktop portals", "flatpak",
         &["pgrep -a 'portal'", "pacman -Qs xdg-desktop-portal | head -10"]),
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
    fn test_flatpak() {
        assert!(match_patterns("flatpak list").is_some());
        assert!(match_patterns("flatpak remotes").is_some());
        assert!(match_patterns("flatpak version").is_some());
    }

    #[test]
    fn test_snap() {
        assert!(match_patterns("snap list").is_some());
        assert!(match_patterns("snap version").is_some());
        assert!(match_patterns("snapd status").is_some());
    }

    #[test]
    fn test_appimage() {
        assert!(match_patterns("appimage list").is_some());
        assert!(match_patterns("appimages").is_some());
    }

    #[test]
    fn test_troubleshoot() {
        assert!(match_patterns("flatpak repair").is_some());
        assert!(match_patterns("flatpak clean").is_some());
        assert!(match_patterns("xdg portal").is_some());
    }
}

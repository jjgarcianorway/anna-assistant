//! AUR patterns for yay, paru, makepkg, AUR helpers.
//! v0.0.978: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an AUR-related DeepUnderstanding
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

type AurPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match AUR-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_yay(q)
        .or_else(|| match_paru(q))
        .or_else(|| match_makepkg(q))
        .or_else(|| match_aur_general(q))
        .or_else(|| match_aur_troubleshoot(q))
}

/// yay patterns
fn match_yay(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AurPattern] = &[
        // yay search
        (&["yay", "search"], "search AUR with yay", "aur",
         &["echo 'Use: yay -Ss <package>'"]),
        // yay install
        (&["yay", "install"], "install with yay", "aur",
         &["echo 'Use: yay -S <package>'"]),
        // yay update
        (&["yay", "update"], "update system with yay", "aur",
         &["echo 'Use: yay -Syu'"]),
        // yay version
        (&["yay", "version"], "show yay version", "aur",
         &["yay --version 2>/dev/null || echo 'yay not installed'"]),
        // yay installed
        (&["yay", "installed"], "check if yay is installed", "aur",
         &["which yay 2>/dev/null && yay --version"]),
        // yay clean
        (&["yay", "clean"], "clean yay cache", "aur",
         &["echo 'Use: yay -Sc to clean cache'"]),
        // yay stats
        (&["yay", "stats"], "show yay statistics", "aur",
         &["yay -P --stats 2>/dev/null"]),
        // yay foreign
        (&["yay", "foreign"], "list AUR packages (yay)", "aur",
         &["yay -Qm"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// paru patterns
fn match_paru(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AurPattern] = &[
        // paru search
        (&["paru", "search"], "search AUR with paru", "aur",
         &["echo 'Use: paru -Ss <package>'"]),
        // paru install
        (&["paru", "install"], "install with paru", "aur",
         &["echo 'Use: paru -S <package>'"]),
        // paru update
        (&["paru", "update"], "update system with paru", "aur",
         &["echo 'Use: paru -Syu'"]),
        // paru version
        (&["paru", "version"], "show paru version", "aur",
         &["paru --version 2>/dev/null || echo 'paru not installed'"]),
        // paru installed
        (&["paru", "installed"], "check if paru is installed", "aur",
         &["which paru 2>/dev/null && paru --version"]),
        // paru clean
        (&["paru", "clean"], "clean paru cache", "aur",
         &["echo 'Use: paru -Sc to clean cache'"]),
        // paru review
        (&["paru", "review"], "review PKGBUILD with paru", "aur",
         &["echo 'paru shows PKGBUILD diff by default'"]),
        // paru foreign
        (&["paru", "foreign"], "list AUR packages (paru)", "aur",
         &["paru -Qm"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// makepkg patterns
fn match_makepkg(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AurPattern] = &[
        // makepkg build
        (&["makepkg", "build"], "build package with makepkg", "aur",
         &["echo 'Use: makepkg -s to build with deps'"]),
        (&["makepkg", "install"], "build and install with makepkg", "aur",
         &["echo 'Use: makepkg -si to build and install'"]),
        // makepkg clean
        (&["makepkg", "clean"], "clean makepkg build", "aur",
         &["echo 'Use: makepkg -c to clean'"]),
        // makepkg srcinfo
        (&["makepkg", "srcinfo"], "generate .SRCINFO", "aur",
         &["echo 'Use: makepkg --printsrcinfo > .SRCINFO'"]),
        // makepkg options
        (&["makepkg", "options"], "show makepkg options", "aur",
         &["makepkg --help 2>&1 | head -30"]),
        // PKGBUILD
        (&["pkgbuild"], "show PKGBUILD info", "aur",
         &["cat PKGBUILD 2>/dev/null | head -30 || echo 'No PKGBUILD in current directory'"]),
        (&["edit", "pkgbuild"], "edit PKGBUILD", "aur",
         &["echo 'Use: $EDITOR PKGBUILD'"]),
        // makepkg deps
        (&["makepkg", "deps"], "install makepkg dependencies", "aur",
         &["echo 'Use: makepkg -s or makepkg --syncdeps'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// General AUR patterns
fn match_aur_general(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AurPattern] = &[
        // AUR helper
        (&["aur", "helper"], "show installed AUR helpers", "aur",
         &["which yay paru pikaur trizen 2>/dev/null"]),
        (&["aur", "helpers"], "list AUR helpers", "aur",
         &["which yay paru pikaur trizen aurman 2>/dev/null"]),
        // AUR packages
        (&["aur", "packages"], "list installed AUR packages", "aur",
         &["pacman -Qm"]),
        (&["foreign", "packages"], "list foreign packages", "aur",
         &["pacman -Qm"]),
        (&["installed", "aur"], "list installed AUR packages", "aur",
         &["pacman -Qm"]),
        // AUR updates
        (&["aur", "updates"], "check for AUR updates", "aur",
         &["yay -Qua 2>/dev/null || paru -Qua 2>/dev/null || echo 'No AUR helper found'"]),
        (&["check", "aur"], "check AUR for updates", "aur",
         &["yay -Qua 2>/dev/null || paru -Qua 2>/dev/null"]),
        // AUR cache
        (&["aur", "cache"], "show AUR cache location", "aur",
         &["ls ~/.cache/yay/ 2>/dev/null | head -10", "ls ~/.cache/paru/clone/ 2>/dev/null | head -10"]),
        // Install from AUR
        (&["install", "aur"], "install from AUR", "aur",
         &["echo 'Use: yay -S <package> or paru -S <package>'"]),
        // AUR orphans
        (&["aur", "orphans"], "check for orphaned AUR packages", "aur",
         &["pacman -Qdt"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// AUR troubleshooting patterns
fn match_aur_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AurPattern] = &[
        // AUR build failed
        (&["aur", "build", "failed"], "troubleshoot AUR build failure", "aur",
         &["echo 'Check PKGBUILD for errors, try makepkg -s'", "echo 'Common issues: missing deps, outdated package'"]),
        (&["makepkg", "failed"], "troubleshoot makepkg failure", "aur",
         &["echo 'Check for missing makedepends'", "echo 'Try: makepkg -s --noconfirm'"]),
        // PGP key issues
        (&["aur", "pgp"], "fix AUR PGP key issues", "aur",
         &["echo 'Use: gpg --recv-keys <KEY_ID>'", "echo 'Or add to PKGBUILD: validpgpkeys=()'"] ),
        (&["gpg", "key", "aur"], "import GPG key for AUR", "aur",
         &["echo 'Use: gpg --keyserver keyserver.ubuntu.com --recv-keys <KEY_ID>'"]),
        // AUR outdated
        (&["aur", "outdated"], "find outdated AUR packages", "aur",
         &["yay -Qua 2>/dev/null || paru -Qua 2>/dev/null"]),
        // AUR conflicts
        (&["aur", "conflict"], "resolve AUR conflicts", "aur",
         &["echo 'Remove conflicting package first: pacman -R <pkg>'"]),
        // Rebuild AUR
        (&["rebuild", "aur"], "rebuild AUR packages", "aur",
         &["echo 'Use: yay -S --rebuild <package>'", "echo 'Or: paru -S --rebuild <package>'"]),
        // AUR clean cache
        (&["clean", "aur", "cache"], "clean AUR cache", "aur",
         &["rm -rf ~/.cache/yay/* 2>/dev/null", "rm -rf ~/.cache/paru/clone/* 2>/dev/null"]),
        // Checksum mismatch
        (&["checksum", "mismatch"], "fix checksum mismatch", "aur",
         &["echo 'Update PKGBUILD checksums: updpkgsums'", "echo 'Or skip with makepkg --skipchecksums'"]),
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
    fn test_yay() {
        assert!(match_patterns("yay version").is_some());
        assert!(match_patterns("yay installed").is_some());
        assert!(match_patterns("yay stats").is_some());
    }

    #[test]
    fn test_paru() {
        assert!(match_patterns("paru version").is_some());
        assert!(match_patterns("paru installed").is_some());
    }

    #[test]
    fn test_makepkg() {
        assert!(match_patterns("makepkg build").is_some());
        assert!(match_patterns("pkgbuild").is_some());
    }

    #[test]
    fn test_aur_general() {
        assert!(match_patterns("aur helper").is_some());
        assert!(match_patterns("aur packages").is_some());
        assert!(match_patterns("foreign packages").is_some());
    }

    #[test]
    fn test_aur_troubleshoot() {
        assert!(match_patterns("aur build failed").is_some());
        assert!(match_patterns("makepkg failed").is_some());
        assert!(match_patterns("checksum mismatch").is_some());
    }
}

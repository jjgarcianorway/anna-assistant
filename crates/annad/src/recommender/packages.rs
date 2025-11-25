//! Packages recommendations

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_orphan_packages(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.orphan_packages.is_empty() {
        let count = facts.orphan_packages.len();
        let msg = if count == 1 {
            "Clean up 1 unused package".to_string()
        } else {
            format!("Clean up {} unused packages", count)
        };
        result.push(Advice {
            id: "orphan-packages".to_string(),
            title: msg,
            reason: format!("You have {} {} that were installed to support other software, but nothing needs them anymore. They're just taking up space.",
                count, if count == 1 { "package" } else { "packages" }),
            action: "Remove unused packages to free up disk space".to_string(),
            command: Some("pacman -Rns --noconfirm $(pacman -Qdtq)".to_string()),
            risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman/Tips_and_tricks".to_string()],
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

pub(crate) fn check_system_updates() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if updates are available
    let output = Command::new("pacman").args(&["-Qu"]).output();

    if let Ok(output) = output {
        let updates = String::from_utf8_lossy(&output.stdout);
        let update_count = updates.lines().count();

        if update_count > 0 {
            let msg = if update_count == 1 {
                "1 package update available".to_string()
            } else {
                format!("{} package updates available", update_count)
            };
            let package_str = if update_count == 1 {
                "1 package".to_string()
            } else {
                format!("{} packages", update_count)
            };
            result.push(Advice {
                id: "system-updates".to_string(),
                title: msg,
                reason: format!("There {} {} waiting to be updated. Updates usually include security fixes, bug fixes, and new features.",
                    if update_count == 1 { "is" } else { "are" },
                    package_str),
                action: "Update your system to stay secure and up-to-date".to_string(),
                command: Some("pacman -Syu --noconfirm".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/System_maintenance".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_pacman_config() -> Vec<Advice> {
    let mut result = Vec::new();

    // Read pacman.conf
    if let Ok(config) = std::fs::read_to_string("/etc/pacman.conf") {
        // Check for Color
        if !config.lines().any(|l| l.trim() == "Color") {
            result.push(Advice {
                id: "pacman-color".to_string(),
                title: "Make pacman output colorful".to_string(),
                reason: "Right now pacman (the package manager) shows everything in plain text. Turning on colors makes it much easier to see what's being installed, updated, or removed.".to_string(),
                action: "Enable colored output in pacman".to_string(),
                command: Some("sed -i 's/^#Color/Color/' /etc/pacman.conf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Customization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Enabling_color_output".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for ParallelDownloads
        if !config
            .lines()
            .any(|l| l.trim().starts_with("ParallelDownloads"))
        {
            result.push(Advice {
                id: "pacman-parallel".to_string(),
                title: "Download packages 5x faster".to_string(),
                reason: "By default, pacman downloads one package at a time. Enabling parallel downloads lets it download 5 packages simultaneously, making updates much faster.".to_string(),
                action: "Enable parallel downloads in pacman".to_string(),
                command: Some("echo 'ParallelDownloads = 5' >> /etc/pacman.conf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Performance & Optimization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Enabling_parallel_downloads".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_aur_helper() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if yay is installed
    let has_yay = Command::new("pacman")
        .args(&["-Q", "yay"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if paru is installed
    let has_paru = Command::new("pacman")
        .args(&["-Q", "paru"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_yay && !has_paru {
        result.push(Advice {
            id: "aur-helper".to_string(),
            title: "Install an AUR helper to access thousands more packages".to_string(),
            reason: "The AUR (Arch User Repository) has over 85,000 community packages that aren't in the official repos. An AUR helper like 'yay' or 'paru' makes it super easy to install them - just like using pacman. Think of it as unlocking the full power of Arch!".to_string(),
            action: "Install yay to access AUR packages easily".to_string(),
            command: Some("pacman -S --needed git base-devel && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si --noconfirm".to_string()),
            risk: RiskLevel::Medium,
            priority: Priority::Recommended,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers".to_string()],
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

pub(crate) fn check_reflector() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if reflector is installed
    let has_reflector = Command::new("pacman")
        .args(&["-Q", "reflector"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_reflector {
        result.push(Advice {
            id: "reflector".to_string(),
            title: "Speed up downloads with better mirrors".to_string(),
            reason: "Reflector automatically finds the fastest Arch mirrors near you and updates your mirror list. This can make package downloads much faster - sometimes 10x faster if you're currently using a slow mirror!".to_string(),
            action: "Install reflector to optimize your mirror list".to_string(),
            command: Some("pacman -S --noconfirm reflector".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Reflector".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    } else {
        // Check when mirrorlist was last updated
        if let Ok(metadata) = std::fs::metadata("/etc/pacman.d/mirrorlist") {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    let days_old = elapsed.as_secs() / 86400;
                    if days_old > 30 {
                        result.push(Advice {
                            id: "reflector-update".to_string(),
                            title: format!("Your mirror list is {} days old", days_old),
                            reason: "Your mirror list hasn't been updated in over a month. Mirrors can change speed over time, and new faster ones might be available. Running reflector will find you the best mirrors right now.".to_string(),
                            action: "Update your mirror list with reflector".to_string(),
                            command: Some("sudo reflector --latest 20 --protocol https --sort rate --save /etc/pacman.d/mirrorlist".to_string()),
                            risk: RiskLevel::Medium,
                            priority: Priority::Recommended,
                            category: "Performance & Optimization".to_string(),
                            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Reflector".to_string()],
                                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
                    }
                }
            }
        }
    }

    result
}

pub(crate) fn check_flatpak() -> Vec<Advice> {
    let mut result = Vec::new();

    let has_flatpak = Command::new("which")
        .arg("flatpak")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_flatpak {
        result.push(Advice {
            id: "install-flatpak".to_string(),
            title: "Install Flatpak for universal app support".to_string(),
            reason: "Flatpak provides access to thousands of desktop applications from Flathub. Many proprietary apps (Spotify, Discord, Slack) are easier to install via Flatpak. It's sandboxed for better security and doesn't conflict with pacman packages!".to_string(),
            action: "Install Flatpak and enable Flathub".to_string(),
            command: Some("sudo pacman -S --noconfirm flatpak && flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Utilities".to_string(),
            alternatives: vec![
                Alternative {
                    name: "AppImage".to_string(),
                    description: "Portable app format (no sandbox)".to_string(),
                    install_command: "yay -S --noconfirm appimaged".to_string(),
                },
                Alternative {
                    name: "Snap".to_string(),
                    description: "Ubuntu's universal package format".to_string(),
                    install_command: "yay -S --noconfirm snapd".to_string(),
                },
            ],
            wiki_refs: vec!["https://wiki.archlinux.org/title/Flatpak".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
            satisfies: Vec::new(),
            popularity: 75,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_aur_helper_safety() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check which AUR helper is installed
    let has_yay = Command::new("pacman")
        .args(&["-Q", "yay"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let has_paru = Command::new("pacman")
        .args(&["-Q", "paru"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let has_pikaur = Command::new("pacman")
        .args(&["-Q", "pikaur"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_yay || has_paru || has_pikaur {
        result.push(Advice {
            id: "aur-safety-reminder".to_string(),
            title: "AUR Safety Reminder: Always review PKGBUILDs".to_string(),
            reason: "You're using an AUR helper - that's great! But remember: ALWAYS review the PKGBUILD before installing AUR packages. AUR packages can run any code during installation, so malicious packages could compromise your system. Think of it like downloading a random script from the internet - check it first!".to_string(),
            action: "Always review PKGBUILDs (use --editmenu or --show)".to_string(),
            command: Some("pacman -Qm | head -10".to_string()), // Show installed AUR packages
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers#Safety".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });

        // Check if user has devel packages installed (needs regular updates)
        let devel_packages = Command::new("pacman")
            .args(&["-Qq"])
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|line| {
                        line.ends_with("-git") || line.ends_with("-svn") || line.ends_with("-hg")
                    })
                    .count()
            })
            .unwrap_or(0);

        if devel_packages > 0 {
            result.push(Advice {
                id: "aur-devel-update".to_string(),
                title: format!("You have {} -git/-svn development packages", devel_packages),
                reason: "Development packages (-git, -svn, -hg) don't get automatic updates! They track upstream development, so you need to rebuild them periodically to get new features and fixes. Run your AUR helper with the devel flag (yay -Syu --devel or paru -Syu --devel) to update them.".to_string(),
                action: "Rebuild development packages regularly".to_string(),
                command: if has_yay { Some("yay -Syu --devel".to_string()) } else if has_paru { Some("paru -Syu --devel".to_string()) } else { None },
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_pkgbuild_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user builds packages
    let builds_packages = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "PKGBUILD",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if builds_packages {
        // Check for namcap
        let has_namcap = Command::new("pacman")
            .args(&["-Q", "namcap"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_namcap {
            result.push(Advice {
                id: "pkgbuild-namcap".to_string(),
                title: "Install namcap for PKGBUILD linting".to_string(),
                reason: "You build packages! namcap checks PKGBUILDs for errors, missing dependencies, naming issues, and packaging problems. Essential for AUR maintainers or anyone building custom packages. Run 'namcap PKGBUILD' before uploading to AUR!".to_string(),
                action: "Install namcap".to_string(),
                command: Some("pacman -S --noconfirm namcap".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Namcap".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for devtools
        let has_devtools = Command::new("pacman")
            .args(&["-Q", "devtools"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_devtools {
            result.push(Advice {
                id: "pkgbuild-devtools".to_string(),
                title: "Install devtools for clean chroot builds".to_string(),
                reason: "Build packages in clean chroots! devtools provides 'extra-x86_64-build' and friends - build in isolated environment, catch missing dependencies, ensure reproducibility. Professional package builders use this. If you're serious about packaging, you need devtools!".to_string(),
                action: "Install devtools".to_string(),
                command: Some("pacman -S --noconfirm devtools".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/DeveloperWiki:Building_in_a_clean_chroot".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_pacman_hooks_recommendations(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.pacman_hooks.iter().any(|h| h.contains("orphans")) {
        result.push(Advice::new(
            "pacman-hook-orphans".to_string(),
            "Add Pacman Hook to List Orphan Packages".to_string(),
            "Create a pacman hook that automatically lists orphaned packages after package operations, helping identify packages to remove and keep your system clean.".to_string(),
            "Create hook to automatically show orphaned packages after pacman operations".to_string(),
            Some("echo '[Trigger]\\nOperation = Remove\\nOperation = Install\\nOperation = Upgrade\\nType = Package\\nTarget = *\\n\\n[Action]\\nDescription = Listing orphaned packages...\\nWhen = PostTransaction\\nExec = /bin/sh -c \"/usr/bin/pacman -Qtdq || echo No orphans found\"' | sudo tee /etc/pacman.d/hooks/orphans.hook".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Pacman#Hooks".to_string()],
            "maintenance".to_string(),
        ).with_bundle("Pacman Enhancements".to_string())
         .with_popularity(35));
    }

    result
}


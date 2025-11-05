//! Smart package recommendation engine
//!
//! Analyzes user's workflow and suggests useful packages

use anna_common::{Advice, Priority, RiskLevel, SystemFacts};
use tracing::info;

/// Generate smart package recommendations based on detected workflow
pub fn generate_smart_recommendations(facts: &SystemFacts) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating smart package recommendations based on workflow");

    // Development workflow recommendations
    recommendations.extend(recommend_for_development(&facts.development_environment));

    // Gaming workflow recommendations
    recommendations.extend(recommend_for_gaming(&facts.gaming_profile));

    // Desktop environment enhancements
    if let Some(ref de) = facts.desktop_environment {
        recommendations.extend(recommend_for_desktop(de));
    }

    // Network tools based on profile
    recommendations.extend(recommend_for_networking(&facts.network_profile));

    // Content creation tools
    if facts.user_preferences.is_content_creator {
        recommendations.extend(recommend_for_content_creation());
    }

    // Laptop-specific recommendations
    if facts.user_preferences.uses_laptop {
        recommendations.extend(recommend_for_laptop());
    }

    recommendations
}

/// Recommend development tools based on detected languages
fn recommend_for_development(profile: &anna_common::DevelopmentProfile) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    for lang_usage in &profile.languages {
        match lang_usage.language.as_str() {
            "Python" if !lang_usage.has_lsp => {
                recommendations.push(Advice::new(
                    "python-lsp-server".to_string(),
                    "Install Python Language Server (pyright)".to_string(),
                    format!("You have {} Python files but no LSP server installed. A language server provides autocomplete, type checking, and refactoring in your editor.", lang_usage.file_count),
                    "Install pyright for Python development".to_string(),
                    Some("sudo pacman -S --noconfirm pyright".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Python".to_string()],
                    "development".to_string(),
                ));
            }
            "Rust" if !lang_usage.has_lsp => {
                recommendations.push(Advice::new(
                    "rust-analyzer".to_string(),
                    "Install Rust Language Server (rust-analyzer)".to_string(),
                    format!("You have {} Rust files but no LSP server installed. rust-analyzer provides excellent IDE features for Rust development.", lang_usage.file_count),
                    "Install rust-analyzer for Rust development".to_string(),
                    Some("sudo pacman -S --noconfirm rust-analyzer".to_string()),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec!["https://wiki.archlinux.org/title/Rust".to_string()],
                    "development".to_string(),
                ));
            }
            "JavaScript" | "TypeScript" if !lang_usage.has_lsp => {
                recommendations.push(Advice::new(
                    "typescript-language-server".to_string(),
                    "Install TypeScript Language Server".to_string(),
                    format!("You have {} {}/{} files but no LSP server installed. TypeScript LSP works for both JavaScript and TypeScript.", lang_usage.file_count, lang_usage.language, "JavaScript"),
                    "Install typescript-language-server".to_string(),
                    Some("sudo pacman -S --noconfirm typescript-language-server".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Node.js".to_string()],
                    "development".to_string(),
                ));
            }
            "Go" if !lang_usage.has_lsp => {
                recommendations.push(Advice::new(
                    "gopls".to_string(),
                    "Install Go Language Server (gopls)".to_string(),
                    format!("You have {} Go files but no LSP server installed. gopls is the official Go language server.", lang_usage.file_count),
                    "Install gopls for Go development".to_string(),
                    Some("sudo pacman -S --noconfirm gopls".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Go".to_string()],
                    "development".to_string(),
                ));
            }
            _ => {}
        }
    }

    // Recommend debuggers
    if profile.languages.iter().any(|l| l.language == "C" || l.language == "C++") {
        recommendations.push(Advice::new(
            "gdb-enhanced".to_string(),
            "Install GDB with enhancements for C/C++".to_string(),
            "GDB is essential for debugging C/C++ programs. Consider gdb with python support.".to_string(),
            "Install GDB debugger".to_string(),
            Some("sudo pacman -S --noconfirm gdb".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Debugging".to_string()],
            "development".to_string(),
        ));
    }

    // Docker/Podman completion tools
    if profile.uses_containers {
        recommendations.push(Advice::new(
            "docker-compose".to_string(),
            "Install Docker Compose for multi-container apps".to_string(),
            "You're using Docker. Docker Compose makes it easy to manage multi-container applications.".to_string(),
            "Install docker-compose".to_string(),
            Some("sudo pacman -S --noconfirm docker-compose".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Docker".to_string()],
            "development".to_string(),
        ));
    }

    recommendations
}

/// Recommend gaming enhancements
fn recommend_for_gaming(profile: &anna_common::GamingProfile) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    if profile.steam_installed && !profile.proton_ge_installed {
        recommendations.push(Advice::new(
            "protonup-qt".to_string(),
            "Install ProtonUp-Qt for better game compatibility".to_string(),
            "You have Steam installed. ProtonUp-Qt lets you easily install Proton-GE for better game compatibility and performance.".to_string(),
            "Install ProtonUp-Qt to manage Proton versions".to_string(),
            Some("yay -S --noconfirm protonup-qt".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Steam".to_string()],
            "gaming".to_string(),
        ));
    }

    if profile.steam_installed && !profile.mangohud_installed {
        recommendations.push(Advice::new(
            "mangohud".to_string(),
            "Install MangoHud for in-game performance overlay".to_string(),
            "MangoHud shows FPS, GPU/CPU usage, and temperatures in games. Great for monitoring performance.".to_string(),
            "Install MangoHud for performance monitoring".to_string(),
            Some("sudo pacman -S --noconfirm mangohud".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Gaming#Performance_overlays".to_string()],
            "gaming".to_string(),
        ));
    }

    if (profile.steam_installed || profile.lutris_installed) && !profile.uses_gamepad {
        recommendations.push(Advice::new(
            "gamepad-support".to_string(),
            "Consider installing gamepad support tools".to_string(),
            "You're gaming on Linux. Installing gamepad tools like xpadneo (Xbox) or ds4drv (PS4) can improve controller support.".to_string(),
            "Install gamepad drivers if needed".to_string(),
            Some("# For Xbox controllers: yay -S xpadneo-dkms\n# For PS4 controllers: sudo pacman -S --noconfirm ds4drv".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Gamepad".to_string()],
            "gaming".to_string(),
        ));
    }

    recommendations
}

/// Recommend desktop environment enhancements
fn recommend_for_desktop(de: &str) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    match de.to_lowercase().as_str() {
        de_str if de_str.contains("gnome") => {
            recommendations.push(Advice::new(
                "gnome-tweaks".to_string(),
                "Install GNOME Tweaks for customization".to_string(),
                "GNOME Tweaks provides access to many settings not available in the default settings app.".to_string(),
                "Install gnome-tweaks".to_string(),
                Some("sudo pacman -S --noconfirm gnome-tweaks".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/GNOME".to_string()],
                "desktop".to_string(),
            ));
        }
        de_str if de_str.contains("kde") || de_str.contains("plasma") => {
            recommendations.push(Advice::new(
                "kde-gtk-config".to_string(),
                "Install GTK theme integration for KDE".to_string(),
                "This package allows you to set GTK themes from KDE System Settings, ensuring consistent theming.".to_string(),
                "Install kde-gtk-config".to_string(),
                Some("sudo pacman -S --noconfirm kde-gtk-config".to_string()),
                RiskLevel::Low,
                Priority::Cosmetic,
                vec!["https://wiki.archlinux.org/title/KDE".to_string()],
                "desktop".to_string(),
            ));
        }
        _ => {}
    }

    recommendations
}

/// Recommend networking tools based on profile
fn recommend_for_networking(profile: &anna_common::NetworkProfile) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    if !profile.vpn_configured {
        recommendations.push(Advice::new(
            "wireguard-tools".to_string(),
            "Consider setting up a VPN (WireGuard)".to_string(),
            "WireGuard is a modern, fast VPN that's easy to set up. Great for secure remote access.".to_string(),
            "Install WireGuard for VPN functionality".to_string(),
            Some("sudo pacman -S --noconfirm wireguard-tools".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/WireGuard".to_string()],
            "networking".to_string(),
        ));
    }

    if !profile.ssh_server_running {
        recommendations.push(Advice::new(
            "openssh-server".to_string(),
            "Consider installing OpenSSH server for remote access".to_string(),
            "OpenSSH server allows you to access your machine remotely. Useful for development and administration.".to_string(),
            "Install and enable OpenSSH server".to_string(),
            Some("sudo pacman -S --noconfirm openssh && sudo systemctl enable --now sshd".to_string()),
            RiskLevel::Medium,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/OpenSSH".to_string()],
            "networking".to_string(),
        ));
    }

    recommendations
}

/// Recommend content creation tools
fn recommend_for_content_creation() -> Vec<Advice> {
    vec![
        Advice::new(
            "obs-studio-plugins".to_string(),
            "Install OBS Studio plugins for enhanced streaming".to_string(),
            "You have OBS Studio installed. Consider adding plugins like obs-websocket for remote control.".to_string(),
            "Install OBS plugins".to_string(),
            Some("sudo pacman -S --noconfirm obs-studio-plugin-websocket".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Open_Broadcaster_Software".to_string()],
            "multimedia".to_string(),
        ),
    ]
}

/// Recommend laptop-specific tools
fn recommend_for_laptop() -> Vec<Advice> {
    vec![
        Advice::new(
            "tlp-power".to_string(),
            "Install TLP for better battery life".to_string(),
            "TLP is an advanced power management tool that can significantly extend your laptop's battery life.".to_string(),
            "Install TLP for power management".to_string(),
            Some("sudo pacman -S --noconfirm tlp && sudo systemctl enable tlp".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/TLP".to_string()],
            "power".to_string(),
        ),
        Advice::new(
            "powertop".to_string(),
            "Install powertop for power usage analysis".to_string(),
            "Powertop helps you identify what's consuming power on your laptop and provides tuning suggestions.".to_string(),
            "Install powertop".to_string(),
            Some("sudo pacman -S --noconfirm powertop".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Powertop".to_string()],
            "power".to_string(),
        ),
    ]
}

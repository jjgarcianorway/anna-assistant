//! Object Metadata v5.2.5 - Descriptions, Relationships, and Classification
//!
//! Provides rich metadata for knowledge objects:
//! - Human-readable descriptions derived from ArchWiki/man/help
//! - Relationship detection (e.g., aquamarine -> hyprland)
//! - Extended category classification


/// Object relationship information
#[derive(Debug, Clone)]
pub struct ObjectRelationship {
    /// The parent/related object name
    pub related_to: String,
    /// Type of relationship
    pub relationship_type: RelationshipType,
}

/// Types of relationships between objects
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    /// Plugin/addon for another tool
    Plugin,
    /// Part of a larger suite
    PartOf,
    /// Configuration tool for another
    ConfigTool,
    /// Theme/style for another
    Theme,
    /// Backend/library for another
    Backend,
    /// Provides functionality to another
    DependencyOf,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::Plugin => "plugin",
            RelationshipType::PartOf => "part of",
            RelationshipType::ConfigTool => "config tool",
            RelationshipType::Theme => "theme",
            RelationshipType::Backend => "backend",
            RelationshipType::DependencyOf => "dependency",
        }
    }
}

/// Get a human-readable description for a known object
/// Derived from ArchWiki, man pages, and --help output
pub fn get_description(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();

    // Editors
    match lower.as_str() {
        "vim" | "nvim" | "neovim" => Some("Vi-improved text editor for the terminal"),
        "nano" => Some("Simple terminal text editor"),
        "code" | "code-oss" | "vscode" => Some("Visual Studio Code editor"),
        "emacs" => Some("Extensible, customizable text editor"),
        "helix" | "hx" => Some("Post-modern modal text editor"),
        "kate" => Some("KDE advanced text editor"),
        "gedit" => Some("GNOME text editor"),

        // Terminals
        "alacritty" => Some("GPU-accelerated terminal emulator"),
        "kitty" => Some("Fast, feature-rich GPU-based terminal"),
        "wezterm" => Some("GPU-accelerated cross-platform terminal"),
        "foot" => Some("Fast, lightweight Wayland terminal"),
        "gnome-terminal" => Some("GNOME terminal emulator"),
        "konsole" => Some("KDE terminal emulator"),
        "st" => Some("Simple terminal from suckless"),

        // Shells
        "zsh" => Some("Z shell with advanced features"),
        "bash" => Some("Bourne Again Shell"),
        "fish" => Some("Friendly interactive shell"),
        "nushell" | "nu" => Some("Modern shell with structured data"),

        // Window Managers
        "i3" => Some("Tiling window manager for X11"),
        "sway" => Some("i3-compatible Wayland compositor"),
        "awesome" => Some("Highly configurable X11 window manager"),
        "bspwm" => Some("Tiling window manager based on binary space partitioning"),
        "dwm" => Some("Dynamic window manager from suckless"),
        "openbox" => Some("Stacking window manager for X11"),

        // Compositors
        "hyprland" => Some("Dynamic tiling Wayland compositor"),
        "wayfire" => Some("3D Wayland compositor"),
        "river" => Some("Dynamic tiling Wayland compositor"),
        "picom" => Some("Compositor for X11 with blur and animations"),

        // Hyprland ecosystem
        "aquamarine" => Some("Wayland rendering backend for Hyprland"),
        "hyprcursor" => Some("Cursor theme library for Hyprland"),
        "hyprutils" => Some("C++ utility library for Hyprland"),
        "hyprlang" => Some("Configuration language parser for Hyprland"),
        "hyprwayland-scanner" => Some("Wayland scanner for Hyprland"),
        "xdg-desktop-portal-hyprland" => Some("XDG desktop portal for Hyprland"),
        "hyprpaper" => Some("Wallpaper utility for Hyprland"),
        "hyprpicker" => Some("Color picker for Hyprland"),
        "hyprlock" => Some("Screen locker for Hyprland"),
        "hypridle" => Some("Idle daemon for Hyprland"),
        "hyprshot" => Some("Screenshot utility for Hyprland"),

        // Browsers
        "firefox" => Some("Mozilla Firefox web browser"),
        "chromium" | "chrome" | "google-chrome" => Some("Google Chrome web browser"),
        "brave" => Some("Privacy-focused Chromium-based browser"),
        "vivaldi" => Some("Feature-rich Chromium-based browser"),
        "qutebrowser" => Some("Keyboard-driven vim-like browser"),

        // Services/Bars
        "waybar" => Some("Highly customizable Wayland bar"),
        "polybar" => Some("Fast and easy-to-use status bar"),
        "dunst" => Some("Lightweight notification daemon"),
        "mako" => Some("Lightweight Wayland notification daemon"),
        "rofi" => Some("Application launcher and dmenu replacement"),
        "wofi" => Some("Wayland application launcher"),
        "fuzzel" => Some("Application launcher for Wayland"),

        // System services
        "systemd" => Some("System and service manager for Linux"),
        "sshd" | "openssh" => Some("OpenSSH secure shell daemon"),
        "networkmanager" => Some("Network connection manager"),
        "pipewire" => Some("Multimedia processing daemon"),
        "wireplumber" => Some("PipeWire session manager"),
        "pulseaudio" => Some("Sound server for Linux"),
        "bluetooth" | "bluez" => Some("Bluetooth protocol stack"),
        "cups" => Some("Common Unix Printing System"),
        "docker" => Some("Container runtime"),
        "ollama" => Some("Local LLM inference server"),

        // Development tools
        "git" => Some("Distributed version control system"),
        "cargo" => Some("Rust package manager and build tool"),
        "rustc" => Some("Rust compiler"),
        "python" | "python3" => Some("Python programming language interpreter"),
        "node" | "nodejs" => Some("JavaScript runtime"),
        "npm" => Some("Node.js package manager"),
        "gcc" => Some("GNU C/C++ compiler"),
        "clang" => Some("LLVM C/C++ compiler"),
        "make" => Some("Build automation tool"),
        "cmake" => Some("Cross-platform build system generator"),

        // CLI tools
        "ls" => Some("List directory contents"),
        "cat" => Some("Concatenate and print files"),
        "grep" => Some("Search text using patterns"),
        "find" => Some("Search for files in directories"),
        "sed" => Some("Stream editor for text transformation"),
        "awk" => Some("Pattern scanning and processing language"),
        "curl" => Some("Transfer data from URLs"),
        "wget" => Some("Network file retriever"),
        "tar" => Some("Archive utility"),
        "zip" | "unzip" => Some("Archive compression utility"),
        "htop" => Some("Interactive process viewer"),
        "btop" => Some("Resource monitor with TUI"),
        "neofetch" | "fastfetch" => Some("System information tool"),

        // Anna
        "annad" => Some("Anna daemon - system knowledge collector"),
        "annactl" => Some("Anna CLI - query system knowledge"),

        _ => None,
    }
}

/// Get relationship information for an object
/// Returns the parent object it relates to and the relationship type
pub fn get_relationship(name: &str) -> Option<ObjectRelationship> {
    let lower = name.to_lowercase();

    // Hyprland ecosystem
    match lower.as_str() {
        "aquamarine" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Backend,
        }),
        "hyprcursor" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "hyprutils" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Backend,
        }),
        "hyprlang" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Backend,
        }),
        "hyprwayland-scanner" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Backend,
        }),
        "xdg-desktop-portal-hyprland" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "hyprpaper" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "hyprpicker" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "hyprlock" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "hypridle" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),

        // Waybar/Polybar related
        "waybar" => Some(ObjectRelationship {
            related_to: "hyprland".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),

        // PipeWire ecosystem
        "wireplumber" => Some(ObjectRelationship {
            related_to: "pipewire".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "pipewire-pulse" => Some(ObjectRelationship {
            related_to: "pipewire".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),
        "pipewire-jack" => Some(ObjectRelationship {
            related_to: "pipewire".to_string(),
            relationship_type: RelationshipType::Plugin,
        }),

        // KDE ecosystem
        "kate" | "dolphin" | "konsole" | "kwin" => Some(ObjectRelationship {
            related_to: "plasma".to_string(),
            relationship_type: RelationshipType::PartOf,
        }),

        // GNOME ecosystem
        "gnome-terminal" | "nautilus" | "gedit" | "gnome-shell" => Some(ObjectRelationship {
            related_to: "gnome".to_string(),
            relationship_type: RelationshipType::PartOf,
        }),

        // Coreutils
        "ls" | "cat" | "cp" | "mv" | "rm" | "mkdir" | "chmod" | "chown" => Some(ObjectRelationship {
            related_to: "coreutils".to_string(),
            relationship_type: RelationshipType::PartOf,
        }),

        _ => None,
    }
}

/// Check if an object is in a known ecosystem
pub fn get_ecosystem(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();

    if lower.starts_with("hypr") || lower == "aquamarine" || lower == "xdg-desktop-portal-hyprland" {
        return Some("Hyprland");
    }
    if lower.starts_with("pipewire") || lower == "wireplumber" {
        return Some("PipeWire");
    }
    if lower.starts_with("kde") || lower.starts_with("k") && matches!(lower.as_str(), "kate" | "konsole" | "kwin" | "dolphin") {
        return Some("KDE Plasma");
    }
    if lower.starts_with("gnome") || matches!(lower.as_str(), "nautilus" | "gedit" | "evolution") {
        return Some("GNOME");
    }

    None
}

/// Get package metadata from pacman description
pub fn parse_pacman_description(desc: &str) -> Option<String> {
    // Take first sentence or first 80 chars
    let first_sentence = desc.split('.').next()?;
    let trimmed = first_sentence.trim();

    if trimmed.len() > 80 {
        Some(format!("{}...", &trimmed[..77]))
    } else if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_description() {
        assert_eq!(get_description("vim"), Some("Vi-improved text editor for the terminal"));
        assert_eq!(get_description("VIM"), Some("Vi-improved text editor for the terminal"));
        assert_eq!(get_description("hyprland"), Some("Dynamic tiling Wayland compositor"));
        assert_eq!(get_description("aquamarine"), Some("Wayland rendering backend for Hyprland"));
    }

    #[test]
    fn test_get_relationship() {
        let rel = get_relationship("aquamarine").unwrap();
        assert_eq!(rel.related_to, "hyprland");
        assert_eq!(rel.relationship_type, RelationshipType::Backend);

        let rel = get_relationship("wireplumber").unwrap();
        assert_eq!(rel.related_to, "pipewire");
    }

    #[test]
    fn test_get_ecosystem() {
        assert_eq!(get_ecosystem("hyprland"), Some("Hyprland"));
        assert_eq!(get_ecosystem("hyprcursor"), Some("Hyprland"));
        assert_eq!(get_ecosystem("aquamarine"), Some("Hyprland"));
        assert_eq!(get_ecosystem("pipewire"), Some("PipeWire"));
        assert_eq!(get_ecosystem("vim"), None);
    }
}

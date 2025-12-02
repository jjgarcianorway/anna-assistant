//! Category Evidence v7.32.0 - Evidence-Based Software Categorization
//!
//! Provides category assignments with full evidence trail.
//! No hardcoded app name lists - all categorization derived from:
//! 1. Desktop entries (.desktop files)
//! 2. Pacman package metadata (description, groups)
//! 3. Man page sections and descriptions
//!
//! Evidence is stored and can be queried for transparency.

use serde::{Deserialize, Serialize};

/// Confidence level for category assignment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// Strong evidence from multiple sources
    High,
    /// Single reliable source
    Medium,
    /// Inferred or weak evidence
    Low,
}

impl Confidence {
    pub fn label(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// Evidence source for category assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    /// From .desktop file Categories field
    DesktopCategories { path: String, categories: Vec<String> },
    /// From pacman package description
    PacmanDescription { description: String },
    /// From pacman package groups
    PacmanGroups { groups: Vec<String> },
    /// From man page section
    ManSection { section: u8, description: String },
    /// From Steam appmanifest
    SteamManifest { appid: u32, name: String },
    /// From Heroic/Legendary manifest
    HeroicManifest { app_name: String },
    /// From Lutris game entry
    LutrisEntry { slug: String },
    /// From Bottles entry
    BottlesEntry { name: String },
}

impl EvidenceSource {
    pub fn describe(&self) -> String {
        match self {
            Self::DesktopCategories { path, categories } => {
                format!("desktop: {} (Categories={})", path, categories.join(";"))
            }
            Self::PacmanDescription { description } => {
                let truncated = if description.len() > 50 {
                    format!("{}...", &description[..50])
                } else {
                    description.clone()
                };
                format!("pacman: description \"{}\"", truncated)
            }
            Self::PacmanGroups { groups } => {
                format!("pacman: groups [{}]", groups.join(", "))
            }
            Self::ManSection { section, description } => {
                format!("man({}): {}", section, description)
            }
            Self::SteamManifest { appid, name } => {
                format!("steam: appid {} \"{}\"", appid, name)
            }
            Self::HeroicManifest { app_name } => {
                format!("heroic: {}", app_name)
            }
            Self::LutrisEntry { slug } => {
                format!("lutris: {}", slug)
            }
            Self::BottlesEntry { name } => {
                format!("bottles: {}", name)
            }
        }
    }
}

/// A category assignment with evidence trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAssignment {
    /// The assigned category
    pub category: String,
    /// Confidence level
    pub confidence: Confidence,
    /// Evidence sources that led to this assignment
    pub evidence: Vec<EvidenceSource>,
}

impl CategoryAssignment {
    pub fn new(category: impl Into<String>, confidence: Confidence) -> Self {
        Self {
            category: category.into(),
            confidence,
            evidence: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, source: EvidenceSource) -> Self {
        self.evidence.push(source);
        self
    }

    pub fn add_evidence(&mut self, source: EvidenceSource) {
        self.evidence.push(source);
    }

    /// Format evidence for display
    pub fn format_evidence(&self) -> String {
        if self.evidence.is_empty() {
            return "(no evidence)".to_string();
        }
        self.evidence.iter()
            .map(|e| e.describe())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Classify software based on evidence from multiple sources
pub fn classify_software(name: &str) -> Option<CategoryAssignment> {
    let mut assignment: Option<CategoryAssignment> = None;

    // Priority 1: Desktop file categories
    if let Some(desktop_cat) = classify_from_desktop(name) {
        return Some(desktop_cat);
    }

    // Priority 2: Pacman metadata
    if let Some(pacman_cat) = classify_from_pacman(name) {
        assignment = Some(pacman_cat);
    }

    // Priority 3: Man page section (lower priority, may override if no pacman)
    if assignment.is_none() {
        if let Some(man_cat) = classify_from_man(name) {
            assignment = Some(man_cat);
        }
    }

    assignment
}

/// Classify from .desktop file Categories field
fn classify_from_desktop(name: &str) -> Option<CategoryAssignment> {
    let search_paths = [
        format!("/usr/share/applications/{}.desktop", name),
        format!("/usr/share/applications/{}.desktop", name.to_lowercase()),
        format!("{}/.local/share/applications/{}.desktop",
            std::env::var("HOME").unwrap_or_default(), name),
    ];

    for path in &search_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(categories) = parse_desktop_categories(&content) {
                if let Some(category) = map_desktop_to_anna_category(&categories) {
                    return Some(CategoryAssignment::new(category, Confidence::High)
                        .with_evidence(EvidenceSource::DesktopCategories {
                            path: path.clone(),
                            categories,
                        }));
                }
            }
        }
    }
    None
}

/// Parse Categories= line from .desktop file
fn parse_desktop_categories(content: &str) -> Option<Vec<String>> {
    for line in content.lines() {
        if line.starts_with("Categories=") {
            let value = line.trim_start_matches("Categories=");
            let categories: Vec<String> = value.split(';')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            if !categories.is_empty() {
                return Some(categories);
            }
        }
    }
    None
}

/// Map freedesktop.org categories to Anna categories
fn map_desktop_to_anna_category(categories: &[String]) -> Option<String> {
    // Skip generic categories, look for specific ones
    let generic = ["Application", "Utility", "GTK", "Qt", "GNOME", "KDE", "X-"];

    for cat in categories {
        // Skip generic categories
        if generic.iter().any(|g| cat.starts_with(g)) {
            continue;
        }

        // Map to Anna categories
        let mapped = match cat.as_str() {
            "Game" | "ActionGame" | "AdventureGame" | "ArcadeGame" | "BoardGame" |
            "BlocksGame" | "CardGame" | "KidsGame" | "LogicGame" | "RolePlaying" |
            "Shooter" | "Simulation" | "SportsGame" | "StrategyGame" => "Games",

            "AudioVideo" | "Audio" | "Video" | "Player" | "Recorder" |
            "Music" | "Mixer" | "Sequencer" | "Tuner" | "TV" => "Multimedia",

            "Development" | "IDE" | "Debugger" | "GUIDesigner" | "Profiling" |
            "RevisionControl" | "Translation" | "WebDevelopment" | "Building" => "Development",

            "TextEditor" | "WordProcessor" => "Editors",

            "Network" | "Dialup" | "InstantMessaging" | "Chat" | "IRCClient" |
            "Feed" | "FileTransfer" | "HamRadio" | "News" | "P2P" |
            "RemoteAccess" | "Telephony" | "VideoConference" | "WebBrowser" => "Network",

            "System" | "FileManager" | "Monitor" | "Security" | "Accessibility" |
            "Core" | "PackageManager" | "Emulator" => "System",

            "TerminalEmulator" => "Terminals",

            "Shell" => "Shells",

            "Settings" | "DesktopSettings" | "HardwareSettings" | "Printing" |
            "Documentation" | "Help" => "System",

            _ => continue,
        };

        return Some(mapped.to_string());
    }
    None
}

/// Classify from pacman package metadata
fn classify_from_pacman(name: &str) -> Option<CategoryAssignment> {
    // Try pacman -Qi for installed packages
    let output = std::process::Command::new("pacman")
        .args(["-Qi", name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut description = String::new();
    let mut groups = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("Description") {
            if let Some(desc) = line.split(':').nth(1) {
                description = desc.trim().to_string();
            }
        } else if line.starts_with("Groups") {
            if let Some(grp) = line.split(':').nth(1) {
                groups = grp.trim()
                    .split_whitespace()
                    .filter(|s| *s != "None")
                    .map(|s| s.to_string())
                    .collect();
            }
        }
    }

    // Try to categorize from groups first
    if !groups.is_empty() {
        if let Some(category) = categorize_from_groups(&groups) {
            return Some(CategoryAssignment::new(category, Confidence::High)
                .with_evidence(EvidenceSource::PacmanGroups { groups }));
        }
    }

    // Then try description
    if !description.is_empty() {
        if let Some(category) = categorize_from_description(&description) {
            return Some(CategoryAssignment::new(category, Confidence::Medium)
                .with_evidence(EvidenceSource::PacmanDescription { description }));
        }
    }

    None
}

/// Categorize from pacman groups
fn categorize_from_groups(groups: &[String]) -> Option<String> {
    for group in groups {
        let lower = group.to_lowercase();
        if lower.contains("game") || lower == "steam" {
            return Some("Games".to_string());
        }
        if lower.contains("multimedia") || lower.contains("audio") || lower.contains("video") {
            return Some("Multimedia".to_string());
        }
        if lower.contains("devel") || lower.contains("base-devel") {
            return Some("Development".to_string());
        }
        if lower.contains("network") {
            return Some("Network".to_string());
        }
        if lower == "xorg" || lower == "wayland" {
            return Some("Compositors".to_string());
        }
    }
    None
}

/// Categorize from description keywords
fn categorize_from_description(desc: &str) -> Option<String> {
    let lower = desc.to_lowercase();

    // Games - check first as Steam description contains "game"
    if lower.contains("game") || lower.contains("gaming") ||
       lower.contains("steam") || lower.contains("launcher for games") {
        return Some("Games".to_string());
    }

    // Editors
    if lower.contains("editor") && !lower.contains("video editor") &&
       !lower.contains("audio editor") {
        return Some("Editors".to_string());
    }

    // Terminals
    if lower.contains("terminal emulator") || lower.contains("terminal application") {
        return Some("Terminals".to_string());
    }

    // Shells
    if lower.contains("command shell") || lower.contains("unix shell") ||
       (lower.contains("shell") && lower.contains("command")) {
        return Some("Shells".to_string());
    }

    // Compositors
    if lower.contains("compositor") || lower.contains("window manager") ||
       lower.contains("wayland compositor") || lower.contains("x11 window") {
        return Some("Compositors".to_string());
    }

    // Browsers
    if lower.contains("web browser") || lower.contains("browser") {
        return Some("Browsers".to_string());
    }

    // Multimedia
    if lower.contains("media player") || lower.contains("video player") ||
       lower.contains("audio player") || lower.contains("music player") ||
       lower.contains("multimedia") || lower.contains("video editor") ||
       lower.contains("audio editor") {
        return Some("Multimedia".to_string());
    }

    // Development
    if lower.contains("compiler") || lower.contains("debugger") ||
       lower.contains("programming") || lower.contains("development") ||
       lower.contains("sdk") || lower.contains("ide") ||
       lower.contains("version control") {
        return Some("Development".to_string());
    }

    // Network
    if lower.contains("network") || lower.contains("vpn") ||
       lower.contains("firewall") || lower.contains("wireless") ||
       lower.contains("wifi") || lower.contains("ethernet") {
        return Some("Network".to_string());
    }

    // Power
    if lower.contains("power") || lower.contains("battery") ||
       lower.contains("cpu frequency") || lower.contains("thermal") {
        return Some("Power".to_string());
    }

    // Virtualization
    if lower.contains("virtual machine") || lower.contains("virtualization") ||
       lower.contains("hypervisor") {
        return Some("Virtualization".to_string());
    }

    // Containers
    if lower.contains("container") || lower.contains("docker") ||
       lower.contains("podman") || lower.contains("kubernetes") {
        return Some("Containers".to_string());
    }

    // System
    if lower.contains("system") || lower.contains("boot") ||
       lower.contains("init") || lower.contains("daemon") {
        return Some("System".to_string());
    }

    // Tools (generic utilities)
    if lower.contains("utility") || lower.contains("tool") ||
       lower.contains("command line") || lower.contains("cli") {
        return Some("Tools".to_string());
    }

    None
}

/// Classify from man page section
fn classify_from_man(name: &str) -> Option<CategoryAssignment> {
    let output = std::process::Command::new("man")
        .args(["-w", name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Extract section number from path like /usr/share/man/man1/foo.1.gz
    let section = path.rsplit('/')
        .next()
        .and_then(|filename| {
            // Parse foo.1.gz or foo.1
            let parts: Vec<_> = filename.split('.').collect();
            if parts.len() >= 2 {
                parts[parts.len() - 2].parse::<u8>().ok()
                    .or_else(|| parts.get(1).and_then(|s| s.parse::<u8>().ok()))
            } else {
                None
            }
        })?;

    let (category, description) = match section {
        1 => ("Tools", "User command"),
        2 => ("Development", "System call"),
        3 => ("Development", "Library function"),
        4 => ("System", "Device/special file"),
        5 => ("System", "File format"),
        6 => ("Games", "Game"),
        7 => ("System", "Miscellaneous"),
        8 => ("System", "System administration"),
        9 => ("Development", "Kernel routine"),
        _ => return None,
    };

    Some(CategoryAssignment::new(category, Confidence::Low)
        .with_evidence(EvidenceSource::ManSection {
            section,
            description: description.to_string(),
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desktop_categories() {
        let content = "[Desktop Entry]\nName=Foo\nCategories=Game;ActionGame;GTK;\n";
        let cats = parse_desktop_categories(content).unwrap();
        assert!(cats.contains(&"Game".to_string()));
    }

    #[test]
    fn test_map_game_category() {
        let cats = vec!["Game".to_string(), "ActionGame".to_string()];
        let mapped = map_desktop_to_anna_category(&cats);
        assert_eq!(mapped, Some("Games".to_string()));
    }

    #[test]
    fn test_categorize_steam_description() {
        let desc = "Steam is a digital distribution platform for video games";
        let cat = categorize_from_description(desc);
        assert_eq!(cat, Some("Games".to_string()));
    }

    #[test]
    fn test_categorize_editor() {
        let desc = "A text editor for programmers";
        let cat = categorize_from_description(desc);
        assert_eq!(cat, Some("Editors".to_string()));
    }
}

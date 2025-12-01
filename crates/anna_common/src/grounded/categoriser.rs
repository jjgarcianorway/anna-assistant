//! Rule-Based Categoriser v7.2.0
//!
//! Categorises packages/commands based on description text and metadata.
//! No hardcoded package lists - uses pattern matching on descriptions.
//!
//! Sources for descriptions:
//! - pacman -Qi -> Description field
//! - man -f -> one-line description
//! - pacman -Qi -> Groups field
//!
//! Rules:
//! - Every category is derived from description content
//! - Multiple categories are possible if multiple rules match
//! - Falls back to "Other" if no rule matches

use std::process::Command;

/// Standard category names (in display order)
pub const CATEGORY_ORDER: &[&str] = &[
    "Editors",
    "Terminals",
    "Shells",
    "Compositors",
    "Browsers",
    "Multimedia",
    "Development",
    "Network",
    "System",
    "Power",
    "Virtualization",
    "Containers",
    "Games",
    "Tools",
    "Other",
];

/// Information about a package/command for categorisation
#[derive(Debug, Clone, Default)]
pub struct PackageInfo {
    pub name: String,
    pub pacman_description: Option<String>,
    pub man_description: Option<String>,
    pub groups: Vec<String>,
}

impl PackageInfo {
    /// Fetch package info from system
    pub fn fetch(name: &str) -> Self {
        let mut info = Self {
            name: name.to_string(),
            ..Default::default()
        };

        // Get pacman description and groups
        if let Ok(output) = Command::new("pacman").args(["-Qi", name]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("Description") {
                        if let Some(desc) = line.split(':').nth(1) {
                            info.pacman_description = Some(desc.trim().to_string());
                        }
                    } else if line.starts_with("Groups") {
                        if let Some(groups) = line.split(':').nth(1) {
                            let groups_str = groups.trim();
                            if groups_str != "None" {
                                info.groups = groups_str.split_whitespace()
                                    .map(|s| s.to_string())
                                    .collect();
                            }
                        }
                    }
                }
            }
        }

        // Get man description
        if let Ok(output) = Command::new("man").args(["-f", name]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Format: "name (section) - description"
                if let Some(line) = stdout.lines().next() {
                    if let Some(desc_start) = line.find(" - ") {
                        info.man_description = Some(line[desc_start + 3..].trim().to_string());
                    }
                }
            }
        }

        info
    }

    /// Get combined description text for matching
    fn combined_description(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref d) = self.pacman_description {
            parts.push(d.to_lowercase());
        }
        if let Some(ref d) = self.man_description {
            parts.push(d.to_lowercase());
        }
        parts.join(" ")
    }
}

/// Categorise a package based on its info
pub fn categorise(info: &PackageInfo) -> Vec<String> {
    let desc = info.combined_description();
    let name_lower = info.name.to_lowercase();
    let mut categories = Vec::new();

    // Rule-based matching on description
    // Order matters - more specific rules first

    // Editors
    if desc.contains("text editor") || desc.contains("editor for")
        || desc.contains("vi improved") || desc.contains("vim")
        || (desc.contains("editor") && !desc.contains("video editor") && !desc.contains("hex editor"))
        || name_lower == "nano" || name_lower == "emacs" || name_lower == "helix" {
        categories.push("Editors".to_string());
    }

    // Terminals
    if desc.contains("terminal emulator") || desc.contains("terminal application")
        || (desc.contains("terminal") && desc.contains("gpu"))
        || name_lower == "alacritty" || name_lower == "kitty" || name_lower == "foot"
        || name_lower == "wezterm" || name_lower == "konsole" {
        categories.push("Terminals".to_string());
    }

    // Shells
    if desc.contains("command interpreter") || desc.contains("unix shell")
        || desc.contains("bourne") || desc.contains("shell for")
        || (desc.contains("shell") && (desc.contains("interactive") || desc.contains("scripting")))
        || info.groups.iter().any(|g| g == "base")
        || name_lower == "bash" || name_lower == "zsh" || name_lower == "fish"
        || name_lower == "dash" || name_lower == "nushell" {
        categories.push("Shells".to_string());
    }

    // Compositors / Window Managers
    if desc.contains("wayland compositor") || desc.contains("window manager")
        || desc.contains("wm for") || desc.contains("tiling")
        || name_lower == "hyprland" || name_lower == "sway" || name_lower == "i3"
        || name_lower == "bspwm" || name_lower == "openbox" {
        categories.push("Compositors".to_string());
    }

    // Browsers
    if desc.contains("web browser") || desc.contains("browser for")
        || (desc.contains("browser") && (desc.contains("internet") || desc.contains("web")))
        || name_lower == "firefox" || name_lower == "chromium" || name_lower == "brave"
        || name_lower.contains("chrome") {
        categories.push("Browsers".to_string());
    }

    // Multimedia
    if desc.contains("video player") || desc.contains("audio player")
        || desc.contains("media player") || desc.contains("multimedia")
        || desc.contains("video edit") || desc.contains("audio edit")
        || desc.contains("music player") || desc.contains("image viewer")
        || name_lower == "mpv" || name_lower == "vlc" || name_lower == "ffmpeg"
        || name_lower == "gimp" || name_lower == "inkscape" {
        categories.push("Multimedia".to_string());
    }

    // Development
    if desc.contains("compiler") || desc.contains("debugger")
        || desc.contains("programming") || desc.contains("development")
        || desc.contains("ide") || desc.contains("sdk")
        || info.groups.iter().any(|g| g == "base-devel")
        || name_lower == "gcc" || name_lower == "clang" || name_lower == "rustc"
        || name_lower == "cargo" || name_lower == "python" || name_lower == "node" {
        categories.push("Development".to_string());
    }

    // Network
    if desc.contains("network") || desc.contains("networking")
        || desc.contains("download") || desc.contains("http client")
        || desc.contains("ssh") || desc.contains("ftp")
        || desc.contains("dns") || desc.contains("vpn")
        || name_lower == "curl" || name_lower == "wget" || name_lower == "ssh"
        || name_lower == "rsync" {
        categories.push("Network".to_string());
    }

    // System
    if desc.contains("system monitor") || desc.contains("system information")
        || desc.contains("process") || desc.contains("service manager")
        || desc.contains("init") || desc.contains("boot")
        || name_lower == "systemd" || name_lower == "htop" || name_lower == "btop"
        || name_lower == "top" || name_lower == "ps" {
        categories.push("System".to_string());
    }

    // Power
    if desc.contains("power management") || desc.contains("battery")
        || desc.contains("acpi") || desc.contains("suspend")
        || desc.contains("hibernate") || desc.contains("power saving")
        || name_lower == "tlp" || name_lower == "powertop" {
        categories.push("Power".to_string());
    }

    // Virtualization
    if desc.contains("virtual machine") || desc.contains("virtualization")
        || desc.contains("hypervisor") || desc.contains("emulator")
        || name_lower == "qemu" || name_lower == "virtualbox" || name_lower == "libvirt" {
        categories.push("Virtualization".to_string());
    }

    // Containers
    if desc.contains("container") || desc.contains("containerization")
        || desc.contains("docker") || desc.contains("oci")
        || name_lower == "docker" || name_lower == "podman" || name_lower == "containerd" {
        categories.push("Containers".to_string());
    }

    // Games
    if desc.contains("game") || desc.contains("gaming")
        || info.groups.iter().any(|g| g.contains("game")) {
        categories.push("Games".to_string());
    }

    // Tools - catch common CLI utilities
    if categories.is_empty() {
        // Check if it's a common tool based on description
        if desc.contains("utility") || desc.contains("tool")
            || desc.contains("command") || desc.contains("cli")
            || is_common_cli_tool(&name_lower) {
            categories.push("Tools".to_string());
        }
    }

    // Fallback to Other
    if categories.is_empty() {
        categories.push("Other".to_string());
    }

    categories
}

/// Check if name is a common CLI tool
fn is_common_cli_tool(name: &str) -> bool {
    matches!(name,
        "grep" | "awk" | "sed" | "tar" | "gzip" | "unzip" | "zip"
        | "find" | "ls" | "cat" | "cp" | "mv" | "rm" | "mkdir"
        | "make" | "cmake" | "ninja" | "meson"
        | "jq" | "yq" | "fzf" | "ripgrep" | "rg" | "fd"
        | "git" | "diff" | "patch"
    )
}

/// Get all installed packages in a category
pub fn packages_in_category(category: &str) -> Vec<(String, String, String)> {
    // Get list of explicitly installed packages
    let output = Command::new("pacman")
        .args(["-Qe"])
        .output();

    let mut results = Vec::new();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0];
                    let version = parts[1];

                    let info = PackageInfo::fetch(name);
                    let cats = categorise(&info);

                    if cats.iter().any(|c| c.eq_ignore_ascii_case(category)) {
                        let desc = info.pacman_description
                            .or(info.man_description)
                            .unwrap_or_default();
                        results.push((name.to_string(), desc, version.to_string()));
                    }
                }
            }
        }
    }

    // Sort alphabetically
    results.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    results
}

/// Get category summary for KDB overview
/// Returns (category_name, list of package names)
pub fn get_category_summary() -> Vec<(String, Vec<String>)> {
    use std::collections::HashMap;

    // Get list of explicitly installed packages
    let output = Command::new("pacman")
        .args(["-Qe"])
        .output();

    let mut category_packages: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.split_whitespace().next() {
                    let info = PackageInfo::fetch(name);
                    let cats = categorise(&info);

                    // Add to first matching category (primary category)
                    if let Some(cat) = cats.first() {
                        category_packages
                            .entry(cat.clone())
                            .or_default()
                            .push(name.to_string());
                    }
                }
            }
        }
    }

    // Sort packages within each category
    for packages in category_packages.values_mut() {
        packages.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    }

    // Return in standard order, only categories with items
    let mut result = Vec::new();
    for cat in CATEGORY_ORDER {
        if let Some(packages) = category_packages.remove(*cat) {
            if !packages.is_empty() {
                result.push((cat.to_string(), packages));
            }
        }
    }

    result
}

/// Check if a string is a valid category name
pub fn is_valid_category(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    CATEGORY_ORDER.iter().any(|c| c.to_lowercase() == name_lower)
}

/// Normalize category name to standard form
pub fn normalize_category(name: &str) -> Option<String> {
    let name_lower = name.to_lowercase();
    CATEGORY_ORDER.iter()
        .find(|c| c.to_lowercase() == name_lower)
        .map(|c| c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorise_editor() {
        let info = PackageInfo {
            name: "vim".to_string(),
            pacman_description: Some("Vi Improved, a highly configurable, improved version of the vi text editor".to_string()),
            man_description: None,
            groups: vec![],
        };
        let cats = categorise(&info);
        assert!(cats.contains(&"Editors".to_string()));
    }

    #[test]
    fn test_categorise_terminal() {
        let info = PackageInfo {
            name: "alacritty".to_string(),
            pacman_description: Some("A cross-platform, GPU-accelerated terminal emulator".to_string()),
            man_description: None,
            groups: vec![],
        };
        let cats = categorise(&info);
        assert!(cats.contains(&"Terminals".to_string()));
    }

    #[test]
    fn test_categorise_shell() {
        let info = PackageInfo {
            name: "bash".to_string(),
            pacman_description: Some("The GNU Bourne Again shell".to_string()),
            man_description: Some("GNU Bourne-Again SHell".to_string()),
            groups: vec!["base".to_string()],
        };
        let cats = categorise(&info);
        assert!(cats.contains(&"Shells".to_string()));
    }

    #[test]
    fn test_categorise_browser() {
        let info = PackageInfo {
            name: "firefox".to_string(),
            pacman_description: Some("Standalone web browser from mozilla.org".to_string()),
            man_description: None,
            groups: vec![],
        };
        let cats = categorise(&info);
        assert!(cats.contains(&"Browsers".to_string()));
    }

    #[test]
    fn test_categorise_compositor() {
        let info = PackageInfo {
            name: "hyprland".to_string(),
            pacman_description: Some("A highly customizable dynamic tiling Wayland compositor".to_string()),
            man_description: None,
            groups: vec![],
        };
        let cats = categorise(&info);
        assert!(cats.contains(&"Compositors".to_string()));
    }

    #[test]
    fn test_is_valid_category() {
        assert!(is_valid_category("editors"));
        assert!(is_valid_category("Editors"));
        assert!(is_valid_category("TERMINALS"));
        assert!(!is_valid_category("unknown_category"));
    }

    #[test]
    fn test_normalize_category() {
        assert_eq!(normalize_category("editors"), Some("Editors".to_string()));
        assert_eq!(normalize_category("SHELLS"), Some("Shells".to_string()));
        assert_eq!(normalize_category("unknown"), None);
    }
}

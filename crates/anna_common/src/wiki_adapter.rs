//! Arch Wiki Adapter - Knowledge adaptation pipeline (6.9.0)
//!
//! This module implements the pipeline that takes:
//! 1. User's natural language question
//! 2. Real system context (WM/DE, tools, config files, directories)
//! 3. Local Arch Wiki content
//!
//! And produces:
//! - Adapted, user-specific action plans
//! - Commands tailored to their environment
//! - Safe execution with backups
//!
//! 6.9.0 Scope: Wallpaper changing on Hyprland + swww ONLY
//! Future: Expand to other WM/DE combos and use cases

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Intent & Context Structures
// ============================================================================

/// User's intent extracted from their natural language question
///
/// The LLM analyzes the question to determine:
/// - What the user wants to accomplish
/// - Any environment hints they provided
/// - Soft constraints or preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIntent {
    /// High-level goal (e.g., "change wallpaper", "configure audio")
    pub goal: String,

    /// Environment hint if user specified it (e.g., "Hyprland", "KDE")
    pub environment: Option<String>,

    /// Soft constraints or preferences
    /// Examples: ["persistent after reboot", "use this specific tool"]
    pub constraints: Vec<String>,
}

/// User's desktop environment context detected from the real system
///
/// This is built by inspecting processes, environment variables,
/// installed tools, config files, and directory structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDesktopContext {
    /// Window manager or desktop environment
    /// Examples: "Hyprland", "KDE Plasma", "GNOME", "i3"
    pub wm_or_de: Option<String>,

    /// Wallpaper tools installed on the system
    /// Examples: ["swww", "hyprpaper", "feh", "nitrogen"]
    pub wallpaper_tools: Vec<String>,

    /// Config files for the detected WM/DE
    /// Examples: ["/home/user/.config/hypr/hyprland.conf"]
    pub config_files: Vec<PathBuf>,

    /// Directories where wallpapers are likely stored
    /// Examples: ["/home/user/Wallpapers", "/home/user/Pictures"]
    pub wallpaper_dirs: Vec<PathBuf>,
}

// ============================================================================
// Wiki Content Structures
// ============================================================================

/// A single Arch Wiki article (or section)
///
/// For 6.9.0, this is sourced from local fixtures.
/// Future: Could be from local corpus, cached wiki, or live fetch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiArticle {
    /// Article title
    pub title: String,

    /// Wiki URL (for citation)
    pub url: String,

    /// Article content (markdown or plain text)
    pub content: String,
}

/// A single step in a wiki-derived action plan
///
/// Each step is either a command execution or a manual instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiActionStep {
    /// Human-readable description of what this step does
    pub description: String,

    /// Shell command to execute (if any)
    /// None means manual step (e.g., "choose a wallpaper file")
    pub command: Option<String>,

    /// True if this step modifies config files
    /// Triggers backup creation and extra confirmation
    pub is_file_write: bool,
}

/// Complete action plan derived from Arch Wiki + user context
///
/// This is what gets shown to the user before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiActionPlan {
    /// Plain language explanation of what will be done
    pub explanation: String,

    /// Ordered list of steps to execute
    pub steps: Vec<WikiActionStep>,

    /// Wiki articles this plan was derived from (for citation)
    pub source_articles: Vec<String>,
}

// ============================================================================
// Desktop Context Detection
// ============================================================================

/// Detect user's desktop environment context from the real system
///
/// Inspects:
/// - Running processes and environment variables for WM/DE
/// - Installed binaries for wallpaper tools
/// - Config file locations
/// - Common wallpaper directories
///
/// 6.9.0: Only supports Hyprland + swww detection
pub fn detect_user_desktop_context() -> UserDesktopContext {
    UserDesktopContext {
        wm_or_de: detect_wm_or_de(),
        wallpaper_tools: detect_wallpaper_tools(),
        config_files: detect_config_files(),
        wallpaper_dirs: detect_wallpaper_dirs(),
    }
}

/// Detect window manager or desktop environment
///
/// 6.9.0: Only Hyprland detection implemented
fn detect_wm_or_de() -> Option<String> {
    use std::env;
    use std::process::Command;

    // Check for Hyprland via environment variable
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Some("Hyprland".to_string());
    }

    // Check for Hyprland process
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("Hyprland").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return Some("Hyprland".to_string());
        }
    }

    // Future: Add detection for other WM/DEs (KDE, GNOME, i3, Sway)
    None
}

/// Detect installed wallpaper tools
///
/// Checks for common tools: swww, hyprpaper, feh, nitrogen
fn detect_wallpaper_tools() -> Vec<String> {
    let mut tools = Vec::new();

    let candidates = ["swww", "hyprpaper", "feh", "nitrogen"];

    for tool in &candidates {
        if is_command_available(tool) {
            tools.push(tool.to_string());
        }
    }

    tools
}

/// Check if a command is available in PATH
fn is_command_available(cmd: &str) -> bool {
    use std::process::Command;

    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Detect config files for the current WM/DE
///
/// 6.9.0: Only Hyprland config detection
fn detect_config_files() -> Vec<PathBuf> {
    use std::env;
    use std::fs;

    let mut configs = Vec::new();

    // Get home directory
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return configs,
    };

    // Check for Hyprland config
    let hypr_config = PathBuf::from(format!("{}/.config/hypr/hyprland.conf", home));
    if hypr_config.exists() {
        configs.push(hypr_config);
    }

    // Check for additional Hyprland config directory
    let hypr_config_d = PathBuf::from(format!("{}/.config/hypr/hyprland.conf.d", home));
    if hypr_config_d.exists() && hypr_config_d.is_dir() {
        // Add .conf files from hyprland.conf.d/
        if let Ok(entries) = fs::read_dir(&hypr_config_d) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("conf") {
                    configs.push(path);
                }
            }
        }
    }

    configs
}

/// Detect common wallpaper directories
///
/// Looks for: ~/Wallpapers, ~/Pictures, ~/Images
fn detect_wallpaper_dirs() -> Vec<PathBuf> {
    use std::env;

    let mut dirs = Vec::new();

    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return dirs,
    };

    let candidates = [
        format!("{}/Wallpapers", home),
        format!("{}/Pictures", home),
        format!("{}/Images", home),
        format!("{}/.local/share/wallpapers", home),
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() && path.is_dir() {
            dirs.push(path);
        }
    }

    dirs
}

// ============================================================================
// Core Functions (to be implemented in following steps)
// ============================================================================

impl UserIntent {
    /// Check if this intent is about changing wallpaper
    pub fn is_wallpaper_request(&self) -> bool {
        self.goal.to_lowercase().contains("wallpaper")
            || self.goal.to_lowercase().contains("background")
    }
}

impl UserDesktopContext {
    /// Check if we have enough context to proceed with wallpaper change
    pub fn can_handle_wallpaper_request(&self) -> bool {
        self.wm_or_de.is_some() && !self.wallpaper_tools.is_empty()
    }

    /// Get the preferred wallpaper tool (first in list)
    pub fn preferred_wallpaper_tool(&self) -> Option<&str> {
        self.wallpaper_tools.first().map(|s| s.as_str())
    }
}

impl WikiActionPlan {
    /// Get all commands that will be executed (for validation)
    pub fn all_commands(&self) -> Vec<&str> {
        self.steps
            .iter()
            .filter_map(|step| step.command.as_deref())
            .collect()
    }

    /// Check if any steps involve file writes
    pub fn has_file_writes(&self) -> bool {
        self.steps.iter().any(|step| step.is_file_write)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_user_desktop_context() {
        // This test runs on the real system, so we can't assert exact values
        // Just verify it doesn't crash and returns reasonable structure
        let ctx = detect_user_desktop_context();

        // Should return a valid context (even if empty)
        // If on Hyprland, should detect it
        // If swww installed, should detect it

        // Just verify the function runs
        assert!(ctx.wallpaper_tools.len() <= 4); // At most 4 known tools
    }

    #[test]
    fn test_user_intent_wallpaper_detection() {
        let intent = UserIntent {
            goal: "change wallpaper".to_string(),
            environment: Some("Hyprland".to_string()),
            constraints: vec![],
        };
        assert!(intent.is_wallpaper_request());

        let intent2 = UserIntent {
            goal: "set desktop background".to_string(),
            environment: None,
            constraints: vec![],
        };
        assert!(intent2.is_wallpaper_request());

        let intent3 = UserIntent {
            goal: "configure audio".to_string(),
            environment: None,
            constraints: vec![],
        };
        assert!(!intent3.is_wallpaper_request());
    }

    #[test]
    fn test_desktop_context_wallpaper_capability() {
        let ctx = UserDesktopContext {
            wm_or_de: Some("Hyprland".to_string()),
            wallpaper_tools: vec!["swww".to_string()],
            config_files: vec![],
            wallpaper_dirs: vec![],
        };
        assert!(ctx.can_handle_wallpaper_request());
        assert_eq!(ctx.preferred_wallpaper_tool(), Some("swww"));

        let empty_ctx = UserDesktopContext {
            wm_or_de: None,
            wallpaper_tools: vec![],
            config_files: vec![],
            wallpaper_dirs: vec![],
        };
        assert!(!empty_ctx.can_handle_wallpaper_request());
    }

    #[test]
    fn test_wiki_action_plan_helpers() {
        let plan = WikiActionPlan {
            explanation: "Change wallpaper using swww".to_string(),
            steps: vec![
                WikiActionStep {
                    description: "Initialize swww daemon".to_string(),
                    command: Some("swww init".to_string()),
                    is_file_write: false,
                },
                WikiActionStep {
                    description: "Set wallpaper".to_string(),
                    command: Some("swww img ~/Wallpapers/bg.png".to_string()),
                    is_file_write: false,
                },
                WikiActionStep {
                    description: "Update config file".to_string(),
                    command: Some("echo 'exec-once = swww init' >> ~/.config/hypr/hyprland.conf".to_string()),
                    is_file_write: true,
                },
            ],
            source_articles: vec!["Hyprland".to_string(), "swww".to_string()],
        };

        assert_eq!(plan.all_commands().len(), 3);
        assert!(plan.has_file_writes());
    }
}

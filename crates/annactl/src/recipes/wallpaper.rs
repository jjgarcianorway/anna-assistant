//! Wallpaper Change Recipe
//!
//! Beta.151: Deterministic recipe for changing desktop wallpaper
//!
//! This module detects the desktop environment / window manager and generates
//! the appropriate commands for wallpaper changes.

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Wallpaper change scenario detector
pub struct WallpaperRecipe;

impl WallpaperRecipe {
    /// Check if user request matches wallpaper change
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        (input_lower.contains("wallpaper") || input_lower.contains("background"))
            && (input_lower.contains("change")
                || input_lower.contains("set")
                || input_lower.contains("update")
                || input_lower.contains("new"))
    }

    /// Generate wallpaper change ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        // Detect desktop environment / window manager
        let de = telemetry
            .get("desktop_environment")
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let wm = telemetry
            .get("window_manager")
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        let display_protocol = telemetry
            .get("display_protocol")
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        // Determine which tool to use
        let (tool, commands) = match (de, wm, display_protocol) {
            // GNOME
            ("gnome", _, _) | (_, _, _) if de.contains("gnome") => {
                ("gsettings", Self::gnome_commands())
            }
            // KDE Plasma
            ("kde", _, _) | ("plasma", _, _) | (_, _, _)
                if de.contains("kde") || de.contains("plasma") =>
            {
                ("plasma-apply-wallpaperimage", Self::kde_commands())
            }
            // Hyprland
            (_, "hyprland", _) | (_, _, _) if wm.contains("hyprland") => {
                ("hyprpaper / swww", Self::hyprland_commands())
            }
            // Sway
            (_, "sway", _) | (_, _, _) if wm.contains("sway") => ("swaybg", Self::sway_commands()),
            // i3 (X11)
            (_, "i3", "x11") => ("feh", Self::i3_commands()),
            // XFCE
            ("xfce", _, _) | (_, _, _) if de.contains("xfce") => {
                ("xfconf-query", Self::xfce_commands())
            }
            // Generic X11 fallback
            (_, _, "x11") => ("feh", Self::x11_fallback_commands()),
            // Unknown - cannot proceed
            _ => {
                return Err(anyhow!(
                    "Cannot determine wallpaper method for DE={}, WM={}, Protocol={}",
                    de,
                    wm,
                    display_protocol
                ));
            }
        };

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-image-exists".to_string(),
                description: "Verify wallpaper image file exists".to_string(),
                command: "ls -la ~/Pictures/wallpapers/ || ls -la ~/Pictures/ || ls -la ~"
                    .to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-current-wallpaper".to_string(),
                description: "Check current wallpaper settings (for backup)".to_string(),
                command: commands.check_current.clone(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "set-wallpaper".to_string(),
                description: format!("Set wallpaper using {}", tool),
                command: commands.set_wallpaper.clone(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("restore-wallpaper".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-wallpaper".to_string(),
                description: "Verify wallpaper was applied".to_string(),
                command: commands.verify.clone(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "restore-wallpaper".to_string(),
            description: "Restore previous wallpaper".to_string(),
            command: commands.restore.clone(),
        }];

        let analysis = format!(
            "User requests wallpaper change. Detected environment: DE={}, WM={}, Protocol={}. \
             Using {} for wallpaper management.",
            de, wm, display_protocol, tool
        );

        let goals = vec![
            format!("Change desktop wallpaper using {}", tool),
            "Verify the new wallpaper is applied".to_string(),
            "Provide rollback to previous wallpaper if needed".to_string(),
        ];

        let notes_for_user = format!(
            "Wallpaper will be changed using {}.\n\n\
             Usage: Specify wallpaper path in the command, e.g.:\n\
             {}\n\n\
             Common wallpaper locations:\n\
             - ~/Pictures/wallpapers/\n\
             - ~/Pictures/\n\
             - /usr/share/backgrounds/",
            tool, commands.example
        );

        // Metadata (Beta.151: Use PlanMeta with detected environment)
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("wallpaper.rs".to_string()),
        );
        other.insert(
            "tool_detected".to_string(),
            serde_json::Value::String(tool.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: Some(de.to_string()),
                wm: Some(wm.to_string()),
                wallpaper_backends: vec![tool.to_string()],
                display_protocol: Some(display_protocol.to_string()),
                other,
            },
            template_used: Some("wallpaper_change".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn gnome_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "gsettings get org.gnome.desktop.background picture-uri".to_string(),
            set_wallpaper: "gsettings set org.gnome.desktop.background picture-uri 'file:///path/to/wallpaper.jpg'".to_string(),
            verify: "gsettings get org.gnome.desktop.background picture-uri".to_string(),
            restore: "gsettings set org.gnome.desktop.background picture-uri '$PREVIOUS_URI'".to_string(),
            example: "gsettings set org.gnome.desktop.background picture-uri 'file:///home/user/Pictures/mountain.jpg'".to_string(),
        }
    }

    fn kde_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "kreadconfig5 --file plasma-org.kde.plasma.desktop-appletsrc --group Wallpaper --key Image".to_string(),
            set_wallpaper: "plasma-apply-wallpaperimage /path/to/wallpaper.jpg".to_string(),
            verify: "kreadconfig5 --file plasma-org.kde.plasma.desktop-appletsrc --group Wallpaper --key Image".to_string(),
            restore: "plasma-apply-wallpaperimage $PREVIOUS_WALLPAPER".to_string(),
            example: "plasma-apply-wallpaperimage ~/Pictures/sunset.jpg".to_string(),
        }
    }

    fn hyprland_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "cat ~/.config/hypr/hyprpaper.conf || echo 'swww query'".to_string(),
            set_wallpaper: "swww img /path/to/wallpaper.jpg --transition-type fade".to_string(),
            verify: "swww query".to_string(),
            restore: "swww img $PREVIOUS_WALLPAPER --transition-type fade".to_string(),
            example: "swww img ~/Pictures/wallpapers/forest.jpg --transition-type fade".to_string(),
        }
    }

    fn sway_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "cat ~/.config/sway/config | grep output".to_string(),
            set_wallpaper: "swaybg -i /path/to/wallpaper.jpg -m fill".to_string(),
            verify: "pgrep swaybg && echo 'swaybg running'".to_string(),
            restore: "swaybg -i $PREVIOUS_WALLPAPER -m fill".to_string(),
            example: "swaybg -i ~/Pictures/ocean.jpg -m fill".to_string(),
        }
    }

    fn i3_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "cat ~/.fehbg || echo 'No .fehbg found'".to_string(),
            set_wallpaper: "feh --bg-scale /path/to/wallpaper.jpg".to_string(),
            verify: "cat ~/.fehbg".to_string(),
            restore: "feh --bg-scale $PREVIOUS_WALLPAPER".to_string(),
            example: "feh --bg-scale ~/Pictures/mountain.jpg".to_string(),
        }
    }

    fn xfce_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "xfconf-query -c xfce4-desktop -l | grep last-image".to_string(),
            set_wallpaper: "xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s /path/to/wallpaper.jpg".to_string(),
            verify: "xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image".to_string(),
            restore: "xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s $PREVIOUS_WALLPAPER".to_string(),
            example: "xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s ~/Pictures/lake.jpg".to_string(),
        }
    }

    fn x11_fallback_commands() -> WallpaperCommands {
        WallpaperCommands {
            check_current: "cat ~/.fehbg || echo 'Using feh fallback'".to_string(),
            set_wallpaper: "feh --bg-scale /path/to/wallpaper.jpg".to_string(),
            verify: "cat ~/.fehbg".to_string(),
            restore: "feh --bg-scale $PREVIOUS_WALLPAPER".to_string(),
            example: "feh --bg-scale ~/Pictures/stars.jpg".to_string(),
        }
    }
}

struct WallpaperCommands {
    check_current: String,
    set_wallpaper: String,
    verify: String,
    restore: String,
    example: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_wallpaper_change() {
        assert!(WallpaperRecipe::matches_request("change my wallpaper"));
        assert!(WallpaperRecipe::matches_request("set a new background"));
        assert!(WallpaperRecipe::matches_request("update wallpaper"));
        assert!(WallpaperRecipe::matches_request("change background image"));

        // Should not match
        assert!(!WallpaperRecipe::matches_request("what is my wallpaper"));
        assert!(!WallpaperRecipe::matches_request("show current wallpaper"));
    }

    #[test]
    fn test_gnome_detection() {
        let mut telemetry = HashMap::new();
        telemetry.insert("desktop_environment".to_string(), "gnome".to_string());
        telemetry.insert("window_manager".to_string(), "mutter".to_string());
        telemetry.insert("display_protocol".to_string(), "wayland".to_string());

        let plan = WallpaperRecipe::build_plan(&telemetry).unwrap();
        assert!(plan.analysis.contains("gnome"));
        assert!(plan.command_plan[0].command.contains("gsettings"));
    }

    #[test]
    fn test_kde_detection() {
        let mut telemetry = HashMap::new();
        telemetry.insert("desktop_environment".to_string(), "kde".to_string());
        telemetry.insert("display_protocol".to_string(), "wayland".to_string());

        let plan = WallpaperRecipe::build_plan(&telemetry).unwrap();
        assert!(plan.analysis.contains("kde") || plan.analysis.contains("plasma"));
        assert!(plan.command_plan[0].command.contains("plasma-apply"));
    }

    #[test]
    fn test_hyprland_detection() {
        let mut telemetry = HashMap::new();
        telemetry.insert("window_manager".to_string(), "hyprland".to_string());
        telemetry.insert("display_protocol".to_string(), "wayland".to_string());

        let plan = WallpaperRecipe::build_plan(&telemetry).unwrap();
        assert!(plan.analysis.contains("hyprland"));
        assert!(plan.command_plan[0].command.contains("swww"));
    }

    #[test]
    fn test_unknown_environment_error() {
        let telemetry = HashMap::new(); // No environment info

        let result = WallpaperRecipe::build_plan(&telemetry);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot determine"));
    }
}

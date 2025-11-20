// Beta.158: Media Player and Codec Recipe
// Handles installation of media players (VLC, MPV) and multimedia codecs

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct MediaRecipe;

#[derive(Debug, PartialEq)]
enum MediaOperation {
    Install,       // Install media player or codecs
    CheckStatus,   // Verify installation
    InstallCodecs, // Install codec pack
    ListPlayers,   // List installed media players
}

impl MediaOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || (input_lower.contains("which") && input_lower.contains("player"))
        {
            return MediaOperation::CheckStatus;
        }

        // List players
        if input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("available")
        {
            return MediaOperation::ListPlayers;
        }

        // Install codecs
        if input_lower.contains("codec")
            || input_lower.contains("ffmpeg")
            || input_lower.contains("gstreamer")
            || (input_lower.contains("media") && input_lower.contains("support"))
        {
            return MediaOperation::InstallCodecs;
        }

        // Default to install
        MediaOperation::Install
    }
}

impl MediaRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_media_context = input_lower.contains("media player")
            || input_lower.contains("video player")
            || input_lower.contains("vlc")
            || input_lower.contains("mpv")
            || input_lower.contains("codec")
            || input_lower.contains("ffmpeg")
            || input_lower.contains("gstreamer")
            || (input_lower.contains("play") && (input_lower.contains("video") || input_lower.contains("audio")))
            || (input_lower.contains("media") && input_lower.contains("support"));

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("add")
            || input_lower.contains("get")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("enable")
            || input_lower.contains("configure");

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_media_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = MediaOperation::detect(user_input);

        match operation {
            MediaOperation::Install => Self::build_install_plan(telemetry),
            MediaOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            MediaOperation::InstallCodecs => Self::build_install_codecs_plan(telemetry),
            MediaOperation::ListPlayers => Self::build_list_players_plan(telemetry),
        }
    }

    fn detect_player(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("vlc") {
            "vlc"
        } else if input_lower.contains("mpv") {
            "mpv"
        } else {
            "vlc" // Default to VLC
        }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let player = Self::detect_player(user_input);

        let (player_name, package_name, description) = match player {
            "vlc" => ("VLC", "vlc", "Feature-rich media player with GUI"),
            "mpv" => ("MPV", "mpv", "Lightweight, keyboard-driven media player"),
            _ => ("VLC", "vlc", "Feature-rich media player with GUI"),
        };

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-player".to_string(),
                description: format!("Check if {} is already installed", player_name),
                command: format!("pacman -Q {} 2>/dev/null || true", package_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: format!("install-{}", player),
                description: format!("Install {} media player", player_name),
                command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                risk_level: RiskLevel::Low,
                rollback_id: Some(format!("remove-{}", player)),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: format!("Verify {} is installed", player_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: format!("remove-{}", player),
                description: format!("Remove {}", player_name),
                command: format!("sudo pacman -Rns --noconfirm {}", package_name),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("media.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("player".to_string(), serde_json::json!(player_name));
        meta_other.insert("package".to_string(), serde_json::json!(package_name));

        Ok(ActionPlan {
            analysis: format!("Installing {} media player from official Arch repositories. {}", player_name, description),
            goals: vec![
                format!("Install {} media player", player_name),
                format!("Verify {} is working", player_name),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: format!("{} will be installed from official Arch repositories.\n\n\
                                   Features:\n\
                                   - Plays most media formats (video, audio)\n\
                                   - No additional codec installation needed for common formats\n\
                                   - {} desktop integration\n\n\
                                   Launch: {}",
                                   player_name,
                                   if player == "vlc" { "Full GUI with" } else { "Lightweight" },
                                   package_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("media_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-vlc".to_string(),
                description: "Check if VLC is installed".to_string(),
                command: "pacman -Q vlc".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-mpv".to_string(),
                description: "Check if MPV is installed".to_string(),
                command: "pacman -Q mpv".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-ffmpeg".to_string(),
                description: "Check if ffmpeg is installed".to_string(),
                command: "pacman -Q ffmpeg".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "list-media-tools".to_string(),
                description: "List all installed media players and tools".to_string(),
                command: "echo '=== Media Players ===' && pacman -Q vlc mpv 2>/dev/null || echo 'No media players installed' && echo '\n=== Codecs & Tools ===' && pacman -Q ffmpeg gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav 2>/dev/null || echo 'No codec packages installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("media.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking which media players and codec packages are currently installed.".to_string(),
            goals: vec![
                "Verify VLC installation status".to_string(),
                "Verify MPV installation status".to_string(),
                "Check codec package installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will show which media players (VLC, MPV) and multimedia codec packages are currently installed.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("media_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_install_codecs_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-codecs".to_string(),
                description: "Check which codec packages are already installed".to_string(),
                command: "pacman -Q ffmpeg gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-ffmpeg".to_string(),
                description: "Install ffmpeg (core multimedia framework)".to_string(),
                command: "sudo pacman -S --needed --noconfirm ffmpeg".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-ffmpeg".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-gstreamer-good".to_string(),
                description: "Install GStreamer good plugins (common formats)".to_string(),
                command: "sudo pacman -S --needed --noconfirm gst-plugins-good".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-gstreamer-good".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-gstreamer-bad".to_string(),
                description: "Install GStreamer bad plugins (additional formats)".to_string(),
                command: "sudo pacman -S --needed --noconfirm gst-plugins-bad".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-gstreamer-bad".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-gstreamer-ugly".to_string(),
                description: "Install GStreamer ugly plugins (proprietary formats)".to_string(),
                command: "sudo pacman -S --needed --noconfirm gst-plugins-ugly".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-gstreamer-ugly".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-gst-libav".to_string(),
                description: "Install GStreamer libav plugin (FFmpeg integration)".to_string(),
                command: "sudo pacman -S --needed --noconfirm gst-libav".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-gst-libav".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: "Verify all codec packages are installed".to_string(),
                command: "pacman -Q ffmpeg gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-ffmpeg".to_string(),
                description: "Remove ffmpeg".to_string(),
                command: "sudo pacman -Rns --noconfirm ffmpeg".to_string(),
            },
            RollbackStep {
                id: "remove-gstreamer-good".to_string(),
                description: "Remove gst-plugins-good".to_string(),
                command: "sudo pacman -Rns --noconfirm gst-plugins-good".to_string(),
            },
            RollbackStep {
                id: "remove-gstreamer-bad".to_string(),
                description: "Remove gst-plugins-bad".to_string(),
                command: "sudo pacman -Rns --noconfirm gst-plugins-bad".to_string(),
            },
            RollbackStep {
                id: "remove-gstreamer-ugly".to_string(),
                description: "Remove gst-plugins-ugly".to_string(),
                command: "sudo pacman -Rns --noconfirm gst-plugins-ugly".to_string(),
            },
            RollbackStep {
                id: "remove-gst-libav".to_string(),
                description: "Remove gst-libav".to_string(),
                command: "sudo pacman -Rns --noconfirm gst-libav".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("media.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("InstallCodecs"));

        Ok(ActionPlan {
            analysis: "Installing comprehensive multimedia codec support: ffmpeg and GStreamer plugins for broad format compatibility.".to_string(),
            goals: vec![
                "Install ffmpeg (core multimedia framework)".to_string(),
                "Install GStreamer plugin packages".to_string(),
                "Enable support for common and proprietary media formats".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will install comprehensive codec support:\n\n\
                           - ffmpeg: Core multimedia framework (encoding, decoding, streaming)\n\
                           - gst-plugins-good: Common formats (MP3, AAC, H.264)\n\
                           - gst-plugins-bad: Additional formats (experimental/new codecs)\n\
                           - gst-plugins-ugly: Proprietary formats (MP3 encoding, DVD)\n\
                           - gst-libav: FFmpeg integration for GStreamer\n\n\
                           After installation, most media players will support a wide range of formats.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("media_install_codecs".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_players_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "list-installed-players".to_string(),
                description: "List installed media players".to_string(),
                command: "echo '=== Installed Media Players ===' && pacman -Q vlc mpv 2>/dev/null | awk '{print \"- \" $1 \" (\" $2 \")\"}' || echo 'No media players installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-available-players".to_string(),
                description: "Show available media players".to_string(),
                command: "echo '\n=== Available Media Players ===\n- VLC: Feature-rich GUI player (most popular)\n- MPV: Lightweight, keyboard-driven player\n\n=== Codec Support ===\n- ffmpeg: Core multimedia framework\n- GStreamer plugins: Additional format support'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("media.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListPlayers"));

        Ok(ActionPlan {
            analysis: "Showing installed media players and available options for installation.".to_string(),
            goals: vec![
                "List currently installed media players".to_string(),
                "Show available players for installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Available media players:\n\n\
                           - VLC: Full-featured GUI player\n  \
                           - Most popular choice\n  \
                           - Plays almost everything\n  \
                           - Built-in codec support\n\n\
                           - MPV: Lightweight player\n  \
                           - Keyboard-driven interface\n  \
                           - Highly configurable\n  \
                           - Lower resource usage\n\n\
                           To install: annactl \"install <player-name>\"".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("media_list_players".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_media_keywords() {
        assert!(MediaRecipe::matches_request("install vlc"));
        assert!(MediaRecipe::matches_request("install mpv"));
        assert!(MediaRecipe::matches_request("install media player"));
        assert!(MediaRecipe::matches_request("setup video player"));
        assert!(MediaRecipe::matches_request("install codecs"));
    }

    #[test]
    fn test_matches_media_actions() {
        assert!(MediaRecipe::matches_request("check media player status"));
        assert!(MediaRecipe::matches_request("list media players"));
        assert!(MediaRecipe::matches_request("install ffmpeg"));
    }

    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!MediaRecipe::matches_request("what is vlc"));
        assert!(!MediaRecipe::matches_request("tell me about mpv"));
        assert!(!MediaRecipe::matches_request("explain codecs"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            MediaOperation::detect("install vlc"),
            MediaOperation::Install
        );
        assert_eq!(
            MediaOperation::detect("check media player status"),
            MediaOperation::CheckStatus
        );
        assert_eq!(
            MediaOperation::detect("install codecs"),
            MediaOperation::InstallCodecs
        );
        assert_eq!(
            MediaOperation::detect("list media players"),
            MediaOperation::ListPlayers
        );
    }

    #[test]
    fn test_player_detection() {
        assert_eq!(MediaRecipe::detect_player("install vlc"), "vlc");
        assert_eq!(MediaRecipe::detect_player("install mpv"), "mpv");
        assert_eq!(MediaRecipe::detect_player("install media player"), "vlc"); // Default
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install vlc".to_string());

        let plan = MediaRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("vlc") || g.to_lowercase().contains("media") || g.to_lowercase().contains("player")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check media player status".to_string());

        let plan = MediaRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_install_codecs_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install codecs".to_string());

        let plan = MediaRecipe::build_install_codecs_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("codec") || g.to_lowercase().contains("ffmpeg") || g.to_lowercase().contains("format")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_list_players_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "list media players".to_string());

        let plan = MediaRecipe::build_list_players_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("list") || g.to_lowercase().contains("available")));
        assert!(!plan.command_plan.is_empty());
    }
}

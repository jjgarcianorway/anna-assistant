// Beta.164: Music Players Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct MusicRecipe;

#[derive(Debug, PartialEq)]
enum MusicOperation {
    Install,
    CheckStatus,
    SetupMpd,
    ListPlayers,
}

impl MusicOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("setup mpd") || input_lower.contains("configure mpd") {
            MusicOperation::SetupMpd
        } else if input_lower.contains("check") || input_lower.contains("status") {
            MusicOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            MusicOperation::ListPlayers
        } else {
            MusicOperation::Install
        }
    }
}

impl MusicRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("spotify") || input_lower.contains("mpd")
            || input_lower.contains("ncmpcpp") || input_lower.contains("music player")
            || input_lower.contains("music streaming");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = MusicOperation::detect(user_input);
        match operation {
            MusicOperation::Install => Self::build_install_plan(telemetry),
            MusicOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            MusicOperation::SetupMpd => Self::build_setup_mpd_plan(telemetry),
            MusicOperation::ListPlayers => Self::build_list_players_plan(telemetry),
        }
    }

    fn detect_player(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("spotify") { "spotify" }
        else if input_lower.contains("mpd") { "mpd" }
        else if input_lower.contains("ncmpcpp") { "ncmpcpp" }
        else { "spotify" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let player = Self::detect_player(user_input);
        let (player_name, package_name, is_aur) = match player {
            "spotify" => ("Spotify", "spotify-launcher", true),
            "mpd" => ("MPD", "mpd", false),
            "ncmpcpp" => ("ncmpcpp", "ncmpcpp", false),
            _ => ("Spotify", "spotify-launcher", true),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("music.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("player".to_string(), serde_json::json!(player_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = match player {
            "mpd" => "MPD installed. Configure in ~/.config/mpd/mpd.conf, start with: systemctl --user start mpd".to_string(),
            "ncmpcpp" => "ncmpcpp installed. Requires MPD to be running. Launch with: ncmpcpp".to_string(),
            _ => format!("{} installed. Launch from application menu.", player_name),
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} music player", player_name),
            goals: vec![format!("Install {}", player_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", player),
                    description: format!("Install {}", player_name),
                    command: install_cmd,
                    risk_level: if is_aur { RiskLevel::Medium } else { RiskLevel::Low },
                    rollback_id: Some(format!("remove-{}", player)),
                    requires_confirmation: is_aur,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", player),
                    description: format!("Remove {}", player_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("music_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("music.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking music players".to_string(),
            goals: vec!["List installed music players".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-music-players".to_string(),
                    description: "List music players".to_string(),
                    command: "pacman -Q spotify-launcher mpd ncmpcpp 2>/dev/null || echo 'No music players installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-mpd-status".to_string(),
                    description: "Check MPD status".to_string(),
                    command: "systemctl --user is-active mpd 2>/dev/null || echo 'MPD not running'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed music players and MPD status".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("music_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_setup_mpd_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("music.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SetupMpd"));

        Ok(ActionPlan {
            analysis: "Setting up MPD music server".to_string(),
            goals: vec![
                "Create MPD configuration directory".to_string(),
                "Generate basic MPD config".to_string(),
                "Start MPD user service".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-mpd".to_string(),
                    description: "Verify MPD is installed".to_string(),
                    command: "pacman -Q mpd".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "create-mpd-dirs".to_string(),
                    description: "Create MPD directories".to_string(),
                    command: "mkdir -p ~/.config/mpd ~/.local/share/mpd/playlists".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "create-mpd-config".to_string(),
                    description: "Create MPD configuration".to_string(),
                    command: r#"cat > ~/.config/mpd/mpd.conf << 'MPDEOF'
music_directory    "~/Music"
playlist_directory "~/.local/share/mpd/playlists"
db_file            "~/.local/share/mpd/database"
log_file           "~/.local/share/mpd/log"
pid_file           "~/.local/share/mpd/pid"
state_file         "~/.local/share/mpd/state"
sticker_file       "~/.local/share/mpd/sticker.sql"
bind_to_address    "127.0.0.1"
port               "6600"
auto_update        "yes"

audio_output {
    type  "pulse"
    name  "PulseAudio Output"
}
MPDEOF"#.to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-mpd-config".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "start-mpd".to_string(),
                    description: "Start MPD service".to_string(),
                    command: "systemctl --user enable --now mpd".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-mpd-config".to_string(),
                    description: "Remove MPD configuration".to_string(),
                    command: "rm -rf ~/.config/mpd ~/.local/share/mpd".to_string(),
                },
            ],
            notes_for_user: "MPD configured. Music directory: ~/Music. Connect with ncmpcpp or other MPD clients.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("music_setup_mpd".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_players_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("music.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListPlayers"));

        Ok(ActionPlan {
            analysis: "Showing available music players".to_string(),
            goals: vec!["List available music players".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-players".to_string(),
                    description: "Show available players".to_string(),
                    command: r"echo 'Available:\n- Spotify (AUR) - Streaming music service\n- MPD (official) - Music Player Daemon\n- ncmpcpp (official) - Terminal MPD client\n- Rhythmbox (official) - GNOME music player'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Music players for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("music_list_players".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matches() {
        assert!(MusicRecipe::matches_request("install spotify"));
        assert!(MusicRecipe::matches_request("setup mpd"));
        assert!(MusicRecipe::matches_request("install music player"));
        assert!(!MusicRecipe::matches_request("what is spotify"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install mpd".to_string());
        let plan = MusicRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}

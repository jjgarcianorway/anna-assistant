//! Configuration and desktop environment-related probe rules.

use crate::deterministic_probes::types::ProbeRule;

pub fn config_rules() -> Vec<ProbeRule> {
    vec![
        // Editor/Config queries
        ProbeRule {
            intent_id: "vim.setup",
            keywords: &["vim"],
            negative_keywords: &["install", "package"],
            probes: &[
                "command_v_vim",
                "command_v_nvim",
                "vimrc_content",
                "nvim_config",
            ],
            description: "Vim/nvim setup",
        },
        ProbeRule {
            intent_id: "neovim.setup",
            keywords: &["neovim"],
            negative_keywords: &["install"],
            probes: &["command_v_nvim", "nvim_config"],
            description: "Neovim setup",
        },
        ProbeRule {
            intent_id: "nvim.setup",
            keywords: &["nvim"],
            negative_keywords: &["install"],
            probes: &["command_v_nvim", "nvim_config"],
            description: "Nvim setup",
        },
        ProbeRule {
            intent_id: "editor.setup",
            keywords: &["editor", "setup"],
            negative_keywords: &[],
            probes: &[
                "command_v_vim",
                "command_v_nvim",
                "command_v_nano",
                "command_v_emacs",
                "command_v_code",
            ],
            description: "Editor setup",
        },
        ProbeRule {
            intent_id: "bash.config",
            keywords: &["bashrc"],
            negative_keywords: &[],
            probes: &["bashrc_content"],
            description: "Bashrc content",
        },
        ProbeRule {
            intent_id: "zsh.config",
            keywords: &["zshrc"],
            negative_keywords: &[],
            probes: &["zshrc_content"],
            description: "Zshrc content",
        },
        // Desktop queries
        ProbeRule {
            intent_id: "desktop.wallpaper",
            keywords: &["wallpaper"],
            negative_keywords: &[],
            probes: &["desktop_wallpaper", "desktop_session"],
            description: "Wallpaper location",
        },
        ProbeRule {
            intent_id: "desktop.session",
            keywords: &["desktop", "environment"],
            negative_keywords: &[],
            probes: &["desktop_session", "installed_desktops"],
            description: "Desktop environment",
        },
        ProbeRule {
            intent_id: "display.server",
            keywords: &["wayland"],
            negative_keywords: &[],
            probes: &["display_server"],
            description: "Display server (Wayland/X11)",
        },
        ProbeRule {
            intent_id: "display.server_alt",
            keywords: &["x11"],
            negative_keywords: &[],
            probes: &["display_server"],
            description: "Display server (X11)",
        },
    ]
}

//! Meta and configuration answer functions (v0.0.175).
//!
//! Small talk, config file locations.

use crate::deterministic::DeterministicResult;

/// Answer meta/small-talk queries with static responses (bypass LLM)
pub fn answer_meta_small_talk(query: &str, route_class: &str) -> DeterministicResult {
    let q = query.to_lowercase();

    let answer = if q.contains("how are you") {
        "I'm functioning well! Ready to help with your Linux system questions."
    } else if q.contains("what is your name")
        || q.contains("what's your name")
        || q.contains("who are you")
    {
        "I'm Anna, your Linux system assistant. I help answer questions about your computer's hardware, software, and configuration."
    } else if q.contains("are you ok") || q.contains("are you okay") {
        "Yes, I'm operational and ready to assist with your system questions."
    } else if q.contains("are you using llm")
        || q.contains("are you an ai")
        || q.contains("are you a bot")
    {
        "Yes, I use an LLM (Large Language Model) to understand questions and generate responses. I combine this with deterministic probes to gather accurate system information."
    } else if q.contains("are you human") || q.contains("are you real") {
        "I'm an AI assistant - not human, but designed to help you with Linux system administration tasks."
    } else if q == "hello" || q == "hi" || q == "hey" {
        "Hello! I'm Anna, your Linux system assistant. How can I help you today?"
    } else if q == "thanks" || q == "thank you" {
        "You're welcome! Let me know if you have more questions."
    } else if q.starts_with("good morning")
        || q.starts_with("good afternoon")
        || q.starts_with("good evening")
    {
        "Hello! How can I help you with your system today?"
    } else {
        "Hello! I'm Anna, ready to help with your Linux system questions."
    };

    DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    }
}

/// Answer config file location queries using known paths
pub fn answer_config_file_location(query: &str, route_class: &str) -> Option<DeterministicResult> {
    let q = query.to_lowercase();

    // Config file location mappings
    let answer = if q.contains("vim") && !q.contains("nvim") {
        "Vim config: `~/.vimrc` or `~/.vim/vimrc`"
    } else if q.contains("nvim") || q.contains("neovim") {
        "Neovim config: `~/.config/nvim/init.lua` or `~/.config/nvim/init.vim`"
    } else if q.contains("hyprland") {
        "Hyprland config: `~/.config/hypr/hyprland.conf`"
    } else if q.contains("sway") {
        "Sway config: `~/.config/sway/config`"
    } else if q.contains("alacritty") {
        "Alacritty config: `~/.config/alacritty/alacritty.toml` or `~/.config/alacritty/alacritty.yml`"
    } else if q.contains("kitty") {
        "Kitty config: `~/.config/kitty/kitty.conf`"
    } else if q.contains("bash") {
        "Bash config: `~/.bashrc` (interactive) or `~/.bash_profile` (login shell)"
    } else if q.contains("zsh") {
        "Zsh config: `~/.zshrc` (main config) or `~/.zshenv` (environment)"
    } else if q.contains("fish") {
        "Fish config: `~/.config/fish/config.fish`"
    } else if q.contains("nano") {
        "Nano config: `~/.nanorc` or `~/.config/nano/nanorc`"
    } else if q.contains("emacs") {
        "Emacs config: `~/.emacs` or `~/.emacs.d/init.el`"
    } else if q.contains("git") {
        "Git config: `~/.gitconfig` (global) or `.git/config` (per-repo)"
    } else if q.contains("ssh") {
        "SSH config: `~/.ssh/config` (client) or `/etc/ssh/sshd_config` (server)"
    } else if q.contains("waybar") {
        "Waybar config: `~/.config/waybar/config` and `~/.config/waybar/style.css`"
    } else if q.contains("rofi") {
        "Rofi config: `~/.config/rofi/config.rasi`"
    } else if q.contains("dunst") {
        "Dunst config: `~/.config/dunst/dunstrc`"
    } else if q.contains("picom") {
        "Picom config: `~/.config/picom/picom.conf` or `~/.config/picom.conf`"
    } else if q.contains("i3") {
        "i3 config: `~/.config/i3/config` or `~/.i3/config`"
    } else if q.contains("polybar") {
        "Polybar config: `~/.config/polybar/config.ini`"
    } else if q.contains("tmux") {
        "Tmux config: `~/.tmux.conf`"
    } else {
        return None; // Unknown config location
    };

    Some(DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

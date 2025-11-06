//! Terminal emulator configuration intelligence
//!
//! Analyzes terminal emulator configuration and suggests improvements

use anna_common::{Advice, Priority, RiskLevel};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalType {
    Alacritty,
    Kitty,
    WezTerm,
    Foot,
    St,
    Terminator,
    Gnome,
    Konsole,
    Xterm,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub terminal_type: TerminalType,
    #[allow(dead_code)]
    pub config_path: Option<PathBuf>,
    pub has_nerd_font: bool,
    pub has_color_scheme: bool,
    pub has_gpu_acceleration: bool,
    pub has_good_font_size: bool,
    #[allow(dead_code)]
    pub has_useful_keybindings: bool,
    pub font_family: Option<String>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            terminal_type: TerminalType::Unknown,
            config_path: None,
            has_nerd_font: false,
            has_color_scheme: false,
            has_gpu_acceleration: false,
            has_good_font_size: true, // Assume good by default
            has_useful_keybindings: false,
            font_family: None,
        }
    }
}

/// Detect and analyze terminal emulator configuration
pub fn analyze_terminal() -> Option<TerminalConfig> {
    let terminal_type = detect_terminal_type();

    if terminal_type == TerminalType::Unknown {
        info!("No known terminal emulator detected");
        return None;
    }

    info!("Detected terminal: {:?}", terminal_type);

    let config_path = find_terminal_config(&terminal_type);
    let mut config = TerminalConfig {
        terminal_type: terminal_type.clone(),
        config_path: config_path.clone(),
        ..Default::default()
    };

    // Detect Nerd Font installation
    detect_nerd_fonts(&mut config);

    // Analyze config file if found
    if let Some(ref path) = config_path {
        if let Ok(content) = fs::read_to_string(path) {
            analyze_terminal_config_content(&content, &mut config);
        }
    }

    Some(config)
}

/// Detect which terminal emulator is being used
fn detect_terminal_type() -> TerminalType {
    // Check installed terminals in order of popularity
    let terminals = vec![
        ("alacritty", TerminalType::Alacritty),
        ("kitty", TerminalType::Kitty),
        ("wezterm", TerminalType::WezTerm),
        ("foot", TerminalType::Foot),
        ("st", TerminalType::St),
        ("terminator", TerminalType::Terminator),
        ("gnome-terminal", TerminalType::Gnome),
        ("konsole", TerminalType::Konsole),
    ];

    for (cmd, term_type) in terminals {
        if Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false) {
            return term_type;
        }
    }

    // Check if using xterm (fallback)
    if let Ok(term_var) = std::env::var("TERM") {
        if term_var.contains("xterm") {
            return TerminalType::Xterm;
        }
    }

    TerminalType::Unknown
}

/// Find terminal configuration file
fn find_terminal_config(terminal_type: &TerminalType) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    let config_path = match terminal_type {
        TerminalType::Alacritty => {
            // Check multiple possible locations
            let paths = vec![
                format!("{}/.config/alacritty/alacritty.toml", home),
                format!("{}/.config/alacritty/alacritty.yml", home),
                format!("{}/.alacritty.yml", home),
            ];
            paths.into_iter().find(|p| PathBuf::from(p).exists())
        }
        TerminalType::Kitty => {
            Some(format!("{}/.config/kitty/kitty.conf", home))
        }
        TerminalType::WezTerm => {
            Some(format!("{}/.config/wezterm/wezterm.lua", home))
        }
        TerminalType::Foot => {
            Some(format!("{}/.config/foot/foot.ini", home))
        }
        TerminalType::St => {
            // st is configured at compile time, no runtime config
            None
        }
        TerminalType::Terminator => {
            Some(format!("{}/.config/terminator/config", home))
        }
        TerminalType::Gnome => {
            // GNOME Terminal uses dconf/gsettings, not a file
            None
        }
        TerminalType::Konsole => {
            Some(format!("{}/.config/konsolerc", home))
        }
        TerminalType::Xterm => {
            Some(format!("{}/.Xresources", home))
        }
        TerminalType::Unknown => None,
    };

    config_path.and_then(|p| {
        let path_buf = PathBuf::from(&p);
        if path_buf.exists() {
            Some(path_buf)
        } else {
            None
        }
    })
}

/// Detect if Nerd Fonts are installed
fn detect_nerd_fonts(config: &mut TerminalConfig) {
    // Check for common Nerd Font packages
    let nerd_fonts = vec![
        "ttf-jetbrains-mono-nerd",
        "ttf-firacode-nerd",
        "ttf-hack-nerd",
        "ttf-meslo-nerd",
        "ttf-sourcecodepro-nerd",
    ];

    for font in nerd_fonts {
        if let Ok(output) = Command::new("pacman").args(&["-Q", font]).output() {
            if output.status.success() {
                config.has_nerd_font = true;
                config.font_family = Some(font.to_string());
                return;
            }
        }
    }
}

/// Analyze terminal config file content
fn analyze_terminal_config_content(content: &str, config: &mut TerminalConfig) {
    match config.terminal_type {
        TerminalType::Alacritty => analyze_alacritty_config(content, config),
        TerminalType::Kitty => analyze_kitty_config(content, config),
        TerminalType::WezTerm => analyze_wezterm_config(content, config),
        TerminalType::Foot => analyze_foot_config(content, config),
        _ => {}
    }
}

/// Analyze Alacritty configuration
fn analyze_alacritty_config(content: &str, config: &mut TerminalConfig) {
    // Check for Nerd Font in config
    if content.contains("Nerd") || content.contains("NF") {
        config.has_nerd_font = true;
    }

    // Check for color scheme
    if content.contains("colors:") || content.contains("[colors") {
        config.has_color_scheme = true;
    }

    // Check font size
    if let Some(size_line) = content.lines().find(|l| l.contains("size") && (l.contains("font") || l.contains("size:"))) {
        if let Some(size_str) = size_line.split(':').nth(1).or_else(|| size_line.split('=').nth(1)) {
            if let Ok(size) = size_str.trim().parse::<f32>() {
                config.has_good_font_size = size >= 10.0 && size <= 16.0;
            }
        }
    }

    // Alacritty doesn't have explicit GPU acceleration toggle (always uses it)
    config.has_gpu_acceleration = true;
}

/// Analyze Kitty configuration
fn analyze_kitty_config(content: &str, config: &mut TerminalConfig) {
    // Check for Nerd Font
    if content.contains("Nerd") || content.contains("NF") {
        config.has_nerd_font = true;
    }

    // Check for color scheme
    if content.contains("foreground") && content.contains("background") {
        config.has_color_scheme = true;
    }

    // Check font size
    if let Some(size_line) = content.lines().find(|l| l.trim().starts_with("font_size")) {
        if let Some(size_str) = size_line.split_whitespace().nth(1) {
            if let Ok(size) = size_str.parse::<f32>() {
                config.has_good_font_size = size >= 10.0 && size <= 16.0;
            }
        }
    }

    // Kitty always uses GPU acceleration
    config.has_gpu_acceleration = true;
}

/// Analyze WezTerm configuration
fn analyze_wezterm_config(content: &str, config: &mut TerminalConfig) {
    // Check for Nerd Font
    if content.contains("Nerd") || content.contains("NF") {
        config.has_nerd_font = true;
    }

    // Check for color scheme
    if content.contains("color_scheme") || content.contains("colors") {
        config.has_color_scheme = true;
    }

    // WezTerm uses GPU by default
    config.has_gpu_acceleration = true;
}

/// Analyze Foot configuration
fn analyze_foot_config(content: &str, config: &mut TerminalConfig) {
    // Check for Nerd Font
    if content.contains("Nerd") || content.contains("NF") {
        config.has_nerd_font = true;
    }

    // Check for color scheme
    if content.contains("[colors]") {
        config.has_color_scheme = true;
    }

    // Check font size
    if let Some(size_line) = content.lines().find(|l| l.trim().starts_with("size=")) {
        if let Some(size_str) = size_line.split('=').nth(1) {
            if let Ok(size) = size_str.trim().parse::<f32>() {
                config.has_good_font_size = size >= 10.0 && size <= 16.0;
            }
        }
    }
}

/// Generate terminal improvement recommendations
pub fn generate_terminal_recommendations(config: &TerminalConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    // Recommend Nerd Fonts if not installed
    if !config.has_nerd_font {
        let install_cmd = match config.terminal_type {
            TerminalType::Alacritty => {
                r#"sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd && \
cat >> ~/.config/alacritty/alacritty.toml <<'EOF'

[font]
normal = { family = "JetBrainsMono Nerd Font", style = "Regular" }
bold = { family = "JetBrainsMono Nerd Font", style = "Bold" }
italic = { family = "JetBrainsMono Nerd Font", style = "Italic" }
size = 12.0
EOF"#
            }
            TerminalType::Kitty => {
                r#"sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd && \
echo 'font_family JetBrainsMono Nerd Font' >> ~/.config/kitty/kitty.conf && \
echo 'font_size 12.0' >> ~/.config/kitty/kitty.conf"#
            }
            TerminalType::WezTerm => {
                r#"sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd && \
cat >> ~/.config/wezterm/wezterm.lua <<'EOF'

config.font = wezterm.font('JetBrainsMono Nerd Font')
config.font_size = 12.0
EOF"#
            }
            TerminalType::Foot => {
                r#"sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd && \
echo 'font=JetBrainsMono Nerd Font:size=12' >> ~/.config/foot/foot.ini"#
            }
            _ => "sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd",
        };

        recommendations.push(Advice::new(
            "terminal-nerd-font".to_string(),
            "Install Nerd Font for your terminal".to_string(),
            format!(
                "Nerd Fonts include icons and glyphs that make your terminal beautiful and work perfectly with modern CLI tools like starship, eza, and bat. JetBrains Mono is an excellent choice for {:?}.",
                config.terminal_type
            ),
            "Install JetBrains Mono Nerd Font".to_string(),
            Some(install_cmd.to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Fonts".to_string()],
            "terminal".to_string(),
        ));
    }

    // Recommend color scheme if not configured
    if !config.has_color_scheme {
        let color_advice = match config.terminal_type {
            TerminalType::Alacritty => {
                Advice::new(
                    "terminal-color-scheme".to_string(),
                    "Add a color scheme to Alacritty".to_string(),
                    "A good color scheme makes your terminal easier on the eyes and improves readability. Consider using Catppuccin, Nord, or Dracula themes.".to_string(),
                    "Configure Alacritty colors".to_string(),
                    Some(r#"# Visit https://github.com/alacritty/alacritty-theme to browse themes
# Example: Install Catppuccin theme
mkdir -p ~/.config/alacritty/themes
git clone https://github.com/catppuccin/alacritty ~/.config/alacritty/themes/catppuccin
echo 'import = ["~/.config/alacritty/themes/catppuccin/catppuccin-mocha.toml"]' >> ~/.config/alacritty/alacritty.toml"#.to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://github.com/alacritty/alacritty-theme".to_string()],
                    "terminal".to_string(),
                )
            }
            TerminalType::Kitty => {
                Advice::new(
                    "terminal-color-scheme".to_string(),
                    "Add a color scheme to Kitty".to_string(),
                    "A good color scheme improves readability and reduces eye strain. Kitty has many built-in themes.".to_string(),
                    "Configure Kitty colors".to_string(),
                    Some(r#"# List available themes: kitty +kitten themes
# Example: Apply Dracula theme
kitty +kitten themes --reload-in=all Dracula"#.to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://sw.kovidgoyal.net/kitty/kittens/themes/".to_string()],
                    "terminal".to_string(),
                )
            }
            TerminalType::WezTerm => {
                Advice::new(
                    "terminal-color-scheme".to_string(),
                    "Add a color scheme to WezTerm".to_string(),
                    "WezTerm supports hundreds of color schemes. A good theme makes your terminal pleasant to use.".to_string(),
                    "Configure WezTerm colors".to_string(),
                    Some(r#"# Add to ~/.config/wezterm/wezterm.lua:
echo "config.color_scheme = 'Catppuccin Mocha'" >> ~/.config/wezterm/wezterm.lua
# Browse themes: https://wezfurlong.org/wezterm/colorschemes/"#.to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wezfurlong.org/wezterm/colorschemes/".to_string()],
                    "terminal".to_string(),
                )
            }
            _ => {
                return recommendations; // No specific recommendation for other terminals
            }
        };
        recommendations.push(color_advice);
    }

    // Recommend modern terminal if using outdated one
    match config.terminal_type {
        TerminalType::Xterm | TerminalType::Gnome => {
            recommendations.push(Advice::new(
                "terminal-upgrade-modern".to_string(),
                format!("Consider upgrading from {:?} to a modern terminal", config.terminal_type),
                format!(
                    "You're using {:?}, which lacks modern features like GPU acceleration, true color support, and ligatures. Consider switching to Alacritty (lightweight), Kitty (feature-rich), or Foot (Wayland-native).",
                    config.terminal_type
                ),
                "Install a modern terminal emulator".to_string(),
                Some("# Lightweight & fast:\nsudo pacman -S --noconfirm alacritty\n\n# Feature-rich:\nsudo pacman -S --noconfirm kitty\n\n# Wayland-native:\nsudo pacman -S --noconfirm foot".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/List_of_applications/Utilities#Terminal_emulators".to_string()],
                "terminal".to_string(),
            ));
        }
        _ => {}
    }

    recommendations
}

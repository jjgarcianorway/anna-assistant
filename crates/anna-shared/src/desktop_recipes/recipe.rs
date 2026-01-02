//! Desktop configuration recipe types and constructors.

use serde::{Deserialize, Serialize};

use super::actions::DesktopAction;
use super::environment::DesktopEnvironment;

/// A desktop configuration recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopRecipe {
    pub id: String,
    pub description: String,
    pub action: DesktopAction,
    pub desktop: DesktopEnvironment,
    pub command: String,
    pub rollback_command: Option<String>,
}

impl DesktopRecipe {
    /// Generate wallpaper command for detected desktop
    pub fn set_wallpaper(path: &str) -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let (command, rollback) = match desktop {
            DesktopEnvironment::Gnome => (
                format!(
                    "gsettings set org.gnome.desktop.background picture-uri 'file://{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Kde => (
                format!(
                    r#"qdbus org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript 'var allDesktops = desktops();for (i=0;i<allDesktops.length;i++){{d = allDesktops[i];d.wallpaperPlugin = "org.kde.image";d.currentConfigGroup = Array("Wallpaper","org.kde.image","General");d.writeConfig("Image","file://{}");}}'"#,
                    path
                ),
                None,
            ),
            DesktopEnvironment::Xfce => (
                format!(
                    "xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s '{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Cinnamon => (
                format!(
                    "gsettings set org.cinnamon.desktop.background picture-uri 'file://{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Mate => (
                format!(
                    "gsettings set org.mate.background picture-filename '{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_wallpaper".to_string(),
            description: format!("Set wallpaper to {}", path),
            action: DesktopAction::SetWallpaper {
                path: path.to_string(),
            },
            desktop,
            command,
            rollback_command: rollback,
        })
    }

    /// Generate dark mode command for detected desktop
    pub fn enable_dark_mode() -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let (command, rollback) = match desktop {
            DesktopEnvironment::Gnome => (
                "gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'".to_string(),
                Some(
                    "gsettings set org.gnome.desktop.interface color-scheme 'default'".to_string(),
                ),
            ),
            DesktopEnvironment::Kde => (
                "lookandfeeltool -a org.kde.breezedark.desktop".to_string(),
                Some("lookandfeeltool -a org.kde.breeze.desktop".to_string()),
            ),
            DesktopEnvironment::Xfce => (
                "xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita-dark'".to_string(),
                Some("xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita'".to_string()),
            ),
            DesktopEnvironment::Cinnamon => (
                "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita-dark'".to_string(),
                Some(
                    "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita'".to_string(),
                ),
            ),
            DesktopEnvironment::Mate => (
                "gsettings set org.mate.interface gtk-theme 'Adwaita-dark'".to_string(),
                Some("gsettings set org.mate.interface gtk-theme 'Adwaita'".to_string()),
            ),
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_dark_mode".to_string(),
            description: "Enable dark mode".to_string(),
            action: DesktopAction::EnableDarkMode,
            desktop,
            command,
            rollback_command: rollback,
        })
    }

    /// Generate disable dark mode command
    pub fn disable_dark_mode() -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let (command, rollback) = match desktop {
            DesktopEnvironment::Gnome => (
                "gsettings set org.gnome.desktop.interface color-scheme 'default'".to_string(),
                Some(
                    "gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'"
                        .to_string(),
                ),
            ),
            DesktopEnvironment::Kde => (
                "lookandfeeltool -a org.kde.breeze.desktop".to_string(),
                Some("lookandfeeltool -a org.kde.breezedark.desktop".to_string()),
            ),
            DesktopEnvironment::Xfce => (
                "xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita'".to_string(),
                Some("xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita-dark'".to_string()),
            ),
            DesktopEnvironment::Cinnamon => (
                "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita'".to_string(),
                Some(
                    "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita-dark'"
                        .to_string(),
                ),
            ),
            DesktopEnvironment::Mate => (
                "gsettings set org.mate.interface gtk-theme 'Adwaita'".to_string(),
                Some("gsettings set org.mate.interface gtk-theme 'Adwaita-dark'".to_string()),
            ),
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_light_mode".to_string(),
            description: "Disable dark mode (enable light mode)".to_string(),
            action: DesktopAction::DisableDarkMode,
            desktop,
            command,
            rollback_command: rollback,
        })
    }

    /// Generate theme change command
    pub fn set_theme(theme: &str) -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let command = match desktop {
            DesktopEnvironment::Gnome => {
                format!(
                    "gsettings set org.gnome.desktop.interface gtk-theme '{}'",
                    theme
                )
            }
            DesktopEnvironment::Kde => {
                format!("lookandfeeltool -a {}", theme)
            }
            DesktopEnvironment::Xfce => {
                format!("xfconf-query -c xsettings -p /Net/ThemeName -s '{}'", theme)
            }
            DesktopEnvironment::Cinnamon => {
                format!(
                    "gsettings set org.cinnamon.desktop.interface gtk-theme '{}'",
                    theme
                )
            }
            DesktopEnvironment::Mate => {
                format!("gsettings set org.mate.interface gtk-theme '{}'", theme)
            }
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_theme".to_string(),
            description: format!("Set theme to {}", theme),
            action: DesktopAction::SetTheme {
                theme: theme.to_string(),
            },
            desktop,
            command,
            rollback_command: None,
        })
    }
}

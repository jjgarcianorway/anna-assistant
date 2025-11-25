//! Applications recommendations

use super::{check_command_usage, is_package_installed};

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_screenshot_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if we're running a GUI
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        return result;
    }

    // flameshot - screenshot tool
    if !Command::new("which")
        .arg("flameshot")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-flameshot".to_string(),
            title: "Install flameshot - powerful screenshot tool".to_string(),
            reason: "flameshot is the best screenshot tool for Linux! Take screenshots with annotations, arrows, text, blur, and more. Much better than the default tools!".to_string(),
            action: "Install flameshot".to_string(),
            command: Some("sudo pacman -S --noconfirm flameshot".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Desktop Utilities".to_string(),
            alternatives: vec![
                Alternative {
                    name: "spectacle".to_string(),
                    description: "KDE screenshot utility".to_string(),
                    install_command: "sudo pacman -S --noconfirm spectacle".to_string(),
                },
                Alternative {
                    name: "gnome-screenshot".to_string(),
                    description: "GNOME screenshot tool".to_string(),
                    install_command: "sudo pacman -S --noconfirm gnome-screenshot".to_string(),
                },
            ],
            wiki_refs: vec!["https://wiki.archlinux.org/title/Screen_capture".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: Some("desktop-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 85,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_media_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for mpv (modern video player)
    let has_mpv = Command::new("pacman")
        .args(&["-Q", "mpv"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_mpv {
        result.push(Advice {
            id: "mpv".to_string(),
            title: "Install mpv media player".to_string(),
            reason: "mpv is a powerful, lightweight video player that plays everything. It's keyboard-driven, highly customizable, and handles any format you throw at it. It's also great for streaming and can be controlled via scripts!".to_string(),
            action: "Install mpv".to_string(),
            command: Some("pacman -S --noconfirm mpv".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Mpv".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for yt-dlp (youtube-dl fork)
    let has_ytdlp = Command::new("pacman")
        .args(&["-Q", "yt-dlp"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_ytdlp {
        result.push(Advice {
            id: "yt-dlp".to_string(),
            title: "Install yt-dlp for downloading videos".to_string(),
            reason: "yt-dlp is the best way to download videos from YouTube and hundreds of other sites. It's actively maintained (unlike youtube-dl), supports playlist downloads, can extract audio, and has tons of options. Essential tool for media archiving!".to_string(),
            action: "Install yt-dlp".to_string(),
            command: Some("pacman -S --noconfirm yt-dlp".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Yt-dlp".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for ffmpeg (video processing)
    let has_ffmpeg = Command::new("pacman")
        .args(&["-Q", "ffmpeg"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_ffmpeg {
        result.push(Advice {
            id: "ffmpeg".to_string(),
            title: "Install FFmpeg for video/audio processing".to_string(),
            reason: "FFmpeg is the Swiss Army knife of media processing. Convert videos, extract audio, resize, crop, merge files - it can do everything! Many apps depend on it, so it's practically essential for any media work.".to_string(),
            action: "Install FFmpeg".to_string(),
            command: Some("pacman -S --noconfirm ffmpeg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/FFmpeg".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_browser_recommendations() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for browsers
    let has_firefox = Command::new("pacman")
        .args(&["-Q", "firefox"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let has_chromium = Command::new("pacman")
        .args(&["-Q", "chromium"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let has_chrome = Command::new("pacman")
        .args(&["-Q", "google-chrome"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_firefox {
        // Suggest uBlock Origin reminder
        result.push(Advice {
            id: "browser-firefox-ublock".to_string(),
            title: "Reminder: Install uBlock Origin in Firefox".to_string(),
            reason: "You have Firefox! Make sure to install uBlock Origin extension for ad blocking and privacy. It's the best ad blocker - blocks ads, trackers, and malware. Essential for web browsing today! Also consider Privacy Badger and HTTPS Everywhere.".to_string(),
            action: "Install uBlock Origin from Firefox Add-ons".to_string(),
            command: Some("firefox --version 2>/dev/null || echo 'Firefox version check failed'".to_string()), // Show Firefox version
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Firefox#Privacy".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    if !has_firefox && !has_chromium && !has_chrome {
        result.push(Advice {
            id: "browser-install".to_string(),
            title: "Install a web browser".to_string(),
            reason: "No web browser detected! You need a browser to access the web. Choose based on your privacy and performance preferences.".to_string(),
            action: "Install Firefox (recommended) or choose an alternative".to_string(),
            command: Some("pacman -S --noconfirm firefox".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: vec![
                Alternative {
                    name: "Firefox".to_string(),
                    description: "Privacy-focused, open-source, excellent extension support, independent engine (Gecko)".to_string(),
                    install_command: "pacman -S --noconfirm firefox".to_string(),
                },
                Alternative {
                    name: "Chromium".to_string(),
                    description: "Fast, open-source base of Chrome, Blink engine, without Google services".to_string(),
                    install_command: "pacman -S --noconfirm chromium".to_string(),
                },
                Alternative {
                    name: "LibreWolf".to_string(),
                    description: "Firefox fork with enhanced privacy, no telemetry, uBlock Origin built-in".to_string(),
                    install_command: "yay -S --noconfirm librewolf-bin".to_string(),
                },
            ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Firefox".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_screen_recording() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for OBS Studio
    let has_obs = Command::new("pacman")
        .args(&["-Q", "obs-studio"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_obs {
        result.push(Advice {
            id: "recording-obs".to_string(),
            title: "Install OBS Studio for screen recording and streaming".to_string(),
            reason: "OBS Studio is THE tool for screen recording, streaming, and video capture! Record tutorials, gameplay, video calls, or live stream to Twitch/YouTube. It's professional-grade software used by streamers worldwide. Captures screen, webcam, audio, and more with tons of customization!".to_string(),
            action: "Install OBS Studio".to_string(),
            command: Some("pacman -S --noconfirm obs-studio".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Open_Broadcaster_Software".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for SimpleScreenRecorder (lighter alternative)
    let has_ssr = Command::new("pacman")
        .args(&["-Q", "simplescreenrecorder"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_obs && !has_ssr {
        result.push(Advice {
            id: "recording-ssr".to_string(),
            title: "Or try SimpleScreenRecorder for easy recording".to_string(),
            reason: "Want something simpler than OBS? SimpleScreenRecorder is lightweight and easy - just open, select area, and record! Great for quick screen recordings, tutorials, or capturing bugs. Less features than OBS but way simpler to use!".to_string(),
            action: "Install SimpleScreenRecorder".to_string(),
            command: Some("pacman -S --noconfirm simplescreenrecorder".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Screen_capture#SimpleScreenRecorder".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_text_editors() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check command history for vim usage
    let vim_usage = check_command_usage(&["vim", "vi"]);

    if vim_usage > 10 {
        // User uses vim frequently, suggest neovim
        let has_neovim = Command::new("pacman")
            .args(&["-Q", "neovim"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_neovim {
            result.push(Advice {
                id: "editor-neovim".to_string(),
                title: "Upgrade to Neovim for modern Vim experience".to_string(),
                reason: format!("You use vim {} times in your history! Neovim is vim with modern features: built-in LSP support, better async performance, Lua scripting, Tree-sitter for syntax highlighting, and an active plugin ecosystem. It's fully compatible with vim configs but way more powerful. Think of it as vim 2.0!", vim_usage),
                action: "Install Neovim".to_string(),
                command: Some("pacman -S --noconfirm neovim".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Neovim".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_mail_clients() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has email-related packages but no GUI client
    let has_thunderbird = Command::new("pacman")
        .args(&["-Q", "thunderbird"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_evolution = Command::new("pacman")
        .args(&["-Q", "evolution"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_thunderbird && !has_evolution {
        result.push(Advice {
            id: "mail-thunderbird".to_string(),
            title: "Install Thunderbird for email management".to_string(),
            reason: "Need an email client? Thunderbird is Mozilla's excellent email app! It handles multiple accounts (Gmail, Outlook, custom IMAP), has great spam filtering, calendar integration, and full PGP encryption support. Modern, fast, and privacy-focused. Perfect for managing all your email in one place!".to_string(),
            action: "Install Thunderbird".to_string(),
            command: Some("pacman -S --noconfirm thunderbird".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Productivity".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Thunderbird".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_torrent_clients() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for torrent files
    let has_torrent_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.torrent",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_torrent_files {
        // Check for qBittorrent
        let has_qbittorrent = Command::new("pacman")
            .args(&["-Q", "qbittorrent"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_qbittorrent {
            result.push(Advice {
                id: "torrent-qbittorrent".to_string(),
                title: "Install qBittorrent for torrent downloads".to_string(),
                reason: "You have torrent files! qBittorrent is an excellent, ad-free torrent client. Clean interface, built-in search, RSS support, sequential downloading, and full torrent creation. It's like uTorrent but open-source and without the bloat. Perfect for Linux ISOs and other legal torrents!".to_string(),
                action: "Install qBittorrent".to_string(),
                command: Some("pacman -S --noconfirm qbittorrent".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/QBittorrent".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_office_suite() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for office documents
    let has_office_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.docx",
            "-o",
            "-name",
            "*.xlsx",
            "-o",
            "-name",
            "*.pptx",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_office_files {
        // Check for LibreOffice
        let has_libreoffice = Command::new("pacman")
            .args(&["-Q", "libreoffice-fresh"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_libreoffice {
            result.push(Advice {
                id: "office-libreoffice".to_string(),
                title: "Install LibreOffice for document editing".to_string(),
                reason: "You have Office documents! LibreOffice is a full-featured office suite - Writer (Word), Calc (Excel), Impress (PowerPoint), plus Draw and Base. Opens Microsoft Office files, exports to PDF, fully compatible. It's the gold standard for open-source office software!".to_string(),
                action: "Install LibreOffice".to_string(),
                command: Some("pacman -S --noconfirm libreoffice-fresh".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Productivity".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/LibreOffice".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_graphics_software() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for image files
    let has_image_files = Command::new("find")
        .args(&[
            &format!("{}/Pictures", std::env::var("HOME").unwrap_or_default()),
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_image_files {
        // Check for GIMP
        let has_gimp = Command::new("pacman")
            .args(&["-Q", "gimp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gimp {
            result.push(Advice {
                id: "graphics-gimp".to_string(),
                title: "Install GIMP for photo editing".to_string(),
                reason: "GIMP is the open-source Photoshop! Professional photo editing, retouching, image manipulation, graphic design. Layers, masks, filters, brushes, everything you need. Used by professional designers and photographers. If you edit images, you need GIMP!".to_string(),
                action: "Install GIMP".to_string(),
                command: Some("pacman -S --noconfirm gimp".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Multimedia & Graphics".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GIMP".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Inkscape (vector graphics)
        let has_inkscape = Command::new("pacman")
            .args(&["-Q", "inkscape"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_inkscape {
            result.push(Advice {
                id: "graphics-inkscape".to_string(),
                title: "Install Inkscape for vector graphics".to_string(),
                reason: "Inkscape is the open-source Illustrator! Create logos, icons, diagrams, illustrations - anything that needs to scale without losing quality. SVG-native, professional features, used by designers worldwide. Perfect companion to GIMP!".to_string(),
                action: "Install Inkscape".to_string(),
                command: Some("pacman -S --noconfirm inkscape".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Multimedia & Graphics".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Inkscape".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_video_editing() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for video files
    let has_video_files = Command::new("find")
        .args(&[
            &format!("{}/Videos", std::env::var("HOME").unwrap_or_default()),
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_video_files {
        // Check for Kdenlive
        let has_kdenlive = Command::new("pacman")
            .args(&["-Q", "kdenlive"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_kdenlive {
            result.push(Advice {
                id: "video-kdenlive".to_string(),
                title: "Install Kdenlive for video editing".to_string(),
                reason: "You have video files! Kdenlive is a powerful, intuitive video editor. Multi-track editing, effects, transitions, color correction, audio mixing. Great for YouTube videos, family movies, or professional projects. It's like Adobe Premiere but free and open-source!".to_string(),
                action: "Install Kdenlive".to_string(),
                command: Some("pacman -S --noconfirm kdenlive".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Multimedia & Graphics".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Kdenlive".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_music_players() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for music files
    let has_music_files = Command::new("find")
        .args(&[
            &format!("{}/Music", std::env::var("HOME").unwrap_or_default()),
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_music_files {
        // Check for music players
        let has_mpd = Command::new("pacman")
            .args(&["-Q", "mpd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mpd {
            result.push(Advice {
                id: "music-mpd".to_string(),
                title: "Install MPD for music playback".to_string(),
                reason: "You have music files! MPD (Music Player Daemon) is a flexible, powerful music server. Control it from your phone, web browser, CLI, or GUI. Gapless playback, playlists, streaming, multiple outputs. It's the audiophile's choice - lightweight and feature-rich!".to_string(),
                action: "Install MPD and ncmpcpp client".to_string(),
                command: Some("pacman -S --noconfirm mpd ncmpcpp".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Multimedia & Graphics".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Music_Player_Daemon".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_pdf_readers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for PDF files
    let has_pdf_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.pdf",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_pdf_files {
        // Check for PDF readers
        let has_zathura = Command::new("pacman")
            .args(&["-Q", "zathura"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_okular = Command::new("pacman")
            .args(&["-Q", "okular"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_zathura && !has_okular {
            result.push(Advice {
                id: "pdf-zathura".to_string(),
                title: "Install Zathura for PDF viewing".to_string(),
                reason: "You have PDF files! Zathura is a minimal, vim-like PDF viewer. Keyboard-driven, fast, no bloat. Perfect for reading papers, books, or documents. If you prefer mouse-based, try Okular (KDE) or Evince (GNOME), but Zathura is the power user's choice!".to_string(),
                action: "Install Zathura with plugins".to_string(),
                command: Some("pacman -S --noconfirm zathura zathura-pdf-mupdf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zathura".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_code_editors() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for development activity
    let has_dev_files = check_command_usage(&["vim", "nano", "code", "emacs"]) > 10;

    if has_dev_files {
        // Check for VS Code
        let has_vscode = Command::new("pacman")
            .args(&["-Q", "code"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_vscode {
            result.push(Advice {
                id: "editor-vscode".to_string(),
                title: "Install Visual Studio Code for modern development".to_string(),
                reason: "VS Code is the most popular code editor! IntelliSense, debugging, Git integration, thousands of extensions, remote development. Works with every language. Industry standard for many developers. The open-source version 'code' is fully featured!".to_string(),
                action: "Install VS Code".to_string(),
                command: Some("pacman -S --noconfirm code".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Visual_Studio_Code".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_image_viewers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for image files
    let has_images = Command::new("find")
        .args(&[
            &format!("{}/Pictures", std::env::var("HOME").unwrap_or_default()),
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_images {
        // Check display server
        let is_x11 = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11";
        let is_wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";

        if is_x11 {
            let has_feh = Command::new("pacman")
                .args(&["-Q", "feh"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_feh {
                result.push(Advice {
                    id: "image-feh".to_string(),
                    title: "Install feh for lightweight image viewing".to_string(),
                    reason: "You have images! feh is a fast, lightweight image viewer for X11. View images, set wallpapers, create slideshows. Keyboard-driven, minimal, perfect for tiling WMs. 'feh image.jpg' or 'feh --bg-scale wallpaper.jpg' to set wallpaper!".to_string(),
                    action: "Install feh".to_string(),
                    command: Some("pacman -S --noconfirm feh".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "Multimedia & Graphics".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Feh".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        } else if is_wayland {
            let has_imv = Command::new("pacman")
                .args(&["-Q", "imv"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_imv {
                result.push(Advice {
                    id: "image-imv".to_string(),
                    title: "Install imv for Wayland image viewing".to_string(),
                    reason: "You have images and use Wayland! imv is a fast image viewer for Wayland (also works on X11). Lightweight, keyboard-driven, supports multiple formats. Like feh but for Wayland. 'imv image.jpg' to view!".to_string(),
                    action: "Install imv".to_string(),
                    command: Some("pacman -S --noconfirm imv".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "Multimedia & Graphics".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wayland#Image_viewers".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_communication_apps() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Discord
    let has_discord = Command::new("pacman")
        .args(&["-Q", "discord"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_discord {
        result.push(Advice {
            id: "chat-discord".to_string(),
            title: "Install Discord for gaming and community chat".to_string(),
            reason: "Discord is THE platform for gaming communities, developer groups, and online communities. Voice chat, screen sharing, servers, bots. Whether you're gaming, learning, or collaborating - Discord is where communities live!".to_string(),
            action: "Install Discord".to_string(),
            command: Some("pacman -S --noconfirm discord".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Communication".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Discord".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_scientific_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let python_usage = check_command_usage(&["python", "python3"]);

    if python_usage > 20 {
        // Check for Jupyter
        let has_jupyter = Command::new("pacman")
            .args(&["-Q", "jupyter-notebook"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_jupyter {
            result.push(Advice {
                id: "science-jupyter".to_string(),
                title: "Install Jupyter for interactive Python notebooks".to_string(),
                reason: "Jupyter notebooks are essential for data science! Interactive Python, inline plots, markdown notes, shareable. Used by researchers, data scientists, educators worldwide. 'jupyter notebook' starts web interface. Perfect for analysis, teaching, or exploration!".to_string(),
                action: "Install Jupyter Notebook".to_string(),
                command: Some("pacman -S --noconfirm jupyter-notebook".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Jupyter".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_3d_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for image files that might indicate 3D work
    let does_graphics = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.blend",
            "-o",
            "-name",
            "*.obj",
            "-o",
            "-name",
            "*.stl",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if does_graphics {
        let has_blender = Command::new("pacman")
            .args(&["-Q", "blender"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_blender {
            result.push(Advice {
                id: "3d-blender".to_string(),
                title: "Install Blender for 3D modeling and animation".to_string(),
                reason: "You have 3D files! Blender is THE free 3D creation suite. Modeling, sculpting, animation, rendering, video editing, game creation. Used by professionals for movies, games, architecture. Industry-standard, open-source, incredibly powerful!".to_string(),
                action: "Install Blender".to_string(),
                command: Some("pacman -S --noconfirm blender".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Multimedia & Graphics".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Blender".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_cad_software() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for CAD files
    let has_cad_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.scad",
            "-o",
            "-name",
            "*.FCStd",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_cad_files {
        let has_freecad = Command::new("pacman")
            .args(&["-Q", "freecad"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_freecad {
            result.push(Advice {
                id: "cad-freecad".to_string(),
                title: "Install FreeCAD for parametric 3D modeling".to_string(),
                reason: "You have CAD files! FreeCAD is open-source parametric CAD. Design parts, assemblies, mechanical systems. Great for 3D printing, engineering, product design. Like SolidWorks but free. Parametric means you can easily modify designs!".to_string(),
                action: "Install FreeCAD".to_string(),
                command: Some("pacman -S --noconfirm freecad".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Engineering & CAD".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/FreeCAD".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_markdown_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for markdown files
    let has_markdown = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.md",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_markdown {
        // Check for glow (markdown renderer)
        let has_glow = Command::new("which")
            .arg("glow")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_glow {
            result.push(Advice {
                id: "markdown-glow".to_string(),
                title: "Install glow for beautiful markdown rendering".to_string(),
                reason: "You have markdown files! glow renders markdown beautifully in the terminal. Syntax highlighting, styled text, images. Read READMEs, documentation, notes in style. 'glow README.md' or just 'glow' to browse. Way better than raw markdown!".to_string(),
                action: "Install glow".to_string(),
                command: Some("pacman -S --noconfirm glow".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications#Markdown".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_note_taking() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for many text/markdown files
    let has_notes = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.md",
            "-o",
            "-name",
            "*.txt",
            "-type",
            "f",
        ])
        .output()
        .map(|o| {
            let count = String::from_utf8_lossy(&o.stdout).lines().count();
            count > 20
        })
        .unwrap_or(false);

    if has_notes {
        let has_obsidian = Command::new("pacman")
            .args(&["-Q", "obsidian"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_obsidian {
            result.push(Advice {
                id: "notes-obsidian".to_string(),
                title: "Install Obsidian for powerful note-taking".to_string(),
                reason: "You have lots of notes! Obsidian is a powerful knowledge base using markdown files. Backlinks, graph view, plugins, themes. Local-first, your files stay yours. Perfect for PKM (Personal Knowledge Management), research, or journaling. Build your second brain!".to_string(),
                action: "Install Obsidian".to_string(),
                command: Some("pacman -S --noconfirm obsidian".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Productivity".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications#Note-taking_organizers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}


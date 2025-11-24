//! Recipe Modules - Deterministic ActionPlan Generators
//!
//! Beta.151: Hard-coded, testable recipes for common user scenarios
//! Beta.152: Expanded with systemd, network, system_update, and AUR recipes
//! Beta.153: Added SSH, firewall (UFW), and user management recipes
//! Beta.154: Added development environment recipes (Rust, Python, Node.js)
//! Beta.155: Added GPU driver management recipes (NVIDIA, AMD, Intel)
//! Beta.156: Added infrastructure recipes (Docker Compose, PostgreSQL, Nginx)
//! Beta.157: Added system management recipes (monitoring, backup, performance)
//! Beta.158: Added desktop application recipes (browser, media, productivity)
//! Beta.159: Added terminal and shell tool recipes (terminal, shell, compression)
//! Beta.160: Added communication and productivity recipes (communication, editor, sync)
//! Beta.161: Added gaming recipes (gaming, wine, gamepad)
//! Beta.162: Added security recipes (security, antivirus, vpn)
//! Beta.163: Added virtualization recipes (virtualization, containers, virt_network)
//! Beta.164: Added audio recipes (audio, music, recording)
//! Beta.165: Added video and desktop recipes (video, desktop_env, display_manager)
//! Beta.166: Added final core recipes (printing, bluetooth, cloud)
//! Beta.167: Added file management recipes (file_manager, archive_manager, pdf_reader)
//! Beta.168: Added screen utility recipes (screenshot, screencast, remote_desktop)
//! Beta.169: Added desktop utility recipes (launcher, clipboard, notifications)
//! Beta.170: Added communication recipes (email, passwords, torrents)
//! Beta.171: Added IRC/chat, image viewer, and note-taking recipes
//! Beta.172: Added productivity recipes (calendar, tasks, diagram)
//! Beta.173: Added content tools (ebooks, rss, sysinfo)
//! Beta.174: Added academic and network tools (latex, scientific, network_tools)
//! Beta.175: Added databases, CAD, and disk management (databases, cad_tools, disk_management)
//! Beta.176: Added build systems, rescue tools, and fonts (build_systems, rescue_tools, fonts)
//!
//! These modules generate predictable ActionPlans without relying on LLM
//! generation, reducing hallucination risk and ensuring consistent, safe
//! behavior for common tasks.
//!
//! Each recipe module:
//! - Detects if it matches a user request
//! - Uses telemetry to generate context-aware commands
//! - Provides proper checks, rollback, and risk classification
//! - Includes comprehensive tests

// Beta.151 recipes
pub mod docker;
pub mod neovim;
pub mod packages;
pub mod wallpaper;

// Beta.152 recipes
pub mod aur;
pub mod network;
pub mod systemd;
pub mod system_update;

// Beta.153 recipes
pub mod firewall;
pub mod ssh;
pub mod users;

// Beta.154 recipes
pub mod nodejs;
pub mod python;
pub mod rust;

// Beta.155 recipes
pub mod amd;
pub mod intel;
pub mod nvidia;

// Beta.156 recipes
pub mod docker_compose;
pub mod postgresql;
pub mod webserver;

// Beta.157 recipes
pub mod backup;
pub mod monitoring;
pub mod performance;

// Beta.158 recipes

// Beta.164 recipes
pub mod audio;
pub mod music;
pub mod recording;

// Beta.165 recipes
pub mod video;
pub mod desktop_env;
pub mod display_manager;

// Beta.166 recipes
pub mod printing;
pub mod bluetooth;
pub mod cloud;

// Beta.167 recipes
pub mod file_manager;
pub mod archive_manager;
pub mod pdf_reader;

// Beta.169 recipes
pub mod launcher;
pub mod clipboard;
pub mod notifications;

// Beta.170 recipes
pub mod email;
pub mod passwords;
pub mod torrents;

// Beta.171 recipes
pub mod irc;
pub mod imageview;
pub mod notes;

// Beta.172 recipes
pub mod calendar;
pub mod tasks;
pub mod diagram;

// Beta.173 recipes
pub mod ebooks;
pub mod rss;
pub mod sysinfo;

// Beta.174 recipes
pub mod latex;
pub mod scientific;
pub mod network_tools;

// Beta.175 recipes
pub mod databases;
pub mod cad_tools;
pub mod disk_management;

// Beta.176 recipes
pub mod build_systems;
pub mod rescue_tools;
pub mod fonts;

// Beta.168 recipes
pub mod screenshot;
pub mod screencast;
pub mod remote_desktop;

// Beta.163 recipes
pub mod virtualization;
pub mod containers;
pub mod virt_network;

// Beta.162 recipes
pub mod security;
pub mod antivirus;
pub mod vpn;

// Beta.161 recipes
pub mod gaming;
pub mod wine;
pub mod gamepad;

// Beta.159 recipes
pub mod communication;
pub mod editor;
pub mod sync;
pub mod compression;
pub mod shell;
pub mod terminal;
pub mod browser;
pub mod media;
pub mod productivity;

use anna_common::action_plan_v3::ActionPlan;
use anyhow::Result;
use std::collections::HashMap;

/// Try to match user request against known recipe patterns
///
/// Returns Some(ActionPlan) if a recipe matches, None if no match found
pub fn try_recipe_match(
    user_input: &str,
    telemetry: &HashMap<String, String>,
) -> Option<Result<ActionPlan>> {
    // Beta.152: Enhanced telemetry with user_request for sub-recipe routing
    let mut telemetry_with_request = telemetry.clone();
    telemetry_with_request.insert("user_request".to_string(), user_input.to_string());

    // Try each recipe in order of specificity
    // Beta.152: More specific recipes first to avoid false matches

    // AUR recipes (very specific)
    if aur::AurRecipe::matches_request(user_input) {
        return Some(aur::AurRecipe::build_plan(&telemetry_with_request));
    }

    // System update recipes (specific)
    if system_update::SystemUpdateRecipe::matches_request(user_input) {
        return Some(system_update::SystemUpdateRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.154 recipes - Development environments (specific)
    if rust::RustRecipe::matches_request(user_input) {
        return Some(rust::RustRecipe::build_plan(&telemetry_with_request));
    }

    if python::PythonRecipe::matches_request(user_input) {
        return Some(python::PythonRecipe::build_plan(&telemetry_with_request));
    }

    if nodejs::NodeJsRecipe::matches_request(user_input) {
        return Some(nodejs::NodeJsRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.164 recipes - Audio & Music (specific)
    if audio::AudioRecipe::matches_request(user_input) {
        return Some(audio::AudioRecipe::build_plan(&telemetry_with_request));
    }

    if music::MusicRecipe::matches_request(user_input) {
        return Some(music::MusicRecipe::build_plan(&telemetry_with_request));
    }

    if recording::RecordingRecipe::matches_request(user_input) {
        return Some(recording::RecordingRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.165 recipes - Video & Desktop (specific)
    if video::VideoRecipe::matches_request(user_input) {
        return Some(video::VideoRecipe::build_plan(&telemetry_with_request));
    }

    if desktop_env::DesktopEnvRecipe::matches_request(user_input) {
        return Some(desktop_env::DesktopEnvRecipe::build_plan(&telemetry_with_request));
    }

    if display_manager::DisplayManagerRecipe::matches_request(user_input) {
        return Some(display_manager::DisplayManagerRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.166 recipes - Final Core (specific)
    if printing::PrintingRecipe::matches_request(user_input) {
        return Some(printing::PrintingRecipe::build_plan(&telemetry_with_request));
    }

    if bluetooth::BluetoothRecipe::matches_request(user_input) {
        return Some(bluetooth::BluetoothRecipe::build_plan(&telemetry_with_request));
    }

    if cloud::CloudRecipe::matches_request(user_input) {
        return Some(cloud::CloudRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.167 recipes - File Management (specific)
    if file_manager::FileManagerRecipe::matches_request(user_input) {
        return Some(file_manager::FileManagerRecipe::build_plan(&telemetry_with_request));
    }

    if archive_manager::ArchiveManagerRecipe::matches_request(user_input) {
        return Some(archive_manager::ArchiveManagerRecipe::build_plan(&telemetry_with_request));
    }

    if pdf_reader::PdfReaderRecipe::matches_request(user_input) {
        return Some(pdf_reader::PdfReaderRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.169 recipes - Desktop Utilities (specific)
    if launcher::LauncherRecipe::matches_request(user_input) {
        return Some(launcher::LauncherRecipe::build_plan(&telemetry_with_request));
    }

    if clipboard::ClipboardRecipe::matches_request(user_input) {
        return Some(clipboard::ClipboardRecipe::build_plan(&telemetry_with_request));
    }

    if notifications::NotificationsRecipe::matches_request(user_input) {
        return Some(notifications::NotificationsRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.170 recipes - User Applications (specific)
    if email::EmailRecipe::matches_request(user_input) {
        return Some(email::EmailRecipe::build_plan(&telemetry_with_request));
    }

    if passwords::PasswordsRecipe::matches_request(user_input) {
        return Some(passwords::PasswordsRecipe::build_plan(&telemetry_with_request));
    }

    if torrents::TorrentsRecipe::matches_request(user_input) {
        return Some(torrents::TorrentsRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.171 recipes - Productivity Tools (specific)
    if irc::IrcRecipe::matches_request(user_input) {
        return Some(irc::IrcRecipe::build_plan(&telemetry_with_request));
    }

    if imageview::ImageViewRecipe::matches_request(user_input) {
        return Some(imageview::ImageViewRecipe::build_plan(&telemetry_with_request));
    }

    if notes::NotesRecipe::matches_request(user_input) {
        return Some(notes::NotesRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.172 recipes - Productivity Applications (specific)
    if calendar::CalendarRecipe::matches_request(user_input) {
        return Some(calendar::CalendarRecipe::build_plan(&telemetry_with_request));
    }

    if tasks::TasksRecipe::matches_request(user_input) {
        return Some(tasks::TasksRecipe::build_plan(&telemetry_with_request));
    }

    if diagram::DiagramRecipe::matches_request(user_input) {
        return Some(diagram::DiagramRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.173 recipes - Information Tools (specific)
    if ebooks::EbooksRecipe::matches_request(user_input) {
        return Some(ebooks::EbooksRecipe::build_plan(&telemetry_with_request));
    }

    if rss::RssRecipe::matches_request(user_input) {
        return Some(rss::RssRecipe::build_plan(&telemetry_with_request));
    }

    if sysinfo::SysinfoRecipe::matches_request(user_input) {
        return Some(sysinfo::SysinfoRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.174 recipes - Academic & Network Tools (specific)
    if latex::LatexRecipe::matches_request(user_input) {
        return Some(latex::LatexRecipe::build_plan(&telemetry_with_request));
    }

    if scientific::ScientificRecipe::matches_request(user_input) {
        return Some(scientific::ScientificRecipe::build_plan(&telemetry_with_request));
    }

    if network_tools::NetworkToolsRecipe::matches_request(user_input) {
        return Some(network_tools::NetworkToolsRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.175 recipes - Databases, CAD, Disk Management (specific)
    if databases::DatabasesRecipe::matches_request(user_input) {
        return Some(databases::DatabasesRecipe::build_plan(&telemetry_with_request));
    }

    if cad_tools::CADToolsRecipe::matches_request(user_input) {
        return Some(cad_tools::CADToolsRecipe::build_plan(&telemetry_with_request));
    }

    if disk_management::DiskManagementRecipe::matches_request(user_input) {
        return Some(disk_management::DiskManagementRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.176 recipes - Build systems, rescue tools, fonts (specific)
    if build_systems::BuildSystemsRecipe::matches_request(user_input) {
        return Some(build_systems::BuildSystemsRecipe::build_plan(&telemetry_with_request));
    }

    if rescue_tools::RescueToolsRecipe::matches_request(user_input) {
        return Some(rescue_tools::RescueToolsRecipe::build_plan(&telemetry_with_request));
    }

    if fonts::FontsRecipe::matches_request(user_input) {
        return Some(fonts::FontsRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.168 recipes - Screen Utilities (specific)
    if screenshot::ScreenshotRecipe::matches_request(user_input) {
        return Some(screenshot::ScreenshotRecipe::build_plan(&telemetry_with_request));
    }

    if screencast::ScreencastRecipe::matches_request(user_input) {
        return Some(screencast::ScreencastRecipe::build_plan(&telemetry_with_request));
    }

    if remote_desktop::RemoteDesktopRecipe::matches_request(user_input) {
        return Some(remote_desktop::RemoteDesktopRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.163 recipes - Virtualization (specific)
    if virtualization::VirtualizationRecipe::matches_request(user_input) {
        return Some(virtualization::VirtualizationRecipe::build_plan(&telemetry_with_request));
    }

    if containers::ContainersRecipe::matches_request(user_input) {
        return Some(containers::ContainersRecipe::build_plan(&telemetry_with_request));
    }

    if virt_network::VirtNetworkRecipe::matches_request(user_input) {
        return Some(virt_network::VirtNetworkRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.162 recipes - Security tools (specific)
    if security::SecurityRecipe::matches_request(user_input) {
        return Some(security::SecurityRecipe::build_plan(&telemetry_with_request));
    }

    if antivirus::AntivirusRecipe::matches_request(user_input) {
        return Some(antivirus::AntivirusRecipe::build_plan(&telemetry_with_request));
    }

    if vpn::VpnRecipe::matches_request(user_input) {
        return Some(vpn::VpnRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.161 recipes - Gaming tools (specific)
    if gaming::GamingRecipe::matches_request(user_input) {
        return Some(gaming::GamingRecipe::build_plan(&telemetry_with_request));
    }

    if wine::WineRecipe::matches_request(user_input) {
        return Some(wine::WineRecipe::build_plan(&telemetry_with_request));
    }

    if gamepad::GamepadRecipe::matches_request(user_input) {
        return Some(gamepad::GamepadRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.160 recipes - Communication and productivity (specific)
    if communication::CommunicationRecipe::matches_request(user_input) {
        return Some(communication::CommunicationRecipe::build_plan(&telemetry_with_request));
    }

    if editor::EditorRecipe::matches_request(user_input) {
        return Some(editor::EditorRecipe::build_plan(&telemetry_with_request));
    }

    if sync::SyncRecipe::matches_request(user_input) {
        return Some(sync::SyncRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.159 recipes - Terminal and shell tools (specific)
    if terminal::TerminalRecipe::matches_request(user_input) {
        return Some(terminal::TerminalRecipe::build_plan(&telemetry_with_request));
    }

    if shell::ShellRecipe::matches_request(user_input) {
        return Some(shell::ShellRecipe::build_plan(&telemetry_with_request));
    }

    if compression::CompressionRecipe::matches_request(user_input) {
        return Some(compression::CompressionRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.158 recipes - Desktop applications (specific)
    if browser::BrowserRecipe::matches_request(user_input) {
        return Some(browser::BrowserRecipe::build_plan(&telemetry_with_request));
    }

    if media::MediaRecipe::matches_request(user_input) {
        return Some(media::MediaRecipe::build_plan(&telemetry_with_request));
    }

    if productivity::ProductivityRecipe::matches_request(user_input) {
        return Some(productivity::ProductivityRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.157 recipes - System management (specific)
    if monitoring::MonitoringRecipe::matches_request(user_input) {
        return Some(monitoring::MonitoringRecipe::build_plan(&telemetry_with_request));
    }

    if backup::BackupRecipe::matches_request(user_input) {
        return Some(backup::BackupRecipe::build_plan(&telemetry_with_request));
    }

    if performance::PerformanceRecipe::matches_request(user_input) {
        return Some(performance::PerformanceRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.156 recipes - Infrastructure (specific)
    if docker_compose::DockerComposeRecipe::matches_request(user_input) {
        return Some(docker_compose::DockerComposeRecipe::build_plan(&telemetry_with_request));
    }

    if postgresql::PostgresqlRecipe::matches_request(user_input) {
        return Some(postgresql::PostgresqlRecipe::build_plan(&telemetry_with_request));
    }

    if webserver::WebServerRecipe::matches_request(user_input) {
        return Some(webserver::WebServerRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.155 recipes - GPU drivers (specific)
    if nvidia::NvidiaRecipe::matches_request(user_input) {
        return Some(nvidia::NvidiaRecipe::build_plan(&telemetry_with_request));
    }

    if amd::AmdRecipe::matches_request(user_input) {
        return Some(amd::AmdRecipe::build_plan(&telemetry_with_request));
    }

    if intel::IntelRecipe::matches_request(user_input) {
        return Some(intel::IntelRecipe::build_plan(&telemetry_with_request));
    }

    // Systemd service management (specific)
    if systemd::SystemdRecipe::matches_request(user_input) {
        return Some(systemd::SystemdRecipe::build_plan(&telemetry_with_request));
    }

    // Network diagnostics (specific)
    if network::NetworkRecipe::matches_request(user_input) {
        return Some(network::NetworkRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.153 recipes
    // SSH management (specific)
    if ssh::SshRecipe::matches_request(user_input) {
        return Some(ssh::SshRecipe::build_plan(&telemetry_with_request));
    }

    // Firewall management (specific)
    if firewall::FirewallRecipe::matches_request(user_input) {
        return Some(firewall::FirewallRecipe::build_plan(&telemetry_with_request));
    }

    // User and group management (specific)
    if users::UsersRecipe::matches_request(user_input) {
        return Some(users::UsersRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.151 recipes
    if docker::DockerRecipe::matches_request(user_input) {
        return Some(docker::DockerRecipe::build_plan(telemetry));
    }

    if wallpaper::WallpaperRecipe::matches_request(user_input) {
        return Some(wallpaper::WallpaperRecipe::build_plan(telemetry));
    }

    if neovim::NeovimRecipe::matches_request(user_input) {
        return Some(neovim::NeovimRecipe::build_plan(telemetry));
    }

    if packages::PackagesRecipe::matches_request(user_input) {
        return Some(packages::PackagesRecipe::build_plan(telemetry));
    }

    // No recipe matched
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_matching() {
        let telemetry = HashMap::new();

        // Beta.151 recipes
        assert!(try_recipe_match("install docker", &telemetry).is_some());
        assert!(try_recipe_match("change my wallpaper", &telemetry).is_some());
        assert!(try_recipe_match("install neovim", &telemetry).is_some());
        assert!(try_recipe_match("fix broken packages", &telemetry).is_some());

        // Beta.152 recipes
        assert!(try_recipe_match("enable NetworkManager service", &telemetry).is_some());
        assert!(try_recipe_match("restart bluetooth", &telemetry).is_some());
        assert!(try_recipe_match("check internet connection", &telemetry).is_some());
        assert!(try_recipe_match("show available wifi networks", &telemetry).is_some());
        assert!(try_recipe_match("check for system updates", &telemetry).is_some());
        assert!(try_recipe_match("update system", &telemetry).is_some());
        assert!(try_recipe_match("install package from AUR", &telemetry).is_some());
        assert!(try_recipe_match("do I have yay installed", &telemetry).is_some());

        // Beta.153 recipes
        assert!(try_recipe_match("install SSH server", &telemetry).is_some());
        assert!(try_recipe_match("generate SSH keys", &telemetry).is_some());
        assert!(try_recipe_match("install firewall", &telemetry).is_some());
        assert!(try_recipe_match("enable ufw", &telemetry).is_some());
        assert!(try_recipe_match("allow SSH through firewall", &telemetry).is_some());
        assert!(try_recipe_match("add user john", &telemetry).is_some());
        assert!(try_recipe_match("remove user testaccount", &telemetry).is_some());
        assert!(try_recipe_match("add user to docker group", &telemetry).is_some());
        assert!(try_recipe_match("list users", &telemetry).is_some());

        // Beta.154 recipes
        assert!(try_recipe_match("install Rust", &telemetry).is_some());
        assert!(try_recipe_match("install cargo and rustup", &telemetry).is_some());
        assert!(try_recipe_match("check Rust status", &telemetry).is_some());
        assert!(try_recipe_match("install Python", &telemetry).is_some());
        assert!(try_recipe_match("setup Python development environment", &telemetry).is_some());
        assert!(try_recipe_match("create Python venv", &telemetry).is_some());
        assert!(try_recipe_match("install Node.js", &telemetry).is_some());
        assert!(try_recipe_match("setup npm", &telemetry).is_some());
        assert!(try_recipe_match("initialize new npm project", &telemetry).is_some());

        // Beta.155 recipes
        assert!(try_recipe_match("install NVIDIA drivers", &telemetry).is_some());
        assert!(try_recipe_match("setup nvidia GPU", &telemetry).is_some());
        assert!(try_recipe_match("install CUDA", &telemetry).is_some());
        assert!(try_recipe_match("install AMD drivers", &telemetry).is_some());
        assert!(try_recipe_match("setup AMD GPU", &telemetry).is_some());
        assert!(try_recipe_match("install ROCm", &telemetry).is_some());
        assert!(try_recipe_match("install Intel drivers", &telemetry).is_some());
        assert!(try_recipe_match("setup Intel GPU", &telemetry).is_some());
        assert!(try_recipe_match("check nvidia status", &telemetry).is_some());

        // Beta.156 recipes
        assert!(try_recipe_match("install docker-compose", &telemetry).is_some());
        assert!(try_recipe_match("init docker compose project", &telemetry).is_some());
        assert!(try_recipe_match("validate docker-compose.yml", &telemetry).is_some());
        assert!(try_recipe_match("install postgresql", &telemetry).is_some());
        assert!(try_recipe_match("create postgres database", &telemetry).is_some());
        assert!(try_recipe_match("configure postgresql security", &telemetry).is_some());
        assert!(try_recipe_match("install nginx", &telemetry).is_some());
        assert!(try_recipe_match("create nginx site", &telemetry).is_some());
        assert!(try_recipe_match("enable nginx SSL", &telemetry).is_some());

        // Beta.157 recipes
        assert!(try_recipe_match("install monitoring tools", &telemetry).is_some());
        assert!(try_recipe_match("install htop", &telemetry).is_some());
        assert!(try_recipe_match("setup btop", &telemetry).is_some());
        assert!(try_recipe_match("install backup tools", &telemetry).is_some());
        assert!(try_recipe_match("setup rsync", &telemetry).is_some());
        assert!(try_recipe_match("install borg", &telemetry).is_some());
        assert!(try_recipe_match("install cpupower", &telemetry).is_some());
        assert!(try_recipe_match("set cpu governor", &telemetry).is_some());
        assert!(try_recipe_match("tune swappiness", &telemetry).is_some());

        // Beta.164 recipes
        assert!(try_recipe_match("install pipewire", &telemetry).is_some());
        assert!(try_recipe_match("install spotify", &telemetry).is_some());
        assert!(try_recipe_match("setup jack", &telemetry).is_some());

        // Beta.165 recipes
        assert!(try_recipe_match("install kdenlive", &telemetry).is_some());
        assert!(try_recipe_match("install blender", &telemetry).is_some());
        assert!(try_recipe_match("install gnome", &telemetry).is_some());
        assert!(try_recipe_match("install kde plasma", &telemetry).is_some());
        assert!(try_recipe_match("install sddm", &telemetry).is_some());
        assert!(try_recipe_match("switch to gdm", &telemetry).is_some());

        // Beta.166 recipes
        assert!(try_recipe_match("install cups", &telemetry).is_some());
        assert!(try_recipe_match("setup printer", &telemetry).is_some());
        assert!(try_recipe_match("install bluetooth", &telemetry).is_some());
        assert!(try_recipe_match("enable bluetooth", &telemetry).is_some());
        assert!(try_recipe_match("install nextcloud", &telemetry).is_some());
        assert!(try_recipe_match("setup dropbox", &telemetry).is_some());

        // Beta.167 recipes
        assert!(try_recipe_match("install nautilus", &telemetry).is_some());
        assert!(try_recipe_match("install file manager", &telemetry).is_some());
        assert!(try_recipe_match("install thunar", &telemetry).is_some());
        assert!(try_recipe_match("install file-roller", &telemetry).is_some());
        assert!(try_recipe_match("install ark", &telemetry).is_some());
        assert!(try_recipe_match("install evince", &telemetry).is_some());
        assert!(try_recipe_match("install pdf reader", &telemetry).is_some());

        // Beta.168 recipes
        assert!(try_recipe_match("install flameshot", &telemetry).is_some());
        assert!(try_recipe_match("install screenshot tool", &telemetry).is_some());
        assert!(try_recipe_match("install obs", &telemetry).is_some());
        assert!(try_recipe_match("install screen recording", &telemetry).is_some());
        assert!(try_recipe_match("install remmina", &telemetry).is_some());
        assert!(try_recipe_match("install remote desktop", &telemetry).is_some());

        // Beta.169 recipes
        assert!(try_recipe_match("install rofi", &telemetry).is_some());
        assert!(try_recipe_match("install launcher", &telemetry).is_some());
        assert!(try_recipe_match("install clipman", &telemetry).is_some());
        assert!(try_recipe_match("install clipboard manager", &telemetry).is_some());
        assert!(try_recipe_match("install dunst", &telemetry).is_some());
        assert!(try_recipe_match("install notification daemon", &telemetry).is_some());

        // Beta.170 recipes
        assert!(try_recipe_match("install thunderbird", &telemetry).is_some());
        assert!(try_recipe_match("install email client", &telemetry).is_some());
        assert!(try_recipe_match("install keepassxc", &telemetry).is_some());
        assert!(try_recipe_match("install password manager", &telemetry).is_some());
        assert!(try_recipe_match("install qbittorrent", &telemetry).is_some());
        assert!(try_recipe_match("install torrent client", &telemetry).is_some());

        // Beta.171 recipes
        assert!(try_recipe_match("install weechat", &telemetry).is_some());
        assert!(try_recipe_match("install irc client", &telemetry).is_some());
        assert!(try_recipe_match("install feh", &telemetry).is_some());
        assert!(try_recipe_match("install image viewer", &telemetry).is_some());
        assert!(try_recipe_match("install obsidian", &telemetry).is_some());
        assert!(try_recipe_match("install note-taking app", &telemetry).is_some());

        // Beta.172 recipes
        assert!(try_recipe_match("install gnome-calendar", &telemetry).is_some());
        assert!(try_recipe_match("install calendar app", &telemetry).is_some());
        assert!(try_recipe_match("install taskwarrior", &telemetry).is_some());
        assert!(try_recipe_match("install task management app", &telemetry).is_some());
        assert!(try_recipe_match("install draw.io", &telemetry).is_some());
        assert!(try_recipe_match("install diagram tool", &telemetry).is_some());

        // Beta.173 recipes
        assert!(try_recipe_match("install calibre", &telemetry).is_some());
        assert!(try_recipe_match("install ebook reader", &telemetry).is_some());
        assert!(try_recipe_match("install newsboat", &telemetry).is_some());
        assert!(try_recipe_match("install rss reader", &telemetry).is_some());
        assert!(try_recipe_match("install neofetch", &telemetry).is_some());
        assert!(try_recipe_match("install system info tool", &telemetry).is_some());

        // Beta.174 recipes
        assert!(try_recipe_match("install latex", &telemetry).is_some());
        assert!(try_recipe_match("install texstudio", &telemetry).is_some());
        assert!(try_recipe_match("install octave", &telemetry).is_some());
        assert!(try_recipe_match("install scientific computing", &telemetry).is_some());
        assert!(try_recipe_match("install wireshark", &telemetry).is_some());
        assert!(try_recipe_match("install nmap", &telemetry).is_some());

        // Beta.175 recipes
        assert!(try_recipe_match("install mariadb", &telemetry).is_some());
        assert!(try_recipe_match("install mongodb", &telemetry).is_some());
        assert!(try_recipe_match("install freecad", &telemetry).is_some());
        assert!(try_recipe_match("install kicad", &telemetry).is_some());
        assert!(try_recipe_match("install gparted", &telemetry).is_some());
        assert!(try_recipe_match("install disk management", &telemetry).is_some());

        // Beta.163 recipes
        assert!(try_recipe_match("install qemu", &telemetry).is_some());
        assert!(try_recipe_match("install podman", &telemetry).is_some());
        assert!(try_recipe_match("setup libvirt network", &telemetry).is_some());

        // Beta.162 recipes
        assert!(try_recipe_match("install fail2ban", &telemetry).is_some());
        assert!(try_recipe_match("install clamav", &telemetry).is_some());
        assert!(try_recipe_match("setup wireguard", &telemetry).is_some());

        // Beta.161 recipes
        assert!(try_recipe_match("install steam", &telemetry).is_some());
        assert!(try_recipe_match("install wine", &telemetry).is_some());
        assert!(try_recipe_match("setup gamepad", &telemetry).is_some());

        // Beta.160 recipes
        assert!(try_recipe_match("install discord", &telemetry).is_some());
        assert!(try_recipe_match("install vscode", &telemetry).is_some());
        assert!(try_recipe_match("install syncthing", &telemetry).is_some());

        // Beta.159 recipes
        assert!(try_recipe_match("install tmux", &telemetry).is_some());
        assert!(try_recipe_match("install zsh", &telemetry).is_some());
        assert!(try_recipe_match("install compression tools", &telemetry).is_some());

        // Beta.158 recipes
        assert!(try_recipe_match("install firefox", &telemetry).is_some());
        assert!(try_recipe_match("install chrome", &telemetry).is_some());
        assert!(try_recipe_match("check browser status", &telemetry).is_some());
        assert!(try_recipe_match("install vlc", &telemetry).is_some());
        assert!(try_recipe_match("install media player", &telemetry).is_some());
        assert!(try_recipe_match("install codecs", &telemetry).is_some());
        assert!(try_recipe_match("install libreoffice", &telemetry).is_some());
        assert!(try_recipe_match("install gimp", &telemetry).is_some());
        assert!(try_recipe_match("install office suite", &telemetry).is_some());

        // Generic query should not match
        assert!(try_recipe_match("what is the weather", &telemetry).is_none());
        assert!(try_recipe_match("tell me a joke", &telemetry).is_none());
    }

    #[test]
    fn test_recipe_priority() {
        let telemetry = HashMap::new();

        // AUR-specific queries should match AUR recipe, not generic package recipe
        let aur_match = try_recipe_match("install yay from AUR", &telemetry);
        assert!(aur_match.is_some());
        let plan = aur_match.unwrap().unwrap();
        assert!(plan.meta.detection_results.other.get("recipe_module")
            .and_then(|v| v.as_str())
            .unwrap_or("").contains("aur.rs"));

        // System update queries should match system_update recipe
        let update_match = try_recipe_match("update all packages", &telemetry);
        assert!(update_match.is_some());
        let plan = update_match.unwrap().unwrap();
        assert!(plan.meta.detection_results.other.get("recipe_module")
            .and_then(|v| v.as_str())
            .unwrap_or("").contains("system_update.rs"));
    }
}

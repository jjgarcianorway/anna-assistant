//! Knowledge Command v6.1.0 - System Highlights
//!
//! v6.1.0: Real highlights about the system Anna learned
//! - Overview counts
//! - Desktop environment (compositor, terminal, shell, editors)
//! - Languages/toolchains detected
//! - Key services
//!
//! This command answers: "What kind of system have I learned I am sitting on?"
//! NOT: "Here are some raw counts again"

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::{PackageCounts, get_package_info},
    commands::count_path_executables,
    services::ServiceCounts,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge".bold());
    println!("{}", THIN_SEP);
    println!();

    // [OVERVIEW] - counts
    print_overview_section();

    // [DESKTOP] - detected desktop environment
    print_desktop_section();

    // [LANGUAGES / TOOLCHAINS]
    print_languages_section();

    // [SERVICES]
    print_services_section();

    // [TIPS]
    print_tips_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_overview_section() {
    println!("{}", "[OVERVIEW]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Packages known:   {}", pkg_counts.total);
    println!("  Commands known:   {}", cmd_count);
    println!("  Services known:   {}", svc_counts.total);

    println!();
}

fn print_desktop_section() {
    println!("{}", "[DESKTOP]".cyan());

    use anna_common::grounded::commands::command_exists;

    // Compositor
    let compositor = if command_exists("hyprland") || command_exists("Hyprland") {
        "hyprland (Wayland)"
    } else if command_exists("sway") {
        "sway (Wayland)"
    } else if command_exists("wayfire") {
        "wayfire (Wayland)"
    } else if command_exists("river") {
        "river (Wayland)"
    } else if command_exists("gnome-shell") {
        "GNOME"
    } else if command_exists("plasmashell") {
        "KDE Plasma"
    } else if command_exists("i3") {
        "i3 (X11)"
    } else if command_exists("bspwm") {
        "bspwm (X11)"
    } else {
        "unknown"
    };
    println!("  Compositor:   {}", compositor.cyan());

    // Terminal
    let terminal = if command_exists("foot") {
        "foot"
    } else if command_exists("alacritty") {
        "alacritty"
    } else if command_exists("kitty") {
        "kitty"
    } else if command_exists("wezterm") {
        "wezterm"
    } else if command_exists("gnome-terminal") {
        "gnome-terminal"
    } else if command_exists("konsole") {
        "konsole"
    } else {
        "unknown"
    };
    println!("  Terminal:     {}", terminal);

    // Shell
    let shell = std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string());
    println!("  Shell:        {}", shell);

    // Editors
    let mut editors = Vec::new();
    let editor_candidates = ["nvim", "vim", "code", "helix", "nano", "emacs"];
    for editor in editor_candidates {
        if command_exists(editor) {
            editors.push(editor);
        }
    }
    if !editors.is_empty() {
        println!("  Editors:      {}", editors.join(", "));
    }

    println!();
}

fn print_languages_section() {
    println!("{}", "[LANGUAGES / TOOLCHAINS]".cyan());

    use anna_common::grounded::commands::command_exists;

    // C/C++
    let mut c_tools = Vec::new();
    for tool in ["gcc", "clang", "make", "cmake", "gdb"] {
        if command_exists(tool) {
            c_tools.push(tool);
        }
    }
    if !c_tools.is_empty() {
        println!("  C/C++:        {}", c_tools.join(", "));
    }

    // Rust
    if command_exists("rustc") {
        let version = get_package_info("rust")
            .map(|p| p.version)
            .unwrap_or_else(|| "installed".to_string());
        println!("  Rust:         rustc {}", version);
    }

    // Python
    if command_exists("python") || command_exists("python3") {
        let version = get_package_info("python")
            .map(|p| p.version)
            .unwrap_or_else(|| "installed".to_string());
        println!("  Python:       python {}", version);
    }

    // Go
    if command_exists("go") {
        let version = get_package_info("go")
            .map(|p| p.version)
            .unwrap_or_else(|| "installed".to_string());
        println!("  Go:           go {}", version);
    }

    // Node
    if command_exists("node") {
        let version = get_package_info("nodejs")
            .or_else(|| get_package_info("node"))
            .map(|p| p.version)
            .unwrap_or_else(|| "installed".to_string());
        println!("  Node.js:      node {}", version);
    }

    // Other tools
    let mut other_tools = Vec::new();
    for tool in ["git", "docker", "podman", "ffmpeg", "jq", "rsync"] {
        if command_exists(tool) {
            other_tools.push(tool);
        }
    }
    if !other_tools.is_empty() {
        println!("  Other:        {}", other_tools.join(", "));
    }

    println!();
}

fn print_services_section() {
    println!("{}", "[SERVICES]".cyan());

    use anna_common::grounded::services::get_service_info;

    // Anna
    if let Some(svc) = get_service_info("annad") {
        let state = match svc.state {
            anna_common::grounded::services::ServiceState::Active => "running".green().to_string(),
            _ => "stopped".to_string(),
        };
        println!("  Anna:         annad ({})", state);
    }

    // Audio
    let mut audio = Vec::new();
    for svc_name in ["pipewire", "pulseaudio", "wireplumber"] {
        if get_service_info(svc_name).map(|s| s.state == anna_common::grounded::services::ServiceState::Active).unwrap_or(false) {
            audio.push(svc_name);
        }
    }
    if !audio.is_empty() {
        println!("  Audio:        {}", audio.join(", "));
    }

    // Network
    let mut network = Vec::new();
    for svc_name in ["NetworkManager", "systemd-networkd", "sshd", "ssh"] {
        if get_service_info(svc_name).is_some() {
            network.push(svc_name);
        }
    }
    if !network.is_empty() {
        println!("  Network:      {}", network.join(", "));
    }

    // Display manager
    let mut dm = Vec::new();
    for svc_name in ["gdm", "sddm", "lightdm", "greetd"] {
        if get_service_info(svc_name).is_some() {
            dm.push(svc_name);
        }
    }
    if !dm.is_empty() {
        println!("  Display:      {}", dm.join(", "));
    }

    println!();
}

fn print_tips_section() {
    println!("{}", "[TIPS]".cyan());
    println!("  '{}' - full profile of a command or package", "annactl knowledge vim".dimmed());
    println!("  '{}' - overview of services", "annactl knowledge services".dimmed());
    println!("  '{}' - overview of editors", "annactl knowledge editors".dimmed());
    println!();
}

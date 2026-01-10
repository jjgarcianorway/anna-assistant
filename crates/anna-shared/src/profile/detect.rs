//! Enhanced profile detection - bootloader, shell, editor, AUR helper, etc.
//!
//! v0.0.863: Comprehensive system detection for better personalized assistance.

use std::process::Command;

use crate::user_context;

/// Detect display server (Wayland vs X11)
pub fn detect_display_server() -> Option<String> {
    // Check loginctl for session type
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("loginctl show-session $(loginctl list-sessions --no-legend | head -1 | awk '{print $1}') -p Type --value 2>/dev/null")
        .output()
    {
        let session_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !session_type.is_empty() && session_type != "tty" {
            return Some(session_type);
        }
    }

    // Check XDG_SESSION_TYPE
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if !session_type.is_empty() {
            return Some(session_type);
        }
    }

    // Check for running display servers
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("pgrep -x Xorg || pgrep -x Xwayland")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                return Some("x11/wayland".to_string());
            }
        }
    }

    None
}

/// Detect bootloader (systemd-boot, grub, limine, refind)
pub fn detect_bootloader() -> Option<String> {
    // Check for systemd-boot (most common on modern Arch)
    if std::path::Path::new("/boot/loader/loader.conf").exists() {
        return Some("systemd-boot".to_string());
    }

    // Check for GRUB
    if std::path::Path::new("/boot/grub/grub.cfg").exists()
        || std::path::Path::new("/etc/default/grub").exists()
    {
        return Some("grub".to_string());
    }

    // Check for Limine
    if std::path::Path::new("/boot/limine.cfg").exists()
        || std::path::Path::new("/boot/limine/limine.cfg").exists()
    {
        return Some("limine".to_string());
    }

    // Check for rEFInd
    if std::path::Path::new("/boot/EFI/refind").exists()
        || std::path::Path::new("/boot/refind_linux.conf").exists()
    {
        return Some("refind".to_string());
    }

    // Check bootctl for systemd-boot
    if let Ok(output) = Command::new("bootctl").arg("status").output() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if stdout.contains("systemd-boot") {
            return Some("systemd-boot".to_string());
        }
    }

    None
}

/// Detect user's default shell
pub fn detect_shell() -> Option<String> {
    // Get the actual logged-in user (not root if daemon is running as root)
    // First try SUDO_USER, then check loginctl for active sessions
    let real_user = std::env::var("SUDO_USER").ok().or_else(|| {
        // Get first logged-in user from loginctl
        Command::new("loginctl")
            .args(["list-users", "--no-legend"])
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .map(String::from)
            })
    });

    // Get shell for the real user
    if let Some(user) = real_user {
        if let Ok(output) = Command::new("getent").args(["passwd", &user]).output() {
            let line = String::from_utf8_lossy(&output.stdout);
            if let Some(shell_path) = line.trim().split(':').nth(6) {
                let shell_name = shell_path.rsplit('/').next().unwrap_or(shell_path);
                if !shell_name.is_empty() {
                    return Some(shell_name.to_string());
                }
            }
        }
    }

    // Fallback: find first user with UID >= 1000
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 {
                if let Ok(uid) = parts[2].parse::<u32>() {
                    if uid >= 1000 && uid < 65534 {
                        let shell_name = parts[6].rsplit('/').next().unwrap_or(parts[6]);
                        if !shell_name.is_empty()
                            && shell_name != "nologin"
                            && shell_name != "false"
                        {
                            return Some(shell_name.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Detect preferred editor
pub fn detect_editor() -> Option<String> {
    // Get user context to check their environment
    let user_ctx = user_context::get_user_context();

    // Check user's EDITOR and VISUAL environment variables
    if let Some(ctx) = user_ctx {
        // Try to get user's EDITOR variable
        if let Ok(output) = ctx.execute("echo $EDITOR") {
            let editor = output.trim();
            if !editor.is_empty() && editor != "$EDITOR" {
                let editor_name = editor.rsplit('/').next().unwrap_or(editor);
                return Some(editor_name.to_string());
            }
        }
        if let Ok(output) = ctx.execute("echo $VISUAL") {
            let editor = output.trim();
            if !editor.is_empty() && editor != "$VISUAL" {
                let editor_name = editor.rsplit('/').next().unwrap_or(editor);
                return Some(editor_name.to_string());
            }
        }
    }

    // Fallback: check daemon's env vars
    for var in &["EDITOR", "VISUAL"] {
        if let Ok(editor) = std::env::var(var) {
            let editor_name = editor.rsplit('/').next().unwrap_or(&editor);
            return Some(editor_name.to_string());
        }
    }

    // Check which editors are installed and pick the most likely default
    let editors = ["nvim", "vim", "nano", "emacs", "code", "vi"];
    for editor in &editors {
        if let Ok(output) = Command::new("which").arg(editor).output() {
            if output.status.success() {
                return Some(editor.to_string());
            }
        }
    }

    None
}

/// Detect AUR helper
pub fn detect_aur_helper() -> Option<String> {
    let helpers = ["paru", "yay", "pikaur", "trizen", "aurman"];
    for helper in &helpers {
        if let Ok(output) = Command::new("which").arg(helper).output() {
            if output.status.success() {
                return Some(helper.to_string());
            }
        }
    }
    None
}

/// Detect root filesystem type
pub fn detect_root_filesystem() -> Option<String> {
    // Use findmnt to get root filesystem
    if let Ok(output) = Command::new("findmnt")
        .args(["-n", "-o", "FSTYPE", "/"])
        .output()
    {
        if output.status.success() {
            let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !fstype.is_empty() {
                return Some(fstype);
            }
        }
    }

    // Fallback: check /proc/mounts
    if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "/" {
                return Some(parts[2].to_string());
            }
        }
    }

    None
}

/// Detect display manager
pub fn detect_display_manager() -> Option<String> {
    let dms = [
        ("gdm", "gdm.service"),
        ("sddm", "sddm.service"),
        ("lightdm", "lightdm.service"),
        ("ly", "ly.service"),
        ("greetd", "greetd.service"),
    ];

    for (name, service) in &dms {
        if let Ok(output) = Command::new("systemctl")
            .args(["is-active", service])
            .output()
        {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if status == "active" {
                return Some(name.to_string());
            }
        }
    }

    // Check display-manager.service symlink
    if let Ok(target) = std::fs::read_link("/etc/systemd/system/display-manager.service") {
        let target_str = target.display().to_string().to_lowercase();
        for (name, _) in &dms {
            if target_str.contains(name) {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Detect audio system
pub fn detect_audio_system() -> Option<String> {
    // Get user context for checking user services
    let user_ctx = user_context::get_user_context();

    // Check for PipeWire using user context if available
    if let Some(ctx) = user_ctx {
        // Run systemctl --user as the actual user
        if let Ok(output) =
            ctx.execute("systemctl --user is-active pipewire.service 2>/dev/null")
        {
            if output.trim() == "active" {
                return Some("pipewire".to_string());
            }
        }
        if let Ok(output) =
            ctx.execute("systemctl --user is-active pulseaudio.service 2>/dev/null")
        {
            if output.trim() == "active" {
                return Some("pulseaudio".to_string());
            }
        }
    }

    // Fallback: Check running processes (works regardless of user)
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("pgrep -x pipewire")
        .output()
    {
        if output.status.success() {
            return Some("pipewire".to_string());
        }
    }

    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("pgrep -x pulseaudio")
        .output()
    {
        if output.status.success() {
            return Some("pulseaudio".to_string());
        }
    }

    // Default to ALSA if nothing else detected
    Some("alsa".to_string())
}

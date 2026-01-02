//! Domain detection for cross-domain guard in similarity matching.
//!
//! v0.0.293: Domain keywords to detect query domain for cross-domain guard
//! v0.0.305: Expanded to cover desktop, editors, services, and more

/// Detect which domain a query belongs to based on keywords.
/// Returns None if query is domain-agnostic.
pub fn detect_query_domain(query: &str) -> Option<&'static str> {
    let q = query.to_lowercase();

    // Git/version control
    if q.contains("git")
        || q.contains("commit")
        || q.contains("push")
        || q.contains("pull")
        || q.contains("branch")
        || q.contains("merge")
        || q.contains("rebase")
    {
        return Some("git");
    }

    // Desktop/UI/Window managers (wallpaper, hyprland, wayland, gnome, kde, etc)
    if q.contains("wallpaper")
        || q.contains("desktop")
        || q.contains("background")
        || q.contains("hyprland")
        || q.contains("wayland")
        || q.contains("x11")
        || q.contains("xorg")
        || q.contains("gnome")
        || q.contains("kde")
        || q.contains("plasma")
        || q.contains("xfce")
        || q.contains("i3")
        || q.contains("sway")
        || q.contains("window")
        || q.contains("display")
        || q.contains("monitor")
        || q.contains("screen")
        || q.contains("resolution")
        || q.contains("theme")
        || q.contains("icon")
        || q.contains("font")
    {
        return Some("desktop");
    }

    // Editors
    if q.contains("nano")
        || q.contains("vim")
        || q.contains("nvim")
        || q.contains("neovim")
        || q.contains("emacs")
        || q.contains("vscode")
        || q.contains("code")
        || q.contains("editor")
        || q.contains("gedit")
        || q.contains("kate")
    {
        return Some("editor");
    }

    // Services/systemd
    if q.contains("service")
        || q.contains("systemctl")
        || q.contains("systemd")
        || q.contains("daemon")
        || q.contains("unit")
        || q.contains("enable")
        || q.contains("disable")
        || q.contains("restart")
        || q.contains("status")
    {
        return Some("services");
    }

    // Storage/disk
    if q.contains("disk")
        || q.contains("storage")
        || q.contains("space")
        || q.contains("df")
        || q.contains("mount")
        || q.contains("partition")
        || q.contains("filesystem")
        || q.contains("drive")
        || q.contains("ssd")
        || q.contains("hdd")
        || q.contains("nvme")
    {
        return Some("storage");
    }

    // Network
    if q.contains("network")
        || q.contains("wifi")
        || q.contains("internet")
        || q.contains("ip")
        || q.contains("dns")
        || q.contains("ping")
        || q.contains("ethernet")
        || q.contains("ssh")
        || q.contains("firewall")
        || q.contains("port")
        || q.contains("connection")
    {
        return Some("network");
    }

    // CPU/performance
    if q.contains("cpu")
        || q.contains("core")
        || q.contains("processor")
        || q.contains("load")
        || q.contains("temperature")
        || q.contains("temp")
        || q.contains("fan")
    {
        return Some("cpu");
    }

    // Memory
    if q.contains("memory") || q.contains("ram") || q.contains("swap") || q.contains("oom") {
        return Some("memory");
    }

    // Packages/installation
    if q.contains("install")
        || q.contains("package")
        || q.contains("pacman")
        || q.contains("apt")
        || q.contains("yum")
        || q.contains("dnf")
        || q.contains("flatpak")
        || q.contains("snap")
        || q.contains("aur")
        || q.contains("yay")
        || q.contains("paru")
    {
        return Some("packages");
    }

    // Processes
    if q.contains("process")
        || q.contains("kill")
        || q.contains("ps")
        || q.contains("top")
        || q.contains("htop")
        || q.contains("running")
        || q.contains("pid")
    {
        return Some("processes");
    }

    // Files/filesystem
    if q.contains("file")
        || q.contains("folder")
        || q.contains("directory")
        || q.contains("path")
        || q.contains("permission")
        || q.contains("chmod")
        || q.contains("chown")
        || q.contains("find")
        || q.contains("search")
        || q.contains("locate")
    {
        return Some("files");
    }

    // Logs/journald
    if q.contains("log")
        || q.contains("journal")
        || q.contains("journalctl")
        || q.contains("dmesg")
        || q.contains("syslog")
        || q.contains("error")
    {
        return Some("logs");
    }

    // Users/groups
    if q.contains("user")
        || q.contains("group")
        || q.contains("sudo")
        || q.contains("root")
        || q.contains("password")
        || q.contains("login")
        || q.contains("session")
    {
        return Some("users");
    }

    // Audio
    if q.contains("audio")
        || q.contains("sound")
        || q.contains("volume")
        || q.contains("speaker")
        || q.contains("microphone")
        || q.contains("pulse")
        || q.contains("pipewire")
        || q.contains("alsa")
    {
        return Some("audio");
    }

    // Time/date
    if q.contains("time")
        || q.contains("date")
        || q.contains("timezone")
        || q.contains("clock")
        || q.contains("ntp")
    {
        return Some("time");
    }

    None
}

//! Helper functions for query scenario tests (v0.0.268).
//!
//! Shared domain and route class inference functions.

#[cfg(test)]
/// Infer domain from query for routing test
pub(super) fn infer_domain(query: &str) -> &'static str {
    let q = query.to_lowercase();

    // Check desktop first since it has many specific keywords
    if q.contains("vim")
        || q.contains("nano")
        || q.contains("editor")
        || q.contains("emacs")
        || q.contains("helix")
        || q.contains("neovim")
        || q.contains("hyprland")
        || q.contains("gnome")
        || q.contains("kde")
        || q.contains("gtk")
        || q.contains("wayland")
        || q.contains("x11")
        || q.contains("theme")
        || q.contains("font")
        || q.contains("hidpi")
        || q.contains("tmux")
        || q.contains("bash prompt")
        || q.contains("ps1")
        || q.contains("sway")
        || q.contains("i3")
        || q.contains("dark mode")
        || q.contains("shortcut")
        || q.contains("keybind")
        || q.contains("screenshot")
    {
        return "desktop";
    }
    // Services - check before network to handle "postgresql connections" correctly
    if q.contains("systemd")
        || q.contains("nginx")
        || q.contains("docker")
        || q.contains("apache")
        || q.contains("cron")
        || q.contains("timer")
        || q.contains("daemon")
        || q.contains("postgresql")
        || q.contains("httpd")
        || q.contains("sshd")
        || q.contains("mysql")
        || q.contains("mariadb")
        || q.contains("redis")
        || q.contains("mongodb")
    {
        return "services";
    }
    // Security - check before logs to handle "login" vs "log" correctly
    if q.contains("permission")
        || q.contains("ssh key")
        || q.contains("security")
        || q.contains("fail2ban")
        || q.contains("gpg")
        || q.contains("encrypt")
        || q.contains("ufw")
        || q.contains("harden")
        || q.contains("login")
    {
        return "security";
    }
    // Logs - check "log" after security to avoid "login" -> "log" match
    if (q.contains("log") && !q.contains("login"))
        || q.contains("journal")
        || q.contains("dmesg")
        || q.contains("syslog")
        || q.contains("crash")
        || q.contains("kernel messages")
    {
        return "logs";
    }
    // Hardware - check before performance to handle "CPU temperature" correctly
    if q.contains("gpu")
        || q.contains("nvidia")
        || q.contains("bluetooth")
        || q.contains("sound")
        || q.contains("audio")
        || q.contains("webcam")
        || q.contains("keyboard backlight")
        || q.contains("driver")
        || q.contains("temperature")
        || q.contains("monitor")
        || q.contains("display")
        || q.contains("cpu cores")
        || q.contains("how many cpu")
        || q.contains("ram speed")
        || q.contains("ram type")
        || q.contains("check ram")
    {
        return "hardware";
    }
    // Network - check "network slow" case specifically
    if q.contains("network")
        || q.contains("wifi")
        || q.contains("ip ")
        || q.contains("dns")
        || q.contains("vpn")
        || q.contains("internet")
        || q.contains("bonding")
        || q.contains("bridge")
        || q.contains("connected")
    {
        return "network";
    }
    // Performance - check before storage to handle "benchmark disk io" correctly
    // v0.0.273: More precise performance keywords (avoid matching general queries)
    if q.contains("benchmark")
        || q.contains("iowait")
        || (q.contains("slow") && !q.contains("network"))
        || q.contains("load") && q.contains("system")
        || q.contains("performance")
        || q.contains("swap")
        || (q.contains("cpu")
            && !q.contains("cpu cores")
            && !q.contains("cpu info")
            && !q.contains("how many cpu")
            && !q.contains("temperature"))
        || (q.contains("ram")
            && !q.contains("ram speed")
            && !q.contains("ram type")
            && !q.contains("check ram")
            && q.contains("using"))
        || (q.contains("memory") && !q.contains("memory info") && q.contains("using"))
        || q.contains("power consumption")
        || (q.contains("tune") && q.contains("kernel"))
        || (q.contains("frequency") && q.contains("cpu"))
    {
        return "performance";
    }
    // Storage
    if q.contains("disk")
        || q.contains("storage")
        || q.contains("mount")
        || q.contains("partition")
        || q.contains("btrfs")
        || q.contains("filesystem")
        || q.contains("space")
        || q.contains("inode")
        || q.contains("lsblk")
    {
        return "storage";
    }

    "system"
}

#[cfg(test)]
/// Infer route class from query - mirrors team_from_route_class patterns
pub(super) fn infer_route_class(query: &str) -> &'static str {
    let q = query.to_lowercase();

    // Desktop - editors and DE/WM
    if q.contains("vim")
        || q.contains("nano")
        || q.contains("editor")
        || q.contains("emacs")
        || q.contains("helix")
        || q.contains("neovim")
        || q.contains("gnome")
        || q.contains("kde")
        || q.contains("hyprland")
        || q.contains("sway")
        || q.contains("i3")
        || q.contains("wayland")
        || q.contains("x11")
        || q.contains("gtk")
        || q.contains("theme")
        || q.contains("font")
        || q.contains("hidpi")
        || q.contains("dark mode")
        || q.contains("tmux")
        || q.contains("bash prompt")
        || q.contains("ps1")
        || q.contains("shortcut")
        || q.contains("keybind")
        || q.contains("screenshot")
    {
        return "desktop";
    }
    // Services - check before network to handle database connection issues
    if q.contains("service")
        || q.contains("systemd")
        || q.contains("nginx")
        || q.contains("docker")
        || q.contains("apache")
        || q.contains("cron")
        || q.contains("timer")
        || q.contains("daemon")
        || q.contains("postgresql")
        || q.contains("httpd")
        || q.contains("sshd")
        || q.contains("mysql")
        || q.contains("mariadb")
        || q.contains("redis")
        || q.contains("mongodb")
    {
        return "service";
    }
    // Security - check before logs to handle "login" vs "log" correctly
    if q.contains("permission")
        || q.contains("ssh key")
        || q.contains("ufw")
        || q.contains("fail2ban")
        || q.contains("gpg")
        || q.contains("encrypt")
        || q.contains("security")
        || q.contains("harden")
        || q.contains("login")
    {
        return "security";
    }
    // Logs - check "log" after security to avoid "login" -> "log" match
    if (q.contains("log") && !q.contains("login"))
        || q.contains("journal")
        || q.contains("dmesg")
        || q.contains("syslog")
        || q.contains("crash")
        || q.contains("kernel messages")
    {
        return "log";
    }
    // Hardware - check before performance to handle "CPU temperature" correctly
    if q.contains("gpu")
        || q.contains("nvidia")
        || q.contains("bluetooth")
        || q.contains("sound")
        || q.contains("audio")
        || q.contains("webcam")
        || q.contains("monitor")
        || q.contains("display")
        || q.contains("driver")
        || q.contains("cpu cores")
        || q.contains("how many cpu")
        || q.contains("ram speed")
        || q.contains("ram type")
        || q.contains("check ram")
        || q.contains("temperature")
    {
        return "hardware";
    }
    // Network
    if q.contains("network")
        || q.contains("wifi")
        || q.contains("ip ")
        || q.contains("dns")
        || q.contains("port")
        || q.contains("firewall")
        || q.contains("vpn")
        || q.contains("internet")
        || q.contains("connection")
        || q.contains("bonding")
        || q.contains("bridge")
    {
        return "network";
    }
    // Performance - more precise matching (avoid matching general queries)
    // v0.0.273: Tightened performance keywords
    if q.contains("benchmark")
        || q.contains("iowait")
        || (q.contains("slow") && !q.contains("network"))
        || (q.contains("load") && q.contains("system"))
        || q.contains("performance")
        || q.contains("swap")
        || (q.contains("cpu")
            && !q.contains("cpu cores")
            && !q.contains("cpu info")
            && !q.contains("how many cpu")
            && !q.contains("temperature"))
        || (q.contains("ram")
            && !q.contains("ram speed")
            && !q.contains("ram type")
            && !q.contains("check ram")
            && q.contains("using"))
        || (q.contains("memory") && !q.contains("memory info") && q.contains("using"))
        || q.contains("power consumption")
        || (q.contains("tune") && q.contains("kernel"))
        || (q.contains("frequency") && q.contains("cpu"))
    {
        return "performance";
    }
    // Storage
    if q.contains("disk")
        || q.contains("space")
        || q.contains("mount")
        || q.contains("partition")
        || q.contains("btrfs")
        || q.contains("filesystem")
        || q.contains("inode")
        || q.contains("lsblk")
        || q.contains("du ")
    {
        return "disk";
    }

    ""
}

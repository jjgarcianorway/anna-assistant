//! System context gathering for LLM prompts.
//! Gathers real system info via quick probes, cached per process lifetime.

/// System context that goes into every prompt.
pub fn system_context() -> String {
    use std::sync::OnceLock;
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED.get_or_init(build_system_context).clone()
}

fn cmd_output(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn env_or_cmd(var: &str, cmd: &str, args: &[&str]) -> String {
    std::env::var(var)
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| cmd_output(cmd, args))
        .unwrap_or_else(|| "unknown".to_string())
}

fn build_system_context() -> String {
    let distro = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Linux".to_string());

    let kernel = cmd_output("uname", &["-r"]).unwrap_or_else(|| "unknown".to_string());
    let de = env_or_cmd("XDG_CURRENT_DESKTOP", "echo", &[]);
    let session_type = env_or_cmd("XDG_SESSION_TYPE", "loginctl", &["show-session", "auto", "-p", "Type", "--value"]);

    let dm = cmd_output("systemctl", &["show", "display-manager.service", "-p", "Id", "--value"])
        .or_else(|| {
            for dm in &["gdm", "sddm", "lightdm"] {
                if cmd_output("systemctl", &["is-active", &format!("{}.service", dm)])
                    .map(|s| s == "active")
                    .unwrap_or(false)
                {
                    return Some(dm.to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| "unknown".to_string());

    let pkg_mgr = if std::path::Path::new("/usr/bin/pacman").exists() {
        "pacman (Arch-based)"
    } else if std::path::Path::new("/usr/bin/apt").exists() {
        "apt (Debian-based)"
    } else if std::path::Path::new("/usr/bin/dnf").exists() {
        "dnf (Fedora-based)"
    } else {
        "unknown"
    };

    let init = if std::path::Path::new("/run/systemd/system").exists() {
        "systemd"
    } else {
        "other"
    };

    format!(
        "You are Anna, an autonomous Linux system administrator.\n\
         System: {distro} (kernel {kernel})\n\
         Desktop: {de} on {session_type}\n\
         Display manager: {dm}\n\
         Package manager: {pkg_mgr}\n\
         Init: {init}\n\
         \n\
         You are an AGENT that DOES things, not just answers questions.\n\
         When asked to fix something, you fix it.\n\
         When you find a problem, you solve it.\n\
         You execute commands and make changes. The user trusts you."
    )
}

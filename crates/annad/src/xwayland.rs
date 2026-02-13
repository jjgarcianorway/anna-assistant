//! Per-app XWayland detection.
//!
//! Identifies which running applications are using XWayland vs native Wayland.
//! Uses: xlsclients (lists X11 clients), /proc/<pid>/environ (WAYLAND_DISPLAY presence),
//! and xprop -root to detect the X11 display server itself.

use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tracing::debug;

/// Session display mode
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayMode {
    NativeWayland,
    XWayland,
    X11Native,
    Unknown,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayMode::NativeWayland => write!(f, "native Wayland"),
            DisplayMode::XWayland => write!(f, "XWayland"),
            DisplayMode::X11Native => write!(f, "native X11"),
            DisplayMode::Unknown => write!(f, "unknown"),
        }
    }
}

/// An application and its display mode
#[derive(Debug, Clone)]
pub struct AppDisplayMode {
    pub pid: u32,
    pub name: String,
    pub mode: DisplayMode,
}

/// Full session display report
#[derive(Debug, Clone)]
pub struct DisplayReport {
    /// Overall session type
    pub session_type: String,
    /// Whether XWayland is running at all
    pub xwayland_running: bool,
    /// Per-app breakdown
    pub apps: Vec<AppDisplayMode>,
}

impl DisplayReport {
    /// Detect current session display report.
    pub fn detect() -> Self {
        let session_type = std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_else(|_| "unknown".to_string())
            .to_lowercase();

        let xwayland_running = is_xwayland_running();
        let apps = if xwayland_running || session_type == "wayland" {
            classify_all_apps()
        } else {
            vec![] // Pure X11 — classification not useful
        };

        Self { session_type, xwayland_running, apps }
    }

    /// Summary for briefing/context
    pub fn summary(&self) -> String {
        if !self.xwayland_running && self.session_type == "wayland" {
            return "Wayland session: all apps running natively (no XWayland)".to_string();
        }
        if self.session_type == "x11" {
            return "X11 session".to_string();
        }

        let native_count = self.apps.iter().filter(|a| a.mode == DisplayMode::NativeWayland).count();
        let xw_count = self.apps.iter().filter(|a| a.mode == DisplayMode::XWayland).count();

        let xw_names: Vec<&str> = self.apps.iter()
            .filter(|a| a.mode == DisplayMode::XWayland)
            .map(|a| a.name.as_str())
            .take(5)
            .collect();

        if xw_count == 0 {
            format!("Wayland: {} apps native, 0 using XWayland", native_count)
        } else {
            format!(
                "Wayland: {} native, {} via XWayland ({})",
                native_count, xw_count,
                xw_names.join(", ")
            )
        }
    }

    /// List apps that could benefit from native Wayland
    pub fn xwayland_apps(&self) -> Vec<&AppDisplayMode> {
        self.apps.iter().filter(|a| a.mode == DisplayMode::XWayland).collect()
    }
}

fn is_xwayland_running() -> bool {
    // Check if Xwayland process is running
    Command::new("pgrep")
        .args(["-x", "Xwayland"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Classify all running apps by their display protocol
fn classify_all_apps() -> Vec<AppDisplayMode> {
    // Get X11 clients (apps using XWayland) via xlsclients
    let xwayland_pids = get_xwayland_pids();

    // Get all running processes
    let procs = list_running_procs();

    let mut apps = Vec::new();
    for (pid, name) in procs {
        // Skip kernel threads and system processes
        if name.starts_with('[') || name.is_empty() {
            continue;
        }

        let mode = if xwayland_pids.contains(&pid) {
            DisplayMode::XWayland
        } else if has_wayland_display(pid) {
            DisplayMode::NativeWayland
        } else {
            continue; // Not a GUI app
        };

        debug!("App {} (PID {}): {}", name, pid, mode);
        apps.push(AppDisplayMode { pid, name, mode });
    }

    // Deduplicate by name (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    apps.retain(|a| seen.insert(a.name.clone()));

    apps
}

/// Get PIDs of processes connected to X11/XWayland via xlsclients
fn get_xwayland_pids() -> std::collections::HashSet<u32> {
    let mut pids = std::collections::HashSet::new();

    // xlsclients -l shows window IDs; we cross-reference with /proc/<pid>/net/unix
    // Simpler: use xdotool or wmctrl, but those aren't always available.
    // Reliable fallback: check /proc/<pid>/fd for X11 socket connections
    let x11_socket = find_x11_socket();
    if let Some(socket_path) = x11_socket {
        pids.extend(pids_connected_to_socket(&socket_path));
    }

    // Also try xlsclients for window names
    if let Ok(output) = Command::new("xlsclients").arg("-l").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("pid:") {
                    if let Some(pid_str) = line.split("pid:").nth(1) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            pids.insert(pid);
                        }
                    }
                }
            }
        }
    }

    pids
}

fn find_x11_socket() -> Option<String> {
    // XWayland typically creates /tmp/.X11-unix/X0 or X1
    for n in 0..5 {
        let path = format!("/tmp/.X11-unix/X{}", n);
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

fn pids_connected_to_socket(socket_path: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    let proc_base = std::path::Path::new("/proc");
    let entries = match fs::read_dir(proc_base) {
        Ok(e) => e,
        Err(_) => return pids,
    };

    for entry in entries.flatten() {
        let pid_str = entry.file_name().to_string_lossy().to_string();
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_dir = entry.path().join("fd");
        let fd_entries = match fs::read_dir(&fd_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for fd_entry in fd_entries.flatten() {
            if let Ok(target) = fs::read_link(fd_entry.path()) {
                if target.to_string_lossy().contains(socket_path) {
                    pids.push(pid);
                    break;
                }
            }
        }
    }

    pids
}

/// Check if a process has WAYLAND_DISPLAY in its environment
fn has_wayland_display(pid: u32) -> bool {
    let environ_path = format!("/proc/{}/environ", pid);
    match fs::read(&environ_path) {
        Ok(bytes) => {
            // environ is NUL-separated KEY=VALUE pairs
            bytes.split(|&b| b == 0)
                .any(|entry| entry.starts_with(b"WAYLAND_DISPLAY="))
        }
        Err(_) => false,
    }
}

/// List running GUI processes (name, pid)
fn list_running_procs() -> Vec<(u32, String)> {
    let output = Command::new("ps")
        .args(["-eo", "pid,comm", "--no-headers"])
        .output();

    match output {
        Ok(o) => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split_whitespace();
                    let pid = parts.next()?.parse::<u32>().ok()?;
                    let name = parts.next()?.to_string();
                    Some((pid, name))
                })
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Build a telemetry section for briefing injection.
pub fn xwayland_telemetry() -> String {
    let report = DisplayReport::detect();
    if report.session_type == "x11" || report.session_type == "unknown" {
        return String::new();
    }
    format!("## Display Protocol\n{}\n", report.summary())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_no_panic() {
        let report = DisplayReport::detect();
        let _ = report.summary();
        let _ = report.xwayland_apps();
    }

    #[test]
    fn test_display_mode_display() {
        assert_eq!(DisplayMode::NativeWayland.to_string(), "native Wayland");
        assert_eq!(DisplayMode::XWayland.to_string(), "XWayland");
    }
}

// Anna v0.10 Persona Radar Module
// Computes 8 persona scores based on observable system characteristics

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::telemetry_v10::TelemetrySnapshot;

/// Persona score with evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaScore {
    pub name: String,
    pub score: f32, // 0.0 to 10.0
    pub evidence: Vec<String>,
}

/// Evidence extracted from system state
#[derive(Debug, Default)]
pub struct SystemEvidence {
    pub distro: String,
    pub kernel: String,
    pub package_count: usize,
    pub installed_programs: HashSet<String>,
    pub running_processes: HashSet<String>,
    pub uptime_days: u64,
    pub has_gui_session: bool,
    pub shell_type: Option<String>,
    pub window_manager: Option<String>,
    pub total_ram_gb: u64,
    pub systemd_timers_count: usize,
    pub cron_jobs_count: usize,
    pub git_repos_in_home: usize,
}

/// Persona radar computer
pub struct PersonaRadar;

impl PersonaRadar {
    /// Compute all 8 persona scores from a telemetry snapshot
    pub fn compute_scores(snapshot: &TelemetrySnapshot) -> Result<Vec<PersonaScore>> {
        let evidence = Self::extract_evidence(snapshot)?;

        let scores = vec![
            Self::score_minimalist(&evidence),
            Self::score_power_user(&evidence),
            Self::score_server(&evidence),
            Self::score_workstation(&evidence),
            Self::score_tiling_wm(&evidence),
            Self::score_heavy_desktop(&evidence),
            Self::score_terminal_focus(&evidence),
            Self::score_automation_affinity(&evidence),
        ];

        Ok(scores)
    }

    /// Extract evidence from snapshot and system inspection
    fn extract_evidence(snapshot: &TelemetrySnapshot) -> Result<SystemEvidence> {
        let mut evidence = SystemEvidence {
            distro: snapshot.distro.clone(),
            kernel: snapshot.kernel.clone(),
            uptime_days: snapshot.uptime_s / 86400,
            total_ram_gb: snapshot.mem.total_mb / 1024,
            ..Default::default()
        };

        // Extract running processes
        for proc in &snapshot.processes {
            evidence.running_processes.insert(proc.name.clone());
        }

        // Detect installed programs
        evidence.installed_programs = Self::detect_installed_programs()?;

        // Detect package count
        evidence.package_count = Self::count_packages()?;

        // Detect shell type
        evidence.shell_type = Self::detect_shell();

        // Detect window manager
        evidence.window_manager = Self::detect_window_manager(&evidence.running_processes);

        // Detect GUI session
        evidence.has_gui_session = Self::has_gui_session(&evidence.running_processes);

        // Count systemd timers and cron jobs
        evidence.systemd_timers_count = Self::count_systemd_timers()?;
        evidence.cron_jobs_count = Self::count_cron_jobs()?;

        // Count Git repos in home (limited scan)
        evidence.git_repos_in_home = Self::count_git_repos()?;

        Ok(evidence)
    }

    /// Detect installed programs by checking common paths
    fn detect_installed_programs() -> Result<HashSet<String>> {
        let mut programs = HashSet::new();
        let bin_paths = vec!["/usr/bin", "/usr/local/bin", "/bin"];

        let common_programs = vec![
            "vim", "nvim", "emacs", "nano",
            "tmux", "screen",
            "zsh", "fish", "bash",
            "i3", "sway", "bspwm", "awesome", "xmonad",
            "polybar", "waybar",
            "rofi", "dmenu",
            "firefox", "chromium", "chrome",
            "code", "subl", "atom",
            "git", "docker", "ansible",
            "steam", "vlc", "gimp",
            "systemctl", "journalctl",
        ];

        for bin_path in bin_paths {
            for program in &common_programs {
                let full_path = Path::new(bin_path).join(program);
                if full_path.exists() {
                    programs.insert(program.to_string());
                }
            }
        }

        Ok(programs)
    }

    /// Count installed packages (distro-specific)
    fn count_packages() -> Result<usize> {
        // Try pacman (Arch)
        if let Ok(output) = std::process::Command::new("pacman")
            .args(&["-Q"])
            .output()
        {
            if output.status.success() {
                return Ok(output.stdout.split(|&b| b == b'\n').count());
            }
        }

        // Try dpkg (Debian/Ubuntu)
        if let Ok(output) = std::process::Command::new("dpkg")
            .args(&["-l"])
            .output()
        {
            if output.status.success() {
                return Ok(output.stdout.split(|&b| b == b'\n').filter(|line| {
                    line.starts_with(b"ii")
                }).count());
            }
        }

        // Try rpm (Fedora/RHEL)
        if let Ok(output) = std::process::Command::new("rpm")
            .args(&["-qa"])
            .output()
        {
            if output.status.success() {
                return Ok(output.stdout.split(|&b| b == b'\n').count());
            }
        }

        Ok(0)
    }

    /// Detect shell type from $SHELL or /etc/passwd
    fn detect_shell() -> Option<String> {
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return Some("zsh".to_string());
            } else if shell.contains("fish") {
                return Some("fish".to_string());
            } else if shell.contains("bash") {
                return Some("bash".to_string());
            }
        }

        None
    }

    /// Detect window manager from running processes
    fn detect_window_manager(processes: &HashSet<String>) -> Option<String> {
        let wm_names = vec![
            "i3", "sway", "bspwm", "awesome", "xmonad", "dwm",
            "kwin", "mutter", "openbox", "fluxbox",
        ];

        for wm in wm_names {
            if processes.contains(wm) {
                return Some(wm.to_string());
            }
        }

        None
    }

    /// Check if GUI session is running
    fn has_gui_session(processes: &HashSet<String>) -> bool {
        let gui_indicators = vec![
            "Xorg", "X", "wayland", "kwin", "mutter", "gnome-shell",
            "plasmashell", "xfce4-panel",
        ];

        gui_indicators.iter().any(|&ind| processes.contains(ind))
    }

    /// Count active systemd timers
    fn count_systemd_timers() -> Result<usize> {
        let output = std::process::Command::new("systemctl")
            .args(&["list-timers", "--no-pager", "--no-legend"])
            .output()?;

        if output.status.success() {
            Ok(output.stdout.split(|&b| b == b'\n').filter(|line| !line.is_empty()).count())
        } else {
            Ok(0)
        }
    }

    /// Count cron jobs (user + system)
    fn count_cron_jobs() -> Result<usize> {
        let mut count = 0;

        // User crontab
        if let Ok(output) = std::process::Command::new("crontab").args(&["-l"]).output() {
            if output.status.success() {
                count += output.stdout.split(|&b| b == b'\n')
                    .filter(|line| !line.is_empty() && !line.starts_with(b"#"))
                    .count();
            }
        }

        // System cron jobs
        if let Ok(entries) = fs::read_dir("/etc/cron.d") {
            count += entries.count();
        }

        Ok(count)
    }

    /// Count Git repositories in home directory (limited depth)
    fn count_git_repos() -> Result<usize> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        let home_path = Path::new(&home);

        let mut count = 0;

        // Scan immediate subdirectories only (avoid deep recursion)
        if let Ok(entries) = fs::read_dir(home_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if entry.path().join(".git").is_dir() {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    // === Scoring Functions ===

    fn score_minimalist(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 5.0;
        let mut evidence = Vec::new();

        // Minimal distro
        let minimal_distros = vec!["alpine", "void", "arch", "gentoo"];
        if minimal_distros.iter().any(|d| ev.distro.to_lowercase().contains(d)) {
            score += 1.5;
            evidence.push(format!("Minimal distro: {}", ev.distro));
        }

        // Low package count
        if ev.package_count > 0 && ev.package_count < 200 {
            score += 2.0;
            evidence.push(format!("{} packages installed", ev.package_count));
        } else if ev.package_count >= 200 && ev.package_count < 500 {
            score += 1.0;
        }

        // Terminal tools
        if ev.installed_programs.contains("tmux") || ev.installed_programs.contains("screen") {
            score += 1.0;
            evidence.push("Terminal multiplexer installed".to_string());
        }

        if ev.installed_programs.contains("vim") || ev.installed_programs.contains("nvim") {
            score += 0.5;
            evidence.push("Vim/Neovim installed".to_string());
        }

        // No GUI
        if !ev.has_gui_session {
            score += 1.0;
            evidence.push("No GUI session detected".to_string());
        } else {
            score -= 1.0;
        }

        PersonaScore {
            name: "Minimalist".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_power_user(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 5.0;
        let mut evidence = Vec::new();

        // Advanced shell
        if let Some(shell) = &ev.shell_type {
            if shell == "zsh" || shell == "fish" {
                score += 1.5;
                evidence.push(format!("Advanced shell: {}", shell));
            }
        }

        // Advanced editors
        if ev.installed_programs.contains("nvim") {
            score += 1.0;
            evidence.push("Neovim installed".to_string());
        }
        if ev.installed_programs.contains("emacs") {
            score += 1.0;
            evidence.push("Emacs installed".to_string());
        }

        // Tiling WM
        if let Some(wm) = &ev.window_manager {
            let tiling_wms = vec!["i3", "sway", "bspwm", "awesome", "xmonad"];
            if tiling_wms.contains(&wm.as_str()) {
                score += 2.0;
                evidence.push(format!("Tiling WM: {}", wm));
            }
        }

        // Dev tools
        if ev.installed_programs.contains("git") {
            score += 0.5;
        }
        if ev.git_repos_in_home > 3 {
            score += 1.0;
            evidence.push(format!("{} Git repos in home", ev.git_repos_in_home));
        }

        // Automation
        if ev.systemd_timers_count > 0 || ev.cron_jobs_count > 0 {
            score += 0.5;
        }

        PersonaScore {
            name: "PowerUser".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_server(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 3.0;
        let mut evidence = Vec::new();

        // High uptime
        if ev.uptime_days > 30 {
            score += 2.0;
            evidence.push(format!("{} days uptime", ev.uptime_days));
        } else if ev.uptime_days > 7 {
            score += 1.0;
            evidence.push(format!("{} days uptime", ev.uptime_days));
        }

        // No GUI
        if !ev.has_gui_session {
            score += 2.0;
            evidence.push("No GUI (headless)".to_string());
        } else {
            score -= 2.0;
        }

        // SSH daemon
        if ev.running_processes.contains("sshd") {
            score += 1.5;
            evidence.push("SSH daemon running".to_string());
        }

        // Webserver
        if ev.running_processes.contains("nginx") || ev.running_processes.contains("apache2") {
            score += 1.5;
            evidence.push("Web server detected".to_string());
        }

        // Monitoring tools
        if ev.installed_programs.contains("systemctl") {
            score += 0.5;
        }

        PersonaScore {
            name: "Server".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_workstation(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 5.0;
        let mut evidence = Vec::new();

        // GUI session
        if ev.has_gui_session {
            score += 2.0;
            evidence.push("GUI session active".to_string());
        } else {
            score -= 2.0;
        }

        // Browser
        if ev.installed_programs.contains("firefox") || ev.installed_programs.contains("chromium") {
            score += 1.0;
            evidence.push("Web browser installed".to_string());
        }

        // IDE/Editor
        if ev.installed_programs.contains("code") {
            score += 1.0;
            evidence.push("VS Code installed".to_string());
        }

        // Office/Media
        if ev.installed_programs.contains("vlc") {
            score += 0.5;
        }
        if ev.installed_programs.contains("gimp") {
            score += 0.5;
        }

        // Reasonable RAM
        if ev.total_ram_gb >= 8 {
            score += 0.5;
        }

        PersonaScore {
            name: "Workstation".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_tiling_wm(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 2.0;
        let mut evidence = Vec::new();

        // Tiling WM detected
        if let Some(wm) = &ev.window_manager {
            let tiling_wms = vec!["i3", "sway", "bspwm", "awesome", "xmonad", "dwm"];
            if tiling_wms.contains(&wm.as_str()) {
                score += 5.0;
                evidence.push(format!("Tiling WM: {}", wm));
            }
        }

        // Status bars
        if ev.installed_programs.contains("polybar") {
            score += 1.0;
            evidence.push("Polybar installed".to_string());
        }
        if ev.installed_programs.contains("waybar") {
            score += 1.0;
            evidence.push("Waybar installed".to_string());
        }

        // Launchers
        if ev.installed_programs.contains("rofi") {
            score += 1.0;
            evidence.push("Rofi launcher".to_string());
        }
        if ev.installed_programs.contains("dmenu") {
            score += 0.5;
        }

        // Terminal emulators (tiling WM users often use many)
        let term_count = ["alacritty", "kitty", "st", "urxvt"]
            .iter()
            .filter(|t| ev.installed_programs.contains(&t.to_string()))
            .count();

        if term_count >= 2 {
            score += 1.0;
            evidence.push(format!("{} terminal emulators", term_count));
        }

        PersonaScore {
            name: "TilingWM".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_heavy_desktop(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 3.0;
        let mut evidence = Vec::new();

        // Heavy DE detected
        if let Some(wm) = &ev.window_manager {
            if wm == "kwin" || wm == "mutter" {
                score += 3.0;
                evidence.push(format!("Heavy DE: {}", wm));
            }
        }

        // KDE/GNOME processes
        if ev.running_processes.contains("plasmashell") {
            score += 2.0;
            evidence.push("KDE Plasma detected".to_string());
        }
        if ev.running_processes.contains("gnome-shell") {
            score += 2.0;
            evidence.push("GNOME Shell detected".to_string());
        }

        // High RAM usage (GUI heavy)
        let gui_ram_mb: f64 = ev.running_processes
            .iter()
            .filter(|p| p.contains("plasma") || p.contains("gnome") || p.contains("electron"))
            .count() as f64 * 500.0; // Estimate 500MB per heavy GUI process

        if gui_ram_mb > 4000.0 {
            score += 1.5;
            evidence.push("High GUI memory usage".to_string());
        }

        // Electron apps
        if ev.running_processes.contains("code") || ev.running_processes.contains("slack") {
            score += 1.0;
            evidence.push("Electron apps detected".to_string());
        }

        PersonaScore {
            name: "HeavyDesktop".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_terminal_focus(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 5.0;
        let mut evidence = Vec::new();

        // Terminal multiplexer
        if ev.running_processes.contains("tmux") || ev.running_processes.contains("screen") {
            score += 2.0;
            evidence.push("Terminal multiplexer active".to_string());
        }

        // Terminal editors
        if ev.installed_programs.contains("vim") || ev.installed_programs.contains("nvim") {
            score += 1.5;
            evidence.push("Terminal editor installed".to_string());
        }

        // No GUI or minimal
        if !ev.has_gui_session {
            score += 2.0;
            evidence.push("No GUI session".to_string());
        } else if ev.window_manager.as_ref().map(|w| w.starts_with("i3") || w.starts_with("sway")).unwrap_or(false) {
            score += 1.0;
        }

        // Shell power user
        if ev.shell_type == Some("zsh".to_string()) || ev.shell_type == Some("fish".to_string()) {
            score += 0.5;
        }

        PersonaScore {
            name: "TerminalFocus".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }

    fn score_automation_affinity(ev: &SystemEvidence) -> PersonaScore {
        let mut score: f32 = 4.0;
        let mut evidence = Vec::new();

        // Systemd timers
        if ev.systemd_timers_count > 0 {
            score += 2.0;
            evidence.push(format!("{} systemd timers", ev.systemd_timers_count));
        }

        // Cron jobs
        if ev.cron_jobs_count > 0 {
            score += 1.5;
            evidence.push(format!("{} cron jobs", ev.cron_jobs_count));
        }

        // Automation tools
        if ev.installed_programs.contains("ansible") {
            score += 1.5;
            evidence.push("Ansible installed".to_string());
        }
        if ev.installed_programs.contains("docker") {
            score += 1.0;
            evidence.push("Docker installed".to_string());
        }

        // Git usage (automation scripts)
        if ev.git_repos_in_home > 5 {
            score += 1.0;
            evidence.push(format!("{} Git repos", ev.git_repos_in_home));
        }

        PersonaScore {
            name: "AutomationAffinity".to_string(),
            score: score.max(0.0).min(10.0),
            evidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry_v10::*;

    fn create_test_snapshot() -> TelemetrySnapshot {
        TelemetrySnapshot {
            ts: 1730390400,
            host_id: "test".to_string(),
            kernel: "6.0.0".to_string(),
            distro: "Arch Linux".to_string(),
            uptime_s: 86400 * 7, // 7 days
            cpu: CpuMetrics {
                cores: vec![CpuCore { core: 0, util_pct: 50.0, temp_c: Some(42.0) }],
                load_avg: [1.0, 1.0, 1.0],
                throttle_flags: Vec::new(),
            },
            mem: MemMetrics {
                total_mb: 16384,
                used_mb: 8192,
                free_mb: 8192,
                cached_mb: 2048,
                swap_total_mb: 4096,
                swap_used_mb: 512,
            },
            disk: Vec::new(),
            net: Vec::new(),
            power: None,
            gpu: Vec::new(),
            processes: vec![
                ProcessMetrics {
                    pid: 1,
                    name: "systemd".to_string(),
                    cpu_pct: 0.1,
                    mem_mb: 10.0,
                    state: "Running".to_string(),
                },
                ProcessMetrics {
                    pid: 2,
                    name: "i3".to_string(),
                    cpu_pct: 1.0,
                    mem_mb: 50.0,
                    state: "Running".to_string(),
                },
            ],
            systemd_units: Vec::new(),
        }
    }

    #[test]
    fn test_persona_scores() -> Result<()> {
        let snapshot = create_test_snapshot();
        let scores = PersonaRadar::compute_scores(&snapshot)?;

        assert_eq!(scores.len(), 8);
        assert!(scores.iter().all(|s| s.score >= 0.0 && s.score <= 10.0));

        // Find specific scores
        let minimalist = scores.iter().find(|s| s.name == "Minimalist").unwrap();
        assert!(minimalist.score > 0.0);

        Ok(())
    }
}

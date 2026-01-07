use super::fs;
use super::rollup;
use super::util;
use crate::config::Config;
use anyhow::Result;
use serde_json::json;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs as stdfs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use time::format_description::well_known::Rfc3339;
use time::{Date, Duration as TimeDuration, OffsetDateTime};
use tokio::time::{self as tokio_time, Duration, MissedTickBehavior};
use tracing::{debug, info, warn};

#[derive(Clone)]
struct SharedConfig {
    interval: Duration,
    max_procs: usize,
    loadavg_cap: f64,
}

struct SamplerState {
    current_date: Date,
    last_heartbeat: OffsetDateTime,
}

impl SamplerState {
    fn new() -> Result<Self> {
        Ok(Self {
            current_date: util::today_local()?,
            last_heartbeat: OffsetDateTime::now_utc(),
        })
    }
}

pub fn spawn(cfg: &Config) -> Result<()> {
    let sampler_cfg = cfg.persona.sampler.clone();
    if !sampler_cfg.enable {
        return Ok(());
    }

    let shared_cfg = SharedConfig {
        interval: Duration::from_secs(sampler_cfg.interval_secs),
        max_procs: sampler_cfg.max_procs,
        loadavg_cap: sampler_cfg.loadavg_cap,
    };

    let state = Arc::new(Mutex::new(SamplerState::new()?));

    let cfg_clone = shared_cfg.clone();
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        run_sampler(cfg_clone, state_clone).await;
    });

    Ok(())
}

async fn run_sampler(cfg: SharedConfig, state: Arc<Mutex<SamplerState>>) {
    let mut interval = tokio_time::interval(cfg.interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let cfg_clone = cfg.clone();
        let state_clone = Arc::clone(&state);
        let res = tokio::task::spawn_blocking(move || sample_once(cfg_clone, state_clone)).await;
        match res {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => warn!(target: "annad", "persona sampler tick error: {err:?}"),
            Err(join_err) => warn!(target: "annad", "persona sampler join error: {join_err:?}"),
        }
    }
}

fn sample_once(cfg: SharedConfig, state: Arc<Mutex<SamplerState>>) -> Result<()> {
    if should_skip(&cfg) {
        return Ok(());
    }

    let now = OffsetDateTime::now_utc();
    let ts = now
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let today = util::today_local()?;
    let today_str = util::format_date(&today);

    let executables = snapshot_processes(cfg.max_procs)?;
    if !executables.is_empty() {
        let mut lines = Vec::with_capacity(executables.len());
        for exe in &executables {
            let cat = categorize(exe);
            let record = json!({
                "ts": ts,
                "cat": cat,
                "exe": exe,
            });
            let mut line = serde_json::to_vec(&record)?;
            line.push(b'\n');
            lines.push(line);
        }
        let path = fs::samples_path(&today_str);
        fs::append_lines(&path, &lines)?;
    }

    // Handle rollup if date changed
    let mut previous_date: Option<Date> = None;
    let mut emit_heartbeat = false;
    {
        let mut guard = state.lock().expect("sampler state poisoned");
        if today != guard.current_date {
            previous_date = Some(guard.current_date);
            guard.current_date = today;
        }
        if now - guard.last_heartbeat >= TimeDuration::hours(1) {
            guard.last_heartbeat = now;
            emit_heartbeat = true;
        }
    }

    if let Some(prev) = previous_date {
        let _ = rollup::generate_for_date(&prev)?;
    }

    if emit_heartbeat {
        info!(target: "annad", "sampler active (hourly heartbeat)");
    }

    Ok(())
}

fn should_skip(cfg: &SharedConfig) -> bool {
    if cfg.loadavg_cap <= 0.0 {
        return false;
    }
    if let Some(load) = load_average_one() {
        if load > cfg.loadavg_cap {
            debug!(target: "annad", "persona sampler skip: loadavg {:.2} > {:.2}", load, cfg.loadavg_cap);
            return true;
        }
    }
    false
}

fn load_average_one() -> Option<f64> {
    let contents = stdfs::read_to_string("/proc/loadavg").ok()?;
    contents.split_whitespace().next()?.parse().ok()
}

fn snapshot_processes(max_procs: usize) -> Result<HashSet<String>> {
    let mut seen = HashSet::new();
    let mut count = 0usize;
    for entry in stdfs::read_dir("/proc")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();
        if !pid_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        count += 1;
        if count > max_procs {
            break;
        }
        if !belongs_to_user_session(&pid_str) {
            continue;
        }
        if let Some(exe) = read_executable(&pid_str) {
            if should_skip_executable(&exe) {
                continue;
            }
            seen.insert(exe);
        }
    }
    Ok(seen)
}

fn read_executable(pid: &str) -> Option<String> {
    let comm_path = format!("/proc/{pid}/comm");
    if let Ok(contents) = stdfs::read_to_string(&comm_path) {
        let name = contents.trim().to_lowercase();
        if !name.is_empty() {
            return Some(name);
        }
    }
    let exe_path = PathBuf::from(format!("/proc/{pid}/exe"));
    match stdfs::read_link(exe_path) {
        Ok(path) => path
            .file_name()
            .and_then(OsStr::to_str)
            .map(|s| s.to_lowercase()),
        Err(_) => None,
    }
}

fn belongs_to_user_session(pid: &str) -> bool {
    let cgroup_path = format!("/proc/{pid}/cgroup");
    let Ok(contents) = stdfs::read_to_string(&cgroup_path) else {
        return false;
    };
    contents.contains("user.slice") || contents.contains("user@")
}

fn should_skip_executable(exe: &str) -> bool {
    if is_kernel_thread(exe) {
        return true;
    }
    if is_boot_daemon(exe) {
        return true;
    }
    false
}

fn is_kernel_thread(exe: &str) -> bool {
    exe.starts_with("kworker/")
        || exe.starts_with("ksoftirqd/")
        || exe.starts_with("migration/")
        || exe.starts_with("rcu")
        || exe.starts_with("watchdog/")
        || exe.starts_with("irq/")
        || exe.starts_with("idle_inject/")
        || exe == "jfsio"
        || exe == "events"
        || exe.starts_with("events_")
        || exe == "kblockd"
        || exe == "mm_percpu_wq"
        || exe.starts_with("kswapd")
}

fn is_boot_daemon(exe: &str) -> bool {
    matches!(
        exe,
        "systemd-udevd"
            | "systemd-journald"
            | "systemd-timesyncd"
            | "systemd-networkd"
            | "dbus-daemon"
    )
}

fn categorize(exe: &str) -> &'static str {
    match exe {
        "nvim" | "vim" | "emacs" | "vscodium" | "code" | "zed" | "nano" | "helix" | "kakoune" => {
            "editor"
        }
        "idea64" | "clion64" | "goland64" | "pycharm64" | "rider64" | "vscode" => "ide",
        "alacritty" | "kitty" | "gnome-terminal" | "konsole" | "wezterm" | "xterm" | "foot" => {
            "terminal"
        }
        "firefox" | "chromium" | "google-chrome" | "brave" | "vivaldi" => "browser",
        "vlc" | "mpv" | "spotify" => "media",
        "steam" | "lutris" | "heroic" => "games",
        "cargo" | "rustc" | "make" | "ninja" | "gradle" | "mvn" => "build",
        "zsh" | "bash" | "fish" => "shell",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::categorize;

    #[test]
    fn classify_known_binaries() {
        assert_eq!(categorize("nvim"), "editor");
        assert_eq!(categorize("firefox"), "browser");
        assert_eq!(categorize("cargo"), "build");
        assert_eq!(categorize("steam"), "games");
        assert_eq!(categorize("unknown"), "other");
    }
}

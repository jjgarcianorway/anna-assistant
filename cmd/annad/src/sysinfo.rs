use serde::Serialize;
use std::{fs, process::Command};
use which::which;

#[derive(Serialize)]
pub struct SystemSnapshot {
    pub os: String,
    pub kernel: String,
    pub uptime_secs: u64,

    pub cpu_model: String,
    pub cpu_cores_logical: usize,

    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub total_swap_mb: u64,
    pub free_swap_mb: u64,

    pub partitions: Vec<String>,
    pub gpu_model: Option<String>,
    pub audio_server: Option<String>,
    pub network_managers: Vec<String>,
}

fn read_to_string(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn parse_meminfo() -> (u64, u64, u64, u64) {
    let txt = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mt = 0u64; // kB
    let mut ma = 0u64;
    let mut st = 0u64;
    let mut sf = 0u64;
    for line in txt.lines() {
        let grab = |pfx: &str| -> Option<u64> {
            if line.starts_with(pfx) {
                return line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse::<u64>().ok());
            }
            None
        };
        if let Some(v) = grab("MemTotal:") { mt = v; }
        if let Some(v) = grab("MemAvailable:") { ma = v; }
        if let Some(v) = grab("SwapTotal:") { st = v; }
        if let Some(v) = grab("SwapFree:") { sf = v; }
    }
    (mt / 1024, ma / 1024, st / 1024, sf / 1024) // -> MiB
}

fn cpu_model_and_cores() -> (String, usize) {
    let txt = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut model = String::new();
    let mut cores = 0usize;
    for line in txt.lines() {
        if line.starts_with("model name") {
            if model.is_empty() {
                model = line
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
            cores += 1;
        }
    }
    (model, cores.max(1))
}

fn uptime_secs() -> u64 {
    read_to_string("/proc/uptime")
        .and_then(|s| s.split_whitespace().next().and_then(|f| f.parse::<f64>().ok()))
        .map(|f| f as u64)
        .unwrap_or(0)
}

fn os_pretty_name() -> String {
    let osr = read_to_string("/etc/os-release").unwrap_or_default();
    for line in osr.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return line
                .trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_string();
        }
    }
    String::from_utf8(
        Command::new("uname")
            .args(["-o"])
            .output()
            .ok()
            .map(|o| o.stdout)
            .unwrap_or_default(),
    )
    .unwrap_or_else(|_| "Linux".into())
    .trim()
    .to_string()
}

fn kernel_release() -> String {
    String::from_utf8(
        Command::new("uname")
            .args(["-r"])
            .output()
            .ok()
            .map(|o| o.stdout)
            .unwrap_or_default(),
    )
    .unwrap_or_default()
    .trim()
    .to_string()
}

fn gpu_model() -> Option<String> {
    let out = Command::new("lspci").output().ok()?;
    let s = String::from_utf8(out.stdout).ok()?;
    s.lines()
        .find(|l| {
            let ll = l.to_lowercase();
            ll.contains(" vga ") || ll.contains(" 3d ") || ll.contains(" display ")
        })
        .map(|l| l.trim().to_string())
}

fn audio_server() -> Option<String> {
    if which("pactl").is_ok() {
        let out = Command::new("pactl").arg("info").output().ok()?;
        let s = String::from_utf8(out.stdout).ok()?;
        s.lines()
            .find(|l| l.contains("Server Name"))
            .map(|l| l.trim().to_string())
    } else {
        None
    }
}

fn partitions() -> Vec<String> {
    // Prefer lsblk JSON; fall back to plain output.
    if which("lsblk").is_ok() {
        if let Some(out) = Command::new("lsblk")
            .args(["-o", "NAME,FSTYPE,SIZE,MOUNTPOINT", "-J"])
            .output()
            .ok()
            .map(|o| o.stdout)
        {
            if let Some(js) = serde_json::from_slice::<serde_json::Value>(&out).ok() {
                let mut v = Vec::new();
                if let Some(blockdevices) = js.get("blockdevices").and_then(|x| x.as_array()) {
                    for bd in blockdevices {
                        let name = bd.get("name").and_then(|x| x.as_str()).unwrap_or("");
                        let fstype = bd.get("fstype").and_then(|x| x.as_str()).unwrap_or("");
                        let size = bd.get("size").and_then(|x| x.as_str()).unwrap_or("");
                        let mnt = bd.get("mountpoint").and_then(|x| x.as_str()).unwrap_or("");
                        v.push(format!("{name} {size} {fstype} {mnt}"));
                    }
                }
                return v;
            }
        }
        if let Some(out) = Command::new("lsblk")
            .args(["-o", "NAME,FSTYPE,SIZE,MOUNTPOINT"])
            .output()
            .ok()
            .map(|o| o.stdout)
        {
            return String::from_utf8(out)
                .unwrap_or_default()
                .lines()
                .skip(1)
                .map(|s| s.trim().to_string())
                .collect();
        }
    }
    Vec::new()
}

fn network_managers() -> Vec<String> {
    ["NetworkManager", "systemd-networkd", "connman"]
        .into_iter()
        .filter(|name| which(name).is_ok())
        .map(|s| s.to_string())
        .collect()
}

pub fn collect() -> SystemSnapshot {
    let (total_mb, avail_mb, swap_total_mb, swap_free_mb) = parse_meminfo();
    let (cpu_model, cpu_cores_logical) = cpu_model_and_cores();
    SystemSnapshot {
        os: os_pretty_name(),
        kernel: kernel_release(),
        uptime_secs: uptime_secs(),
        cpu_model,
        cpu_cores_logical,
        total_memory_mb: total_mb,
        available_memory_mb: avail_mb,
        total_swap_mb: swap_total_mb,
        free_swap_mb: swap_free_mb,
        partitions: partitions(),
        gpu_model: gpu_model(),
        audio_server: audio_server(),
        network_managers: network_managers(),
    }
}

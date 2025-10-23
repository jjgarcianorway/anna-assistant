use crate::advice;
use crate::advice::types::AdvicePlan;
use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::net::ToSocketAddrs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::process::Command;
use tokio::time::{self as tokio_time, Duration};
use tracing::info;
use which::which;

const REPORT_ROOT: &str = "/var/lib/anna/reports";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Action,
}

#[derive(Debug, Clone, Copy, Default)]
struct SwapTotals {
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwapKind {
    Partition,
    File,
    Zram,
    Other,
}

#[derive(Debug, Clone)]
struct SwapEntry {
    name: String,
    kind: SwapKind,
    size_bytes: u64,
    used_bytes: u64,
    priority: Option<i32>,
}

#[derive(Debug, Clone)]
struct ZramDevice {
    name: String,
    disksize_bytes: u64,
    orig_data_bytes: u64,
    mem_used_bytes: u64,
    comp_algorithm: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ZswapState {
    enabled: bool,
    limit_bytes: Option<u64>,
    used_bytes: Option<u64>,
    headroom_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
struct SwapSnapshot {
    mem_total_bytes: u64,
    entries: Vec<SwapEntry>,
    zram_devices: Vec<ZramDevice>,
    zswap: ZswapState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeadroomSource {
    RealSwap,
    Zram,
    Zswap,
    None,
}

impl SwapSnapshot {
    fn real_swap_totals(&self) -> SwapTotals {
        let mut totals = SwapTotals::default();
        for entry in &self.entries {
            if matches!(entry.kind, SwapKind::Partition | SwapKind::File) {
                totals.total_bytes += entry.size_bytes;
                totals.used_bytes += entry.used_bytes;
            }
        }
        totals.free_bytes = totals.total_bytes.saturating_sub(totals.used_bytes);
        totals
    }

    fn zram_totals(&self) -> SwapTotals {
        let mut totals = SwapTotals::default();
        for entry in &self.entries {
            if entry.kind == SwapKind::Zram {
                totals.total_bytes += entry.size_bytes;
                totals.used_bytes += entry.used_bytes;
            }
        }
        totals.free_bytes = totals.total_bytes.saturating_sub(totals.used_bytes);
        totals
    }

    fn effective_headroom(&self) -> u64 {
        let real = self.real_swap_totals().free_bytes;
        let zram = self.zram_totals().free_bytes;
        let zswap = self.zswap.headroom_bytes.unwrap_or(0);
        real.max(zram).max(zswap)
    }

    fn dominant_headroom_source(&self) -> HeadroomSource {
        let real = self.real_swap_totals().free_bytes;
        let zram = self.zram_totals().free_bytes;
        let zswap = self.zswap.headroom_bytes.unwrap_or(0);
        let mut best = (HeadroomSource::None, 0u64);
        if real > best.1 {
            best = (HeadroomSource::RealSwap, real);
        }
        if zram > best.1 {
            best = (HeadroomSource::Zram, zram);
        }
        if zswap > best.1 {
            best = (HeadroomSource::Zswap, zswap);
        }
        best.0
    }

    fn has_real_swap(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry.kind, SwapKind::Partition | SwapKind::File))
    }
}

async fn collect_swap_snapshot() -> Result<SwapSnapshot> {
    let mem_total_bytes = parse_meminfo()?.unwrap_or(0);
    let proc_entries = collect_proc_swaps()?;
    let entries = if let Some(swapon_entries) = collect_swapon_entries().await? {
        if swapon_entries.is_empty() {
            proc_entries
        } else {
            swapon_entries
        }
    } else {
        proc_entries
    };
    let zram_devices = collect_zram_devices();
    let zswap = collect_zswap_state(mem_total_bytes);
    Ok(SwapSnapshot {
        mem_total_bytes,
        entries,
        zram_devices,
        zswap,
    })
}

fn parse_meminfo() -> Result<Option<u64>> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let value = rest.trim().trim_end_matches(" kB");
            let kb = value.parse::<u64>().unwrap_or(0);
            return Ok(Some(kb * 1024));
        }
    }
    Ok(None)
}

async fn collect_swapon_entries() -> Result<Option<Vec<SwapEntry>>> {
    if which("swapon").is_err() {
        return Ok(None);
    }
    let output = Command::new("swapon")
        .args([
            "--show",
            "--bytes",
            "--noheadings",
            "--output=NAME,TYPE,SIZE,USED,PRIO",
        ])
        .output()
        .await?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        let name = parts[0].to_string();
        let kind = match parts[1] {
            "partition" => SwapKind::Partition,
            "file" => SwapKind::File,
            "zram" => SwapKind::Zram,
            _ => SwapKind::Other,
        };
        let size_bytes = parts[2].parse::<u64>().unwrap_or(0);
        let used_bytes = parts[3].parse::<u64>().unwrap_or(0);
        let priority = parts[4].parse::<i32>().ok();
        entries.push(SwapEntry {
            name,
            kind,
            size_bytes,
            used_bytes,
            priority,
        });
    }
    Ok(Some(entries))
}

fn collect_proc_swaps() -> Result<Vec<SwapEntry>> {
    let mut entries = Vec::new();
    let contents = fs::read_to_string("/proc/swaps")?;
    for line in contents.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        let name = parts[0].to_string();
        let kind = match parts[1] {
            "partition" => SwapKind::Partition,
            "file" => SwapKind::File,
            "zram" => SwapKind::Zram,
            _ => SwapKind::Other,
        };
        let size_bytes = parts[2].parse::<u64>().unwrap_or(0) * 1024;
        let used_bytes = parts[3].parse::<u64>().unwrap_or(0) * 1024;
        let priority = parts[4].parse::<i32>().ok();
        entries.push(SwapEntry {
            name,
            kind,
            size_bytes,
            used_bytes,
            priority,
        });
    }
    Ok(entries)
}

fn collect_zram_devices() -> Vec<ZramDevice> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = match name.to_str() {
                Some(s) if s.starts_with("zram") => s.to_string(),
                _ => continue,
            };
            let base = entry.path();
            let disksize_bytes = read_sys_value(base.join("disksize"));
            let mm_stat_path = base.join("mm_stat");
            let (orig_data_bytes, mem_used_bytes) = read_mm_stat(&mm_stat_path);
            let comp_algorithm = fs::read_to_string(base.join("comp_algorithm"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            devices.push(ZramDevice {
                name: name_str,
                disksize_bytes,
                orig_data_bytes,
                mem_used_bytes,
                comp_algorithm,
            });
        }
    }
    devices
}

fn read_sys_value(path: std::path::PathBuf) -> u64 {
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

fn read_mm_stat(path: &std::path::Path) -> (u64, u64) {
    if let Ok(contents) = fs::read_to_string(path) {
        let parts: Vec<&str> = contents.split_whitespace().collect();
        if parts.len() >= 3 {
            let orig = parts[0].parse::<u64>().unwrap_or(0);
            let mem = parts[2].parse::<u64>().unwrap_or(0);
            return (orig, mem);
        }
    }
    (0, 0)
}

fn collect_zswap_state(mem_total_bytes: u64) -> ZswapState {
    let mut state = ZswapState::default();
    if let Ok(enabled) = fs::read_to_string("/sys/module/zswap/parameters/enabled") {
        let trimmed = enabled.trim();
        state.enabled = matches!(trimmed, "Y" | "y" | "1" | "true");
    }
    if !state.enabled {
        return state;
    }
    let limit_percent = fs::read_to_string("/sys/module/zswap/parameters/max_pool_percent")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    let debug_dir = std::path::Path::new("/sys/kernel/debug/zswap");
    let pool_limit = fs::read_to_string(debug_dir.join("pool_limit"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    let pool_total = fs::read_to_string(debug_dir.join("pool_total_size"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());

    state.limit_bytes = match (pool_limit, limit_percent) {
        (Some(limit), _) if limit > 0 => Some(limit),
        (_, Some(percent)) if percent > 0 => Some(mem_total_bytes * percent / 100),
        _ => None,
    };
    state.used_bytes = pool_total;
    if let (Some(limit), Some(used)) = (state.limit_bytes, state.used_bytes) {
        state.headroom_bytes = Some(limit.saturating_sub(used));
    }
    state
}

async fn detect_zram_managers() -> Result<Vec<String>> {
    let mut hits = Vec::new();
    if which("systemctl").is_ok() {
        for unit in [
            "systemd-zram-setup@zram0.service",
            "zramswap.service",
            "zram.service",
            "dev-zram0.swap",
        ] {
            let status = Command::new("systemctl")
                .args(["is-enabled", unit])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
                .unwrap_or_else(|_| std::process::ExitStatus::from_raw(1));
            if status.success() {
                hits.push(format!("{} enabled", unit));
                continue;
            }
            let active = Command::new("systemctl")
                .args(["is-active", unit])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
                .unwrap_or_else(|_| std::process::ExitStatus::from_raw(1));
            if active.success() {
                hits.push(format!("{} active", unit));
            }
        }
    }
    let generator_conf = std::path::Path::new("/etc/systemd/zram-generator.conf");
    if generator_conf.exists() {
        hits.push("zram-generator.conf present".into());
    }
    let generator_dir = std::path::Path::new("/etc/systemd/zram-generator.conf.d");
    if generator_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(generator_dir) {
            if entries.filter_map(|res| res.ok()).any(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "conf")
                    .unwrap_or(false)
            }) {
                hits.push("zram-generator drop-ins".into());
            }
        }
    }
    Ok(hits)
}

fn detect_portable_chassis() -> Option<bool> {
    let path = std::path::Path::new("/sys/class/dmi/id/chassis_type");
    let value = fs::read_to_string(path).ok()?;
    let code = value.trim().parse::<u32>().ok()?;
    let portable = matches!(code, 8 | 9 | 10 | 14);
    Some(portable)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Distro {
    Arch,
    Debian,
    Ubuntu,
    Fedora,
    Unknown,
}

fn detect_distro() -> Distro {
    let path = std::path::Path::new("/etc/os-release");
    let contents = fs::read_to_string(path).unwrap_or_default();
    let mut id = String::new();
    let mut id_like = String::new();
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("ID=") {
            id = rest.trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("ID_LIKE=") {
            id_like = rest.trim_matches('"').to_string();
        }
    }
    let mut ids = id_like
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if !id.is_empty() {
        ids.push(id);
    }
    for candidate in ids {
        match candidate.as_str() {
            "arch" | "manjaro" | "endeavouros" => return Distro::Arch,
            "debian" | "raspbian" => return Distro::Debian,
            "ubuntu" | "pop" | "linuxmint" => return Distro::Ubuntu,
            "fedora" | "rhel" | "centos" => return Distro::Fedora,
            _ => {}
        }
    }
    Distro::Unknown
}

fn fmt_bytes(bytes: u64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("TiB", 1024.0 * 1024.0 * 1024.0 * 1024.0),
        ("GiB", 1024.0 * 1024.0 * 1024.0),
        ("MiB", 1024.0 * 1024.0),
        ("KiB", 1024.0),
    ];
    if bytes == 0 {
        return "0 B".into();
    }
    for (unit, size) in UNITS {
        if (bytes as f64) >= *size {
            let value = (bytes as f64) / *size;
            return format!("{value:.1} {unit}");
        }
    }
    format!("{} B", bytes)
}

fn fmt_bytes_plain(bytes: u64) -> String {
    if bytes % (1024 * 1024 * 1024) == 0 {
        return format!("{}G", bytes / (1024 * 1024 * 1024));
    }
    if bytes % (1024 * 1024) == 0 {
        return format!("{}M", bytes / (1024 * 1024));
    }
    format!("{}", bytes)
}

fn default_swap_target(mem_total_bytes: u64) -> u64 {
    if mem_total_bytes == 0 {
        return 8 * 1024 * 1024 * 1024;
    }
    let half = mem_total_bytes / 2;
    half.clamp(4 * 1024 * 1024 * 1024, 16 * 1024 * 1024 * 1024)
}

fn zram_enable_plan(distro: Distro, recommended: u64) -> FixPlan {
    let dry_run = vec!["swapon --show".into()];
    let mut apply = Vec::new();
    match distro {
        Distro::Arch => {
            apply.push("sudo pacman -S --needed zram-generator".into());
        }
        Distro::Debian | Distro::Ubuntu => {
            apply.push("sudo apt install --yes zram-tools".into());
        }
        Distro::Fedora => {
            apply.push("sudo dnf install -y zram-generator-defaults".into());
        }
        Distro::Unknown => {}
    }
    apply.push(
        "sudo tee /etc/systemd/zram-generator.conf >/dev/null <<'EOF'\n[zram0]\nzram-size = ram / 2\ncompression-algorithm = zstd\nEOF"
            .into(),
    );
    apply.push("sudo systemctl daemon-reload".into());
    apply.push("sudo systemctl restart dev-zram0.swap".into());
    apply.push("sudo systemctl enable dev-zram0.swap".into());
    FixPlan {
        summary: format!("Enable zram swap (~{})", fmt_bytes(recommended)),
        dry_run_cmds: dry_run,
        apply_cmds: apply,
        undo_cmds: vec!["sudo systemctl disable --now dev-zram0.swap".into()],
    }
}

fn swapfile_plan(recommended: u64) -> FixPlan {
    let size_arg = fmt_bytes_plain(recommended);
    FixPlan {
        summary: format!("Create swapfile of ~{}", fmt_bytes(recommended)),
        dry_run_cmds: vec!["swapon --show --bytes".into()],
        apply_cmds: vec![
            format!("sudo fallocate -l {} /swapfile", size_arg),
            "sudo chmod 600 /swapfile".into(),
            "sudo mkswap /swapfile".into(),
            "sudo swapon /swapfile".into(),
            "echo /swapfile none swap defaults 0 0 | sudo tee -a /etc/fstab".into(),
        ],
        undo_cmds: vec!["sudo swapoff /swapfile".into(), "sudo rm /swapfile".into()],
    }
}

fn build_swap_fix(
    snapshot: &SwapSnapshot,
    distro: Distro,
    portable: Option<bool>,
    headroom_bytes: u64,
) -> FixPlan {
    let recommended = default_swap_target(snapshot.mem_total_bytes)
        .max(headroom_bytes.saturating_mul(2))
        .min(32 * 1024 * 1024 * 1024);
    if !snapshot.zram_devices.is_empty() {
        return FixPlan {
            summary: format!(
                "Increase zram capacity toward {} (currently {})",
                fmt_bytes(recommended),
                fmt_bytes(snapshot.zram_totals().total_bytes)
            ),
            dry_run_cmds: vec!["systemctl status dev-zram0.swap".into(), "zramctl".into()],
            apply_cmds: vec![
                "sudoedit /etc/systemd/zram-generator.conf".into(),
                "sudo systemctl restart dev-zram0.swap".into(),
            ],
            undo_cmds: vec!["sudo systemctl restart dev-zram0.swap".into()],
        };
    }
    if let Some(file_entry) = snapshot
        .entries
        .iter()
        .find(|entry| entry.kind == SwapKind::File)
    {
        return resize_swapfile_plan(&file_entry.name, recommended);
    }
    if snapshot.has_real_swap() {
        return swapfile_plan(recommended);
    }
    if portable.unwrap_or(false) {
        return zram_enable_plan(distro, recommended);
    }
    swapfile_plan(recommended)
}

fn resize_swapfile_plan(path: &str, recommended: u64) -> FixPlan {
    let size_arg = fmt_bytes_plain(recommended);
    FixPlan {
        summary: format!("Resize {} to ~{}", path, fmt_bytes(recommended)),
        dry_run_cmds: vec![format!("sudo ls -lh {}", path)],
        apply_cmds: vec![
            format!("sudo swapoff {}", path),
            format!("sudo fallocate -l {} {}", size_arg, path),
            format!("sudo chmod 600 {}", path),
            format!("sudo mkswap {}", path),
            format!("sudo swapon {}", path),
        ],
        undo_cmds: vec![format!("sudo swapoff {}", path)],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlan {
    pub summary: String,
    #[serde(default)]
    pub apply_cmds: Vec<String>,
    #[serde(default)]
    pub dry_run_cmds: Vec<String>,
    #[serde(default)]
    pub undo_cmds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub fix: Option<FixPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickscanReport {
    pub generated: String,
    pub duration_ms: u128,
    pub ok: usize,
    pub warn: usize,
    pub action: usize,
    pub findings: Vec<Finding>,
}

pub async fn run(config: Arc<Config>) -> Result<QuickscanReport> {
    if !config.quickscan.enable {
        return Err(anyhow!("quickscan disabled"));
    }

    let start = Instant::now();
    let mut findings = Vec::new();
    let timeout = Duration::from_secs(config.quickscan.timeout_secs);

    if config.quickscan.check_network {
        findings.push(run_with_timeout(timeout, network_probe()).await);
    }
    if config.quickscan.check_cpu_power {
        findings.push(run_with_timeout(timeout, cpu_power_probe()).await);
    }
    if config.quickscan.check_memory_swap {
        findings.push(run_with_timeout(timeout, memory_swap_probe()).await);
    }
    if config.quickscan.check_storage {
        findings.push(run_with_timeout(timeout, storage_probe()).await);
    }
    if config.quickscan.check_fs_trim {
        findings.push(run_with_timeout(timeout, fs_trim_probe()).await);
    }
    if config.quickscan.check_ntp {
        findings.push(run_with_timeout(timeout, ntp_probe()).await);
    }
    if config.quickscan.check_pkg_cache {
        findings.push(run_with_timeout(timeout, pkg_cache_probe()).await);
    }
    if config.quickscan.check_orphans {
        findings.push(run_with_timeout(timeout, orphan_probe()).await);
    }

    let (ok, warn_count, action) = count_severity(&findings);
    let report = QuickscanReport {
        generated: now_rfc3339(),
        duration_ms: start.elapsed().as_millis(),
        ok,
        warn: warn_count,
        action,
        findings: findings.clone(),
    };

    persist_report(&report)?;
    seed_advice(&findings)?;

    info!(
        target: "annad",
        "quickscan done ok={} warn={} action={} in {:.1}s",
        ok,
        warn_count,
        action,
        start.elapsed().as_secs_f32()
    );

    Ok(report)
}

async fn run_with_timeout(
    falloff: Duration,
    fut: impl std::future::Future<Output = Finding>,
) -> Finding {
    match tokio_time::timeout(falloff, fut).await {
        Ok(finding) => finding,
        Err(_) => Finding {
            id: "quickscan.timeout".into(),
            title: "Quickscan timeout".into(),
            severity: Severity::Warn,
            summary: "Probe timed out".into(),
            detail: "Probe exceeded configured timeout window".into(),
            fix: None,
        },
    }
}

fn count_severity(findings: &[Finding]) -> (usize, usize, usize) {
    let mut ok = 0;
    let mut warn = 0;
    let mut action = 0;
    for finding in findings {
        match finding.severity {
            Severity::Info => ok += 1,
            Severity::Warn => warn += 1,
            Severity::Action => action += 1,
        }
    }
    (ok, warn, action)
}

fn persist_report(report: &QuickscanReport) -> Result<()> {
    ensure_dir(Path::new(REPORT_ROOT), 0o700)?;
    let date = OffsetDateTime::parse(&report.generated, &Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .date();
    let dir = Path::new(REPORT_ROOT).join(format!("{}", date));
    ensure_dir(&dir, 0o700)?;
    let json_path = dir.join("quickscan.json");
    let txt_path = dir.join("quickscan.txt");

    write_secure(&json_path, &serde_json::to_vec_pretty(report)?)?;

    let mut txt = String::new();
    txt.push_str(&format!(
        "Quick Health Check — {}\nok: {}  warn: {}  action: {}\n\n",
        report.generated, report.ok, report.warn, report.action
    ));
    for finding in &report.findings {
        txt.push_str(&format!(
            "[{:?}] {}\n{}\n{}\n",
            finding.severity, finding.title, finding.summary, finding.detail
        ));
        if let Some(fix) = &finding.fix {
            if !fix.apply_cmds.is_empty() {
                txt.push_str("Suggested commands:\n");
                for cmd in &fix.apply_cmds {
                    txt.push_str(&format!("  $ {}\n", cmd));
                }
            }
        }
        txt.push('\n');
    }
    write_secure(&txt_path, txt.as_bytes())?;

    Ok(())
}

fn seed_advice(findings: &[Finding]) -> Result<()> {
    for finding in findings {
        if let Some(fix) = &finding.fix {
            let plan = AdvicePlan {
                dry_run_cmds: fix.dry_run_cmds.clone(),
                apply_cmds: fix.apply_cmds.clone(),
                undo_cmds: fix.undo_cmds.clone(),
            };
            advice::create_quickscan_advice(&finding.id, &finding.summary, plan)?;
        }
    }
    Ok(())
}

fn ensure_dir(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create dir {}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("chmod {}", path.display()))?;
    Ok(())
}

fn write_secure(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent, 0o700)?;
    }
    let mut file = fs::File::create(path).with_context(|| format!("write {}", path.display()))?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    file.write_all(data)?;
    Ok(())
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

// --- Probes ---------------------------------------------------------------

async fn network_probe() -> Finding {
    let mut detail = Vec::new();
    let mut severity = Severity::Info;
    let mut summary = String::from("Network connectivity looks good");

    let dns_ok = ("example.com", 443)
        .to_socket_addrs()
        .map(|mut addrs| addrs.next().is_some())
        .unwrap_or(false);
    if dns_ok {
        detail.push("DNS resolution succeeded".to_string());
    } else {
        severity = Severity::Warn;
        summary = "DNS resolution failed".into();
        detail.push("Could not resolve example.com".into());
    }

    if let Ok(status) = Command::new("curl")
        .args(["-I", "https://example.com", "--max-time", "1", "--silent"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
    {
        if !status.success() {
            severity = Severity::Warn;
            summary = "HTTPS check failed".into();
            detail.push("curl could not reach https://example.com".into());
        }
    } else {
        detail.push("curl not available; skipped HTTPS probe".into());
    }

    Finding {
        id: "network.connectivity".into(),
        title: "Network connectivity".to_string(),
        severity,
        summary,
        detail: detail.join("; "),
        fix: None,
    }
}

async fn cpu_power_probe() -> Finding {
    let governor_info = gather_cpu_governors();
    let manager = detect_power_manager().await;

    let mut detail_segments = Vec::new();
    match &manager {
        PowerManager::PowerProfiles { profile } => {
            detail_segments.push(format!("Manager: power-profiles-daemon ({profile})"));
        }
        PowerManager::Tlp { mode } => match mode {
            Some(mode) => detail_segments.push(format!("Manager: TLP ({mode} mode)")),
            None => detail_segments.push("Manager: TLP active".into()),
        },
        PowerManager::Tuned { profile } => {
            detail_segments.push(format!("Manager: tuned ({profile})"));
        }
        PowerManager::Thermald => {
            detail_segments.push("Manager: thermald active".into());
        }
        PowerManager::Powertop { notes } => {
            if notes.is_empty() {
                detail_segments.push("Manager: powertop autotune active".into());
            } else {
                detail_segments.push(format!("Manager: powertop autotune ({})", notes.join(", ")));
            }
        }
        PowerManager::None => {
            detail_segments.push("Manager: none detected".into());
        }
    }

    if !governor_info.governor_counts.is_empty() {
        detail_segments.push(format!(
            "Governors: {}",
            format_counts(&governor_info.governor_counts)
        ));
    } else if governor_info.cpufreq_present {
        detail_segments.push("Governors: no active cpufreq policies detected".into());
    } else {
        detail_segments.push("Governors: cpufreq interface not exposed".into());
    }

    if !governor_info.epp_counts.is_empty() {
        detail_segments.push(format!("EPP: {}", format_counts(&governor_info.epp_counts)));
    }
    if let Some(status) = &governor_info.intel_pstate_status {
        detail_segments.push(format!("intel_pstate status: {status}"));
    }

    let mut severity = Severity::Info;
    let mut fix = None;

    let summary = match &manager {
        PowerManager::PowerProfiles { profile } => match profile.as_str() {
            "performance" => "✓ power-profiles-daemon on performance profile; no action".into(),
            "balanced" => "✓ power-profiles-daemon on balanced profile".into(),
            "power-saver" => {
                severity = Severity::Warn;
                fix = Some(FixPlan {
                    summary: "Switch to performance profile".into(),
                    dry_run_cmds: vec!["powerprofilesctl list".into()],
                    apply_cmds: vec!["powerprofilesctl set performance".into()],
                    undo_cmds: vec!["powerprofilesctl set balanced".into()],
                });
                "⚠ power-profiles-daemon in power-saver; switch to performance on AC".into()
            }
            other => format!("✓ power-profiles-daemon active ({other}); adjust as needed"),
        },
        PowerManager::Tlp { mode } => match mode.as_deref() {
            Some("AC") | Some("ac") => "✓ TLP managing AC profile; no action".into(),
            Some(other) => {
                severity = Severity::Warn;
                fix = Some(FixPlan {
                    summary: "Switch TLP to AC profile".into(),
                    dry_run_cmds: vec!["tlp-stat -p".into()],
                    apply_cmds: vec!["sudo tlp ac".into()],
                    undo_cmds: vec!["sudo tlp bat".into()],
                });
                format!("⚠ TLP running in {other} mode; switch to AC profile when plugged in")
            }
            None => "✓ TLP active".into(),
        },
        PowerManager::Tuned { profile } => {
            if profile.contains("power") || profile.contains("save") {
                severity = Severity::Warn;
                fix = Some(FixPlan {
                    summary: "Set tuned profile to performance".into(),
                    dry_run_cmds: vec!["tuned-adm list".into()],
                    apply_cmds: vec!["sudo tuned-adm profile performance".into()],
                    undo_cmds: vec![format!("sudo tuned-adm profile {}", profile)],
                });
                format!(
                    "⚠ tuned profile '{}' prioritises power savings; switch to performance",
                    profile
                )
            } else {
                format!("✓ tuned active ({profile})")
            }
        }
        PowerManager::Thermald => {
            "✓ thermald handling CPU thermals; governors look reasonable".into()
        }
        PowerManager::Powertop { .. } => "✓ powertop autotune applied".into(),
        PowerManager::None => {
            if let Some(majority) = governor_info.majority_governor() {
                if majority == "powersave" {
                    severity = Severity::Warn;
                    fix = Some(cpupower_fix_plan());
                    "⚠ CPU governors set to powersave; switch to schedutil/performance".into()
                } else {
                    format!("✓ Governors default to {}; no manager detected", majority)
                }
            } else if let Some(epp) = governor_info.majority_epp() {
                if matches!(epp, "power" | "balance_power") {
                    severity = Severity::Warn;
                    if which("powerprofilesctl").is_ok() {
                        fix = Some(FixPlan {
                            summary: "Set performance profile via powerprofilesctl".into(),
                            dry_run_cmds: vec!["powerprofilesctl list".into()],
                            apply_cmds: vec!["powerprofilesctl set performance".into()],
                            undo_cmds: vec!["powerprofilesctl set balanced".into()],
                        });
                    } else {
                        fix = Some(cpupower_fix_plan());
                    }
                    format!("⚠ CPU EPP is {}; favour performance when on AC", epp)
                } else {
                    format!("✓ CPU EPP majority {}; no action", epp)
                }
            } else {
                severity = Severity::Warn;
                "⚠ Unable to read CPU governor state".into()
            }
        }
    };

    Finding {
        id: "cpu.governor".into(),
        title: "CPU power management".to_string(),
        severity,
        summary,
        detail: detail_segments.join("; "),
        fix,
    }
}

#[derive(Default)]
struct CpuGovernorInfo {
    cpufreq_present: bool,
    total: usize,
    governor_counts: HashMap<String, usize>,
    epp_counts: HashMap<String, usize>,
    intel_pstate_status: Option<String>,
}

impl CpuGovernorInfo {
    fn majority_governor(&self) -> Option<&str> {
        self.governor_counts
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.cmp(b.0)))
            .map(|(name, _)| name.as_str())
    }

    fn majority_epp(&self) -> Option<&str> {
        self.epp_counts
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.cmp(b.0)))
            .map(|(name, _)| name.as_str())
    }
}

fn gather_cpu_governors() -> CpuGovernorInfo {
    let mut info = CpuGovernorInfo::default();
    if let Ok(status) = fs::read_to_string("/sys/devices/system/cpu/intel_pstate/status") {
        let trimmed = status.trim();
        if !trimmed.is_empty() {
            info.intel_pstate_status = Some(trimmed.to_string());
        }
    }

    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu/cpufreq") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if !matches!(name.to_str(), Some(s) if s.starts_with("policy")) {
                continue;
            }
            let base = entry.path();
            info.cpufreq_present = true;
            let weight = read_related_cpus(base.join("related_cpus")).len().max(1);
            if let Ok(governor) = fs::read_to_string(base.join("scaling_governor")) {
                let key = governor.trim().to_string();
                if !key.is_empty() {
                    *info.governor_counts.entry(key).or_insert(0) += weight;
                    info.total += weight;
                }
            }
            if let Ok(epp) = fs::read_to_string(base.join("energy_performance_preference")) {
                let key = epp.trim().to_string();
                if !key.is_empty() {
                    *info.epp_counts.entry(key).or_insert(0) += weight;
                }
            }
        }
        if info.total > 0 || info.cpufreq_present {
            return info;
        }
    }

    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let cpu_dir = match name.to_str() {
                Some(s) if s.starts_with("cpu") && s[3..].chars().all(|c| c.is_ascii_digit()) => {
                    entry.path()
                }
                _ => continue,
            };
            let online_path = cpu_dir.join("online");
            if let Ok(state) = fs::read_to_string(&online_path) {
                if state.trim() == "0" {
                    continue;
                }
            }
            let governor_path = cpu_dir.join("cpufreq/scaling_governor");
            if let Ok(governor) = fs::read_to_string(&governor_path) {
                let key = governor.trim().to_string();
                if !key.is_empty() {
                    info.cpufreq_present = true;
                    *info.governor_counts.entry(key).or_insert(0) += 1;
                    info.total += 1;
                }
            }
            let epp_path = cpu_dir.join("cpufreq/energy_performance_preference");
            if let Ok(epp) = fs::read_to_string(&epp_path) {
                let key = epp.trim().to_string();
                if !key.is_empty() {
                    *info.epp_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    info
}

fn read_related_cpus(path: PathBuf) -> Vec<usize> {
    let contents = fs::read_to_string(path).unwrap_or_default();
    let mut cpus = Vec::new();
    for token in contents.split_whitespace() {
        if let Some((start, end)) = token.split_once('-') {
            if let (Ok(start), Ok(end)) = (start.parse::<usize>(), end.parse::<usize>()) {
                for cpu in start..=end {
                    cpus.push(cpu);
                }
            }
        } else if let Ok(cpu) = token.parse::<usize>() {
            cpus.push(cpu);
        }
    }
    cpus
}

fn format_counts(map: &HashMap<String, usize>) -> String {
    let mut pairs: Vec<(&String, &usize)> = map.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    pairs
        .into_iter()
        .map(|(k, v)| format!("{k} x{v}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug)]
enum PowerManager {
    PowerProfiles { profile: String },
    Tlp { mode: Option<String> },
    Tuned { profile: String },
    Thermald,
    Powertop { notes: Vec<String> },
    None,
}

struct TlpStatus {
    mode: Option<String>,
}

async fn detect_power_manager() -> PowerManager {
    if let Some(profile) = detect_power_profiles().await {
        return PowerManager::PowerProfiles { profile };
    }
    if let Some(tlp) = detect_tlp().await {
        return PowerManager::Tlp { mode: tlp.mode };
    }
    if let Some(profile) = detect_tuned().await {
        return PowerManager::Tuned { profile };
    }
    if detect_thermald().await {
        return PowerManager::Thermald;
    }
    if let Some(notes) = detect_powertop().await {
        return PowerManager::Powertop { notes };
    }
    PowerManager::None
}

async fn detect_power_profiles() -> Option<String> {
    if which("powerprofilesctl").is_err() {
        return None;
    }
    let output = Command::new("powerprofilesctl")
        .arg("get")
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if profile.is_empty() {
        None
    } else {
        Some(profile)
    }
}

async fn detect_tlp() -> Option<TlpStatus> {
    if which("systemctl").is_err() {
        return None;
    }
    let status = Command::new("systemctl")
        .args(["is-active", "tlp"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .ok()?;
    if !status.success() {
        return None;
    }
    let mut mode = None;
    if which("tlp-stat").is_ok() {
        if let Ok(output) = Command::new("tlp-stat").arg("-p").output().await {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                mode = parse_tlp_mode(&text);
            }
        }
    }
    Some(TlpStatus { mode })
}

fn parse_tlp_mode(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(rest) = line.split_once('=') {
            let label = rest.0.trim();
            let value = rest.1.trim();
            if label.eq_ignore_ascii_case("Mode") && !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

async fn detect_tuned() -> Option<String> {
    if which("tuned-adm").is_err() {
        return None;
    }
    let output = Command::new("tuned-adm")
        .arg("active")
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(rest) = line.split_once(':') {
            if rest.0.trim().eq_ignore_ascii_case("Current active profile") {
                let value = rest.1.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

async fn detect_thermald() -> bool {
    if which("systemctl").is_err() {
        return false;
    }
    Command::new("systemctl")
        .args(["is-active", "thermald"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
}

async fn detect_powertop() -> Option<Vec<String>> {
    let mut notes = Vec::new();
    if which("systemctl").is_ok() {
        if let Ok(status) = Command::new("systemctl")
            .args(["is-active", "powertop.service"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            if status.success() {
                notes.push("powertop.service active".into());
            }
        }
    }
    if let Ok(rc_local) = fs::read_to_string("/etc/rc.local") {
        if rc_local.contains("powertop --auto-tune") {
            notes.push("/etc/rc.local powertop --auto-tune".into());
        }
    }
    if notes.is_empty() {
        None
    } else {
        Some(notes)
    }
}

fn cpupower_fix_plan() -> FixPlan {
    FixPlan {
        summary: "Switch governors to schedutil".into(),
        dry_run_cmds: vec!["cpupower frequency-info".into()],
        apply_cmds: vec!["sudo cpupower frequency-set --governor schedutil".into()],
        undo_cmds: vec!["sudo cpupower frequency-set --governor powersave".into()],
    }
}

async fn memory_swap_probe() -> Finding {
    match collect_swap_snapshot().await {
        Ok(snapshot) => {
            let distro = detect_distro();
            let portable = detect_portable_chassis();
            let zram_managers = detect_zram_managers().await.unwrap_or_default();

            let real = snapshot.real_swap_totals();
            let dominant = snapshot.dominant_headroom_source();

            let headroom_bytes = snapshot.effective_headroom();
            let mem_total_bytes = snapshot.mem_total_bytes;
            let headroom_ratio = if mem_total_bytes > 0 {
                headroom_bytes as f64 / mem_total_bytes as f64
            } else {
                0.0
            };

            let warn_threshold = 0.12;
            let action_threshold = 0.08;

            let mut severity = if headroom_bytes == 0 {
                Severity::Action
            } else if headroom_ratio < action_threshold {
                Severity::Action
            } else if headroom_ratio < warn_threshold {
                Severity::Warn
            } else {
                Severity::Info
            };

            let mut detail_segments = Vec::new();
            if real.total_bytes > 0 {
                let mut per_entry = Vec::new();
                for entry in &snapshot.entries {
                    if matches!(entry.kind, SwapKind::Partition | SwapKind::File) {
                        let mut piece = format!(
                            "{} {} total, {} used",
                            entry.name,
                            fmt_bytes(entry.size_bytes),
                            fmt_bytes(entry.used_bytes)
                        );
                        if let Some(prio) = entry.priority {
                            piece.push_str(&format!(" (prio {})", prio));
                        }
                        per_entry.push(piece);
                    }
                }
                detail_segments.push(format!(
                    "Real swap: {}; {} free",
                    per_entry.join(", "),
                    fmt_bytes(real.free_bytes)
                ));
            }
            if !snapshot.zram_devices.is_empty() {
                let mut parts = Vec::new();
                for dev in &snapshot.zram_devices {
                    parts.push(format!(
                        "{} {} logical, {} stored, {} resident{}",
                        dev.name,
                        fmt_bytes(dev.disksize_bytes),
                        fmt_bytes(dev.orig_data_bytes),
                        fmt_bytes(dev.mem_used_bytes),
                        dev.comp_algorithm
                            .as_ref()
                            .map(|algo| format!(" ({} compression)", algo))
                            .unwrap_or_default()
                    ));
                }
                detail_segments.push(format!("zram: {}", parts.join(", ")));
            }
            if snapshot.zswap.enabled {
                let limit = snapshot
                    .zswap
                    .limit_bytes
                    .map(fmt_bytes)
                    .unwrap_or_else(|| "unknown".into());
                let used = snapshot
                    .zswap
                    .used_bytes
                    .map(fmt_bytes)
                    .unwrap_or_else(|| "unknown".into());
                detail_segments.push(format!("zswap enabled (limit {}, used {})", limit, used));
            }
            if !zram_managers.is_empty() {
                detail_segments.push(format!("zram units: {}", zram_managers.join(", ")));
            }

            let headroom_pct = (headroom_ratio * 100.0).round();
            detail_segments.push(format!(
                "Effective headroom {} (~{:.0}% of RAM)",
                fmt_bytes(headroom_bytes),
                headroom_ratio * 100.0
            ));

            let mut summary = match dominant {
                HeadroomSource::Zram => "✓ Swap headroom is healthy via zram; no action".into(),
                HeadroomSource::RealSwap => {
                    "✓ Swap headroom is healthy via swapfile/device; no action".into()
                }
                HeadroomSource::Zswap => "✓ Swap headroom is healthy via zswap; no action".into(),
                HeadroomSource::None => "⚠ No swap headroom detected; configure swap".into(),
            };

            let mut fix = None;
            if matches!(severity, Severity::Warn | Severity::Action) {
                summary = match severity {
                    Severity::Warn => format!(
                        "⚠ Swap headroom is low (~{headroom_pct:.0}% of RAM); consider expanding"
                    ),
                    Severity::Action => format!(
                        "⚠ Swap headroom is critical (~{headroom_pct:.0}% of RAM); expand now"
                    ),
                    _ => summary,
                };
                severity = if severity == Severity::Warn && headroom_bytes == 0 {
                    Severity::Action
                } else {
                    severity
                };
                fix = Some(build_swap_fix(&snapshot, distro, portable, headroom_bytes));
            }

            Finding {
                id: "memory.swap".into(),
                title: "Swap availability".to_string(),
                severity,
                summary,
                detail: detail_segments.join("; "),
                fix,
            }
        }
        Err(err) => Finding {
            id: "memory.swap".into(),
            title: "Swap availability".to_string(),
            severity: Severity::Warn,
            summary: "⚠ Unable to read swap configuration".into(),
            detail: format!("{err:?}"),
            fix: None,
        },
    }
}

async fn storage_probe() -> Finding {
    let entries = gather_mount_entries().await;
    if entries.is_empty() {
        return Finding {
            id: "storage.free".into(),
            title: "Disk free space".to_string(),
            severity: Severity::Info,
            summary: "✓ No writable mounts detected; skipping free-space check".into(),
            detail: String::new(),
            fix: None,
        };
    }

    let mut issues = Vec::new();
    let mut details = Vec::new();
    for entry in &entries {
        details.push(describe_mount(entry));
        issues.extend(evaluate_mount(entry));
    }

    let mut severity = Severity::Info;
    for issue in &issues {
        severity = max_severity(severity, issue.severity);
    }

    let summary = match severity {
        Severity::Info => "✓ Storage volumes healthy; no action".into(),
        Severity::Warn | Severity::Action => {
            if let Some(top) = issues
                .iter()
                .filter(|issue| issue.severity == severity)
                .next()
            {
                let prefix = if severity == Severity::Action {
                    "⚠ Storage critical"
                } else {
                    "⚠ Storage warning"
                };
                format!("{prefix}: {}", top.summary_fragment)
            } else if let Some(first) = issues.first() {
                let prefix = if severity == Severity::Action {
                    "⚠ Storage critical"
                } else {
                    "⚠ Storage warning"
                };
                format!("{prefix}: {}", first.summary_fragment)
            } else {
                "⚠ Storage warning detected".into()
            }
        }
    };

    let fix = if severity == Severity::Info {
        None
    } else {
        build_storage_fix(&issues)
    };

    Finding {
        id: "storage.free".into(),
        title: "Disk free space".to_string(),
        severity,
        summary,
        detail: details.join("; "),
        fix,
    }
}

#[derive(Clone)]
struct MountEntry {
    mount: String,
    fs_type: String,
    total_bytes: u64,
    free_bytes: u64,
    btrfs: Option<BtrfsInfo>,
}

#[derive(Clone, Default)]
struct BtrfsInfo {
    data_used: u64,
    data_total: u64,
    metadata_used: u64,
    metadata_total: u64,
    system_used: u64,
    system_total: u64,
    snapshot_count: Option<usize>,
}

#[derive(Clone)]
struct StorageIssue {
    kind: StorageIssueKind,
    severity: Severity,
    summary_fragment: String,
}

#[derive(Clone)]
enum StorageIssueKind {
    LowFree { mount: String },
    BtrfsData { mount: String },
    BtrfsMetadata { mount: String },
    BtrfsSnapshots { mount: String },
}

async fn gather_mount_entries() -> Vec<MountEntry> {
    let mounts = fs::read_to_string("/proc/mounts").unwrap_or_default();
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let mountpoint = unescape_mount(parts[1]);
        let fs_type = parts[2];
        let options = parts[3];
        if !options.contains("rw") {
            continue;
        }
        if should_skip_fs(fs_type) {
            continue;
        }
        if !seen.insert(mountpoint.clone()) {
            continue;
        }
        let stat = match statvfs::statvfs(&mountpoint) {
            Ok(stat) => stat,
            Err(_) => continue,
        };
        if stat.blocks() == 0 {
            continue;
        }
        let total_bytes = stat.blocks() * stat.block_size();
        let free_bytes = stat.blocks_available() * stat.block_size();
        let mut entry = MountEntry {
            mount: mountpoint.clone(),
            fs_type: fs_type.to_string(),
            total_bytes,
            free_bytes,
            btrfs: None,
        };
        if entry.fs_type == "btrfs" {
            entry.btrfs = collect_btrfs_info(&entry.mount).await;
        }
        entries.push(entry);
    }
    entries
}

fn should_skip_fs(fs_type: &str) -> bool {
    matches!(
        fs_type,
        "proc"
            | "sysfs"
            | "tmpfs"
            | "devtmpfs"
            | "cgroup"
            | "cgroup2"
            | "overlay"
            | "squashfs"
            | "autofs"
            | "rpc_pipefs"
            | "pstore"
            | "debugfs"
            | "tracefs"
            | "configfs"
            | "hugetlbfs"
            | "fusectl"
            | "binfmt_misc"
            | "nsfs"
            | "securityfs"
            | "ramfs"
            | "efivarfs"
    )
}

fn unescape_mount(raw: &str) -> String {
    let mut result = String::new();
    let mut iter = raw.chars().peekable();
    while let Some(ch) = iter.next() {
        if ch == '\\' {
            let mut code = String::new();
            for _ in 0..3 {
                if let Some(&next) = iter.peek() {
                    if next.is_ascii_digit() {
                        code.push(next);
                        iter.next();
                    }
                }
            }
            match code.as_str() {
                "040" => result.push(' '),
                "011" => result.push('\t'),
                "012" => result.push('\n'),
                _ => {
                    result.push('\\');
                    result.push_str(&code);
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

async fn collect_btrfs_info(mount: &str) -> Option<BtrfsInfo> {
    if which("btrfs").is_err() {
        return None;
    }
    let usage = Command::new("btrfs")
        .args(["filesystem", "usage", "-b", mount])
        .output()
        .await
        .ok()?;
    if !usage.status.success() {
        return None;
    }
    let mut info = parse_btrfs_usage(&String::from_utf8_lossy(&usage.stdout))?;
    info.snapshot_count = collect_btrfs_snapshot_count(mount).await;
    Some(info)
}

fn parse_btrfs_usage(output: &str) -> Option<BtrfsInfo> {
    let mut info = BtrfsInfo::default();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Data") {
            if let Some(total) =
                parse_value_after(trimmed, "total=").or_else(|| parse_value_after(trimmed, "Size:"))
            {
                info.data_total = total;
            }
            if let Some(used) =
                parse_value_after(trimmed, "used=").or_else(|| parse_value_after(trimmed, "Used:"))
            {
                info.data_used = used;
            }
        } else if trimmed.starts_with("Metadata") {
            if let Some(total) =
                parse_value_after(trimmed, "total=").or_else(|| parse_value_after(trimmed, "Size:"))
            {
                info.metadata_total = total;
            }
            if let Some(used) =
                parse_value_after(trimmed, "used=").or_else(|| parse_value_after(trimmed, "Used:"))
            {
                info.metadata_used = used;
            }
        } else if trimmed.starts_with("System") {
            if let Some(total) =
                parse_value_after(trimmed, "total=").or_else(|| parse_value_after(trimmed, "Size:"))
            {
                info.system_total = total;
            }
            if let Some(used) =
                parse_value_after(trimmed, "used=").or_else(|| parse_value_after(trimmed, "Used:"))
            {
                info.system_used = used;
            }
        }
    }
    Some(info)
}

fn parse_value_after(line: &str, key: &str) -> Option<u64> {
    let idx = line.find(key)? + key.len();
    let rest = &line[idx..];
    let digits: String = rest
        .chars()
        .skip_while(|c| *c == ' ' || *c == '=' || *c == ':')
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u64>().ok()
    }
}

async fn collect_btrfs_snapshot_count(mount: &str) -> Option<usize> {
    if which("btrfs").is_err() {
        return None;
    }
    let output = Command::new("btrfs")
        .args(["subvolume", "list", "-s", mount])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    Some(count)
}

fn evaluate_mount(entry: &MountEntry) -> Vec<StorageIssue> {
    let mut issues = Vec::new();
    let free_ratio = if entry.total_bytes > 0 {
        entry.free_bytes as f64 / entry.total_bytes as f64
    } else {
        1.0
    };
    if free_ratio < 0.05 {
        issues.push(StorageIssue {
            kind: StorageIssueKind::LowFree {
                mount: entry.mount.clone(),
            },
            severity: Severity::Action,
            summary_fragment: format!("{} {:.1}% free", entry.mount, free_ratio * 100.0),
        });
    } else if free_ratio < 0.12 {
        issues.push(StorageIssue {
            kind: StorageIssueKind::LowFree {
                mount: entry.mount.clone(),
            },
            severity: Severity::Warn,
            summary_fragment: format!("{} {:.1}% free", entry.mount, free_ratio * 100.0),
        });
    }

    if let Some(btrfs) = &entry.btrfs {
        if btrfs.data_total > 0 {
            let ratio = btrfs.data_used as f64 / btrfs.data_total as f64;
            if ratio > 0.92 {
                issues.push(StorageIssue {
                    kind: StorageIssueKind::BtrfsData {
                        mount: entry.mount.clone(),
                    },
                    severity: Severity::Action,
                    summary_fragment: format!(
                        "{} Btrfs data {:.0}% used",
                        entry.mount,
                        ratio * 100.0
                    ),
                });
            } else if ratio > 0.85 {
                issues.push(StorageIssue {
                    kind: StorageIssueKind::BtrfsData {
                        mount: entry.mount.clone(),
                    },
                    severity: Severity::Warn,
                    summary_fragment: format!(
                        "{} Btrfs data {:.0}% used",
                        entry.mount,
                        ratio * 100.0
                    ),
                });
            }
        }
        if btrfs.metadata_total > 0 {
            let ratio = btrfs.metadata_used as f64 / btrfs.metadata_total as f64;
            if ratio > 0.9 {
                issues.push(StorageIssue {
                    kind: StorageIssueKind::BtrfsMetadata {
                        mount: entry.mount.clone(),
                    },
                    severity: Severity::Action,
                    summary_fragment: format!(
                        "{} Btrfs metadata {:.0}% used",
                        entry.mount,
                        ratio * 100.0
                    ),
                });
            } else if ratio > 0.8 {
                issues.push(StorageIssue {
                    kind: StorageIssueKind::BtrfsMetadata {
                        mount: entry.mount.clone(),
                    },
                    severity: Severity::Warn,
                    summary_fragment: format!(
                        "{} Btrfs metadata {:.0}% used",
                        entry.mount,
                        ratio * 100.0
                    ),
                });
            }
        }
        if let Some(snaps) = btrfs.snapshot_count {
            if snaps > 200 {
                issues.push(StorageIssue {
                    kind: StorageIssueKind::BtrfsSnapshots {
                        mount: entry.mount.clone(),
                    },
                    severity: Severity::Action,
                    summary_fragment: format!("{} {} snapshots", entry.mount, snaps),
                });
            } else if snaps > 100 {
                issues.push(StorageIssue {
                    kind: StorageIssueKind::BtrfsSnapshots {
                        mount: entry.mount.clone(),
                    },
                    severity: Severity::Warn,
                    summary_fragment: format!("{} {} snapshots", entry.mount, snaps),
                });
            }
        }
    }

    issues
}

fn describe_mount(entry: &MountEntry) -> String {
    let percent = if entry.total_bytes > 0 {
        entry.free_bytes as f64 / entry.total_bytes as f64 * 100.0
    } else {
        100.0
    };
    let mut detail = format!(
        "{} ({}) free {} ({:.1}%)",
        entry.mount,
        entry.fs_type,
        fmt_bytes(entry.free_bytes),
        percent
    );
    if let Some(btrfs) = &entry.btrfs {
        if btrfs.data_total > 0 {
            let ratio = btrfs.data_used as f64 / btrfs.data_total as f64 * 100.0;
            detail.push_str(&format!("; data {:.0}%", ratio));
        }
        if btrfs.metadata_total > 0 {
            let ratio = btrfs.metadata_used as f64 / btrfs.metadata_total as f64 * 100.0;
            detail.push_str(&format!("; metadata {:.0}%", ratio));
        }
        if let Some(snaps) = btrfs.snapshot_count {
            detail.push_str(&format!("; {} snapshots", snaps));
        }
    }
    detail
}

fn max_severity(a: Severity, b: Severity) -> Severity {
    match (a, b) {
        (Severity::Action, _) | (_, Severity::Action) => Severity::Action,
        (Severity::Warn, _) | (_, Severity::Warn) => Severity::Warn,
        _ => Severity::Info,
    }
}

fn build_storage_fix(issues: &[StorageIssue]) -> Option<FixPlan> {
    if issues.is_empty() {
        return None;
    }
    if let Some(issue) = issues
        .iter()
        .find(|i| matches!(i.kind, StorageIssueKind::BtrfsSnapshots { .. }))
    {
        if let StorageIssueKind::BtrfsSnapshots { mount } = &issue.kind {
            return Some(FixPlan {
                summary: format!("Prune Btrfs snapshots on {}", mount),
                dry_run_cmds: vec![format!("sudo btrfs subvolume list -s {}", mount)],
                apply_cmds: vec!["sudo btrfs subvolume delete <snapshot-path>".into()],
                undo_cmds: Vec::new(),
            });
        }
    }
    if let Some(issue) = issues.iter().find(|i| {
        matches!(
            i.kind,
            StorageIssueKind::BtrfsData { .. } | StorageIssueKind::BtrfsMetadata { .. }
        )
    }) {
        let mount = match &issue.kind {
            StorageIssueKind::BtrfsData { mount } | StorageIssueKind::BtrfsMetadata { mount } => {
                mount
            }
            _ => unreachable!(),
        };
        return Some(FixPlan {
            summary: format!("Reclaim Btrfs space on {}", mount),
            dry_run_cmds: vec![format!("sudo btrfs filesystem usage {}", mount)],
            apply_cmds: vec![format!(
                "sudo btrfs balance start -dusage=75 -musage=75 {}",
                mount
            )],
            undo_cmds: Vec::new(),
        });
    }
    if let Some(issue) = issues
        .iter()
        .find(|i| matches!(i.kind, StorageIssueKind::LowFree { .. }))
    {
        let mount = match &issue.kind {
            StorageIssueKind::LowFree { mount } => mount,
            _ => unreachable!(),
        };
        return Some(FixPlan {
            summary: format!("Free disk space on {}", mount),
            dry_run_cmds: vec![
                "sudo du -sh /var/log/*".into(),
                "sudo du -sh /var/cache/*".into(),
            ],
            apply_cmds: vec!["sudo journalctl --vacuum-time=14d".into()],
            undo_cmds: Vec::new(),
        });
    }
    None
}

async fn fs_trim_probe() -> Finding {
    let ssd_devices = detect_ssd_devices();
    if ssd_devices.is_empty() {
        return Finding {
            id: "storage.trim".into(),
            title: "Filesystem TRIM".to_string(),
            severity: Severity::Info,
            summary: "✓ No SSD detected; periodic fstrim not required".into(),
            detail: "Checked /sys/block for non-rotational devices".into(),
            fix: None,
        };
    }

    if which("systemctl").is_err() {
        return Finding {
            id: "storage.trim".into(),
            title: "Filesystem TRIM".to_string(),
            severity: Severity::Warn,
            summary: "⚠ Cannot verify fstrim.timer (systemctl unavailable)".into(),
            detail: format!("SSD devices: {}", ssd_devices.join(", ")),
            fix: None,
        };
    }

    let enabled = Command::new("systemctl")
        .args(["is-enabled", "fstrim.timer"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .unwrap_or_else(|_| std::process::ExitStatus::from_raw(1))
        .success();

    let last_run = read_fstrim_last_run().await;
    let detail = match last_run {
        Some(ref when) => format!("SSD devices: {}; last run {}", ssd_devices.join(", "), when),
        None => format!(
            "SSD devices: {}; last run unavailable",
            ssd_devices.join(", ")
        ),
    };

    if enabled {
        let summary = if let Some(when) = last_run {
            format!("✓ fstrim.timer enabled (last run {})", when)
        } else {
            "✓ fstrim.timer enabled".into()
        };
        Finding {
            id: "storage.trim".into(),
            title: "Filesystem TRIM".to_string(),
            severity: Severity::Info,
            summary,
            detail,
            fix: None,
        }
    } else {
        let fix = FixPlan {
            summary: "Enable weekly fstrim".into(),
            dry_run_cmds: vec!["systemctl status fstrim.timer".into()],
            apply_cmds: vec!["sudo systemctl enable --now fstrim.timer".into()],
            undo_cmds: vec!["sudo systemctl disable --now fstrim.timer".into()],
        };
        Finding {
            id: "storage.trim".into(),
            title: "Filesystem TRIM".to_string(),
            severity: Severity::Warn,
            summary: "⚠ fstrim.timer disabled on SSD system; enable weekly trimming".into(),
            detail,
            fix: Some(fix),
        }
    }
}

fn detect_ssd_devices() -> Vec<String> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("fd")
                || name.starts_with("sr")
            {
                continue;
            }
            let rotational_path = entry.path().join("queue/rotational");
            if let Ok(value) = fs::read_to_string(rotational_path) {
                if value.trim() == "0" {
                    devices.push(name);
                }
            }
        }
    }
    devices
}

async fn read_fstrim_last_run() -> Option<String> {
    let output = Command::new("systemctl")
        .args(["status", "fstrim.timer"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Last Trigger:") {
            return Some(rest.trim().to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("Last run:") {
            return Some(rest.trim().to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("Last Successful Run:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

async fn ntp_probe() -> Finding {
    if which("timedatectl").is_err() {
        return Finding {
            id: "time.ntp".into(),
            title: "System time sync".to_string(),
            severity: Severity::Warn,
            summary: "timedatectl missing; unable to verify NTP".into(),
            detail: "Install systemd-timesyncd or chrony".into(),
            fix: None,
        };
    }
    let output = match Command::new("timedatectl")
        .args(["show", "-p", "NTPSynchronized", "--value"])
        .output()
        .await
    {
        Ok(out) => out,
        Err(_) => {
            return Finding {
                id: "time.ntp".into(),
                title: "System time sync".to_string(),
                severity: Severity::Warn,
                summary: "timedatectl invocation failed".into(),
                detail: "Unable to determine NTP status".into(),
                fix: None,
            }
        }
    };
    let synced = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if synced == "yes" {
        Finding {
            id: "time.ntp".into(),
            title: "System time sync".to_string(),
            severity: Severity::Info,
            summary: "NTP is enabled".into(),
            detail: "timedatectl reports NTPSynchronized=yes".into(),
            fix: None,
        }
    } else {
        let fix = FixPlan {
            summary: "Enable systemd-timesyncd".into(),
            dry_run_cmds: vec!["timedatectl status".into()],
            apply_cmds: vec!["sudo timedatectl set-ntp true".into()],
            undo_cmds: vec!["sudo timedatectl set-ntp false".into()],
        };
        Finding {
            id: "time.ntp".into(),
            title: "System time sync".to_string(),
            severity: Severity::Action,
            summary: "NTP is disabled".into(),
            detail: format!("timedatectl NTPSynchronized={}", synced),
            fix: Some(fix),
        }
    }
}

async fn pkg_cache_probe() -> Finding {
    let mut caches: Vec<(String, PathBuf, u64)> = Vec::new();
    if let Some(path) = pacman_cache() {
        let size = dir_size(&path).unwrap_or(0);
        caches.push(("pacman".into(), path, size));
    }
    if let Some(path) = apt_cache() {
        let size = dir_size(&path).unwrap_or(0);
        caches.push(("apt".into(), path, size));
    }
    if let Some(path) = dnf_cache() {
        let size = dir_size(&path).unwrap_or(0);
        caches.push(("dnf".into(), path, size));
    }

    let threshold = 2 * 1024 * 1024 * 1024u64;
    let mut heavy = Vec::new();
    let mut fixes = Vec::new();
    let mut detail_lines = Vec::new();
    for (kind, path, size) in &caches {
        detail_lines.push(format!(
            "{}: {} ({})",
            kind,
            fmt_bytes(*size),
            path.display()
        ));
        if *size > threshold {
            heavy.push((kind.clone(), *size));
            fixes.push(cache_cleanup_plan(kind));
        }
    }

    if heavy.is_empty() {
        Finding {
            id: "packages.cache".into(),
            title: "Package cache size".to_string(),
            severity: Severity::Info,
            summary: "✓ Package caches are under 2 GiB".into(),
            detail: detail_lines.join("; "),
            fix: None,
        }
    } else {
        let summary_bits: Vec<String> = heavy
            .iter()
            .map(|(kind, size)| format!("{} {}", kind, fmt_bytes(*size)))
            .collect();
        Finding {
            id: "packages.cache".into(),
            title: "Package cache size".to_string(),
            severity: Severity::Warn,
            summary: format!("⚠ Large package caches: {}", summary_bits.join(", ")),
            detail: detail_lines.join("; "),
            fix: merge_plans(&fixes),
        }
    }
}

async fn orphan_probe() -> Finding {
    let mut hits = Vec::new();
    let mut checks = Vec::new();
    let mut fixes = Vec::new();

    if which("pacman").is_ok() {
        let mut cmd = Command::new("pacman");
        cmd.args(["-Qtdq"]);
        if let Ok(count) = count_lines(cmd).await {
            checks.push(format!("pacman ({})", count));
            if count > 0 {
                hits.push(format!("pacman {}", count));
                fixes.push(FixPlan {
                    summary: "Review pacman orphans".into(),
                    dry_run_cmds: vec!["pacman -Qtdq".into()],
                    apply_cmds: vec!["sudo pacman -Qtdq | sudo pacman -Rns -".into()],
                    undo_cmds: Vec::new(),
                });
            }
        }
    }
    if which("dnf").is_ok() {
        let mut cmd = Command::new("dnf");
        cmd.args(["repoquery", "--unneeded"]);
        if let Ok(count) = count_lines(cmd).await {
            checks.push(format!("dnf ({})", count));
            if count > 0 {
                hits.push(format!("dnf {}", count));
                fixes.push(FixPlan {
                    summary: "Review dnf unneeded packages".into(),
                    dry_run_cmds: vec!["dnf repoquery --unneeded".into()],
                    apply_cmds: vec!["sudo dnf autoremove".into()],
                    undo_cmds: Vec::new(),
                });
            }
        }
    }
    if which("apt-get").is_ok() {
        if let Ok(output) = Command::new("apt-get")
            .args(["-s", "autoremove"])
            .output()
            .await
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let count = text.lines().filter(|line| line.contains("Remv")).count();
            checks.push(format!("apt ({})", count));
            if count > 0 {
                hits.push(format!("apt {}", count));
                fixes.push(FixPlan {
                    summary: "Clean apt orphans".into(),
                    dry_run_cmds: vec!["apt-get -s autoremove".into()],
                    apply_cmds: vec!["sudo apt-get autoremove".into()],
                    undo_cmds: Vec::new(),
                });
            }
        }
    }

    if hits.is_empty() {
        Finding {
            id: "packages.orphans".into(),
            title: "Orphan packages".to_string(),
            severity: Severity::Info,
            summary: "✓ No package orphans detected".into(),
            detail: if checks.is_empty() {
                String::from("No package managers detected")
            } else {
                format!("Checks: {}", checks.join(", "))
            },
            fix: None,
        }
    } else {
        Finding {
            id: "packages.orphans".into(),
            title: "Orphan packages".to_string(),
            severity: Severity::Warn,
            summary: format!("⚠ Package orphans: {}", hits.join(", ")),
            detail: if checks.is_empty() {
                String::from("Consider removing unused packages")
            } else {
                format!(
                    "Checks: {}; remove unused packages to tidy dependencies",
                    checks.join(", ")
                )
            },
            fix: merge_plans(&fixes),
        }
    }
}

fn merge_plans(plans: &[FixPlan]) -> Option<FixPlan> {
    if plans.is_empty() {
        return None;
    }
    let mut merged = FixPlan {
        summary: "Follow the listed maintenance commands".into(),
        dry_run_cmds: Vec::new(),
        apply_cmds: Vec::new(),
        undo_cmds: Vec::new(),
    };
    for plan in plans {
        merged
            .dry_run_cmds
            .extend(plan.dry_run_cmds.iter().cloned());
        merged.apply_cmds.extend(plan.apply_cmds.iter().cloned());
        merged.undo_cmds.extend(plan.undo_cmds.iter().cloned());
    }
    Some(merged)
}

fn pacman_cache() -> Option<PathBuf> {
    let path = Path::new("/var/cache/pacman/pkg");
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn apt_cache() -> Option<PathBuf> {
    let path = Path::new("/var/cache/apt/archives");
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn dnf_cache() -> Option<PathBuf> {
    let path = Path::new("/var/cache/dnf");
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn cache_cleanup_plan(kind: &str) -> FixPlan {
    match kind {
        "pacman" => FixPlan {
            summary: "Clean pacman cache".into(),
            dry_run_cmds: vec!["paccache -dv".into()],
            apply_cmds: vec!["sudo paccache -rk2".into()],
            undo_cmds: Vec::new(),
        },
        "apt" => FixPlan {
            summary: "Clean apt cache".into(),
            dry_run_cmds: vec!["apt-get clean --dry-run".into()],
            apply_cmds: vec!["sudo apt-get clean".into()],
            undo_cmds: Vec::new(),
        },
        "dnf" => FixPlan {
            summary: "Clean dnf cache".into(),
            dry_run_cmds: vec!["dnf clean packages --verbose".into()],
            apply_cmds: vec!["sudo dnf clean packages".into()],
            undo_cmds: Vec::new(),
        },
        _ => FixPlan {
            summary: "Clean package cache".into(),
            dry_run_cmds: Vec::new(),
            apply_cmds: Vec::new(),
            undo_cmds: Vec::new(),
        },
    }
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut total = 0u64;
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            total += dir_size(&p)?;
        } else {
            total += p.metadata()?.len();
        }
    }
    Ok(total)
}

async fn count_lines(mut cmd: Command) -> Result<u64> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());
    let output = cmd.output().await?;
    let count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    Ok(count as u64)
}

mod statvfs {
    use anyhow::{Context, Result};
    use libc::statvfs as Statvfs;

    pub fn statvfs(path: &str) -> Result<StatStruct> {
        let c_path = std::ffi::CString::new(path).context("invalid path")?;
        let mut stat: Statvfs = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
        if rc != 0 {
            let err = std::io::Error::last_os_error();
            return Err(err.into());
        }
        Ok(StatStruct(stat))
    }

    pub struct StatStruct(Statvfs);

    impl StatStruct {
        pub fn blocks(&self) -> u64 {
            self.0.f_blocks
        }
        pub fn blocks_available(&self) -> u64 {
            self.0.f_bavail
        }
        pub fn block_size(&self) -> u64 {
            self.0.f_frsize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_plans_combines_commands() {
        let plan_a = FixPlan {
            summary: "a".into(),
            dry_run_cmds: vec!["a".into()],
            apply_cmds: vec!["b".into()],
            undo_cmds: vec!["c".into()],
        };
        let plan_b = FixPlan {
            summary: "b".into(),
            dry_run_cmds: vec!["d".into()],
            apply_cmds: vec!["e".into()],
            undo_cmds: Vec::new(),
        };
        let merged = merge_plans(&[plan_a, plan_b]).expect("plan");
        assert_eq!(merged.dry_run_cmds.len(), 2);
        assert_eq!(merged.apply_cmds.len(), 2);
    }

    #[test]
    fn count_severity_splits_levels() {
        let findings = vec![
            Finding {
                id: "1".into(),
                title: "a".into(),
                severity: Severity::Info,
                summary: String::new(),
                detail: String::new(),
                fix: None,
            },
            Finding {
                id: "2".into(),
                title: "b".into(),
                severity: Severity::Warn,
                summary: String::new(),
                detail: String::new(),
                fix: None,
            },
            Finding {
                id: "3".into(),
                title: "c".into(),
                severity: Severity::Action,
                summary: String::new(),
                detail: String::new(),
                fix: None,
            },
        ];
        let (ok, warn, action) = count_severity(&findings);
        assert_eq!((ok, warn, action), (1, 1, 1));
    }
}

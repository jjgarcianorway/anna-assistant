use crate::advice;
use crate::advice::types::AdvicePlan;
use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Action,
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
    let governor = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();
    let mut detail = format!("Current CPU governor: {}", governor);
    let mut severity = Severity::Info;
    let mut summary = "CPU power management looks reasonable".to_string();
    let mut fix = None;

    if governor == "powersave" {
        severity = Severity::Warn;
        summary = "CPU scaling governor is set to powersave".into();
        detail.push_str("; consider enabling schedutil/performance for AC workloads");
        fix = Some(FixPlan {
            summary: "Switch to schedutil governor".into(),
            dry_run_cmds: vec!["cpupower frequency-info".into()],
            apply_cmds: vec!["sudo cpupower frequency-set --governor schedutil".into()],
            undo_cmds: vec!["sudo cpupower frequency-set --governor powersave".into()],
        });
    }

    Finding {
        id: "cpu.governor".into(),
        title: "CPU power management".to_string(),
        severity,
        summary,
        detail,
        fix,
    }
}

async fn memory_swap_probe() -> Finding {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut values = HashMap::new();
    for line in meminfo.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if let Some(kb) = value.trim().strip_suffix(" kB") {
                if let Ok(num) = kb.trim().parse::<u64>() {
                    values.insert(key.trim().to_string(), num);
                }
            }
        }
    }
    let mem_total = *values.get("MemTotal").unwrap_or(&0);
    let swap_total = *values.get("SwapTotal").unwrap_or(&0);

    if swap_total == 0 && mem_total < 16 * 1024 * 1024 {
        let summary = "No swap detected on a system with <16 GiB RAM".into();
        let detail = format!("MemTotal={} kB, SwapTotal={} kB", mem_total, swap_total);
        let fix = FixPlan {
            summary: "Enable zram swap".into(),
            dry_run_cmds: vec!["systemctl status systemd-zram-setup@zram0".into()],
            apply_cmds: vec![
                "sudo systemctl enable --now systemd-zram-generator".into(),
                "sudo systemctl start systemd-zram-setup@zram0".into(),
            ],
            undo_cmds: vec!["sudo systemctl disable --now systemd-zram-generator".into()],
        };
        Finding {
            id: "memory.swap".into(),
            title: "Swap availability".to_string(),
            severity: Severity::Action,
            summary,
            detail,
            fix: Some(fix),
        }
    } else {
        Finding {
            id: "memory.swap".into(),
            title: "Swap availability".to_string(),
            severity: Severity::Info,
            summary: "Swap configuration looks fine".into(),
            detail: format!("MemTotal={} kB, SwapTotal={} kB", mem_total, swap_total),
            fix: None,
        }
    }
}

async fn storage_probe() -> Finding {
    let mounts = fs::read_to_string("/proc/mounts").unwrap_or_default();
    let mut warnings = Vec::new();
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let device = parts[0];
        let mountpoint = parts[1];
        let fs_type = parts[2];
        let options = parts[3];
        if !options.contains("rw") {
            continue;
        }
        if matches!(
            fs_type,
            "proc" | "sysfs" | "tmpfs" | "devtmpfs" | "cgroup" | "overlay"
        ) {
            continue;
        }
        if let Ok(stat) = statvfs::statvfs(mountpoint) {
            if stat.blocks() == 0 {
                continue;
            }
            let free = stat.blocks_available() as f64 * stat.block_size() as f64;
            let total = stat.blocks() as f64 * stat.block_size() as f64;
            if total > 0.0 {
                let ratio = free / total;
                if ratio < 0.15 {
                    warnings.push(format!(
                        "{} ({}): {:.1}% free",
                        mountpoint,
                        device,
                        ratio * 100.0
                    ));
                }
            }
        }
    }

    if warnings.is_empty() {
        Finding {
            id: "storage.free".into(),
            title: "Disk free space".to_string(),
            severity: Severity::Info,
            summary: "Disk free space looks healthy".into(),
            detail: "All monitored mounts above 15% free".into(),
            fix: None,
        }
    } else {
        let summary = format!("Low disk space on {} mount(s)", warnings.len());
        let detail = warnings.join("; ");
        let fix = FixPlan {
            summary: "Review large files and remove old caches".into(),
            dry_run_cmds: vec!["sudo du -sh /var/log/*".into()],
            apply_cmds: vec!["sudo journalctl --vacuum-time=14d".into()],
            undo_cmds: Vec::new(),
        };
        Finding {
            id: "storage.free".into(),
            title: "Disk free space".to_string(),
            severity: Severity::Action,
            summary,
            detail,
            fix: Some(fix),
        }
    }
}

async fn fs_trim_probe() -> Finding {
    let mut severity = Severity::Info;
    let mut summary = "FSTRIM service active".to_string();
    let mut detail = String::new();
    let mut fix = None;

    if which("systemctl").is_err() {
        detail = "systemctl not available".into();
    } else {
        let status = Command::new("systemctl")
            .args(["is-enabled", "fstrim.timer"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .unwrap_or_else(|_| std::process::ExitStatus::from_raw(1));
        if !status.success() {
            severity = Severity::Action;
            summary = "fstrim.timer disabled".into();
            detail = "SSD systems benefit from regular fstrim".into();
            fix = Some(FixPlan {
                summary: "Enable weekly fstrim".into(),
                dry_run_cmds: vec!["systemctl list-timers fstrim.timer".into()],
                apply_cmds: vec!["sudo systemctl enable --now fstrim.timer".into()],
                undo_cmds: vec!["sudo systemctl disable --now fstrim.timer".into()],
            });
        }
    }

    Finding {
        id: "storage.trim".into(),
        title: "Filesystem TRIM".to_string(),
        severity,
        summary,
        detail,
        fix,
    }
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
    let mut caches = Vec::new();
    if let Some(path) = pacman_cache() {
        caches.push(("pacman", path));
    }
    if let Some(path) = apt_cache() {
        caches.push(("apt", path));
    }
    if let Some(path) = dnf_cache() {
        caches.push(("dnf", path));
    }

    let mut warn_entries = Vec::new();
    let mut fixes = Vec::new();
    for (kind, path) in caches {
        let size = dir_size(&path).unwrap_or(0);
        if size > 2 * 1024 * 1024 * 1024 {
            warn_entries.push(format!(
                "{} cache {:.1} GiB",
                kind,
                size as f64 / 1_073_741_824.0
            ));
            fixes.push(cache_cleanup_plan(kind));
        }
    }

    if warn_entries.is_empty() {
        Finding {
            id: "packages.cache".into(),
            title: "Package cache size".to_string(),
            severity: Severity::Info,
            summary: "Package caches are within limits".into(),
            detail: "All monitored caches < 2 GiB".into(),
            fix: None,
        }
    } else {
        let fix = merge_plans(&fixes);
        Finding {
            id: "packages.cache".into(),
            title: "Package cache size".to_string(),
            severity: Severity::Warn,
            summary: format!("Large cache detected ({})", warn_entries.join(", ")),
            detail: "Reduce cache size to reclaim disk space".into(),
            fix,
        }
    }
}

async fn orphan_probe() -> Finding {
    let mut details = Vec::new();
    let mut fixes = Vec::new();

    if which("pacman").is_ok() {
        let mut cmd = Command::new("pacman");
        cmd.args(["-Qtdq"]);
        if let Ok(count) = count_lines(cmd).await {
            if count > 0 {
                details.push(format!("{} pacman orphans", count));
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
            if count > 0 {
                details.push(format!("{} dnf orphans", count));
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
            if count > 0 {
                details.push(format!("apt may remove {} packages", count));
                fixes.push(FixPlan {
                    summary: "Clean apt orphans".into(),
                    dry_run_cmds: vec!["apt-get -s autoremove".into()],
                    apply_cmds: vec!["sudo apt-get autoremove".into()],
                    undo_cmds: Vec::new(),
                });
            }
        }
    }

    if details.is_empty() {
        Finding {
            id: "packages.orphans".into(),
            title: "Orphan packages".to_string(),
            severity: Severity::Info,
            summary: "No package orphans detected".into(),
            detail: String::new(),
            fix: None,
        }
    } else {
        Finding {
            id: "packages.orphans".into(),
            title: "Orphan packages".to_string(),
            severity: Severity::Warn,
            summary: details.join(", "),
            detail: "Consider removing unused packages".into(),
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

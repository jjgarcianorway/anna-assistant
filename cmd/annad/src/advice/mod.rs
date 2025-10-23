pub mod fs;
pub mod types;

use crate::config::AdviceConfig;
use crate::persona::store::Store;
use crate::persona::types::{Persona, PersonaState};
use anyhow::{anyhow, Context, Result};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Duration as StdDuration;
use time::macros::format_description;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};
use tokio::time as tokio_time;
use tracing::{info, warn};
use types::{Advice, AdvicePlan};
use which::which;

const KIND_STORAGE_FREE: &str = "storage.free-space";

#[derive(Debug, Clone, Copy)]
enum PackageManager {
    Pacman,
    Apt,
    Dnf,
    Unknown,
}

#[derive(Debug, Clone)]
struct VolumeStatus {
    label: String,
    free_ratio: f64,
}

pub fn init(cfg: &AdviceConfig) -> Result<()> {
    if cfg.enabled {
        fs::ensure_dirs()?;
    }
    Ok(())
}

pub fn run_once(cfg: &AdviceConfig) -> Result<Option<Advice>> {
    if !cfg.enabled {
        return Ok(None);
    }
    let store = Store::new()?;
    let state = match store.read_current()? {
        Some(state) => state,
        None => return Ok(None),
    };
    evaluate(&state, cfg)
}

pub fn start_background(cfg: AdviceConfig) -> Result<()> {
    if !cfg.enabled {
        return Ok(());
    }
    run_once(&cfg)?;
    let interval = StdDuration::from_secs(cfg.check_interval_minutes.saturating_mul(60));
    let cfg_clone = cfg.clone();
    tokio::spawn(async move {
        let mut ticker = tokio_time::interval(interval.max(StdDuration::from_secs(300)));
        loop {
            ticker.tick().await;
            if let Err(err) = run_once(&cfg_clone) {
                warn!(target: "annad", "advice check failed: {err:?}");
            }
        }
    });
    Ok(())
}

fn evaluate(state: &PersonaState, cfg: &AdviceConfig) -> Result<Option<Advice>> {
    if state.persona != Persona::CasualMinimal {
        return Ok(None);
    }
    if let Some(advice) = maybe_disk_pressure(state, cfg)? {
        return Ok(Some(advice));
    }
    Ok(None)
}

fn maybe_disk_pressure(state: &PersonaState, cfg: &AdviceConfig) -> Result<Option<Advice>> {
    let threshold = cfg.disk_free_threshold as f64;
    let mut statuses = collect_volumes(threshold)?;
    if statuses.is_empty() {
        return Ok(None);
    }

    if within_cooldown(KIND_STORAGE_FREE, state.persona, cfg)? {
        return Ok(None);
    }

    statuses.sort_by(|a, b| {
        a.free_ratio
            .partial_cmp(&b.free_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let reason = format_reason(&statuses, threshold);
    let plan = build_cleanup_plan(detect_package_manager());
    let advice = Advice {
        id: advice_id(KIND_STORAGE_FREE),
        kind: KIND_STORAGE_FREE.to_string(),
        persona_hint: state.persona,
        reason,
        created_at: now_rfc3339(),
        plan,
    };
    fs::write_advice(&advice)?;
    info!(
        target: "annad",
        "advice created: {} persona={} reason={}",
        advice.id,
        advice.persona_hint.as_str(),
        advice.reason
    );
    Ok(Some(advice))
}

fn within_cooldown(kind: &str, persona: Persona, cfg: &AdviceConfig) -> Result<bool> {
    let window = Duration::hours(cfg.cooldown_hours as i64);
    if window.is_zero() {
        return Ok(false);
    }
    let now = OffsetDateTime::now_utc();
    let records = fs::read_all()?;
    for rec in records {
        if rec.kind != kind || rec.persona_hint != persona {
            continue;
        }
        if let Ok(created) = OffsetDateTime::parse(&rec.created_at, &Rfc3339) {
            if now - created < window {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn collect_volumes(threshold: f64) -> Result<Vec<VolumeStatus>> {
    let mut statuses = Vec::new();
    let root = Path::new("/");
    if let Some(ratio) = free_ratio(root)? {
        if ratio < threshold {
            statuses.push(VolumeStatus {
                label: "root".to_string(),
                free_ratio: ratio,
            });
        }
    }

    if let Some(home_path) = dirs::home_dir() {
        let home_meta = std::fs::metadata(&home_path).ok();
        let root_meta = std::fs::metadata(root).ok();
        let same_device = match (home_meta, root_meta) {
            (Some(h), Some(r)) => h.dev() == r.dev(),
            _ => false,
        };
        if same_device {
            if let Some(entry) = statuses.iter_mut().find(|s| s.label == "root") {
                entry.label = "root/home".to_string();
            }
        } else if let Some(ratio) = free_ratio(&home_path)? {
            if ratio < threshold {
                statuses.push(VolumeStatus {
                    label: "home".to_string(),
                    free_ratio: ratio,
                });
            }
        }
    }

    Ok(statuses)
}

pub fn create_quickscan_advice(id_suffix: &str, summary: &str, plan: AdvicePlan) -> Result<()> {
    let advice = Advice {
        id: advice_id(&format!("quickscan.{}", id_suffix)),
        kind: format!("system/quickscan/{}", id_suffix),
        persona_hint: Persona::Unknown,
        reason: format!("Quickscan: {}", summary),
        created_at: now_rfc3339(),
        plan,
    };
    fs::write_advice(&advice)?;
    Ok(())
}

fn free_ratio(path: &Path) -> Result<Option<f64>> {
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| anyhow!("invalid path"))?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        return Err(err).with_context(|| format!("statvfs {}", path.display()));
    }
    if stat.f_blocks == 0 {
        return Ok(None);
    }
    let total = (stat.f_frsize as f64) * (stat.f_blocks as f64);
    if total <= 0.0 {
        return Ok(None);
    }
    let avail = (stat.f_frsize as f64) * (stat.f_bavail as f64);
    let ratio = (avail / total).clamp(0.0, 1.0);
    Ok(Some(ratio))
}

fn build_cleanup_plan(manager: PackageManager) -> AdvicePlan {
    let mut cmds = Vec::new();
    cmds.push("sudo du -sh /var/cache/*".to_string());
    let pkg_cmd = match manager {
        PackageManager::Pacman => "sudo pacman -Qtdq".to_string(),
        PackageManager::Apt => "sudo apt-get -s autoremove".to_string(),
        PackageManager::Dnf => "sudo dnf repoquery --unneeded".to_string(),
        PackageManager::Unknown => "sudo du -sh ~/.cache".to_string(),
    };
    cmds.push(pkg_cmd);
    cmds.push("sudo find $HOME -type f -size +500M -print | head -n 20".to_string());
    AdvicePlan::dry_run_only(cmds)
}

fn detect_package_manager() -> PackageManager {
    if which("pacman").is_ok() {
        return PackageManager::Pacman;
    }
    if which("apt-get").is_ok() {
        return PackageManager::Apt;
    }
    if which("dnf").is_ok() {
        return PackageManager::Dnf;
    }
    PackageManager::Unknown
}

fn advice_id(kind: &str) -> String {
    let stamp = OffsetDateTime::now_utc()
        .format(&format_description!(
            "[year][month][day]T[hour][minute][second]Z"
        ))
        .unwrap_or_else(|_| "00000000T000000Z".to_string());
    format!("{}-{}", kind.replace('.', "-"), stamp)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn format_reason(entries: &[VolumeStatus], threshold: f64) -> String {
    let mut parts = Vec::new();
    for entry in entries {
        parts.push(format!(
            "{} {:.1}% free",
            entry.label,
            entry.free_ratio * 100.0
        ));
    }
    format!(
        "low disk space on {}; threshold {:.0}%",
        parts.join("; "),
        threshold * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_has_three_steps() {
        let plan = build_cleanup_plan(PackageManager::Unknown);
        assert_eq!(plan.dry_run_cmds.len(), 3);
        assert!(plan.apply_cmds.is_empty());
        assert!(plan.undo_cmds.is_empty());
    }

    #[test]
    fn reason_formatting() {
        let entries = vec![
            VolumeStatus {
                label: "root".into(),
                free_ratio: 0.12,
            },
            VolumeStatus {
                label: "home".into(),
                free_ratio: 0.10,
            },
        ];
        let reason = format_reason(&entries, 0.15);
        assert!(reason.contains("root 12.0% free"));
        assert!(reason.contains("home 10.0% free"));
    }
}

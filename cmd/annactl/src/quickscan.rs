use crate::ui::{self, Style, UiCfg};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const REPORT_ROOT: &str = "/var/lib/anna/reports";

#[derive(Debug, Deserialize)]
struct QuickscanReport {
    generated: String,
    ok: usize,
    warn: usize,
    action: usize,
    findings: Vec<Finding>,
}

#[derive(Debug, Deserialize)]
struct Finding {
    id: String,
    title: String,
    severity: Severity,
    summary: String,
    detail: String,
    #[serde(default)]
    fix: Option<FixPlan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Severity {
    Info,
    Warn,
    Action,
}

#[derive(Debug, Deserialize)]
struct FixPlan {
    summary: String,
    #[serde(default)]
    apply_cmds: Vec<String>,
    #[serde(default)]
    dry_run_cmds: Vec<String>,
    #[serde(default)]
    undo_cmds: Vec<String>,
}

pub struct QuickscanArgs {
    pub raw: bool,
}

pub fn run(args: QuickscanArgs, ui_cfg: &UiCfg, style: &Style) -> Result<()> {
    let before = latest_mtime();
    trigger_daemon()?;
    let (json_path, report) = wait_for_report(before, Duration::from_secs(30))?;

    if args.raw {
        let contents = fs::read_to_string(&json_path)
            .with_context(|| format!("read {}", json_path.display()))?;
        println!("{}", contents.trim());
        return Ok(());
    }

    render_report(&report, ui_cfg, style);
    Ok(())
}

fn trigger_daemon() -> Result<()> {
    let status = std::process::Command::new("systemctl")
        .args(["kill", "-s", "SIGUSR1", "annad"])
        .status()
        .context("invoke systemctl kill")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("systemctl kill annad failed"))
    }
}

fn wait_for_report(
    previous: Option<SystemTime>,
    timeout: Duration,
) -> Result<(PathBuf, QuickscanReport)> {
    let start = Instant::now();
    loop {
        if let Some(path) = find_latest_json()? {
            if is_newer(&path, previous)? {
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                let report: QuickscanReport =
                    serde_json::from_str(&contents).context("parse quickscan json")?;
                return Ok((path, report));
            }
        }

        if start.elapsed() > timeout {
            return Err(anyhow!("timed out waiting for quickscan report"));
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn find_latest_json() -> Result<Option<PathBuf>> {
    let mut latest: Option<(SystemTime, PathBuf)> = None;
    let root = Path::new(REPORT_ROOT);
    if !root.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let json = entry.path().join("quickscan.json");
            if json.exists() {
                let meta = json.metadata()?;
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                match &mut latest {
                    Some((best_time, _)) if modified <= *best_time => {}
                    _ => latest = Some((modified, json.clone())),
                }
            }
        }
    }
    Ok(latest.map(|(_, path)| path))
}

fn latest_mtime() -> Option<SystemTime> {
    find_latest_json()
        .ok()
        .flatten()
        .and_then(|path| fs::metadata(path).ok()?.modified().ok())
}

fn is_newer(path: &Path, previous: Option<SystemTime>) -> Result<bool> {
    let meta = path.metadata()?;
    if let Some(prev) = previous {
        Ok(meta.modified().unwrap_or(SystemTime::UNIX_EPOCH) > prev)
    } else {
        Ok(true)
    }
}

fn render_report(report: &QuickscanReport, ui_cfg: &UiCfg, style: &Style) {
    println!("{}", ui::head(style, "Quick Health Check"));
    println!(
        "{}",
        ui::kv(
            style,
            "Generated",
            &ui::fmt_local(&report.generated, ui_cfg)
        )
    );
    println!(
        "{}",
        ui::kv(
            style,
            "Summary",
            &format!(
                "ok {}  warn {}  action {}",
                report.ok, report.warn, report.action
            )
        )
    );
    println!();

    for finding in &report.findings {
        let title = match finding.severity {
            Severity::Info => ui::ok(style, &finding.title),
            Severity::Warn => ui::warn(style, &finding.title),
            Severity::Action => ui::err(style, &finding.title),
        };
        println!("{}", title);
        println!("{}", ui::kv(style, "ID", &finding.id));
        println!("{}", ui::bullet(style, &finding.summary));
        if !finding.detail.is_empty() {
            println!("{}", ui::bullet(style, &finding.detail));
        }
        if let Some(fix) = &finding.fix {
            println!("{}", ui::kv(style, "Action", &fix.summary));
            if !fix.dry_run_cmds.is_empty() {
                println!("{}", ui::kv(style, "Dry-run", ""));
                for cmd in &fix.dry_run_cmds {
                    println!("{}", ui::bullet(style, &format!("$ {}", cmd)));
                }
            }
            for cmd in &fix.apply_cmds {
                println!("{}", ui::bullet(style, &format!("$ {}", cmd)));
            }
            if !fix.undo_cmds.is_empty() {
                println!("{}", ui::kv(style, "Undo", ""));
                for cmd in &fix.undo_cmds {
                    println!("{}", ui::bullet(style, &format!("$ {}", cmd)));
                }
            }
        }
        println!();
    }
}

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::ui::{self, Style, UiCfg};

const ADVICE_DIR: &str = "/var/lib/anna/advice";

#[derive(Debug, Clone)]
pub struct AdviceCliError {
    code: i32,
    message: String,
}

impl AdviceCliError {
    pub fn usage(msg: impl Into<String>) -> Self {
        Self {
            code: 64,
            message: msg.into(),
        }
    }

    pub fn io(context: impl Into<String>, err: io::Error) -> Self {
        Self {
            code: 1,
            message: format!("{}: {}", context.into(), err),
        }
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: msg.into(),
        }
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}

impl std::fmt::Display for AdviceCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AdviceCliError {}

pub type Result<T> = std::result::Result<T, AdviceCliError>;

#[derive(Debug, Deserialize, Serialize)]
struct AdvicePlan {
    #[serde(default)]
    dry_run_cmds: Vec<String>,
    #[serde(default)]
    apply_cmds: Vec<String>,
    #[serde(default)]
    undo_cmds: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AdviceRecord {
    id: String,
    kind: String,
    persona_hint: String,
    reason: String,
    created_at: String,
    plan: AdvicePlan,
}

#[derive(Debug, Clone, Copy)]
pub enum AdviceCommand {
    List,
    Show,
    Apply,
}

pub struct AdviceArgs<'a> {
    pub command: AdviceCommand,
    pub id: Option<&'a str>,
    pub dry_run: bool,
    pub raw: bool,
}

pub fn run(args: AdviceArgs<'_>, cfg: &UiCfg, style: &Style) -> Result<()> {
    match args.command {
        AdviceCommand::List => list(cfg, style, args.raw),
        AdviceCommand::Show => {
            let id = args
                .id
                .ok_or_else(|| AdviceCliError::usage("advice show <id>"))?;
            show(id, cfg, style, args.raw)
        }
        AdviceCommand::Apply => {
            let id = args
                .id
                .ok_or_else(|| AdviceCliError::usage("advice apply <id> --dry-run"))?;
            apply(id, args.dry_run, cfg, style, args.raw)
        }
    }
}

fn list(cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    let entries = read_all()?;
    if entries.is_empty() {
        println!(
            "{}",
            ui::warn(style, "no advice entries (run annad to generate)")
        );
        return Ok(());
    }

    if raw {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".into())
        );
        return Ok(());
    }

    println!("{}", ui::head(style, "Advice entries"));
    for entry in entries {
        let created_local = ui::fmt_local(&entry.created_at, cfg);
        println!("{}", ui::kv(style, "ID", &entry.id));
        println!("{}", ui::kv(style, "Persona", &entry.persona_hint));
        println!("{}", ui::kv(style, "Kind", &entry.kind));
        println!("{}", ui::kv(style, "Created", &created_local));
        println!("{}", ui::kv(style, "Reason", &entry.reason));
        println!();
    }
    Ok(())
}

fn show(id: &str, cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    let record = read_one(id)?;
    if raw {
        println!(
            "{}",
            serde_json::to_string_pretty(&record).unwrap_or_else(|_| "{}".into())
        );
        return Ok(());
    }

    let created_local = ui::fmt_local(&record.created_at, cfg);
    println!("{}", ui::head(style, &format!("Advice {}", record.id)));
    println!("{}", ui::kv(style, "Persona", &record.persona_hint));
    println!("{}", ui::kv(style, "Kind", &record.kind));
    println!("{}", ui::kv(style, "Created", &created_local));
    println!("{}", ui::kv(style, "Reason", &record.reason));

    if !record.plan.dry_run_cmds.is_empty() {
        println!("{}", ui::head(style, "Dry-run commands"));
        for cmd in &record.plan.dry_run_cmds {
            println!("{}", ui::bullet(style, cmd));
        }
    }
    if !record.plan.apply_cmds.is_empty() {
        println!("{}", ui::head(style, "Apply commands"));
        for cmd in &record.plan.apply_cmds {
            println!("{}", ui::bullet(style, cmd));
        }
    }
    if !record.plan.undo_cmds.is_empty() {
        println!("{}", ui::head(style, "Undo commands"));
        for cmd in &record.plan.undo_cmds {
            println!("{}", ui::bullet(style, cmd));
        }
    }
    Ok(())
}

fn apply(id: &str, dry_run: bool, cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    if !dry_run {
        return Err(AdviceCliError::usage(
            "--dry-run is required; automation not enabled yet",
        ));
    }
    let record = read_one(id)?;
    if raw {
        println!(
            "{}",
            serde_json::to_string_pretty(&record).unwrap_or_else(|_| "{}".into())
        );
        return Ok(());
    }

    println!("{}", ui::head(style, &format!("Preview: {}", record.id)));
    println!("{}", ui::kv(style, "Kind", &record.kind));
    println!("{}", ui::kv(style, "Reason", &record.reason));
    let created_local = ui::fmt_local(&record.created_at, cfg);
    println!("{}", ui::kv(style, "Created", &created_local));
    if record.plan.dry_run_cmds.is_empty() {
        println!("{}", ui::warn(style, "no dry-run commands recorded"));
        return Ok(());
    }
    println!("{}", ui::head(style, "Commands to preview"));
    for cmd in &record.plan.dry_run_cmds {
        println!("{}", ui::bullet(style, cmd));
    }
    Ok(())
}

fn read_all() -> Result<Vec<AdviceRecord>> {
    let dir = Path::new(ADVICE_DIR);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| AdviceCliError::io("read advice dir", e))? {
        let entry = entry.map_err(|e| AdviceCliError::io("read advice entry", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match fs::read(&path) {
            Ok(data) => match serde_json::from_slice::<AdviceRecord>(&data) {
                Ok(rec) => records.push(rec),
                Err(err) => {
                    return Err(AdviceCliError::other(format!(
                        "parse {}: {}",
                        path.display(),
                        err
                    )));
                }
            },
            Err(err) => return Err(AdviceCliError::io("read advice", err)),
        }
    }
    records.sort_by(
        |a, b| match parse_ts(&b.created_at).cmp(&parse_ts(&a.created_at)) {
            Ordering::Equal => b.id.cmp(&a.id),
            other => other,
        },
    );
    Ok(records)
}

fn read_one(id: &str) -> Result<AdviceRecord> {
    let path = normalize_id(id);
    let data = fs::read(&path).map_err(|e| AdviceCliError::io("read advice", e))?;
    serde_json::from_slice::<AdviceRecord>(&data)
        .map_err(|e| AdviceCliError::other(format!("parse {}: {}", path.display(), e)))
}

fn normalize_id(id: &str) -> PathBuf {
    let mut path = Path::new(ADVICE_DIR).join(id);
    if path.extension().is_none() {
        path = path.with_extension("json");
    }
    path
}

fn parse_ts(ts: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(ts, &Rfc3339).ok()
}

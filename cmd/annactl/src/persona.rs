use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use time::{format_description::well_known::Rfc3339, Date, Month, OffsetDateTime};

use crate::ui::{self, Style, UiCfg};

const PERSONA_DIR: &str = "/var/lib/anna/persona";
const SAMPLES_DIR: &str = "/var/lib/anna/persona/samples";
const ROLLUPS_DIR: &str = "/var/lib/anna/persona/rollups";
const CURRENT_PATH: &str = "/var/lib/anna/persona/current.json";
const OVERRIDE_PATH: &str = "/etc/anna/persona_override";
const TRIGGER_PATH: &str = "/var/lib/anna/persona/last_trigger.json";

const VALID_PERSONAS: &[&str] = &[
    "admin-pragmatic",
    "dev-enthusiast",
    "power-nerd",
    "casual-minimal",
    "creator-writer",
    "unknown",
];

#[derive(Debug, Clone)]
pub struct PersonaCliError {
    code: i32,
    message: String,
}

impl PersonaCliError {
    pub fn usage(msg: impl Into<String>) -> Self {
        Self {
            code: 64,
            message: msg.into(),
        }
    }

    pub fn io(context: impl Into<String>, err: std::io::Error) -> Self {
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

impl std::fmt::Display for PersonaCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PersonaCliError {}

pub type Result<T> = std::result::Result<T, PersonaCliError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonaState {
    persona: String,
    confidence: f32,
    updated: String,
    source: String,
    #[serde(default)]
    explanations: Vec<String>,
    #[serde(default)]
    window_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TriggerSnapshot {
    time: String,
    pkg_churn: u32,
    shell_lines: u32,
    browser_navs: u32,
    #[serde(default)]
    debounced: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PersonaCommand {
    Show,
    Set,
    Unset,
    Samples,
    Rollup,
    Explain,
    Triggers,
}

pub struct PersonaArgs<'a> {
    pub command: PersonaCommand,
    pub name: Option<&'a str>,
    pub date: Option<&'a str>,
    pub tail: Option<usize>,
    pub raw: bool,
}

pub fn run(args: PersonaArgs<'_>, cfg: &UiCfg, style: &Style) -> Result<()> {
    match args.command {
        PersonaCommand::Show => show(cfg, style, args.raw),
        PersonaCommand::Set => {
            let name = args
                .name
                .ok_or_else(|| PersonaCliError::usage("persona set requires a name"))?;
            set(name, style)
        }
        PersonaCommand::Unset => unset(style),
        PersonaCommand::Samples => {
            let date = args
                .date
                .ok_or_else(|| PersonaCliError::usage("persona samples --date YYYY-MM-DD"))?;
            let tail = args.tail.unwrap_or(0);
            samples(date, tail)
        }
        PersonaCommand::Rollup => {
            let date = args
                .date
                .ok_or_else(|| PersonaCliError::usage("persona rollup --date YYYY-MM-DD"))?;
            rollup(date)
        }
        PersonaCommand::Explain => explain(cfg, style, args.raw),
        PersonaCommand::Triggers => triggers(cfg, style, args.raw),
    }
}

fn show(cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    match read_current()? {
        Some(state) => {
            if raw {
                let data = fs::read_to_string(CURRENT_PATH)
                    .map_err(|e| PersonaCliError::io("read current persona", e))?;
                println!("{}", data.trim());
                return Ok(());
            }
            print_persona_summary(&state, cfg, style, true);
        }
        None => {
            println!(
                "{}",
                ui::warn(style, "current persona not set (start annad to initialize)")
            );
            return Ok(());
        }
    }

    if let Some(name) = read_override()? {
        println!("{}", ui::kv(style, "Override", &name));
    }

    Ok(())
}

fn print_persona_summary(state: &PersonaState, cfg: &UiCfg, style: &Style, show_source: bool) {
    let header = format!(
        "Persona: {}  •  Confidence: {:.2}  •  Window: {} day(s)",
        state.persona, state.confidence, state.window_days
    );
    println!("{}", header);
    let updated_local = ui::fmt_local(&state.updated, cfg);
    println!("{}", ui::kv(style, "Updated", &updated_local));
    if show_source {
        println!("{}", ui::kv(style, "Source", state.source.as_str()));
    }
    if !state.explanations.is_empty() {
        for item in &state.explanations {
            println!("{}", ui::bullet(style, item));
        }
    } else {
        println!("{}", ui::warn(style, "no supporting signals recorded"));
    }
}

fn set(name: &str, style: &Style) -> Result<()> {
    validate_persona(name)?;
    ensure_persona_dir()?;

    write_override(name)?;

    let state = PersonaState {
        persona: name.to_string(),
        confidence: 1.0,
        updated: now_rfc3339(),
        source: "override".into(),
        explanations: vec![format!("manual override set to {}", name)],
        window_days: 0,
    };
    write_current(&state)?;
    println!(
        "{}",
        ui::ok(style, &format!("Persona override set to {name}"))
    );
    Ok(())
}

fn unset(style: &Style) -> Result<()> {
    remove_override()?;
    ensure_persona_dir()?;
    let state = PersonaState {
        persona: "unknown".into(),
        confidence: 0.0,
        updated: now_rfc3339(),
        source: "default".into(),
        explanations: vec!["manual override cleared".into()],
        window_days: 0,
    };
    write_current(&state)?;
    println!("{}", ui::ok(style, "Persona override removed"));
    Ok(())
}

fn samples(date: &str, tail: usize) -> Result<()> {
    let date = parse_date(date)?;
    let date_str = date.to_string();
    let path = Path::new(SAMPLES_DIR).join(format!("{date_str}.ndjson"));
    if !path.exists() {
        return Err(PersonaCliError::other(format!(
            "no samples for {date_str} (run annad and try later)"
        )));
    }

    if tail == 0 {
        let data = fs::read_to_string(&path).map_err(|e| PersonaCliError::io("read samples", e))?;
        print!("{}", data);
        return Ok(());
    }

    let file = File::open(&path).map_err(|e| PersonaCliError::io("open samples", e))?;
    let reader = BufReader::new(file);
    let mut buf = VecDeque::with_capacity(tail.max(1));
    for line in reader.lines() {
        let line = line.map_err(|e| PersonaCliError::io("read samples", e))?;
        buf.push_back(line);
        if buf.len() > tail {
            buf.pop_front();
        }
    }
    for line in buf {
        println!("{}", line);
    }
    Ok(())
}

fn rollup(date: &str) -> Result<()> {
    let date = parse_date(date)?;
    let date_str = date.to_string();
    let path = Path::new(ROLLUPS_DIR).join(format!("{date_str}.json"));
    if !path.exists() {
        return Err(PersonaCliError::other(
            "no rollup yet; run tomorrow or restart annad to force catch-up",
        ));
    }
    let data = fs::read_to_string(&path).map_err(|e| PersonaCliError::io("read rollup", e))?;
    let value: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| PersonaCliError::other(format!("parse rollup: {}", e)))?;
    println!("{}", serde_json::to_string_pretty(&value).unwrap_or(data));
    Ok(())
}

fn explain(cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    if raw {
        let data = fs::read_to_string(CURRENT_PATH)
            .map_err(|e| PersonaCliError::io("read current persona", e))?;
        println!("{}", data.trim());
        return Ok(());
    }
    let state = read_current()?.ok_or_else(|| PersonaCliError::other("current persona not set"))?;
    print_persona_summary(&state, cfg, style, false);
    Ok(())
}

fn triggers(cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    if raw {
        match fs::read_to_string(TRIGGER_PATH) {
            Ok(data) => {
                println!("{}", data.trim());
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("{}", ui::warn(style, "no trigger recorded yet"));
                return Ok(());
            }
            Err(e) => return Err(PersonaCliError::io("read last trigger", e)),
        }
    }

    match read_last_trigger()? {
        Some(snapshot) => {
            let header = ui::head(style, "Last Persona Trigger");
            println!("{}", header);
            let when = ui::fmt_local(&snapshot.time, cfg);
            println!("{}", ui::kv(style, "When", &when));
            println!(
                "{}",
                ui::kv(
                    style,
                    "Counts",
                    &format!(
                        "pkg={} shell={} browser={}",
                        snapshot.pkg_churn, snapshot.shell_lines, snapshot.browser_navs
                    )
                )
            );
            println!(
                "{}",
                ui::kv(
                    style,
                    "Debounced",
                    if snapshot.debounced { "yes" } else { "no" }
                )
            );
        }
        None => println!("{}", ui::warn(style, "no trigger recorded yet")),
    }
    Ok(())
}

fn validate_persona(name: &str) -> Result<()> {
    if VALID_PERSONAS.contains(&name) {
        Ok(())
    } else {
        Err(PersonaCliError::usage(format!(
            "invalid persona '{}'. allowed: {}",
            name,
            VALID_PERSONAS.join(", ")
        )))
    }
}

fn ensure_persona_dir() -> Result<()> {
    let dir = Path::new(PERSONA_DIR);
    if let Err(e) = fs::create_dir_all(dir) {
        return Err(PersonaCliError::io("create persona dir", e));
    }
    if let Err(e) = fs::set_permissions(dir, fs::Permissions::from_mode(0o700)) {
        return Err(PersonaCliError::io("set persona dir permissions", e));
    }
    Ok(())
}

fn read_current() -> Result<Option<PersonaState>> {
    match fs::read_to_string(CURRENT_PATH) {
        Ok(data) => serde_json::from_str(&data)
            .map(Some)
            .map_err(|e| PersonaCliError::other(format!("parse current persona: {}", e))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(PersonaCliError::io("read current persona", e)),
    }
}

fn write_current(state: &PersonaState) -> Result<()> {
    let payload = serde_json::to_vec_pretty(state)
        .map_err(|e| PersonaCliError::other(format!("encode persona state: {}", e)))?;
    write_atomic(Path::new(CURRENT_PATH), &payload)
}

fn read_override() -> Result<Option<String>> {
    match fs::read_to_string(OVERRIDE_PATH) {
        Ok(data) => {
            let value = data.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_string()))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(PersonaCliError::io("read persona override", e)),
    }
}

fn read_last_trigger() -> Result<Option<TriggerSnapshot>> {
    match fs::read_to_string(TRIGGER_PATH) {
        Ok(data) => serde_json::from_str(&data)
            .map(Some)
            .map_err(|e| PersonaCliError::other(format!("parse trigger snapshot: {}", e))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(PersonaCliError::io("read last trigger", e)),
    }
}

fn write_override(name: &str) -> Result<()> {
    let data = format!("{}\n", name);
    write_atomic(Path::new(OVERRIDE_PATH), data.as_bytes())
}

fn remove_override() -> Result<()> {
    match fs::remove_file(OVERRIDE_PATH) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(PersonaCliError::io("remove persona override", e)),
    }
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(PersonaCliError::io("create directories", e));
        }
    }
    let tmp = tmp_path(path);
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|e| PersonaCliError::io("open temp file", e))?;
        file.write_all(data)
            .map_err(|e| PersonaCliError::io("write temp file", e))?;
        file.sync_all()
            .map_err(|e| PersonaCliError::io("sync temp file", e))?;
    }
    if let Some(parent) = path.parent() {
        let dir = File::open(parent).map_err(|e| PersonaCliError::io("open dir", e))?;
        dir.sync_all()
            .map_err(|e| PersonaCliError::io("sync dir", e))?;
    }
    fs::rename(&tmp, path).map_err(|e| PersonaCliError::io("rename temp file", e))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|e| PersonaCliError::io("set permissions", e))?;
    Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let fname = path
        .file_name()
        .and_then(|f| f.to_str())
        .map(|name| format!("{}.tmp", name))
        .unwrap_or_else(|| String::from("tmp"));
    tmp.set_file_name(fname);
    tmp
}

fn parse_date(text: &str) -> Result<Date> {
    let mut parts = text.split('-');
    let year = parts
        .next()
        .and_then(|s| s.parse::<i32>().ok())
        .ok_or_else(|| PersonaCliError::usage("date must be YYYY-MM-DD"))?;
    let month_num = parts
        .next()
        .and_then(|s| s.parse::<u8>().ok())
        .ok_or_else(|| PersonaCliError::usage("date must be YYYY-MM-DD"))?;
    let day = parts
        .next()
        .and_then(|s| s.parse::<u8>().ok())
        .ok_or_else(|| PersonaCliError::usage("date must be YYYY-MM-DD"))?;
    if parts.next().is_some() {
        return Err(PersonaCliError::usage("date must be YYYY-MM-DD"));
    }
    let month = Month::try_from(month_num)
        .map_err(|_| PersonaCliError::usage("date must be YYYY-MM-DD"))?;
    Date::from_calendar_date(year, month, day)
        .map_err(|_| PersonaCliError::usage("date must be YYYY-MM-DD"))
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

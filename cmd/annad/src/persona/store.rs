use crate::persona::fs;
use crate::persona::types::{Persona, PersonaSource, PersonaState};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs as stdfs;
use std::path::PathBuf;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const CURRENT_PATH: &str = "/var/lib/anna/persona/current.json";
const OVERRIDE_PATH: &str = "/etc/anna/persona_override";
const LAST_TRIGGER_PATH: &str = "/var/lib/anna/persona/last_trigger.json";

#[derive(Clone)]
pub struct Store {
    current_path: PathBuf,
    override_path: PathBuf,
    trigger_path: PathBuf,
}

impl Store {
    pub fn new() -> Result<Self> {
        fs::ensure_dirs()?;
        Ok(Self {
            current_path: PathBuf::from(CURRENT_PATH),
            override_path: PathBuf::from(OVERRIDE_PATH),
            trigger_path: PathBuf::from(LAST_TRIGGER_PATH),
        })
    }

    pub fn ensure_current_exists(&self) -> Result<PersonaState> {
        match self.read_current()? {
            Some(state) => Ok(state),
            None => {
                let state = create_state(Persona::Unknown, 0.0, PersonaSource::Default);
                self.write_current(&state)?;
                Ok(state)
            }
        }
    }

    pub fn read_current(&self) -> Result<Option<PersonaState>> {
        match stdfs::read_to_string(&self.current_path) {
            Ok(data) => {
                let state: PersonaState =
                    serde_json::from_str(&data).context("parse current persona")?;
                Ok(Some(state))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context("read current persona"),
        }
    }

    pub fn write_current(&self, state: &PersonaState) -> Result<()> {
        let payload = serde_json::to_vec_pretty(state)?;
        fs::write_atomic(&self.current_path, &payload)
    }

    pub fn write_last_trigger(&self, snapshot: &TriggerSnapshot) -> Result<()> {
        let payload = serde_json::to_vec_pretty(snapshot)?;
        fs::write_atomic(&self.trigger_path, &payload)
    }

    pub fn read_last_trigger(&self) -> Result<Option<TriggerSnapshot>> {
        match stdfs::read_to_string(&self.trigger_path) {
            Ok(data) => {
                let snapshot: TriggerSnapshot =
                    serde_json::from_str(&data).context("parse last trigger snapshot")?;
                Ok(Some(snapshot))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context("read last trigger snapshot"),
        }
    }

    pub fn read_override(&self) -> Result<Option<Persona>> {
        match stdfs::read_to_string(&self.override_path) {
            Ok(data) => {
                let value = data.trim();
                if value.is_empty() {
                    return Ok(None);
                }
                match value.parse::<Persona>() {
                    Ok(persona) => Ok(Some(persona)),
                    Err(_) => Ok(None),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context("read persona override"),
        }
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn create_state(persona: Persona, confidence: f32, source: PersonaSource) -> PersonaState {
    PersonaState {
        persona,
        confidence,
        updated: now_rfc3339(),
        source,
        explanations: Vec::new(),
        window_days: 0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSnapshot {
    pub time: String,
    pub pkg_churn: u32,
    pub shell_lines: u32,
    pub browser_navs: u32,
    pub debounced: bool,
}

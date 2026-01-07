use crate::persona::fs;
use crate::persona::types::{Persona, PersonaSource, PersonaState};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs as stdfs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

static PERSONA_PATHS: OnceLock<PersonaPaths> = OnceLock::new();

struct PersonaPaths {
    current: PathBuf,
    last_trigger: PathBuf,
}

pub fn set_persona_paths(persona_dir: &PathBuf, _config_dir: &PathBuf) {
    PERSONA_PATHS
        .set(PersonaPaths {
            current: persona_dir.join("current.json"),
            last_trigger: persona_dir.join("last_trigger.json"),
        })
        .ok();
}

fn persona_paths() -> &'static PersonaPaths {
    PERSONA_PATHS.get().expect("persona paths not initialized")
}

#[derive(Clone)]
pub struct Store {
    current_path: PathBuf,
    override_path: PathBuf,
    trigger_path: PathBuf,
    persona_dir: PathBuf,
}

impl Store {
    /// Create a Store using the global persona paths (for backward compatibility)
    pub fn new() -> Result<Self> {
        fs::ensure_dirs()?;
        let paths = persona_paths();
        Ok(Self {
            current_path: paths.current.clone(),
            override_path: PathBuf::from("/etc/anna/persona_override"), // Config override - kept as-is
            trigger_path: paths.last_trigger.clone(),
            persona_dir: persona_paths().current.parent().unwrap().to_path_buf(),
        })
    }

    /// Create a Store for a specific persona directory (per-user)
    pub fn for_dir(persona_dir: &Path) -> Result<Self> {
        // Ensure persona directory exists with proper permissions
        ensure_persona_dir(persona_dir)?;

        Ok(Self {
            current_path: persona_dir.join("current.json"),
            override_path: PathBuf::from("/etc/anna/persona_override"), // Config override - global
            trigger_path: persona_dir.join("last_trigger.json"),
            persona_dir: persona_dir.to_path_buf(),
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
        // Ensure parent directory exists with proper permissions
        if let Some(parent) = self.current_path.parent() {
            ensure_persona_dir(parent)?;
        }
        let payload = serde_json::to_vec_pretty(state)?;
        fs::write_atomic(&self.current_path, &payload)
    }

    pub fn write_last_trigger(&self, snapshot: &TriggerSnapshot) -> Result<()> {
        // Ensure parent directory exists with proper permissions
        if let Some(parent) = self.trigger_path.parent() {
            ensure_persona_dir(parent)?;
        }
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

/// Ensure persona directory exists with proper permissions (0700)
fn ensure_persona_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        stdfs::create_dir_all(dir).with_context(|| format!("create persona dir {}", dir.display()))?;
    }

    // Set permissions to 0700 (owner only)
    let perms = stdfs::Permissions::from_mode(0o700);
    stdfs::set_permissions(dir, perms)
        .with_context(|| format!("set persona dir permissions {}", dir.display()))?;

    Ok(())
}

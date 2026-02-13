//! Unified registry of everything Anna has created or deployed.
//!
//! Tracks services, timers, scripts, wallpaper automations, user accounts,
//! and audit reports. Persisted to /var/lib/anna/registry.json.
//!
//! All created systemd units are prefixed `anna-` for easy identification.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use anna_shared::config::anna_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactKind {
    SystemdService,
    SystemdTimer,
    Script,
    WallpaperTimer,
    UserAccount,
    SshAuditReport,
    KernelConfig,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactKind::SystemdService => write!(f, "systemd service"),
            ArtifactKind::SystemdTimer => write!(f, "systemd timer"),
            ArtifactKind::Script => write!(f, "script"),
            ArtifactKind::WallpaperTimer => write!(f, "wallpaper timer"),
            ArtifactKind::UserAccount => write!(f, "user account"),
            ArtifactKind::SshAuditReport => write!(f, "SSH audit"),
            ArtifactKind::KernelConfig => write!(f, "kernel config"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactStatus {
    Active,
    Paused,
    Removed,
}

impl std::fmt::Display for ArtifactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactStatus::Active => write!(f, "Active"),
            ArtifactStatus::Paused => write!(f, "Paused"),
            ArtifactStatus::Removed => write!(f, "Removed"),
        }
    }
}

/// A single artifact Anna has created or deployed on this system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedArtifact {
    /// Unique ID (UUID-style slug)
    pub id: String,
    pub kind: ArtifactKind,
    /// Human-readable name (e.g., "downloads cleanup timer")
    pub name: String,
    /// One-sentence description of what it does
    pub description: String,
    /// All file paths created or modified
    pub paths: Vec<String>,
    /// Unix timestamp of creation
    pub created_at: u64,
    pub status: ArtifactStatus,
    /// Shell commands that fully remove this artifact
    pub remove_cmds: Vec<String>,
    /// Optional: systemd unit name for timers/services
    pub unit_name: Option<String>,
}

impl CreatedArtifact {
    pub fn new(
        kind: ArtifactKind,
        name: impl Into<String>,
        description: impl Into<String>,
        paths: Vec<String>,
        remove_cmds: Vec<String>,
    ) -> Self {
        let now = unix_now();
        // Generate a simple slug-based ID
        let name_str: String = name.into();
        let id = format!("{}-{}", slug(&name_str), now % 100000);
        Self {
            id,
            kind,
            name: name_str,
            description: description.into(),
            paths,
            created_at: now,
            status: ArtifactStatus::Active,
            remove_cmds,
            unit_name: None,
        }
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit_name = Some(unit.into());
        self
    }

    pub fn age_days(&self) -> u64 {
        (unix_now() - self.created_at) / 86400
    }
}

/// Persistent store of all artifacts Anna has created.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactRegistry {
    pub artifacts: Vec<CreatedArtifact>,
}

impl ArtifactRegistry {
    fn storage_path() -> PathBuf {
        anna_data_dir().join("registry.json")
    }

    pub fn load() -> Self {
        let path = Self::storage_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(s) => {
                if let Err(e) = std::fs::write(&path, s) {
                    warn!("Failed to save artifact registry: {}", e);
                }
            }
            Err(e) => warn!("Failed to serialize artifact registry: {}", e),
        }
    }

    /// Add an artifact and persist.
    pub fn add(&mut self, artifact: CreatedArtifact) {
        info!("Registering artifact: {} ({})", artifact.name, artifact.kind);
        self.artifacts.push(artifact);
        self.save();
    }

    /// Mark an artifact as removed by name or ID (fuzzy match).
    /// Returns the removal commands to run.
    pub fn remove_by_name(&mut self, query: &str) -> Option<(CreatedArtifact, Vec<String>)> {
        let query_lower = query.to_lowercase();
        let idx = self.artifacts.iter().position(|a| {
            a.status == ArtifactStatus::Active
                && (a.name.to_lowercase().contains(&query_lower)
                    || a.id.to_lowercase().contains(&query_lower)
                    || a.description.to_lowercase().contains(&query_lower))
        })?;

        let cmds = self.artifacts[idx].remove_cmds.clone();
        self.artifacts[idx].status = ArtifactStatus::Removed;
        let artifact = self.artifacts[idx].clone();
        self.save();
        info!("Marked artifact as removed: {}", artifact.name);
        Some((artifact, cmds))
    }

    /// All active artifacts.
    pub fn list_active(&self) -> Vec<&CreatedArtifact> {
        self.artifacts
            .iter()
            .filter(|a| a.status == ArtifactStatus::Active)
            .collect()
    }

    /// Whether Anna has created anything yet.
    pub fn is_empty(&self) -> bool {
        self.list_active().is_empty()
    }

    /// Format active artifacts for morning briefing injection.
    pub fn summary_for_briefing(&self) -> String {
        let active = self.list_active();
        if active.is_empty() {
            return String::new();
        }
        let mut out = "## Anna's Active Automations\n".to_string();
        for a in active.iter().take(5) {
            out.push_str(&format!("- [{}] {} — {}\n", a.kind, a.name, a.description));
        }
        if active.len() > 5 {
            out.push_str(&format!("  ... and {} more\n", active.len() - 5));
        }
        out
    }

    /// Format active artifacts for user-facing listing.
    pub fn format_for_user(&self) -> String {
        let active = self.list_active();
        if active.is_empty() {
            return "I haven't created any automations or artifacts yet. Ask me to set something up!".to_string();
        }
        let mut out = format!("I have {} active automation{} running on your system:\n\n",
            active.len(), if active.len() == 1 { "" } else { "s" });
        for (i, a) in active.iter().enumerate() {
            out.push_str(&format!(
                "{}. [{}] {}\n   {}\n   Created {} day{} ago\n",
                i + 1,
                a.kind,
                a.name,
                a.description,
                a.age_days(),
                if a.age_days() == 1 { "" } else { "s" },
            ));
            if let Some(ref unit) = a.unit_name {
                out.push_str(&format!("   Unit: {}\n", unit));
            }
            out.push_str(&format!(
                "   To remove: ask me to \"remove {}\"\n\n",
                a.name
            ));
        }
        out
    }
}

fn slug(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_roundtrip() {
        let mut reg = ArtifactRegistry::default();
        let art = CreatedArtifact::new(
            ArtifactKind::SystemdTimer,
            "test cleanup",
            "Deletes old test files daily",
            vec!["/etc/systemd/system/anna-test-cleanup.timer".into()],
            vec!["systemctl disable --now anna-test-cleanup.timer".into()],
        );
        reg.add(art);
        assert_eq!(reg.list_active().len(), 1);
    }

    #[test]
    fn test_remove_by_name() {
        let mut reg = ArtifactRegistry::default();
        reg.artifacts.push(CreatedArtifact::new(
            ArtifactKind::SystemdTimer,
            "downloads cleanup",
            "Cleans downloads",
            vec![],
            vec!["systemctl disable --now anna-downloads-cleanup.timer".into()],
        ));
        let result = reg.remove_by_name("downloads");
        assert!(result.is_some());
        assert!(reg.list_active().is_empty());
    }

    #[test]
    fn test_format_for_user_empty() {
        let reg = ArtifactRegistry::default();
        let out = reg.format_for_user();
        assert!(out.contains("haven't created"));
    }
}

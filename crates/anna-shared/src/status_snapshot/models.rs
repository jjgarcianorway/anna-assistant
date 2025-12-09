//! Models subsystem types (v0.0.211).

use crate::specialists::SpecialistRole;
use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// Model download status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadStatus {
    pub name: String,
    pub downloading: bool,
    pub progress_pct: Option<f32>,
    pub error: Option<String>,
}

/// Role-model binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleModelBinding {
    pub team: Team,
    pub role: SpecialistRole,
    pub model_name: String,
    pub model_present: bool,
}

/// Models subsystem information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelsInfo {
    /// Ollama binary present
    pub ollama_present: bool,
    /// Ollama service running
    pub ollama_running: bool,
    /// Ollama version if available
    pub ollama_version: Option<String>,
    /// Role-model bindings
    pub roles: Vec<RoleModelBinding>,
    /// Active downloads
    pub downloads: Vec<ModelDownloadStatus>,
}

impl ModelsInfo {
    pub fn is_ready(&self) -> bool {
        self.ollama_present && self.ollama_running && self.roles.iter().all(|r| r.model_present)
    }

    pub fn missing_models(&self) -> Vec<&str> {
        self.roles
            .iter()
            .filter(|r| !r.model_present)
            .map(|r| r.model_name.as_str())
            .collect()
    }
}

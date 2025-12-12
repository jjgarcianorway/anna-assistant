//! Model health and verification (v0.0.434).
//!
//! Tracks model installation status, verifies models work, and manages lifecycle.

use super::model_plan::ModelPlan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Model installation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model is installed and verified working.
    Ok,
    /// Model is installed but not yet verified.
    Unverified,
    /// Model is not installed.
    Missing,
    /// Model is installed but verification failed.
    Broken,
    /// Model is being installed.
    Installing,
}

impl ModelStatus {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Unverified => "UNVERIFIED",
            Self::Missing => "MISSING",
            Self::Broken => "BROKEN",
            Self::Installing => "INSTALLING",
        }
    }

    /// Whether the model is usable.
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Ok | Self::Unverified)
    }
}

/// Who installed the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstalledBy {
    /// Anna installed this model.
    Anna,
    /// User installed this model.
    User,
    /// Unknown (pre-existing).
    Unknown,
}

impl InstalledBy {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Anna => "anna",
            Self::User => "user",
            Self::Unknown => "unknown",
        }
    }
}

/// Health record for a single model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealthRecord {
    /// Model name.
    pub name: String,
    /// Current status.
    pub status: ModelStatus,
    /// Who installed it.
    pub installed_by: InstalledBy,
    /// When first detected/installed.
    pub installed_at: Option<String>,
    /// Last verification time.
    pub last_verified: Option<String>,
    /// Last verification result message.
    pub last_verify_message: Option<String>,
    /// Number of successful uses.
    pub use_count: u64,
    /// Last error message (if broken).
    pub last_error: Option<String>,
}

impl ModelHealthRecord {
    /// Create a new record for a missing model.
    pub fn missing(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ModelStatus::Missing,
            installed_by: InstalledBy::Unknown,
            installed_at: None,
            last_verified: None,
            last_verify_message: None,
            use_count: 0,
            last_error: None,
        }
    }

    /// Create a new record for a model installed by Anna.
    pub fn installed_by_anna(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ModelStatus::Unverified,
            installed_by: InstalledBy::Anna,
            installed_at: Some(timestamp_now()),
            last_verified: None,
            last_verify_message: None,
            use_count: 0,
            last_error: None,
        }
    }

    /// Create a new record for a pre-existing model.
    pub fn detected(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ModelStatus::Unverified,
            installed_by: InstalledBy::User,
            installed_at: Some(timestamp_now()),
            last_verified: None,
            last_verify_message: None,
            use_count: 0,
            last_error: None,
        }
    }

    /// Mark as verified OK.
    pub fn mark_ok(&mut self, message: &str) {
        self.status = ModelStatus::Ok;
        self.last_verified = Some(timestamp_now());
        self.last_verify_message = Some(message.to_string());
        self.last_error = None;
    }

    /// Mark as broken.
    pub fn mark_broken(&mut self, error: &str) {
        self.status = ModelStatus::Broken;
        self.last_verified = Some(timestamp_now());
        self.last_error = Some(error.to_string());
    }

    /// Record a successful use.
    pub fn record_use(&mut self) {
        self.use_count += 1;
    }
}

/// Overall model health tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    /// Per-model health records.
    pub models: HashMap<String, ModelHealthRecord>,
    /// Last full check time.
    pub last_check: Option<String>,
}

impl ModelHealth {
    /// Create empty health tracker.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            last_check: None,
        }
    }

    /// Load from file.
    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    /// Get or create record for a model.
    pub fn get_or_create(&mut self, name: &str) -> &mut ModelHealthRecord {
        self.models
            .entry(name.to_string())
            .or_insert_with(|| ModelHealthRecord::missing(name))
    }

    /// Get status of a model.
    pub fn status(&self, name: &str) -> ModelStatus {
        self.models
            .get(name)
            .map(|r| r.status)
            .unwrap_or(ModelStatus::Missing)
    }

    /// Check if all plan models are usable.
    pub fn all_usable(&self, plan: &ModelPlan) -> bool {
        plan.model_names()
            .iter()
            .all(|name| self.status(name).is_usable())
    }

    /// Get missing models from plan.
    pub fn missing_models(&self, plan: &ModelPlan) -> Vec<String> {
        plan.model_names()
            .iter()
            .filter(|name| self.status(name) == ModelStatus::Missing)
            .map(|s| s.to_string())
            .collect()
    }

    /// Get broken models from plan.
    pub fn broken_models(&self, plan: &ModelPlan) -> Vec<String> {
        plan.model_names()
            .iter()
            .filter(|name| self.status(name) == ModelStatus::Broken)
            .map(|s| s.to_string())
            .collect()
    }

    /// Mark check complete.
    pub fn mark_checked(&mut self) {
        self.last_check = Some(timestamp_now());
    }

    /// Get models installed by Anna.
    pub fn anna_installed_models(&self) -> Vec<&str> {
        self.models
            .iter()
            .filter(|(_, r)| r.installed_by == InstalledBy::Anna)
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

impl Default for ModelHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Model verifier that tests models.
pub struct ModelVerifier {
    /// Ollama endpoint.
    ollama_url: String,
    /// Timeout for verification in seconds.
    timeout_secs: u64,
}

impl ModelVerifier {
    /// Create a new verifier.
    pub fn new() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            timeout_secs: 30,
        }
    }

    /// Set Ollama URL.
    pub fn with_url(mut self, url: &str) -> Self {
        self.ollama_url = url.to_string();
        self
    }

    /// List installed models from Ollama.
    pub fn list_installed(&self) -> Result<Vec<String>, VerifyError> {
        use std::process::Command;

        let output = Command::new("ollama")
            .arg("list")
            .output()
            .map_err(|e| VerifyError::OllamaError(format!("Failed to run ollama: {}", e)))?;

        if !output.status.success() {
            return Err(VerifyError::OllamaError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let models: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| line.split_whitespace().next())
            .map(|s| s.to_string())
            .collect();

        Ok(models)
    }

    /// Verify a model works with a test completion.
    pub fn verify_model(&self, model: &str) -> Result<String, VerifyError> {
        use std::process::Command;

        let output = Command::new("ollama")
            .args(["run", model, "Reply with only: OK"])
            .output()
            .map_err(|e| VerifyError::OllamaError(format!("Failed to run model: {}", e)))?;

        if !output.status.success() {
            return Err(VerifyError::ModelError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if response.is_empty() {
            return Err(VerifyError::ModelError("Empty response".to_string()));
        }

        Ok(response)
    }

    /// Pull (install) a model.
    pub fn pull_model(&self, model: &str) -> Result<(), VerifyError> {
        use std::process::Command;

        let output = Command::new("ollama")
            .args(["pull", model])
            .output()
            .map_err(|e| VerifyError::OllamaError(format!("Failed to pull model: {}", e)))?;

        if !output.status.success() {
            return Err(VerifyError::InstallError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    /// Remove a model.
    pub fn remove_model(&self, model: &str) -> Result<(), VerifyError> {
        use std::process::Command;

        let output = Command::new("ollama")
            .args(["rm", model])
            .output()
            .map_err(|e| VerifyError::OllamaError(format!("Failed to remove model: {}", e)))?;

        if !output.status.success() {
            return Err(VerifyError::OllamaError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    /// Sync health with actual Ollama state.
    pub fn sync_health(&self, health: &mut ModelHealth, plan: &ModelPlan) -> Result<(), VerifyError> {
        let installed = self.list_installed()?;

        for model_name in plan.model_names() {
            let record = health.get_or_create(model_name);

            if installed.iter().any(|m| m.starts_with(model_name)) {
                // Model is installed
                if record.status == ModelStatus::Missing {
                    // Was missing, now detected
                    *record = ModelHealthRecord::detected(model_name);
                }
            } else {
                // Model is not installed
                if record.status != ModelStatus::Missing {
                    record.status = ModelStatus::Missing;
                    record.last_error = Some("Model was removed from Ollama".to_string());
                }
            }
        }

        health.mark_checked();
        Ok(())
    }

    /// Verify all models in plan and update health.
    pub fn verify_all(&self, health: &mut ModelHealth, plan: &ModelPlan) -> VerifyReport {
        let mut report = VerifyReport::default();

        for model_name in plan.model_names() {
            let record = health.get_or_create(model_name);

            if record.status == ModelStatus::Missing {
                report.missing.push(model_name.to_string());
                continue;
            }

            match self.verify_model(model_name) {
                Ok(msg) => {
                    record.mark_ok(&msg);
                    report.ok.push(model_name.to_string());
                }
                Err(e) => {
                    record.mark_broken(&e.to_string());
                    report.broken.push((model_name.to_string(), e.to_string()));
                }
            }
        }

        report
    }
}

impl Default for ModelVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification error.
#[derive(Debug, Clone)]
pub enum VerifyError {
    /// Ollama is not available or errored.
    OllamaError(String),
    /// Model-specific error.
    ModelError(String),
    /// Installation error.
    InstallError(String),
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OllamaError(msg) => write!(f, "Ollama error: {}", msg),
            Self::ModelError(msg) => write!(f, "Model error: {}", msg),
            Self::InstallError(msg) => write!(f, "Install error: {}", msg),
        }
    }
}

impl std::error::Error for VerifyError {}

/// Report from verification run.
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    /// Models that verified OK.
    pub ok: Vec<String>,
    /// Models that are missing.
    pub missing: Vec<String>,
    /// Models that are broken (name, error).
    pub broken: Vec<(String, String)>,
}

impl VerifyReport {
    /// Whether all models are OK.
    pub fn all_ok(&self) -> bool {
        self.missing.is_empty() && self.broken.is_empty()
    }

    /// Format for display.
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        if !self.ok.is_empty() {
            parts.push(format!("OK: {}", self.ok.join(", ")));
        }
        if !self.missing.is_empty() {
            parts.push(format!("MISSING: {}", self.missing.join(", ")));
        }
        if !self.broken.is_empty() {
            let broken: Vec<_> = self.broken.iter().map(|(n, _)| n.as_str()).collect();
            parts.push(format!("BROKEN: {}", broken.join(", ")));
        }

        parts.join(" | ")
    }
}

/// Get current timestamp.
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::profile::CapabilityTier;

    fn mock_plan() -> ModelPlan {
        ModelPlan {
            catalog_version: 1,
            profile_version: 1,
            tier: CapabilityTier::Small,
            translator_model: "qwen3:0.6b".to_string(),
            junior_model: "qwen3:1.7b".to_string(),
            senior_model: "qwen3:4b".to_string(),
            prefer_small: false,
            estimated_disk_gb: 6,
            created_at: "0".to_string(),
            rationale: "Test plan".to_string(),
        }
    }

    #[test]
    fn test_model_status_labels() {
        assert_eq!(ModelStatus::Ok.label(), "OK");
        assert_eq!(ModelStatus::Missing.label(), "MISSING");
        assert_eq!(ModelStatus::Broken.label(), "BROKEN");
    }

    #[test]
    fn test_model_status_usable() {
        assert!(ModelStatus::Ok.is_usable());
        assert!(ModelStatus::Unverified.is_usable());
        assert!(!ModelStatus::Missing.is_usable());
        assert!(!ModelStatus::Broken.is_usable());
    }

    #[test]
    fn test_health_tracker() {
        let mut health = ModelHealth::new();
        let plan = mock_plan();

        // Initially all missing
        assert_eq!(health.status("qwen3:0.6b"), ModelStatus::Missing);
        assert!(!health.all_usable(&plan));

        // Add records
        health.models.insert(
            "qwen3:0.6b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:0.6b"),
        );
        health.models.insert(
            "qwen3:1.7b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:1.7b"),
        );
        health.models.insert(
            "qwen3:4b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:4b"),
        );

        // All unverified but usable
        assert!(health.all_usable(&plan));
    }

    #[test]
    fn test_missing_models() {
        let mut health = ModelHealth::new();
        let plan = mock_plan();

        let missing = health.missing_models(&plan);
        assert_eq!(missing.len(), 3);

        // Add one model
        health.models.insert(
            "qwen3:0.6b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:0.6b"),
        );

        let missing = health.missing_models(&plan);
        assert_eq!(missing.len(), 2);
    }

    #[test]
    fn test_mark_ok_and_broken() {
        let mut record = ModelHealthRecord::missing("test_model");

        record.mark_ok("Test passed");
        assert_eq!(record.status, ModelStatus::Ok);
        assert!(record.last_verified.is_some());

        record.mark_broken("Test failed");
        assert_eq!(record.status, ModelStatus::Broken);
        assert!(record.last_error.is_some());
    }

    #[test]
    fn test_verify_report() {
        let report = VerifyReport {
            ok: vec!["model1".to_string()],
            missing: vec!["model2".to_string()],
            broken: vec![("model3".to_string(), "error".to_string())],
        };

        assert!(!report.all_ok());

        let formatted = report.format();
        assert!(formatted.contains("OK: model1"));
        assert!(formatted.contains("MISSING: model2"));
        assert!(formatted.contains("BROKEN: model3"));
    }

    #[test]
    fn test_anna_installed_models() {
        let mut health = ModelHealth::new();

        health.models.insert(
            "model1".to_string(),
            ModelHealthRecord::installed_by_anna("model1"),
        );
        health.models.insert(
            "model2".to_string(),
            ModelHealthRecord::detected("model2"),
        );

        let anna_models = health.anna_installed_models();
        assert_eq!(anna_models.len(), 1);
        assert!(anna_models.contains(&"model1"));
    }
}

//! Model verification and installation (v0.0.434).
//!
//! Tests models work correctly and manages model lifecycle via Ollama.

use super::health_record::ModelHealthRecord;
use super::health_tracker::ModelHealth;
use super::model_status::ModelStatus;
use crate::hardware_aware::model_plan::ModelPlan;

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
    pub fn sync_health(
        &self,
        health: &mut ModelHealth,
        plan: &ModelPlan,
    ) -> Result<(), VerifyError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

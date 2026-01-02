//! Model usage statistics (v0.0.434).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsageStats {
    /// Per-model usage counts.
    pub models: HashMap<String, ModelUsage>,
    /// Last error.
    pub last_error: Option<ModelError>,
}

impl ModelUsageStats {
    /// Record a model call.
    pub fn record_call(&mut self, model: &str, duration_ms: u64, success: bool) {
        let usage = self.models.entry(model.to_string()).or_default();
        usage.call_count += 1;
        usage.total_duration_ms += duration_ms;

        if !success {
            usage.error_count += 1;
        }
    }

    /// Record an error.
    pub fn record_error(&mut self, model: &str, error_type: &str) {
        self.last_error = Some(ModelError {
            model: model.to_string(),
            error_type: error_type.to_string(),
            timestamp: timestamp_now(),
        });

        if let Some(usage) = self.models.get_mut(model) {
            usage.error_count += 1;
        }
    }

    /// Format for stats display.
    pub fn format(&self) -> String {
        let mut lines = Vec::new();
        lines.push("[llm_usage]".to_string());

        for (role, model_key) in [
            ("translator", "translator"),
            ("junior", "junior"),
            ("senior", "senior"),
        ] {
            if let Some((model, usage)) = self.models.iter().find(|(m, _)| m.contains(model_key)) {
                let avg_ms = if usage.call_count > 0 {
                    usage.total_duration_ms / usage.call_count
                } else {
                    0
                };
                lines.push(format!(
                    "  {}_calls     {} (avg {:.1}s)",
                    role,
                    usage.call_count,
                    avg_ms as f64 / 1000.0
                ));
            }
        }

        if let Some(error) = &self.last_error {
            lines.push(format!(
                "  last_model_error     {} ({}, {})",
                error.timestamp, error.error_type, error.model
            ));
        }

        lines.join("\n")
    }
}

/// Usage stats for a single model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Number of calls.
    pub call_count: u64,
    /// Total duration in ms.
    pub total_duration_ms: u64,
    /// Error count.
    pub error_count: u64,
}

impl ModelUsage {
    /// Average duration in ms.
    pub fn avg_duration_ms(&self) -> u64 {
        if self.call_count > 0 {
            self.total_duration_ms / self.call_count
        } else {
            0
        }
    }
}

/// Model error record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelError {
    /// Model name.
    pub model: String,
    /// Error type.
    pub error_type: String,
    /// Timestamp.
    pub timestamp: String,
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

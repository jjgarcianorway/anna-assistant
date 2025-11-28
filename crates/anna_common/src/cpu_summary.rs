//! CPU Summary Helper v0.79.0
//!
//! Extracts meaningful CPU information from lscpu -J output.
//! Used to provide clear evidence summary to Senior auditor.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Structured CPU summary extracted from lscpu output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuSummary {
    /// Total logical CPUs (threads) - "CPU(s)" field
    pub logical_cpus: u32,
    /// Physical cores per socket - "Core(s) per socket" field
    pub cores_per_socket: u32,
    /// Number of CPU sockets - "Socket(s)" field
    pub sockets: u32,
    /// Threads per physical core - "Thread(s) per core" field
    pub threads_per_core: u32,
    /// CPU model name if available
    pub model_name: Option<String>,
}

impl CpuSummary {
    /// Calculate total physical cores
    pub fn physical_cores(&self) -> u32 {
        self.cores_per_socket * self.sockets
    }

    /// Check if hyperthreading is enabled
    pub fn has_hyperthreading(&self) -> bool {
        self.threads_per_core > 1
    }

    /// Format as human-readable summary for Senior prompt
    pub fn format_for_senior(&self) -> String {
        let mut lines = Vec::new();
        lines.push("CPU SUMMARY (from cpu.info):".to_string());
        lines.push(format!("- logical_cpus (CPU(s)): {}", self.logical_cpus));
        lines.push(format!("- cores_per_socket: {}", self.cores_per_socket));
        lines.push(format!("- sockets: {}", self.sockets));
        lines.push(format!("- threads_per_core: {}", self.threads_per_core));
        lines.push(format!("- physical_cores (computed): {}", self.physical_cores()));
        if let Some(ref model) = self.model_name {
            lines.push(format!("- model: {}", model));
        }
        lines.join("\n")
    }

    /// v0.80.0: Format as compact JSON for probe summary
    pub fn to_compact_json(&self) -> String {
        let model = self.model_name.as_deref().unwrap_or("unknown");
        format!(
            r#"{{"threads_total":{},"physical_cores":{},"model":"{}"}}"#,
            self.logical_cpus,
            self.physical_cores(),
            model.replace('"', "'")
        )
    }
}

/// Parse lscpu -J output to extract CPU summary
///
/// The lscpu -J output format is:
/// ```json
/// {"lscpu": [
///   {"field": "CPU(s):", "data": "32"},
///   {"field": "Core(s) per socket:", "data": "24"},
///   {"field": "Socket(s):", "data": "1"},
///   {"field": "Thread(s) per core:", "data": "2"},
///   {"field": "Model name:", "data": "AMD Ryzen ..."}
/// ]}
/// ```
pub fn summarize_cpu(lscpu_json: &Value) -> CpuSummary {
    let mut summary = CpuSummary::default();

    // Try to extract from lscpu -J format
    if let Some(lscpu_array) = lscpu_json.get("lscpu").and_then(|v| v.as_array()) {
        for entry in lscpu_array {
            let field = entry.get("field").and_then(|f| f.as_str()).unwrap_or("");
            let data = entry.get("data").and_then(|d| d.as_str()).unwrap_or("");

            match field.trim_end_matches(':') {
                "CPU(s)" => {
                    summary.logical_cpus = data.parse().unwrap_or(0);
                }
                "Core(s) per socket" => {
                    summary.cores_per_socket = data.parse().unwrap_or(0);
                }
                "Socket(s)" => {
                    summary.sockets = data.parse().unwrap_or(0);
                }
                "Thread(s) per core" => {
                    summary.threads_per_core = data.parse().unwrap_or(0);
                }
                "Model name" => {
                    summary.model_name = Some(data.to_string());
                }
                _ => {}
            }
        }
    }

    // Also try flat JSON format (from parsed cpuinfo.rs)
    if summary.logical_cpus == 0 {
        if let Some(v) = lscpu_json.get("logical_cores").and_then(|v| v.as_u64()) {
            summary.logical_cpus = v as u32;
        }
    }
    if summary.cores_per_socket == 0 {
        if let Some(v) = lscpu_json.get("physical_cores").and_then(|v| v.as_u64()) {
            // In flat format, physical_cores is total, not per socket
            let sockets = summary.sockets.max(1);
            summary.cores_per_socket = v as u32 / sockets;
        }
    }
    if summary.sockets == 0 {
        if let Some(v) = lscpu_json.get("sockets").and_then(|v| v.as_u64()) {
            summary.sockets = v as u32;
        }
    }
    if summary.threads_per_core == 0 {
        if let Some(v) = lscpu_json.get("threads_per_core").and_then(|v| v.as_u64()) {
            summary.threads_per_core = v as u32;
        }
    }
    if summary.model_name.is_none() {
        if let Some(v) = lscpu_json.get("model_name").and_then(|v| v.as_str()) {
            summary.model_name = Some(v.to_string());
        }
    }

    // Default to 1 for zero values to avoid division by zero
    if summary.sockets == 0 {
        summary.sockets = 1;
    }
    if summary.threads_per_core == 0 {
        summary.threads_per_core = 1;
    }

    summary
}

/// Parse lscpu text output (non-JSON) to extract CPU summary
///
/// Format:
/// ```text
/// CPU(s):                  32
/// Core(s) per socket:      24
/// Socket(s):               1
/// Thread(s) per core:      2
/// ```
pub fn summarize_cpu_from_text(text: &str) -> CpuSummary {
    let mut summary = CpuSummary::default();

    for line in text.lines() {
        let line = line.trim();
        if let Some((field, value)) = line.split_once(':') {
            let field = field.trim();
            let value = value.trim();

            match field {
                "CPU(s)" => {
                    summary.logical_cpus = value.parse().unwrap_or(0);
                }
                "Core(s) per socket" => {
                    summary.cores_per_socket = value.parse().unwrap_or(0);
                }
                "Socket(s)" => {
                    summary.sockets = value.parse().unwrap_or(0);
                }
                "Thread(s) per core" => {
                    summary.threads_per_core = value.parse().unwrap_or(0);
                }
                "Model name" => {
                    summary.model_name = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    // Default to 1 for zero values
    if summary.sockets == 0 {
        summary.sockets = 1;
    }
    if summary.threads_per_core == 0 {
        summary.threads_per_core = 1;
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_cpu_lscpu_json() {
        let json: Value = serde_json::from_str(r#"{
            "lscpu": [
                {"field": "CPU(s):", "data": "32"},
                {"field": "Core(s) per socket:", "data": "24"},
                {"field": "Socket(s):", "data": "1"},
                {"field": "Thread(s) per core:", "data": "2"},
                {"field": "Model name:", "data": "AMD Ryzen Threadripper 3960X"}
            ]
        }"#).unwrap();

        let summary = summarize_cpu(&json);
        assert_eq!(summary.logical_cpus, 32);
        assert_eq!(summary.cores_per_socket, 24);
        assert_eq!(summary.sockets, 1);
        assert_eq!(summary.threads_per_core, 2);
        assert_eq!(summary.physical_cores(), 24);
        assert!(summary.has_hyperthreading());
        assert_eq!(summary.model_name, Some("AMD Ryzen Threadripper 3960X".to_string()));
    }

    #[test]
    fn test_summarize_cpu_flat_json() {
        let json: Value = serde_json::from_str(r#"{
            "logical_cores": 32,
            "physical_cores": 24,
            "sockets": 1,
            "threads_per_core": 2,
            "model_name": "AMD Ryzen"
        }"#).unwrap();

        let summary = summarize_cpu(&json);
        assert_eq!(summary.logical_cpus, 32);
        assert_eq!(summary.cores_per_socket, 24);
        assert_eq!(summary.sockets, 1);
        assert_eq!(summary.threads_per_core, 2);
    }

    #[test]
    fn test_summarize_cpu_from_text() {
        let text = r#"
CPU(s):                          32
Core(s) per socket:              24
Socket(s):                       1
Thread(s) per core:              2
Model name:                      AMD Ryzen Threadripper 3960X 24-Core Processor
"#;

        let summary = summarize_cpu_from_text(text);
        assert_eq!(summary.logical_cpus, 32);
        assert_eq!(summary.cores_per_socket, 24);
        assert_eq!(summary.sockets, 1);
        assert_eq!(summary.threads_per_core, 2);
        assert_eq!(summary.physical_cores(), 24);
    }

    #[test]
    fn test_format_for_senior() {
        let summary = CpuSummary {
            logical_cpus: 32,
            cores_per_socket: 24,
            sockets: 1,
            threads_per_core: 2,
            model_name: Some("AMD Ryzen".to_string()),
        };

        let formatted = summary.format_for_senior();
        assert!(formatted.contains("logical_cpus (CPU(s)): 32"));
        assert!(formatted.contains("cores_per_socket: 24"));
        assert!(formatted.contains("physical_cores (computed): 24"));
    }

    #[test]
    fn test_defaults_for_zero_values() {
        let json: Value = serde_json::from_str(r#"{}"#).unwrap();
        let summary = summarize_cpu(&json);

        // Should default to 1 to avoid division by zero
        assert_eq!(summary.sockets, 1);
        assert_eq!(summary.threads_per_core, 1);
    }
}

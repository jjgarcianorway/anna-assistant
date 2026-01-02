//! Debug Trace File Manager - v0.0.443.

use super::trace_types::{RequestTrace, TraceEvent};

/// Trace file manager.
pub struct TraceFileManager {
    /// Debug directory.
    debug_dir: String,
}

impl TraceFileManager {
    /// Default debug directory.
    pub const DEFAULT_DIR: &'static str = "/var/lib/anna/debug";

    /// Create new manager.
    pub fn new() -> Self {
        Self {
            debug_dir: Self::DEFAULT_DIR.to_string(),
        }
    }

    /// Create with custom directory.
    pub fn with_dir(dir: &str) -> Self {
        Self {
            debug_dir: dir.to_string(),
        }
    }

    /// Get trace file path for request.
    pub fn trace_path(&self, request_id: &str) -> String {
        format!("{}/{}.jsonl", self.debug_dir, request_id)
    }

    /// Write event to trace file.
    pub fn write_event(&self, event: &TraceEvent) -> Result<(), String> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.debug_dir)
            .map_err(|e| format!("Failed to create debug dir: {}", e))?;

        let path = self.trace_path(&event.request_id);
        let line = event.to_jsonl()?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open trace file: {}", e))?;

        writeln!(file, "{}", line).map_err(|e| format!("Failed to write trace: {}", e))
    }

    /// Read trace for request.
    pub fn read_trace(&self, request_id: &str) -> Result<RequestTrace, String> {
        let path = self.trace_path(request_id);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read trace file: {}", e))?;

        let mut trace = RequestTrace::new(request_id);

        for line in content.lines() {
            if let Ok(event) = serde_json::from_str::<TraceEvent>(line) {
                trace.add_event(event);
            }
        }

        Ok(trace)
    }

    /// List available traces.
    pub fn list_traces(&self) -> Result<Vec<String>, String> {
        let entries = std::fs::read_dir(&self.debug_dir)
            .map_err(|e| format!("Failed to read debug dir: {}", e))?;

        let mut ids = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".jsonl") {
                    ids.push(name.trim_end_matches(".jsonl").to_string());
                }
            }
        }

        ids.sort();
        ids.reverse(); // Most recent first
        Ok(ids)
    }
}

impl Default for TraceFileManager {
    fn default() -> Self {
        Self::new()
    }
}

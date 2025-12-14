// v0.0.615: Settings Processor (Phase 191)
// Process settings changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Processor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProcessorMode {
    /// Synchronous
    #[default]
    Sync,
    /// Asynchronous
    Async,
    /// Batch
    Batch,
    /// Streaming
    Streaming,
}

impl std::fmt::Display for ProcessorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync => write!(f, "sync"),
            Self::Async => write!(f, "async"),
            Self::Batch => write!(f, "batch"),
            Self::Streaming => write!(f, "streaming"),
        }
    }
}

/// Processor state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProcessorState {
    /// Idle
    #[default]
    Idle,
    /// Processing
    Processing,
    /// Paused
    Paused,
    /// Error
    Error,
    /// Shutdown
    Shutdown,
}

impl std::fmt::Display for ProcessorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Processing => write!(f, "processing"),
            Self::Paused => write!(f, "paused"),
            Self::Error => write!(f, "error"),
            Self::Shutdown => write!(f, "shutdown"),
        }
    }
}

/// Processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    /// Unique ID
    pub id: String,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Key
    pub key: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl ProcessingJob {
    /// Create new job
    pub fn new(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category: None,
            key: key.into(),
            old_value: None,
            new_value: None,
            timestamp: 0,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set old value
    pub fn old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn new_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Is create
    pub fn is_create(&self) -> bool {
        self.old_value.is_none() && self.new_value.is_some()
    }

    /// Is update
    pub fn is_update(&self) -> bool {
        self.old_value.is_some() && self.new_value.is_some()
    }

    /// Is delete
    pub fn is_delete(&self) -> bool {
        self.old_value.is_some() && self.new_value.is_none()
    }
}

/// Processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    /// Job ID
    pub job_id: String,
    /// Success
    pub success: bool,
    /// Message
    pub message: String,
    /// Duration ms
    pub duration_ms: u64,
}

impl ProcessingResult {
    /// Create success result
    pub fn success(job_id: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            success: true,
            message: String::new(),
            duration_ms: 0,
        }
    }

    /// Create failure result
    pub fn failure(job_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            success: false,
            message: message.into(),
            duration_ms: 0,
        }
    }

    /// Set duration
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Settings processor
#[derive(Debug, Clone, Default)]
pub struct SettingsProcessor {
    /// Mode
    mode: ProcessorMode,
    /// State
    state: ProcessorState,
    /// Pending jobs
    pending: HashMap<String, ProcessingJob>,
    /// Results
    results: Vec<ProcessingResult>,
    /// Max results
    max_results: usize,
    /// Total processed
    total_processed: usize,
}

impl SettingsProcessor {
    /// Create new processor
    pub fn new() -> Self {
        Self {
            max_results: 200,
            ..Default::default()
        }
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: ProcessorMode) {
        self.mode = mode;
    }

    /// Get mode
    pub fn mode(&self) -> ProcessorMode {
        self.mode
    }

    /// Get state
    pub fn state(&self) -> ProcessorState {
        self.state
    }

    /// Submit job
    pub fn submit(&mut self, job: ProcessingJob) {
        self.pending.insert(job.id.clone(), job);
    }

    /// Get pending job
    pub fn get_pending(&self, id: &str) -> Option<&ProcessingJob> {
        self.pending.get(id)
    }

    /// Complete job
    pub fn complete(&mut self, result: ProcessingResult) {
        self.pending.remove(&result.job_id);
        self.total_processed += 1;
        self.results.push(result);
        while self.results.len() > self.max_results {
            self.results.remove(0);
        }
    }

    /// Start processing
    pub fn start(&mut self) {
        self.state = ProcessorState::Processing;
    }

    /// Pause
    pub fn pause(&mut self) {
        self.state = ProcessorState::Paused;
    }

    /// Resume
    pub fn resume(&mut self) {
        if self.state == ProcessorState::Paused {
            self.state = ProcessorState::Processing;
        }
    }

    /// Shutdown
    pub fn shutdown(&mut self) {
        self.state = ProcessorState::Shutdown;
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Total processed
    pub fn total_processed(&self) -> usize {
        self.total_processed
    }

    /// Results
    pub fn results(&self) -> &[ProcessingResult] {
        &self.results
    }
}

/// Format processor
pub fn format_processor(processor: &SettingsProcessor) -> String {
    let mut output = String::new();
    output.push_str("Settings Processor:\n");
    output.push_str(&format!("  Mode: {}\n", processor.mode()));
    output.push_str(&format!("  State: {}\n", processor.state()));
    output.push_str(&format!("  Pending: {}\n", processor.pending_count()));
    output.push_str(&format!("  Processed: {}\n", processor.total_processed()));
    output
}

/// Check if query is about processor
pub fn is_processor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("processor")
        || lower.contains("processing")
        || lower.contains("batch process")
}

/// Fun fact about processor
pub fn processor_fun_fact() -> &'static str {
    "Anna's processor can handle settings changes in sync, async, batch, or streaming mode!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", ProcessorMode::Sync), "sync");
        assert_eq!(format!("{}", ProcessorMode::Async), "async");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", ProcessorState::Idle), "idle");
        assert_eq!(format!("{}", ProcessorState::Processing), "processing");
    }

    #[test]
    fn test_job_new() {
        let j = ProcessingJob::new("j1", "test_key");
        assert!(j.old_value.is_none());
    }

    #[test]
    fn test_job_operations() {
        let create = ProcessingJob::new("j1", "k").new_value("v");
        assert!(create.is_create());

        let update = ProcessingJob::new("j2", "k").old_value("a").new_value("b");
        assert!(update.is_update());

        let delete = ProcessingJob::new("j3", "k").old_value("v");
        assert!(delete.is_delete());
    }

    #[test]
    fn test_result_success() {
        let r = ProcessingResult::success("j1");
        assert!(r.success);
    }

    #[test]
    fn test_result_failure() {
        let r = ProcessingResult::failure("j1", "error");
        assert!(!r.success);
    }

    #[test]
    fn test_processor_new() {
        let p = SettingsProcessor::new();
        assert_eq!(p.state(), ProcessorState::Idle);
    }

    #[test]
    fn test_processor_submit() {
        let mut p = SettingsProcessor::new();
        p.submit(ProcessingJob::new("j1", "key"));
        assert_eq!(p.pending_count(), 1);
    }

    #[test]
    fn test_processor_lifecycle() {
        let mut p = SettingsProcessor::new();
        p.start();
        assert_eq!(p.state(), ProcessorState::Processing);
        p.pause();
        assert_eq!(p.state(), ProcessorState::Paused);
        p.resume();
        assert_eq!(p.state(), ProcessorState::Processing);
    }

    #[test]
    fn test_is_processor_query() {
        assert!(is_processor_query("batch processing"));
        assert!(!is_processor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = processor_fun_fact();
        assert!(fact.contains("processor"));
    }
}

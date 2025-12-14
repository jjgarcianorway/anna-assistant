// v0.0.614: Settings Pipeline (Phase 190)
// Pipeline processing for settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    /// Input validation
    Validate,
    /// Transform data
    Transform,
    /// Apply changes
    Apply,
    /// Verify results
    Verify,
    /// Notify observers
    Notify,
    /// Cleanup
    Cleanup,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validate => write!(f, "validate"),
            Self::Transform => write!(f, "transform"),
            Self::Apply => write!(f, "apply"),
            Self::Verify => write!(f, "verify"),
            Self::Notify => write!(f, "notify"),
            Self::Cleanup => write!(f, "cleanup"),
        }
    }
}

/// Pipeline status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PipelineStatus {
    /// Ready
    #[default]
    Ready,
    /// Running
    Running,
    /// Paused
    Paused,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

impl std::fmt::Display for PipelineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "ready"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Stage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    /// Stage
    pub stage: PipelineStage,
    /// Success
    pub success: bool,
    /// Message
    pub message: String,
    /// Duration ms
    pub duration_ms: u64,
}

impl StageResult {
    /// Create success result
    pub fn success(stage: PipelineStage) -> Self {
        Self {
            stage,
            success: true,
            message: String::new(),
            duration_ms: 0,
        }
    }

    /// Create failure result
    pub fn failure(stage: PipelineStage, message: impl Into<String>) -> Self {
        Self {
            stage,
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

/// Pipeline run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRun {
    /// Unique ID
    pub id: String,
    /// Status
    pub status: PipelineStatus,
    /// Current stage
    pub current_stage: Option<PipelineStage>,
    /// Stage results
    pub results: Vec<StageResult>,
    /// Started timestamp
    pub started_at: u64,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

impl PipelineRun {
    /// Create new run
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: PipelineStatus::Ready,
            current_stage: None,
            results: Vec::new(),
            started_at: 0,
            completed_at: None,
        }
    }

    /// Start run
    pub fn start(&mut self, timestamp: u64) {
        self.status = PipelineStatus::Running;
        self.started_at = timestamp;
    }

    /// Set current stage
    pub fn set_stage(&mut self, stage: PipelineStage) {
        self.current_stage = Some(stage);
    }

    /// Add result
    pub fn add_result(&mut self, result: StageResult) {
        if !result.success {
            self.status = PipelineStatus::Failed;
        }
        self.results.push(result);
    }

    /// Complete run
    pub fn complete(&mut self, timestamp: u64) {
        if self.status == PipelineStatus::Running {
            self.status = PipelineStatus::Completed;
        }
        self.completed_at = Some(timestamp);
        self.current_stage = None;
    }

    /// Is success
    pub fn is_success(&self) -> bool {
        self.status == PipelineStatus::Completed && self.results.iter().all(|r| r.success)
    }

    /// Total duration
    pub fn total_duration(&self) -> u64 {
        self.results.iter().map(|r| r.duration_ms).sum()
    }
}

/// Settings pipeline
#[derive(Debug, Clone, Default)]
pub struct SettingsPipeline {
    /// Pipeline runs
    runs: HashMap<String, PipelineRun>,
    /// History
    history: Vec<PipelineRun>,
    /// Max history
    max_history: usize,
}

impl SettingsPipeline {
    /// Create new pipeline
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Create run
    pub fn create_run(&mut self, id: impl Into<String>) -> &mut PipelineRun {
        let run = PipelineRun::new(id);
        let run_id = run.id.clone();
        self.runs.insert(run_id.clone(), run);
        self.runs.get_mut(&run_id).unwrap()
    }

    /// Get run
    pub fn get_run(&self, id: &str) -> Option<&PipelineRun> {
        self.runs.get(id)
    }

    /// Get run mut
    pub fn get_run_mut(&mut self, id: &str) -> Option<&mut PipelineRun> {
        self.runs.get_mut(id)
    }

    /// Complete and archive run
    pub fn archive(&mut self, id: &str) {
        if let Some(run) = self.runs.remove(id) {
            self.history.push(run);
            while self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Active runs
    pub fn active_count(&self) -> usize {
        self.runs.len()
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Get history
    pub fn history(&self) -> &[PipelineRun] {
        &self.history
    }

    /// Success count
    pub fn success_count(&self) -> usize {
        self.history.iter().filter(|r| r.is_success()).count()
    }
}

/// Format pipeline
pub fn format_pipeline(pipeline: &SettingsPipeline) -> String {
    let mut output = String::new();
    output.push_str("Settings Pipeline:\n");
    output.push_str(&format!("  Active: {}\n", pipeline.active_count()));
    output.push_str(&format!("  History: {}\n", pipeline.history_count()));
    output.push_str(&format!("  Successes: {}\n", pipeline.success_count()));
    output
}

/// Check if query is about pipeline
pub fn is_pipeline_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("pipeline")
        || lower.contains("processing stage")
        || lower.contains("workflow")
}

/// Fun fact about pipeline
pub fn pipeline_fun_fact() -> &'static str {
    "Anna's pipeline processes settings through multiple stages for reliability!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_display() {
        assert_eq!(format!("{}", PipelineStage::Validate), "validate");
        assert_eq!(format!("{}", PipelineStage::Apply), "apply");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PipelineStatus::Running), "running");
        assert_eq!(format!("{}", PipelineStatus::Completed), "completed");
    }

    #[test]
    fn test_stage_result_success() {
        let r = StageResult::success(PipelineStage::Validate);
        assert!(r.success);
    }

    #[test]
    fn test_stage_result_failure() {
        let r = StageResult::failure(PipelineStage::Apply, "error");
        assert!(!r.success);
    }

    #[test]
    fn test_run_new() {
        let r = PipelineRun::new("r1");
        assert_eq!(r.status, PipelineStatus::Ready);
    }

    #[test]
    fn test_run_lifecycle() {
        let mut r = PipelineRun::new("r1");
        r.start(100);
        assert_eq!(r.status, PipelineStatus::Running);
        r.add_result(StageResult::success(PipelineStage::Validate));
        r.complete(200);
        assert!(r.is_success());
    }

    #[test]
    fn test_pipeline_new() {
        let p = SettingsPipeline::new();
        assert_eq!(p.active_count(), 0);
    }

    #[test]
    fn test_pipeline_create_run() {
        let mut p = SettingsPipeline::new();
        p.create_run("r1");
        assert_eq!(p.active_count(), 1);
    }

    #[test]
    fn test_pipeline_archive() {
        let mut p = SettingsPipeline::new();
        p.create_run("r1");
        p.archive("r1");
        assert_eq!(p.active_count(), 0);
        assert_eq!(p.history_count(), 1);
    }

    #[test]
    fn test_is_pipeline_query() {
        assert!(is_pipeline_query("processing pipeline"));
        assert!(!is_pipeline_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = pipeline_fun_fact();
        assert!(fact.contains("pipeline"));
    }
}

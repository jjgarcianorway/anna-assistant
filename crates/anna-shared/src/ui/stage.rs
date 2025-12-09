//! Stage progress tracker for pipeline visualization (v0.0.213).

use super::colors;

/// Stage status enum
#[derive(Clone, Copy, PartialEq)]
pub enum StageStatus {
    Pending,
    Running,
    Complete,
    Skipped,
    Error,
}

/// Internal stage info
struct StageInfo {
    name: String,
    status: StageStatus,
    duration_ms: Option<u64>,
}

/// Stage progress tracker for pipeline visualization
pub struct StageProgress {
    stages: Vec<StageInfo>,
    current: Option<usize>,
}

impl StageProgress {
    /// Create with stage names
    pub fn new(stage_names: &[&str]) -> Self {
        Self {
            stages: stage_names
                .iter()
                .map(|n| StageInfo {
                    name: n.to_string(),
                    status: StageStatus::Pending,
                    duration_ms: None,
                })
                .collect(),
            current: None,
        }
    }

    /// Start a stage
    pub fn start(&mut self, name: &str) {
        if let Some(idx) = self.stages.iter().position(|s| s.name == name) {
            self.stages[idx].status = StageStatus::Running;
            self.current = Some(idx);
        }
    }

    /// Complete current stage
    pub fn complete(&mut self, duration_ms: u64) {
        if let Some(idx) = self.current {
            self.stages[idx].status = StageStatus::Complete;
            self.stages[idx].duration_ms = Some(duration_ms);
        }
    }

    /// Skip a stage
    pub fn skip(&mut self, name: &str) {
        if let Some(idx) = self.stages.iter().position(|s| s.name == name) {
            self.stages[idx].status = StageStatus::Skipped;
        }
    }

    /// Mark stage as error
    pub fn error(&mut self, duration_ms: u64) {
        if let Some(idx) = self.current {
            self.stages[idx].status = StageStatus::Error;
            self.stages[idx].duration_ms = Some(duration_ms);
        }
    }

    /// Render progress line
    pub fn render_line(&self) -> String {
        self.stages
            .iter()
            .map(|s| match s.status {
                StageStatus::Pending => format!("{}○{}", colors::DIM, colors::RESET),
                StageStatus::Running => format!("{}◉{}", colors::CYAN, colors::RESET),
                StageStatus::Complete => format!("{}●{}", colors::OK, colors::RESET),
                StageStatus::Skipped => format!("{}-{}", colors::DIM, colors::RESET),
                StageStatus::Error => format!("{}●{}", colors::ERR, colors::RESET),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        let completed = self
            .stages
            .iter()
            .filter(|s| s.status == StageStatus::Complete)
            .count();
        let total = self.stages.len();
        let total_ms: u64 = self.stages.iter().filter_map(|s| s.duration_ms).sum();
        format!("{}/{} stages ({}ms)", completed, total, total_ms)
    }
}

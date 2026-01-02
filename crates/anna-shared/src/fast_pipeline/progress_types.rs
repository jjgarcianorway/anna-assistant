//! Progress Types (Part G.1) - v0.0.438.
//!
//! Core types for tracking pipeline phase status and progress.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::fast_pipeline::budget::Phase;

/// Status of a pipeline phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    /// Not yet started.
    Pending,
    /// Currently executing.
    Running,
    /// Completed successfully.
    Completed,
    /// Timed out.
    TimedOut,
    /// Failed with error.
    Failed,
    /// Skipped.
    Skipped,
}

impl PhaseStatus {
    /// Whether this phase is done.
    pub fn is_done(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::TimedOut | Self::Failed | Self::Skipped
        )
    }

    /// Whether this phase succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Emoji indicator.
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "◐",
            Self::Completed => "●",
            Self::TimedOut => "⏱",
            Self::Failed => "✗",
            Self::Skipped => "−",
        }
    }

    /// Honest label (no lies).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "waiting",
            Self::Running => "running",
            Self::Completed => "done",
            Self::TimedOut => "timeout",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// Progress for a single phase.
#[derive(Debug, Clone)]
pub struct PhaseProgress {
    /// Phase type.
    pub phase: Phase,
    /// Current status.
    pub status: PhaseStatus,
    /// When started (if running or done).
    pub started_at: Option<Instant>,
    /// When completed (if done).
    pub completed_at: Option<Instant>,
    /// Duration in milliseconds.
    pub duration_ms: Option<u64>,
    /// Additional detail message.
    pub detail: Option<String>,
}

impl PhaseProgress {
    /// Create pending phase.
    pub fn pending(phase: Phase) -> Self {
        Self {
            phase,
            status: PhaseStatus::Pending,
            started_at: None,
            completed_at: None,
            duration_ms: None,
            detail: None,
        }
    }

    /// Start the phase.
    pub fn start(&mut self) {
        self.status = PhaseStatus::Running;
        self.started_at = Some(Instant::now());
    }

    /// Complete the phase.
    pub fn complete(&mut self) {
        self.status = PhaseStatus::Completed;
        self.completed_at = Some(Instant::now());
        if let Some(start) = self.started_at {
            self.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
    }

    /// Mark as timed out.
    pub fn timeout(&mut self) {
        self.status = PhaseStatus::TimedOut;
        self.completed_at = Some(Instant::now());
        if let Some(start) = self.started_at {
            self.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
    }

    /// Mark as failed.
    pub fn fail(&mut self, reason: &str) {
        self.status = PhaseStatus::Failed;
        self.completed_at = Some(Instant::now());
        self.detail = Some(reason.to_string());
        if let Some(start) = self.started_at {
            self.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
    }

    /// Skip the phase.
    pub fn skip(&mut self, reason: &str) {
        self.status = PhaseStatus::Skipped;
        self.detail = Some(reason.to_string());
    }

    /// Set detail message.
    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    /// Get elapsed time if running.
    pub fn elapsed_ms(&self) -> u64 {
        if let Some(dur) = self.duration_ms {
            dur
        } else if let Some(start) = self.started_at {
            start.elapsed().as_millis() as u64
        } else {
            0
        }
    }

    /// Format for display.
    pub fn format_display(&self) -> String {
        let indicator = self.status.indicator();
        let phase_label = self.phase.label();
        let status_label = self.status.label();

        let time_str = if self.status == PhaseStatus::Running {
            format!(" {}ms", self.elapsed_ms())
        } else if let Some(dur) = self.duration_ms {
            format!(" {}ms", dur)
        } else {
            String::new()
        };

        format!(
            "{} {}: {}{}",
            indicator, phase_label, status_label, time_str
        )
    }
}

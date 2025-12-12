//! Progress Rendering (Part G) - v0.0.438.
//!
//! Show honest progress without lying or breaking parsing:
//! - Show what phase we're in
//! - Show elapsed time
//! - Don't claim "Analyzing..." if we're waiting on network
//! - Never inject progress text into parsed responses

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

        format!("{} {}: {}{}", indicator, phase_label, status_label, time_str)
    }
}

/// Overall pipeline progress.
#[derive(Debug, Clone)]
pub struct PipelineProgress {
    /// Progress for each phase.
    pub phases: Vec<PhaseProgress>,
    /// When pipeline started.
    pub started_at: Option<Instant>,
    /// Current active phase.
    pub current_phase: Option<Phase>,
}

impl PipelineProgress {
    /// Create new pipeline progress.
    pub fn new() -> Self {
        Self {
            phases: vec![
                PhaseProgress::pending(Phase::TranslatorIntent),
                PhaseProgress::pending(Phase::ProbeCollection),
                PhaseProgress::pending(Phase::JuniorSpecialist),
                PhaseProgress::pending(Phase::SeniorSpecialist),
                PhaseProgress::pending(Phase::Renderer),
            ],
            started_at: None,
            current_phase: None,
        }
    }

    /// Start pipeline.
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
    }

    /// Get phase progress.
    pub fn get_phase(&self, phase: Phase) -> Option<&PhaseProgress> {
        self.phases.iter().find(|p| p.phase == phase)
    }

    /// Get mutable phase progress.
    pub fn get_phase_mut(&mut self, phase: Phase) -> Option<&mut PhaseProgress> {
        self.phases.iter_mut().find(|p| p.phase == phase)
    }

    /// Start a phase.
    pub fn start_phase(&mut self, phase: Phase) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.start();
        }
        self.current_phase = Some(phase);
    }

    /// Complete a phase.
    pub fn complete_phase(&mut self, phase: Phase) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.complete();
        }
        if self.current_phase == Some(phase) {
            self.current_phase = None;
        }
    }

    /// Timeout a phase.
    pub fn timeout_phase(&mut self, phase: Phase) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.timeout();
        }
        if self.current_phase == Some(phase) {
            self.current_phase = None;
        }
    }

    /// Fail a phase.
    pub fn fail_phase(&mut self, phase: Phase, reason: &str) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.fail(reason);
        }
        if self.current_phase == Some(phase) {
            self.current_phase = None;
        }
    }

    /// Skip a phase.
    pub fn skip_phase(&mut self, phase: Phase, reason: &str) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.skip(reason);
        }
    }

    /// Total elapsed time.
    pub fn total_elapsed_ms(&self) -> u64 {
        self.started_at
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    /// Check if pipeline is done.
    pub fn is_done(&self) -> bool {
        self.phases.iter().all(|p| p.status.is_done())
    }

    /// Check if any phase timed out.
    pub fn has_timeout(&self) -> bool {
        self.phases.iter().any(|p| p.status == PhaseStatus::TimedOut)
    }

    /// Check if any phase failed.
    pub fn has_failure(&self) -> bool {
        self.phases.iter().any(|p| p.status == PhaseStatus::Failed)
    }
}

impl Default for PipelineProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders progress without lying.
pub struct ProgressRenderer {
    /// Whether to show timing.
    pub show_timing: bool,
    /// Whether to show indicators.
    pub show_indicators: bool,
    /// Compact mode (single line).
    pub compact: bool,
}

impl ProgressRenderer {
    /// Create new renderer.
    pub fn new() -> Self {
        Self {
            show_timing: true,
            show_indicators: true,
            compact: false,
        }
    }

    /// Enable compact mode.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Disable timing.
    pub fn no_timing(mut self) -> Self {
        self.show_timing = false;
        self
    }

    /// Render progress.
    pub fn render(&self, progress: &PipelineProgress) -> String {
        if self.compact {
            self.render_compact(progress)
        } else {
            self.render_full(progress)
        }
    }

    /// Render compact (single line).
    fn render_compact(&self, progress: &PipelineProgress) -> String {
        let parts: Vec<String> = progress
            .phases
            .iter()
            .filter(|p| p.status != PhaseStatus::Pending && p.status != PhaseStatus::Skipped)
            .map(|p| {
                let indicator = if self.show_indicators {
                    p.status.indicator()
                } else {
                    ""
                };
                let time = if self.show_timing {
                    format!(" {}ms", p.elapsed_ms())
                } else {
                    String::new()
                };
                format!("{}{}{}", indicator, p.phase.label(), time)
            })
            .collect();

        let total = if self.show_timing {
            format!(" [{}ms]", progress.total_elapsed_ms())
        } else {
            String::new()
        };

        format!("{}{}", parts.join(" → "), total)
    }

    /// Render full (multi-line).
    fn render_full(&self, progress: &PipelineProgress) -> String {
        let mut lines = Vec::new();

        for phase in &progress.phases {
            if phase.status == PhaseStatus::Skipped {
                continue;
            }
            lines.push(phase.format_display());
        }

        if self.show_timing {
            lines.push(format!("Total: {}ms", progress.total_elapsed_ms()));
        }

        lines.join("\n")
    }

    /// Render current status (for streaming).
    pub fn render_current(&self, progress: &PipelineProgress) -> String {
        if let Some(phase) = progress.current_phase {
            if let Some(p) = progress.get_phase(phase) {
                let elapsed = p.elapsed_ms();
                format!("{} {}ms...", phase.label(), elapsed)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

impl Default for ProgressRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_status() {
        assert!(PhaseStatus::Completed.is_done());
        assert!(PhaseStatus::Completed.is_success());
        assert!(!PhaseStatus::Running.is_done());
        assert!(!PhaseStatus::Failed.is_success());
    }

    #[test]
    fn test_phase_progress() {
        let mut progress = PhaseProgress::pending(Phase::JuniorSpecialist);
        assert_eq!(progress.status, PhaseStatus::Pending);

        progress.start();
        assert_eq!(progress.status, PhaseStatus::Running);

        progress.complete();
        assert_eq!(progress.status, PhaseStatus::Completed);
        assert!(progress.duration_ms.is_some());
    }

    #[test]
    fn test_pipeline_progress() {
        let mut progress = PipelineProgress::new();
        progress.start();

        progress.start_phase(Phase::TranslatorIntent);
        assert_eq!(progress.current_phase, Some(Phase::TranslatorIntent));

        progress.complete_phase(Phase::TranslatorIntent);
        assert!(progress.current_phase.is_none());
    }

    #[test]
    fn test_progress_renderer_compact() {
        let mut progress = PipelineProgress::new();
        progress.start();
        progress.start_phase(Phase::TranslatorIntent);
        progress.complete_phase(Phase::TranslatorIntent);

        let renderer = ProgressRenderer::new().compact();
        let output = renderer.render(&progress);

        assert!(output.contains("intent extraction"));
    }

    #[test]
    fn test_progress_renderer_full() {
        let mut progress = PipelineProgress::new();
        progress.start();
        progress.start_phase(Phase::TranslatorIntent);
        progress.complete_phase(Phase::TranslatorIntent);

        let renderer = ProgressRenderer::new();
        let output = renderer.render(&progress);

        assert!(output.contains("●"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_timeout_and_failure() {
        let mut progress = PipelineProgress::new();
        progress.start();

        progress.start_phase(Phase::JuniorSpecialist);
        progress.timeout_phase(Phase::JuniorSpecialist);
        assert!(progress.has_timeout());

        progress.start_phase(Phase::SeniorSpecialist);
        progress.fail_phase(Phase::SeniorSpecialist, "test error");
        assert!(progress.has_failure());
    }
}

//! Pipeline Progress (Part G.2) - v0.0.438.
//!
//! Overall pipeline progress tracking across all phases.

use std::time::Instant;

use crate::fast_pipeline::budget::Phase;
use crate::fast_pipeline::progress_types::{PhaseProgress, PhaseStatus};

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
        self.phases
            .iter()
            .any(|p| p.status == PhaseStatus::TimedOut)
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

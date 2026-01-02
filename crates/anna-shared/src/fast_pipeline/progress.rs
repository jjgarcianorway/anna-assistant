//! Progress Rendering (Part G) - v0.0.438.
//!
//! Show honest progress without lying or breaking parsing:
//! - Show what phase we're in
//! - Show elapsed time
//! - Don't claim "Analyzing..." if we're waiting on network
//! - Never inject progress text into parsed responses

// Re-export from sibling modules
pub use super::progress_pipeline::PipelineProgress;
pub use super::progress_renderer::ProgressRenderer;
pub use super::progress_types::{PhaseProgress, PhaseStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fast_pipeline::budget::Phase;

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

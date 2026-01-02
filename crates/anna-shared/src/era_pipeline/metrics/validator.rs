//! Pipeline validation and resolution checking.

use crate::era_pipeline::pipeline::{EraPipeline, PipelineStatus};

use super::types::{ResolutionCriteria, ResolutionStatus};

/// Default confidence threshold.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.6;

/// Validate pipeline result and return honest status.
pub fn validate_resolution(pipeline: &EraPipeline, confidence_threshold: f64) -> ResolutionStatus {
    // Check pipeline status first
    match pipeline.status() {
        PipelineStatus::Failed => return ResolutionStatus::Failed,
        PipelineStatus::InProgress(_) => return ResolutionStatus::InProgress,
        PipelineStatus::Complete => {}
    }

    // Check resolution criteria
    let criteria = ResolutionCriteria::from_pipeline(pipeline, confidence_threshold);
    criteria.status()
}

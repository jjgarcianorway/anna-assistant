//! Data extraction methods for the Hollywood renderer.

use super::super::types::{HollywoodTranscript, InternalComm, ProbeResult, ProbeStatus};
use super::core::HollywoodRenderer;
use crate::transcript_segment::SegmentKind;

/// Trait for extraction methods on HollywoodRenderer
pub(super) trait RendererExtractors {
    fn extract_internal_comms(&self, transcript: &HollywoodTranscript) -> Vec<InternalComm>;
    fn extract_probes(&self, transcript: &HollywoodTranscript) -> Vec<ProbeResult>;
    fn find_error_message(&self, transcript: &HollywoodTranscript) -> Option<String>;
}

impl RendererExtractors for HollywoodRenderer {
    /// Extract internal comms from transcript
    fn extract_internal_comms(&self, transcript: &HollywoodTranscript) -> Vec<InternalComm> {
        transcript
            .segments()
            .iter()
            .filter(|s| s.kind == SegmentKind::InternalComms)
            .map(|s| InternalComm::from_actor(&s.actor, &s.content, s.relative_secs))
            .collect()
    }

    /// Extract probes from transcript
    fn extract_probes(&self, transcript: &HollywoodTranscript) -> Vec<ProbeResult> {
        transcript
            .segments()
            .iter()
            .filter(|s| s.kind == SegmentKind::ProbeRun)
            .map(|s| {
                let name = s
                    .meta
                    .get("probe_id")
                    .map(|s| s.as_str())
                    .unwrap_or("probe");
                let status = s.meta.get("status").map(|s| s.as_str()).unwrap_or("ok");
                let duration: u64 = s
                    .meta
                    .get("duration_ms")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                let probe_status = match status {
                    "ok" | "success" => ProbeStatus::Ok,
                    "failed" | "error" => ProbeStatus::Failed,
                    "timeout" => ProbeStatus::Timeout,
                    _ => ProbeStatus::Ok,
                };

                ProbeResult {
                    name: name.to_string(),
                    command: s.meta.get("command").cloned(),
                    status: probe_status,
                    duration_ms: duration,
                    summary: Some(s.content.clone()),
                    raw_output: s.meta.get("raw_output").cloned(),
                }
            })
            .collect()
    }

    /// Find first error message in transcript
    fn find_error_message(&self, transcript: &HollywoodTranscript) -> Option<String> {
        transcript
            .segments()
            .iter()
            .find(|s| s.kind == SegmentKind::Error)
            .map(|s| s.content.clone())
    }
}

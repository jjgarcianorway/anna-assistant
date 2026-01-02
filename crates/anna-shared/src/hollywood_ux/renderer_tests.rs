//! Tests for Hollywood renderer (v0.0.431).

#[cfg(test)]
mod tests {
    use crate::hollywood_ux::renderer_helpers::{format_simple_answer, render_cinematic};
    use crate::hollywood_ux::types::HollywoodTranscript;
    use crate::transcript_segment::{staff, TranscriptSegment};

    #[test]
    fn test_render_simple_transcript() {
        let mut t = HollywoodTranscript::new("REQ-001", "how much free ram?");
        t.set_answer("You have 17.0 GiB free out of 31.0 GiB total (54% available).");
        t.add_evidence("/proc/meminfo");
        t.set_confidence(0.95);
        t.set_handler("Sofia", "Desktop");

        let rendered = render_cinematic(&t);

        assert!(rendered.contains("[you]"));
        assert!(rendered.contains("[anna]"));
        assert!(rendered.contains("17.0 GiB"));
        assert!(rendered.contains("Evidence:"));
        assert!(rendered.contains("95%"));
    }

    #[test]
    fn test_render_with_internal_comms() {
        let mut t = HollywoodTranscript::new("REQ-002", "why is my boot slow?");
        t.add(TranscriptSegment::internal_comms(
            staff::sofia(),
            "Checking boot services...",
        ));
        t.add(TranscriptSegment::internal_comms(
            staff::tomas(),
            "Found slow service: NetworkManager",
        ));
        t.set_answer("Your boot is slow due to NetworkManager taking 2.5s.");
        t.add_evidence("systemd-analyze");
        t.set_confidence(0.90);

        let rendered = render_cinematic(&t);

        assert!(rendered.contains("internal comms"));
        assert!(rendered.contains("Sofia"));
        assert!(rendered.contains("Tomas"));
        assert!(rendered.contains("NetworkManager"));
    }

    #[test]
    fn test_format_simple_answer() {
        let output = format_simple_answer(
            "what time is it?",
            "The current time is 14:32.",
            &["system clock"],
            Some(1.0),
        );

        assert!(output.contains("[you]"));
        assert!(output.contains("14:32"));
        assert!(output.contains("Evidence:"));
        assert!(output.contains("100%"));
    }
}

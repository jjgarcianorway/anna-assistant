//! Protocol Framing (Part A) - v0.0.436.
//!
//! All model outputs must be wrapped in exactly one framed block:
//! - Start: <<<ANNA_PROTO_V1>>>
//! - End: <<<END_ANNA_PROTO_V1>>>
//! - Inside: single JSON object, UTF-8, no trailing commentary

/// Protocol version identifier.
pub const PROTO_VERSION: &str = "V1";

/// Start token for protocol frame.
pub const PROTO_START: &str = "<<<ANNA_PROTO_V1>>>";

/// End token for protocol frame.
pub const PROTO_END: &str = "<<<END_ANNA_PROTO_V1>>>";

/// Result of frame extraction.
#[derive(Debug, Clone)]
pub enum FrameResult {
    /// Successfully found framed content.
    Found {
        /// The JSON content inside the frame.
        content: String,
        /// Byte position where frame started.
        start_pos: usize,
        /// Byte position where frame ended.
        end_pos: usize,
    },
    /// No frame markers found.
    NoFrame,
    /// Found start but no end.
    IncompleteFrame {
        /// Content after start marker (may be partial).
        partial_content: String,
    },
    /// Multiple frames found (protocol violation).
    MultipleFrames,
}

/// Extract the framed content from model output.
///
/// # Arguments
/// * `output` - Raw model output string
///
/// # Returns
/// * `FrameResult` indicating extraction status
pub fn extract_framed_content(output: &str) -> FrameResult {
    // Find all start markers
    let start_positions: Vec<usize> = output
        .match_indices(PROTO_START)
        .map(|(pos, _)| pos)
        .collect();

    // No frame found
    if start_positions.is_empty() {
        return FrameResult::NoFrame;
    }

    // Multiple frames (protocol violation)
    if start_positions.len() > 1 {
        return FrameResult::MultipleFrames;
    }

    let start_pos = start_positions[0];
    let content_start = start_pos + PROTO_START.len();

    // Find end marker after start
    if let Some(relative_end) = output[content_start..].find(PROTO_END) {
        let content_end = content_start + relative_end;
        let end_pos = content_end + PROTO_END.len();

        let content = output[content_start..content_end].trim().to_string();

        FrameResult::Found {
            content,
            start_pos,
            end_pos,
        }
    } else {
        // Start found but no end - incomplete
        let partial_content = output[content_start..].trim().to_string();
        FrameResult::IncompleteFrame { partial_content }
    }
}

/// Create a properly framed response.
///
/// # Arguments
/// * `json_content` - The JSON string to frame
///
/// # Returns
/// * Properly framed string
pub fn create_frame(json_content: &str) -> String {
    format!("{}\n{}\n{}", PROTO_START, json_content, PROTO_END)
}

/// Check if output contains a complete frame.
pub fn has_complete_frame(output: &str) -> bool {
    matches!(extract_framed_content(output), FrameResult::Found { .. })
}

/// Check if output has frame start but is incomplete.
pub fn has_incomplete_frame(output: &str) -> bool {
    matches!(
        extract_framed_content(output),
        FrameResult::IncompleteFrame { .. }
    )
}

/// Validate frame markers are balanced.
pub fn validate_frame_markers(output: &str) -> FrameValidation {
    let start_count = output.matches(PROTO_START).count();
    let end_count = output.matches(PROTO_END).count();

    match (start_count, end_count) {
        (0, 0) => FrameValidation::NoMarkers,
        (1, 1) => FrameValidation::Valid,
        (1, 0) => FrameValidation::MissingEnd,
        (0, 1) => FrameValidation::MissingStart,
        (s, e) if s > 1 || e > 1 => FrameValidation::MultipleMarkers { starts: s, ends: e },
        _ => FrameValidation::Unbalanced {
            starts: start_count,
            ends: end_count,
        },
    }
}

/// Frame validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameValidation {
    /// No markers present.
    NoMarkers,
    /// Exactly one start and one end.
    Valid,
    /// Start present but no end.
    MissingEnd,
    /// End present but no start.
    MissingStart,
    /// Multiple markers (protocol violation).
    MultipleMarkers { starts: usize, ends: usize },
    /// Unbalanced markers.
    Unbalanced { starts: usize, ends: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_valid_frame() {
        let output = r#"Some preamble text
<<<ANNA_PROTO_V1>>>
{"ok": true, "summary": "test"}
<<<END_ANNA_PROTO_V1>>>
Some trailing text"#;

        match extract_framed_content(output) {
            FrameResult::Found { content, .. } => {
                assert!(content.contains("ok"));
                assert!(content.contains("test"));
            }
            other => panic!("Expected Found, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_no_frame() {
        let output = "Just regular text without any frame markers";
        assert!(matches!(
            extract_framed_content(output),
            FrameResult::NoFrame
        ));
    }

    #[test]
    fn test_extract_incomplete_frame() {
        let output = "<<<ANNA_PROTO_V1>>>\n{\"ok\": true}";
        match extract_framed_content(output) {
            FrameResult::IncompleteFrame { partial_content } => {
                assert!(partial_content.contains("ok"));
            }
            other => panic!("Expected IncompleteFrame, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_multiple_frames() {
        let output = "<<<ANNA_PROTO_V1>>>{}<<<END_ANNA_PROTO_V1>>><<<ANNA_PROTO_V1>>>{}<<<END_ANNA_PROTO_V1>>>";
        assert!(matches!(
            extract_framed_content(output),
            FrameResult::MultipleFrames
        ));
    }

    #[test]
    fn test_create_frame() {
        let json = r#"{"ok": true}"#;
        let framed = create_frame(json);
        assert!(framed.contains(PROTO_START));
        assert!(framed.contains(PROTO_END));
        assert!(framed.contains(json));
    }

    #[test]
    fn test_has_complete_frame() {
        let complete = "<<<ANNA_PROTO_V1>>>{}<<<END_ANNA_PROTO_V1>>>";
        let incomplete = "<<<ANNA_PROTO_V1>>>{}";

        assert!(has_complete_frame(complete));
        assert!(!has_complete_frame(incomplete));
    }

    #[test]
    fn test_validate_frame_markers() {
        assert_eq!(
            validate_frame_markers("no markers"),
            FrameValidation::NoMarkers
        );
        assert_eq!(
            validate_frame_markers("<<<ANNA_PROTO_V1>>>{}<<<END_ANNA_PROTO_V1>>>"),
            FrameValidation::Valid
        );
        assert_eq!(
            validate_frame_markers("<<<ANNA_PROTO_V1>>>{}"),
            FrameValidation::MissingEnd
        );
    }
}

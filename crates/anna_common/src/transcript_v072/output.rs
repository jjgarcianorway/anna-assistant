//! Transcript File Output v0.0.72
//!
//! Writes transcript logs to case directories.
//! Both modes are generated from the same event stream.

use super::events::TranscriptStreamV72;
use super::render::{render_stream_v72, RenderedLineV72};
use super::validation::strip_ansi;
use crate::transcript_events::TranscriptMode;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Write both human and debug logs to case directory
pub fn write_case_logs_v72(stream: &TranscriptStreamV72, case_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(case_dir)?;
    write_debug_log_v72(stream, case_dir)?;
    write_human_log_v72(stream, case_dir)?;
    Ok(())
}

/// Write debug log (JSON events + rendered debug output)
pub fn write_debug_log_v72(stream: &TranscriptStreamV72, case_dir: &Path) -> std::io::Result<()> {
    let path = case_dir.join("transcript.debug.log");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "# Anna Debug Transcript")?;
    if let Some(case_id) = stream.case_id() {
        writeln!(writer, "# Case: {}", case_id)?;
    }
    writeln!(writer, "# Mode: Debug")?;
    writeln!(writer, "#")?;

    // Write rendered debug output
    let lines = render_stream_v72(stream, TranscriptMode::Debug);
    for line in &lines {
        let clean = strip_ansi(&line.text);
        writeln!(writer, "{}", clean)?;
    }

    writeln!(writer)?;
    writeln!(writer, "# Raw Events (JSON):")?;

    // Write JSON events for full fidelity
    for event in stream.events() {
        let json = serde_json::to_string(&event).unwrap_or_default();
        writeln!(writer, "{}", json)?;
    }

    writer.flush()?;
    Ok(())
}

/// Write human log (clean, no internals)
pub fn write_human_log_v72(stream: &TranscriptStreamV72, case_dir: &Path) -> std::io::Result<()> {
    let path = case_dir.join("transcript.human.log");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "# Anna Transcript")?;
    if let Some(case_id) = stream.case_id() {
        writeln!(writer, "# Case: {}", case_id)?;
    }
    writeln!(writer)?;

    // Write rendered human output
    let lines = render_stream_v72(stream, TranscriptMode::Human);
    for line in &lines {
        let clean = strip_ansi(&line.text);
        writeln!(writer, "{}", clean)?;
    }

    writer.flush()?;
    Ok(())
}

/// Print transcript to stdout based on current mode
pub fn print_transcript_v72(stream: &TranscriptStreamV72, mode: TranscriptMode) {
    let lines = render_stream_v72(stream, mode);
    for line in &lines {
        println!("{}", line.text);
    }
}

/// Get transcript as string for display
pub fn format_transcript_v72(stream: &TranscriptStreamV72, mode: TranscriptMode) -> String {
    let lines: Vec<String> = render_stream_v72(stream, mode)
        .into_iter()
        .map(|l| l.text)
        .collect();
    lines.join("\n")
}

/// Streaming output helper - writes a single rendered line
pub fn write_line_v72<W: Write>(writer: &mut W, line: &RenderedLineV72) -> std::io::Result<()> {
    if line.is_progress {
        // Progress lines can be overwritten (carriage return)
        write!(writer, "\r{}", line.text)?;
    } else {
        writeln!(writer, "{}", line.text)?;
    }
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::super::events::{EventDataV72, RoleV72, ToneV72};
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_both_logs() {
        let mut stream = TranscriptStreamV72::new().with_case_id("test-001");

        stream.push_data(EventDataV72::UserMessage {
            text: "What is my CPU?".to_string(),
        });

        stream.push_data(EventDataV72::Evidence {
            evidence_id: "E1".to_string(),
            tool_name: "hw_snapshot_summary".to_string(),
            human_label: "hardware inventory".to_string(),
            summary_human: "Intel i9-14900HX".to_string(),
            summary_debug: Some("cpu=Intel i9-14900HX cores=24".to_string()),
            duration_ms: 42,
        });

        stream.push_data(EventDataV72::StaffMessage {
            role: RoleV72::Anna,
            tone: ToneV72::Neutral,
            content_human: "You have an Intel i9-14900HX processor.".to_string(),
            content_debug: None,
        });

        let temp_dir = TempDir::new().unwrap();
        write_case_logs_v72(&stream, temp_dir.path()).unwrap();

        // Check human log
        let human_content =
            fs::read_to_string(temp_dir.path().join("transcript.human.log")).unwrap();
        assert!(human_content.contains("hardware inventory"));
        assert!(human_content.contains("Intel i9-14900HX"));
        assert!(!human_content.contains("[E1]"));
        assert!(!human_content.contains("hw_snapshot_summary"));

        // Check debug log
        let debug_content =
            fs::read_to_string(temp_dir.path().join("transcript.debug.log")).unwrap();
        assert!(debug_content.contains("[E1]"));
        assert!(debug_content.contains("hw_snapshot_summary"));
        assert!(debug_content.contains("42ms"));
    }

    #[test]
    fn test_format_transcript() {
        let mut stream = TranscriptStreamV72::new();
        stream.push_data(EventDataV72::UserMessage {
            text: "Hello".to_string(),
        });

        let human = format_transcript_v72(&stream, TranscriptMode::Human);
        let debug = format_transcript_v72(&stream, TranscriptMode::Debug);

        assert!(human.contains("Hello"));
        assert!(debug.contains("Hello"));
    }
}

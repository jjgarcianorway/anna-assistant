//! Hollywood UX acceptance tests (v0.0.431).

use super::*;
use crate::transcript_segment::{staff, TranscriptSegment};

/// Test scenario 1: Simple RAM query
/// annactl "how much free ram do I have right now?"
#[test]
fn test_simple_ram_query() {
    let mut t = HollywoodTranscript::new("REQ-RAM-001", "how much free ram do I have right now?");
    t.add_user_input("how much free ram do I have right now?");

    // Internal comms
    t.add(TranscriptSegment::internal_comms(
        staff::sofia(),
        "Checking memory status...",
    ));

    // Probe run
    let mut probe = TranscriptSegment::probe_run("proc_meminfo", "Read /proc/meminfo");
    probe.meta.insert("status".to_string(), "ok".to_string());
    probe
        .meta
        .insert("duration_ms".to_string(), "3".to_string());
    t.add(probe);

    // Answer
    t.set_answer(
        "You have 17.0 GiB free out of 31.0 GiB total (54% available).\n\n\
         The system has plenty of memory for normal use.",
    );
    t.add_evidence("/proc/meminfo");
    t.set_confidence(0.98);
    t.set_handler("Sofia", "Desktop");
    t.finalize();

    // Render
    let output = render_cinematic(&t);

    // Verify structure
    assert!(output.contains("[you]"), "Should have user header");
    assert!(output.contains("how much free ram"), "Should have query");
    assert!(output.contains("[anna]"), "Should have anna section");
    assert!(output.contains("17.0 GiB"), "Should have answer");
    assert!(output.contains("Evidence:"), "Should have evidence");
    assert!(output.contains("/proc/meminfo"), "Should cite source");
    assert!(output.contains("98%"), "Should have confidence");
    assert!(
        !output.contains("--- debug ---"),
        "Should not have debug in cinematic"
    );

    // Verify cleanliness
    assert!(!output.contains("null"), "Should not have JSON nulls");
    assert!(!output.contains("{"), "Should not have raw JSON");
}

/// Test scenario 2: Complex boot time query
/// annactl "why is my boot time so slow?"
#[test]
fn test_complex_boot_query() {
    let mut t = HollywoodTranscript::new("REQ-BOOT-001", "why is my boot time so slow?");
    t.add_user_input("why is my boot time so slow?");

    // Internal comms from Desktop team
    t.add(TranscriptSegment::internal_comms(
        staff::sofia(),
        "Checking boot timing analysis...",
    ));
    t.add(TranscriptSegment::internal_comms(
        staff::tomas(),
        "Running systemd-analyze to get breakdown.",
    ));
    t.add(TranscriptSegment::internal_comms(
        staff::sofia(),
        "Found the numbers. Analyzing components.",
    ));

    // Probes
    let mut probe1 = TranscriptSegment::probe_run("systemd_boot_time", "systemd-analyze");
    probe1.meta.insert("status".to_string(), "ok".to_string());
    probe1
        .meta
        .insert("duration_ms".to_string(), "132".to_string());
    t.add(probe1);

    let mut probe2 = TranscriptSegment::probe_run("systemd_blame", "systemd-analyze blame");
    probe2.meta.insert("status".to_string(), "ok".to_string());
    probe2
        .meta
        .insert("duration_ms".to_string(), "87".to_string());
    t.add(probe2);

    // Answer
    t.set_answer(
        "Your boot time is about 25.6 seconds:\n\
         * Firmware: 5.8s\n\
         * Bootloader: 8.3s\n\
         * Kernel: 5.0s\n\
         * Userspace: 6.3s\n\n\
         The main slow part is the bootloader (8.3s). On this system that's typical \
         and not alarming.\n\n\
         If you want to improve it, we can:\n\
         1) Disable unneeded boot entries.\n\
         2) Reduce timeout in your bootloader config.",
    );

    t.add_evidence("systemd-analyze");
    t.add_evidence("systemd-analyze blame");
    t.set_confidence(0.90);
    t.set_handler("Sofia", "Desktop");
    t.finalize();

    // Render cinematic
    let output = render_cinematic(&t);

    // Verify structure
    assert!(
        output.contains("internal comms"),
        "Should have internal comms section"
    );
    assert!(output.contains("Sofia"), "Should show Sofia");
    assert!(output.contains("Tomas"), "Should show Tomas");
    assert!(output.contains("[anna]"), "Should have answer section");
    assert!(output.contains("25.6 seconds"), "Should have timing");
    assert!(output.contains("Firmware"), "Should break down components");
    assert!(output.contains("Evidence:"), "Should have evidence");
    assert!(output.contains("90%"), "Should have confidence");

    // Now render debug mode
    let debug_output = render_debug(&t);
    assert!(
        debug_output.contains("--- debug ---"),
        "Debug should have debug section"
    );
    assert!(
        debug_output.contains("[probes raw]"),
        "Debug should show raw probes"
    );
    assert!(
        debug_output.contains("request_id"),
        "Debug should show request ID"
    );
}

/// Test scenario 3: Parse error handling
/// When LLM output can't be parsed
#[test]
fn test_parse_error_handling() {
    let mut t = HollywoodTranscript::new("REQ-ERR-001", "what's using my disk?");
    t.add_user_input("what's using my disk?");

    // Some internal comms before error
    t.add(TranscriptSegment::internal_comms(
        staff::lars(),
        "Analyzing disk usage...",
    ));

    // Probe ran but parse failed
    let mut probe = TranscriptSegment::probe_run("disk_usage", "df -h");
    probe.meta.insert("status".to_string(), "ok".to_string());
    probe
        .meta
        .insert("duration_ms".to_string(), "45".to_string());
    t.add(probe);

    // Set parse error
    t.set_parse_error("Could not interpret the disk analysis results.");
    t.add_evidence("df");
    t.finalize();

    let output = render_cinematic(&t);

    // Should show gentle error
    assert!(output.contains("[anna]"), "Should have anna section");
    assert!(
        output.contains("couldn't turn it into a reliable answer")
            || output.contains("Something went wrong"),
        "Should have gentle error message"
    );
    assert!(
        output.contains("Evidence:"),
        "Should still show what was collected"
    );

    // Should NOT claim success
    assert!(
        !output.contains("System Status") || output.contains("Parse Error"),
        "Should not claim success"
    );
}

/// Test visual consistency across modes
#[test]
fn test_mode_consistency() {
    let mut t = HollywoodTranscript::new("REQ-001", "test query");
    t.set_answer("Test answer");
    t.set_confidence(0.85);
    t.finalize();

    // Cinematic
    let cinematic = render_cinematic(&t);
    assert!(cinematic.contains("[you]"));
    assert!(cinematic.contains("[anna]"));
    assert!(!cinematic.contains("--- debug ---"));

    // Debug
    let debug = render_debug(&t);
    assert!(debug.contains("[you]"));
    assert!(debug.contains("[anna]"));
    assert!(debug.contains("--- debug ---"));
    assert!(debug.contains("request_id"));
}

/// Test storage round-trip
#[test]
fn test_storage_persistence() {
    let path = format!("/tmp/anna_hollywood_test_{}", std::process::id());
    let storage = TranscriptStorage::new(&path);

    // Create and save
    let mut t = HollywoodTranscript::new("REQ-STORE-001", "persistence test");
    t.set_answer("Stored answer");
    t.set_handler("Sofia", "Desktop");
    t.set_confidence(0.99);
    t.add_evidence("test_source");
    t.finalize();

    storage.save(&t).unwrap();

    // Load and verify
    let loaded = storage.load("REQ-STORE-001").unwrap();
    assert_eq!(loaded.user_query, "persistence test");
    assert!(loaded.final_answer.is_some());
    assert_eq!(loaded.handled_by, Some("Sofia".to_string()));
    assert_eq!(loaded.confidence, Some(0.99));
    assert!(loaded.evidence_sources.contains(&"test_source".to_string()));

    // Cleanup
    let _ = std::fs::remove_dir_all(&path);
}

/// Test header formatting
#[test]
fn test_header_formatting() {
    let header = styles::header_block("how much ram?", 40);

    // Should have separators
    let lines: Vec<&str> = header.lines().collect();
    assert!(lines.len() >= 3, "Should have separator, query, separator");

    // First and last lines should be separators
    assert!(
        lines[0].chars().all(|c| c == '-'),
        "Top should be separator"
    );
    assert!(
        lines[2].chars().all(|c| c == '-'),
        "Bottom should be separator"
    );

    // Middle should have query
    assert!(lines[1].contains("[you]"), "Should have user label");
    assert!(lines[1].contains("how much ram"), "Should have query");
}

/// Test evidence footer formatting
#[test]
fn test_evidence_footer() {
    let sources = vec!["source1".to_string(), "source2".to_string()];
    let footer = styles::evidence_footer(&sources);

    assert!(footer.contains("Evidence:"));
    assert!(footer.contains("source1"));
    assert!(footer.contains("source2"));
    assert!(footer.contains(", ")); // Comma-separated
}

/// Test status footer formatting
#[test]
fn test_status_footer() {
    let footer = styles::status_footer(
        "System Status",
        Some(0.95),
        Some("Sofia (Desktop Jr)"),
        true,
    );

    assert!(footer.contains("System Status"));
    assert!(footer.contains("95%"));
    assert!(footer.contains("Sofia"));
    assert!(footer.contains("verified"));
    assert!(footer.contains("|")); // Pipe separators
}

/// Test probe line formatting
#[test]
fn test_probe_formatting() {
    let line = styles::probe_line("systemd_boot_time", "ok", 132);

    assert!(line.contains("systemd_boot_time"));
    assert!(line.contains("ok"));
    assert!(line.contains("132ms"));
}

/// Test internal comms formatting
#[test]
fn test_internal_comms_formatting() {
    let line = styles::internal_comm_line(
        0.5,
        "Sofia (Desktop Administrator)",
        "Checking services...",
        true,
    );

    assert!(line.contains("[0.5s]"));
    assert!(line.contains("Sofia"));
    assert!(line.contains("Checking services"));
}

/// Test minimal render options
#[test]
fn test_minimal_render() {
    let mut t = HollywoodTranscript::new("REQ-MIN-001", "minimal test");
    t.add(TranscriptSegment::internal_comms(
        staff::sofia(),
        "Should not show",
    ));
    t.set_answer("Just the answer");
    t.finalize();

    let renderer = HollywoodRenderer::new(RenderOptions::minimal());
    let output = renderer.render(&t);

    assert!(output.contains("[you]"));
    assert!(output.contains("[anna]"));
    assert!(
        !output.contains("internal comms"),
        "Minimal should not show internal comms"
    );
}

/// Test streaming renderer states
#[test]
fn test_streaming_states() {
    let mut renderer = StreamingRenderer::cinematic();

    assert!(!renderer.header_printed);
    assert!(!renderer.spinner.active);
    assert_eq!(renderer.rendered_count, 0);

    // Simulate some state changes
    renderer.header_printed = true;
    renderer.internal_section_started = true;
    renderer.probe_buffer.push("test_probe".to_string());

    // Reset should clear all state
    renderer.reset();
    assert!(!renderer.header_printed);
    assert!(!renderer.internal_section_started);
    assert!(renderer.probe_buffer.is_empty());
}

/// Test complete transcript detection
#[test]
fn test_complete_detection() {
    let mut t = HollywoodTranscript::new("REQ-001", "test");
    let renderer = StreamingRenderer::cinematic();

    // Not complete without answer
    assert!(!renderer.is_transcript_complete(&t));

    // Complete with answer
    t.add(TranscriptSegment::answer("done"));
    assert!(renderer.is_transcript_complete(&t));
}

/// Test width constraints
#[test]
fn test_width_constraints() {
    let long_query = "a".repeat(200);
    let header = styles::header_block(&long_query, 80);

    // No line should exceed width
    for line in header.lines() {
        assert!(line.len() <= 80, "Line exceeds width: len={}", line.len());
    }
}

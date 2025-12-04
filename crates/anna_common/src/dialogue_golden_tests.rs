//! Golden Tests for Dialogue Renderer v0.0.56
//!
//! Snapshot tests ensuring transcript realism stays stable.
//! Tests assert:
//! - Contains natural language dialogue with correct actors
//! - Contains evidence IDs
//! - Contains explicit agree/disagree lines
//! - Contains no raw command spam

#[cfg(test)]
mod tests {
    use crate::case_engine::IntentType;
    use crate::case_file_v1::CaseFileV1;
    use crate::dialogue_renderer::*;
    use crate::doctor_registry::DoctorDomain;

    // ========================================================================
    // Helper to create test cases
    // ========================================================================

    fn create_cpu_query_case() -> CaseFileV1 {
        let mut case = CaseFileV1::new("cpu-query-001", "what cpu do i have");
        case.set_intent(IntentType::SystemQuery, 95);
        case.add_evidence(
            "E1",
            "hw_snapshot_cpu",
            "AMD Ryzen 7 5800X, 8 cores, 16 threads, 3.8GHz base",
            45,
            false,
        );
        case.set_response("You have an AMD Ryzen 7 5800X [E1]. It has 8 cores and 16 threads with a base clock of 3.8GHz.", 92);
        case.complete(true, None);
        case
    }

    fn create_memory_query_case() -> CaseFileV1 {
        let mut case = CaseFileV1::new("mem-query-001", "how much memory do i have");
        case.set_intent(IntentType::SystemQuery, 93);
        case.add_evidence(
            "E1",
            "memory_info",
            "Total: 32 GiB, Available: 24.5 GiB, Used: 7.5 GiB",
            32,
            false,
        );
        case.set_response(
            "You have 32 GiB of RAM [E1], with 24.5 GiB currently available.",
            90,
        );
        case.complete(true, None);
        case
    }

    fn create_wifi_diagnose_case() -> CaseFileV1 {
        let mut case = CaseFileV1::new("wifi-diag-001", "wifi keeps disconnecting");
        case.set_intent(IntentType::Diagnose, 88);
        case.set_problem_domain("network");
        case.set_doctor(DoctorDomain::Network, "networking_doctor", 92);
        case.add_evidence(
            "E1",
            "network_status",
            "wlan0: connected to HomeNetwork, signal -65dBm",
            55,
            false,
        );
        case.add_evidence(
            "E2",
            "nm_status",
            "NetworkManager running, connection active",
            40,
            false,
        );
        case.add_evidence(
            "E3",
            "journalctl_network",
            "3 disconnection events in last hour",
            120,
            false,
        );
        case.set_response("Your WiFi [E1] shows frequent disconnections [E3]. Signal strength is moderate at -65dBm. NetworkManager is running [E2]. Consider checking for interference or driver issues.", 85);
        case.complete(true, None);
        case
    }

    fn create_action_request_case() -> CaseFileV1 {
        let mut case = CaseFileV1::new("action-001", "disable NetworkManager");
        case.set_intent(IntentType::ActionRequest, 90);
        case.add_evidence(
            "E1",
            "systemd_service_probe",
            "NetworkManager.service: active, running",
            35,
            false,
        );
        case.set_response("NetworkManager is currently active [E1]. This is a MEDIUM risk operation. To disable, you must confirm with: I CONFIRM (medium risk)", 88);
        case.complete(true, None);
        case
    }

    // ========================================================================
    // SYSTEM_QUERY: what cpu do i have
    // ========================================================================

    #[test]
    fn test_golden_cpu_query_has_correct_actors() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // Must have [you], [anna], [translator], [junior], [annad]
        assert!(
            transcript.contains("[you] to [anna]"),
            "Missing user message"
        );
        assert!(
            transcript.contains("[anna] to [translator]"),
            "Missing anna to translator"
        );
        assert!(
            transcript.contains("[translator] to [anna]"),
            "Missing translator response"
        );
        assert!(
            transcript.contains("to [annad]"),
            "Missing message to annad"
        );
        assert!(transcript.contains("[annad]"), "Missing annad response");
        assert!(
            transcript.contains("[anna] to [junior]"),
            "Missing anna to junior"
        );
        assert!(
            transcript.contains("[junior] to [anna]"),
            "Missing junior response"
        );
        assert!(
            transcript.contains("[anna] to [you]"),
            "Missing final response"
        );
    }

    #[test]
    fn test_golden_cpu_query_has_evidence_ids() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(transcript.contains("[E1]"), "Missing evidence ID E1");
        assert!(
            transcript.contains("Evidence collected: [E1]"),
            "Missing evidence summary"
        );
    }

    #[test]
    fn test_golden_cpu_query_has_agree_disagree() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // Should have Anna acknowledging translator
        assert!(
            transcript.contains("Acknowledged") || transcript.contains("agree"),
            "Missing translator acknowledgment"
        );

        // Should have junior signoff
        assert!(
            transcript.contains("Ship it") || transcript.contains("Shipping"),
            "Missing junior signoff"
        );
    }

    #[test]
    fn test_golden_cpu_query_no_raw_command_spam() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // Should not contain raw shell commands
        assert!(!transcript.contains("$ "), "Contains raw command prompt");
        assert!(
            !transcript.contains("/proc/cpuinfo"),
            "Contains raw proc path"
        );
        assert!(!transcript.contains("cat /"), "Contains raw cat command");
    }

    #[test]
    fn test_golden_cpu_query_has_phase_separators() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(
            transcript.contains("----- triage -----"),
            "Missing triage separator"
        );
        assert!(
            transcript.contains("----- evidence -----"),
            "Missing evidence separator"
        );
        assert!(
            transcript.contains("----- verification -----"),
            "Missing verification separator"
        );
        assert!(
            transcript.contains("----- response -----"),
            "Missing response separator"
        );
    }

    #[test]
    fn test_golden_cpu_query_has_reliability_footer() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(
            transcript.contains("Reliability: 92%"),
            "Missing reliability score"
        );
        assert!(
            transcript.contains("Verified"),
            "Missing verification verdict"
        );
    }

    // ========================================================================
    // SYSTEM_QUERY: how much memory do i have
    // ========================================================================

    #[test]
    fn test_golden_memory_query_full_transcript() {
        let case = create_memory_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // Check complete structure
        assert!(transcript.contains("[you] to [anna]: how much memory"));
        assert!(transcript.contains("SYSTEM_QUERY"));
        assert!(transcript.contains("[E1]"));
        assert!(transcript.contains("memory_info"));
        assert!(transcript.contains("32 GiB"));
        assert!(transcript.contains("Reliability: 90%"));
    }

    // ========================================================================
    // DIAGNOSE: wifi keeps disconnecting
    // ========================================================================

    #[test]
    fn test_golden_diagnose_has_doctor_handoff() {
        let case = create_wifi_diagnose_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // Should have doctor selection phase
        assert!(
            transcript.contains("----- doctor-select -----"),
            "Missing doctor-select separator"
        );

        // Should mention the doctor department
        assert!(
            transcript.contains("[networking-doctor]"),
            "Missing networking-doctor actor"
        );

        // Should have handoff dialogue
        assert!(
            transcript.contains("DIAGNOSE case for Network"),
            "Missing DIAGNOSE handoff"
        );
        assert!(
            transcript.contains("You're up"),
            "Missing doctor activation"
        );
        assert!(
            transcript.contains("Pulling evidence"),
            "Missing doctor acknowledgment"
        );
    }

    #[test]
    fn test_golden_diagnose_has_multiple_evidence() {
        let case = create_wifi_diagnose_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(transcript.contains("[E1]"), "Missing E1");
        assert!(transcript.contains("[E2]"), "Missing E2");
        assert!(transcript.contains("[E3]"), "Missing E3");
        assert!(
            transcript.contains("Evidence collected: [E1, E2, E3]"),
            "Missing evidence summary"
        );
    }

    #[test]
    fn test_golden_diagnose_has_correct_actors() {
        let case = create_wifi_diagnose_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(transcript.contains("[you]"));
        assert!(transcript.contains("[anna]"));
        assert!(transcript.contains("[translator]"));
        assert!(transcript.contains("[networking-doctor]"));
        assert!(transcript.contains("[annad]"));
        assert!(transcript.contains("[junior]"));
    }

    // ========================================================================
    // ACTION_REQUEST: disable NetworkManager
    // ========================================================================

    #[test]
    fn test_golden_action_has_evidence() {
        let case = create_action_request_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(transcript.contains("[E1]"), "Missing evidence ID");
        assert!(
            transcript.contains("systemd_service_probe"),
            "Missing probe name"
        );
    }

    #[test]
    fn test_golden_action_mentions_confirmation() {
        let case = create_action_request_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // The response should mention confirmation requirement
        assert!(
            transcript.contains("MEDIUM risk") || transcript.contains("CONFIRM"),
            "Missing risk/confirmation mention in response"
        );
    }

    // ========================================================================
    // Low confidence / disagreement scenarios
    // ========================================================================

    #[test]
    fn test_low_confidence_shows_uncertainty() {
        let mut case = CaseFileV1::new("low-conf-001", "something ambiguous");
        case.set_intent(IntentType::SystemQuery, 55); // Low confidence
        case.add_evidence("E1", "hw_snapshot", "General system info", 50, false);
        case.set_response("Based on available information [E1]...", 70);
        case.complete(true, None);

        let transcript = render_dialogue_transcript(&case, 80);

        // Translator should express uncertainty
        assert!(
            transcript.contains("Not fully certain") || transcript.contains("not fully confident"),
            "Missing uncertainty expression"
        );

        // Anna should acknowledge carefully
        assert!(
            transcript.contains("carefully") || transcript.contains("conservative"),
            "Missing careful acknowledgment"
        );
    }

    #[test]
    fn test_junior_disagrees_on_low_reliability() {
        let mut case = CaseFileV1::new("low-rel-001", "what is happening");
        case.set_intent(IntentType::SystemQuery, 85);
        case.add_evidence("E1", "hw_snapshot", "Some info", 50, false);
        case.reliability_score = 60; // Below threshold
        case.set_response("Something is happening [E1].", 60);
        case.complete(true, None);

        let transcript = render_dialogue_transcript(&case, 80);

        // Junior should not say "Ship it"
        assert!(
            transcript.contains("Not good enough"),
            "Junior should disagree on low reliability"
        );

        // Anna should acknowledge disagreement
        assert!(
            transcript.contains("collect more") || transcript.contains("can't conclude"),
            "Anna should acknowledge junior's concerns"
        );
    }

    // ========================================================================
    // Format / structure tests
    // ========================================================================

    #[test]
    fn test_transcript_has_case_header() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        assert!(transcript.starts_with("=== Case:"), "Missing case header");
    }

    #[test]
    fn test_transcript_lines_are_readable() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // No line should be excessively long (allowing some margin)
        for line in transcript.lines() {
            assert!(
                line.len() <= 100,
                "Line too long: {} chars - '{}'",
                line.len(),
                &line[..line.len().min(50)]
            );
        }
    }

    #[test]
    fn test_transcript_no_empty_actor_blocks() {
        let case = create_cpu_query_case();
        let transcript = render_dialogue_transcript(&case, 80);

        // Should not have "[] to []:" or similar malformed lines
        assert!(!transcript.contains("[] "), "Contains empty actor");
        assert!(
            !transcript.contains("to []:"),
            "Contains empty target actor"
        );
    }
}

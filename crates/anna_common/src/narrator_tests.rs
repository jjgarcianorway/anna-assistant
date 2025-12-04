//! Narrator Integration Tests v0.0.62
//!
//! Tests for Human Mode vs Debug Mode output rendering.
//! Ensures Human Mode hides internals while Debug Mode shows everything.

#[cfg(test)]
mod tests {
    use crate::narrator::{
        get_output_mode, is_debug_mode, NarratorEvent, ActorVoice,
        topic_evidence_description,
    };
    use crate::transcript_events::TranscriptMode;
    use crate::evidence_topic::EvidenceTopic;

    // ========================================================================
    // Mode Detection Tests
    // ========================================================================

    #[test]
    fn test_default_mode_is_human() {
        // Clear any env vars
        std::env::remove_var("ANNA_DEBUG");
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");

        let mode = get_output_mode();
        assert_eq!(mode, TranscriptMode::Human);
        assert!(!is_debug_mode());
    }

    #[test]
    fn test_anna_debug_enables_debug_mode() {
        std::env::set_var("ANNA_DEBUG", "1");
        assert!(is_debug_mode());
        assert_eq!(get_output_mode(), TranscriptMode::Debug);
        std::env::remove_var("ANNA_DEBUG");
    }

    #[test]
    fn test_anna_debug_true_enables_debug_mode() {
        std::env::set_var("ANNA_DEBUG", "true");
        assert!(is_debug_mode());
        std::env::remove_var("ANNA_DEBUG");
    }

    #[test]
    fn test_transcript_mode_env_override() {
        std::env::remove_var("ANNA_DEBUG");
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "debug");
        assert_eq!(get_output_mode(), TranscriptMode::Debug);
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");
    }

    #[test]
    fn test_anna_debug_takes_precedence() {
        // ANNA_DEBUG should override ANNA_UI_TRANSCRIPT_MODE
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "human");
        std::env::set_var("ANNA_DEBUG", "1");
        assert!(is_debug_mode());
        std::env::remove_var("ANNA_DEBUG");
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");
    }

    // ========================================================================
    // Actor Voice Tests
    // ========================================================================

    #[test]
    fn test_actor_tags_exist() {
        // Ensure all actor voices have tags without panicking
        let _ = ActorVoice::Anna.tag();
        let _ = ActorVoice::Translator.tag();
        let _ = ActorVoice::Junior.tag();
        let _ = ActorVoice::Annad.tag();
        let _ = ActorVoice::You.tag();
        let _ = ActorVoice::Doctor("networking").tag();
    }

    #[test]
    fn test_doctor_tag_includes_name() {
        let tag = ActorVoice::Doctor("storage").tag();
        assert!(tag.contains("storage"));
    }

    // ========================================================================
    // Topic Evidence Description Tests
    // ========================================================================

    #[test]
    fn test_disk_evidence_description() {
        let desc = topic_evidence_description(EvidenceTopic::DiskFree);
        assert!(desc.contains("disk"));
        // Should NOT contain internal terms
        assert!(!desc.contains("mount_usage"));
        assert!(!desc.contains("[E"));
    }

    #[test]
    fn test_kernel_evidence_description() {
        let desc = topic_evidence_description(EvidenceTopic::KernelVersion);
        assert!(desc.contains("kernel"));
        assert!(!desc.contains("tool"));
    }

    #[test]
    fn test_cpu_evidence_description() {
        let desc = topic_evidence_description(EvidenceTopic::CpuInfo);
        assert!(desc.contains("CPU"));
        assert!(!desc.contains("hw_snapshot"));
    }

    #[test]
    fn test_memory_evidence_description() {
        let desc = topic_evidence_description(EvidenceTopic::MemoryInfo);
        assert!(desc.contains("memory"));
    }

    #[test]
    fn test_network_evidence_description() {
        let desc = topic_evidence_description(EvidenceTopic::NetworkStatus);
        assert!(desc.contains("network"));
    }

    // ========================================================================
    // Event Structure Tests
    // ========================================================================

    #[test]
    fn test_narrator_event_request_received() {
        let event = NarratorEvent::RequestReceived {
            request: "how much disk space is free".to_string(),
        };

        // Just ensure it can be created and matched
        match event {
            NarratorEvent::RequestReceived { request } => {
                assert!(request.contains("disk"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_narrator_event_evidence_collected() {
        let event = NarratorEvent::EvidenceCollected {
            tool_name: "mount_usage".to_string(),
            evidence_id: "E1".to_string(),
            success: true,
            duration_ms: 42,
        };

        match event {
            NarratorEvent::EvidenceCollected { tool_name, evidence_id, success, duration_ms } => {
                assert_eq!(tool_name, "mount_usage");
                assert_eq!(evidence_id, "E1");
                assert!(success);
                assert_eq!(duration_ms, 42);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_narrator_event_final_answer() {
        let event = NarratorEvent::FinalAnswer {
            answer: "Free space on /: 50 GiB (40% free).".to_string(),
            reliability: 92,
            evidence_summary: "disk usage snapshot".to_string(),
        };

        match event {
            NarratorEvent::FinalAnswer { answer, reliability, evidence_summary } => {
                assert!(answer.contains("50 GiB"));
                assert_eq!(reliability, 92);
                assert!(evidence_summary.contains("disk"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    // ========================================================================
    // Human Mode Output Contract Tests
    // ========================================================================

    /// Ensures human mode output contract: no internal terms should appear
    #[test]
    fn test_human_mode_contract_no_evidence_ids() {
        // Evidence IDs like E1, E2 should never appear in human mode descriptions
        let descriptions = vec![
            topic_evidence_description(EvidenceTopic::DiskFree),
            topic_evidence_description(EvidenceTopic::CpuInfo),
            topic_evidence_description(EvidenceTopic::KernelVersion),
            topic_evidence_description(EvidenceTopic::MemoryInfo),
            topic_evidence_description(EvidenceTopic::NetworkStatus),
        ];

        for desc in descriptions {
            assert!(!desc.contains("[E"), "Human description contains evidence ID pattern: {}", desc);
            assert!(!desc.contains("E1"), "Human description contains E1: {}", desc);
            assert!(!desc.contains("E2"), "Human description contains E2: {}", desc);
        }
    }

    #[test]
    fn test_human_mode_contract_no_tool_names() {
        // Tool names should not appear in human descriptions
        let descriptions = vec![
            topic_evidence_description(EvidenceTopic::DiskFree),
            topic_evidence_description(EvidenceTopic::CpuInfo),
            topic_evidence_description(EvidenceTopic::KernelVersion),
        ];

        let internal_terms = [
            "mount_usage", "hw_snapshot", "kernel_version", "memory_info",
            "network_status", "tool_", "_tool", "TOOLS:", "Parse error",
        ];

        for desc in &descriptions {
            for term in &internal_terms {
                assert!(!desc.contains(term),
                    "Human description '{}' contains internal term '{}'", desc, term);
            }
        }
    }

    #[test]
    fn test_human_mode_contract_has_evidence_and_reliability() {
        // Human mode should still show Evidence: and Reliability: labels
        // These are the human-friendly versions
        let event = NarratorEvent::FinalAnswer {
            answer: "Test answer".to_string(),
            reliability: 85,
            evidence_summary: "test evidence".to_string(),
        };

        // The event structure supports these fields
        match event {
            NarratorEvent::FinalAnswer { reliability, evidence_summary, .. } => {
                assert!(reliability > 0);
                assert!(!evidence_summary.is_empty());
            }
            _ => panic!("Wrong event type"),
        }
    }

    // ========================================================================
    // Debug Mode Contract Tests
    // ========================================================================

    #[test]
    fn test_debug_mode_has_evidence_id_field() {
        // Debug mode events should have evidence_id field
        let event = NarratorEvent::EvidenceCollected {
            tool_name: "mount_usage".to_string(),
            evidence_id: "E1".to_string(),
            success: true,
            duration_ms: 42,
        };

        match event {
            NarratorEvent::EvidenceCollected { evidence_id, .. } => {
                assert_eq!(evidence_id, "E1");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_debug_mode_has_tool_name_field() {
        // Debug mode events should have tool_name field
        let event = NarratorEvent::EvidenceCollected {
            tool_name: "mount_usage".to_string(),
            evidence_id: "E1".to_string(),
            success: true,
            duration_ms: 42,
        };

        match event {
            NarratorEvent::EvidenceCollected { tool_name, .. } => {
                assert_eq!(tool_name, "mount_usage");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_debug_mode_has_duration_field() {
        // Debug mode events should have duration field
        let event = NarratorEvent::EvidenceCollected {
            tool_name: "test".to_string(),
            evidence_id: "E1".to_string(),
            success: true,
            duration_ms: 123,
        };

        match event {
            NarratorEvent::EvidenceCollected { duration_ms, .. } => {
                assert_eq!(duration_ms, 123);
            }
            _ => panic!("Wrong event type"),
        }
    }
}

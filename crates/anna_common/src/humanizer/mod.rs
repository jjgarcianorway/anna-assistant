//! Humanizer v0.0.73 - Real IT Department Transcript Layer
//!
//! Makes the Human transcript read like a competent IT org operating in real time,
//! while remaining a faithful rendering of internal events.
//!
//! Key constraint: human transcript is produced from real event data, not improvised theater.
//!
//! ## Features (v0.0.73)
//! - Role-based natural phrasing per department
//! - Evidence labels with topic + source (no IDs): "CPU model (from hardware snapshot)"
//! - Doctor selection shows ownership and first check
//! - Tone-appropriate messages based on confidence
//!
//! ## v0.0.73 Role Phrasing Examples
//! - Service Desk: "I'm triaging the request and deciding who should handle it."
//! - Hardware: "Checking the latest hardware snapshot."
//! - Network: "Looking at link state and active connections."
//! - Anna (Lead): "Summarizing what we know and what we don't."
//!
//! ## Allowed Realism (reflects real uncertainty)
//! - "I can't confirm link state from snapshots alone; I need live signals."
//! - "Let's keep it read-only for now and gather more evidence."
//! - "Your draft is okay, but you didn't cite the storage evidence."
//!
//! ## NOT Allowed (fabrication)
//! - Pretending a command ran if it did not
//! - Claiming a file was read when snapshots didn't include it
//! - Inventing device names or numbers

pub mod labels;
pub mod phrases;
pub mod roles;
pub mod threads;
pub mod transform;

// Re-exports
pub use labels::{humanize_evidence, EvidenceSummary, HumanLabel};
pub use phrases::{
    phrase_case_open, phrase_evidence_label, phrase_evidence_start, phrase_finding_prefix,
    phrase_first_check, phrase_missing_evidence, phrase_summarizing, phrase_taking_ownership,
    HumanEvidenceLabel,
};
pub use roles::{ConfidenceHint, DepartmentTag, MessageTone, StaffRole};
pub use threads::{ThreadBuilder, ThreadSegment, ThreadType, ThreadedTranscript};
pub use transform::{
    humanize_case_open, humanize_caution, humanize_doctor_selection, humanize_evidence_gather,
    humanize_final_answer, humanize_finding, humanize_junior_critique, humanize_missing_evidence,
    humanize_triage, validate_answer_relevance, AnswerValidation, HumanizedMessage,
    HumanizerContext,
};

/// Standard transcript tags (v0.0.73)
pub mod tags {
    pub const SERVICE_DESK: &str = "service desk";
    pub const NETWORK: &str = "network";
    pub const STORAGE: &str = "storage";
    pub const PERFORMANCE: &str = "performance";
    pub const AUDIO: &str = "audio";
    pub const GRAPHICS: &str = "graphics";
    pub const BOOT: &str = "boot";
    pub const SECURITY: &str = "security";
    pub const INFO_DESK: &str = "info desk";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_workflow() {
        // Simulate a memory query workflow
        let mut builder = ThreadBuilder::new();
        let ctx = HumanizerContext {
            confidence: 85,
            ..Default::default()
        };

        // Case open
        builder.case_open(humanize_case_open("how much memory do I have", &ctx));

        // Triage
        builder.triage(humanize_triage(DepartmentTag::Performance, &[], &ctx));

        // Evidence gathering (side thread)
        builder.start_evidence(DepartmentTag::Performance);
        builder.evidence(humanize_evidence_gather(
            DepartmentTag::Performance,
            &HumanLabel::HardwareInventory,
            "32 GiB total, 45% used",
            &ctx,
        ));
        builder.end_evidence();

        // Finding
        builder.finding(humanize_finding(
            DepartmentTag::Performance,
            "Your system has 32 GiB of RAM with 45% currently used.",
            &ctx,
        ));

        // Final answer
        builder.final_answer(humanize_final_answer(
            "You have 32 GiB of RAM, with approximately 45% currently in use.",
            85,
            "direct hardware inventory evidence",
        ));

        let transcript = builder.build();
        let lines = transcript.render();

        // Should have service desk messages
        assert!(lines.iter().any(|l| l.contains("[service desk]")));

        // Should have performance messages
        assert!(lines.iter().any(|l| l.contains("[performance]")));

        // Should have reliability
        assert!(lines.iter().any(|l| l.contains("Reliability: 85%")));

        // Should NOT have tool names or evidence IDs
        assert!(!lines.iter().any(|l| l.contains("hw_snapshot")));
        assert!(!lines.iter().any(|l| l.contains("[E1]")));
    }

    #[test]
    fn test_missing_evidence_workflow() {
        let mut builder = ThreadBuilder::new();
        let ctx = HumanizerContext {
            confidence: 40,
            evidence_missing: true,
            ..Default::default()
        };

        builder.case_open(humanize_case_open("is my NVMe healthy", &ctx));
        builder.triage(humanize_triage(DepartmentTag::Storage, &[], &ctx));

        // Missing evidence
        builder.start_evidence(DepartmentTag::Storage);
        builder.evidence(humanize_missing_evidence(
            DepartmentTag::Storage,
            "NVMe health",
            &ctx,
        ));
        builder.end_evidence();

        // Caution
        builder.finding(humanize_caution("gather more evidence"));

        let transcript = builder.build();
        let lines = transcript.render();

        // v0.0.73: Storage dept uses role-specific phrasing
        // Should express uncertainty (either "doesn't have" or "can't confirm")
        assert!(
            lines
                .iter()
                .any(|l| l.contains("doesn't have") || l.contains("can't confirm")),
            "Expected uncertainty phrase in: {:?}",
            lines
        );
        assert!(lines.iter().any(|l| l.contains("read-only")));
    }

    #[test]
    fn test_answer_validation_prevents_wrong_topic() {
        // Memory query should not get CPU answer
        let result = validate_answer_relevance(
            "how much memory do I have",
            "Intel Core i9-14900HX with 24 cores",
        );
        assert!(matches!(result, AnswerValidation::WrongTopic { .. }));

        // Disk query should not get CPU answer
        let result = validate_answer_relevance(
            "how much disk space is free",
            "Your processor is an Intel i9",
        );
        assert!(matches!(result, AnswerValidation::WrongTopic { .. }));

        // Correct answers pass
        let result =
            validate_answer_relevance("how much memory do I have", "You have 32 GiB of RAM");
        assert_eq!(result, AnswerValidation::Ok);
    }

    #[test]
    fn test_human_labels_no_snapshot_suffix() {
        assert!(!HumanLabel::HardwareInventory
            .short_name()
            .contains("snapshot"));
        assert!(!HumanLabel::NetworkSignals.short_name().contains("snapshot"));
        assert!(!HumanLabel::StorageStatus.short_name().contains("snapshot"));
    }

    #[test]
    fn test_junior_critique_phrasing() {
        let ctx = HumanizerContext::default();
        let msg = humanize_junior_critique("hardware inventory", &ctx);
        assert!(msg.text.contains("didn't ground"));
        assert!(!msg.text.contains("[E")); // No evidence IDs
    }

    #[test]
    fn test_multi_department_workflow() {
        // Test network query that also involves performance
        let mut builder = ThreadBuilder::new();
        let ctx = HumanizerContext {
            confidence: 75,
            is_complex: true,
            ..Default::default()
        };

        builder.case_open(humanize_case_open("why is my network slow", &ctx));
        builder.triage(humanize_triage(
            DepartmentTag::Network,
            &[DepartmentTag::Performance],
            &ctx,
        ));

        // Network evidence
        builder.start_evidence(DepartmentTag::Network);
        builder.evidence(humanize_evidence_gather(
            DepartmentTag::Network,
            &HumanLabel::NetworkSignals,
            "wlan0 up, signal -65 dBm",
            &ctx,
        ));
        builder.end_evidence();

        // Performance evidence
        builder.start_evidence(DepartmentTag::Performance);
        builder.evidence(humanize_evidence_gather(
            DepartmentTag::Performance,
            &HumanLabel::SystemLoad,
            "CPU at 85%, possible throttling",
            &ctx,
        ));
        builder.end_evidence();

        let transcript = builder.build();
        let lines = transcript.render();

        // Should have both departments
        assert!(lines.iter().any(|l| l.contains("[network]")));
        assert!(lines.iter().any(|l| l.contains("[performance]")));

        // Triage should mention help from performance
        assert!(lines.iter().any(|l| l.contains("help from")));
    }

    #[test]
    fn test_tone_from_confidence() {
        // High confidence (>=85) = Brisk
        assert_eq!(MessageTone::from_confidence(90), MessageTone::Brisk);
        // Good confidence (>=70) = Neutral
        assert_eq!(MessageTone::from_confidence(75), MessageTone::Neutral);
        // Medium confidence (>=50) = Helpful
        assert_eq!(MessageTone::from_confidence(55), MessageTone::Helpful);
        // Low confidence (<50) = Skeptical
        assert_eq!(MessageTone::from_confidence(30), MessageTone::Skeptical);
    }

    #[test]
    fn test_department_tag_display_names() {
        assert_eq!(DepartmentTag::Network.display_name(), "network");
        assert_eq!(DepartmentTag::Storage.display_name(), "storage");
        assert_eq!(DepartmentTag::Performance.display_name(), "performance");
        assert_eq!(DepartmentTag::Audio.display_name(), "audio");
    }

    #[test]
    fn test_side_thread_indentation() {
        let mut builder = ThreadBuilder::new();
        let ctx = HumanizerContext::default();

        builder.case_open(humanize_case_open("disk check", &ctx));
        builder.start_evidence(DepartmentTag::Storage);
        builder.evidence(humanize_evidence_gather(
            DepartmentTag::Storage,
            &HumanLabel::StorageStatus,
            "250 GiB free on /",
            &ctx,
        ));
        builder.end_evidence();

        let transcript = builder.build();
        let lines = transcript.render();

        // Side thread messages should be indented (start with 2 spaces)
        let side_lines: Vec<_> = lines.iter().filter(|l| l.starts_with("  [")).collect();
        assert!(
            !side_lines.is_empty(),
            "Should have indented side thread messages"
        );
    }

    #[test]
    fn test_systemd_query_validation() {
        // Systemd running query with generic answer (no systemd/running/pid 1 keywords)
        let result =
            validate_answer_relevance("is systemd running", "Your computer is working normally.");
        assert!(matches!(result, AnswerValidation::TooGeneric { .. }));

        // Correct systemd answer passes
        let result =
            validate_answer_relevance("is systemd running", "systemd 256 is running as PID 1");
        assert_eq!(result, AnswerValidation::Ok);

        // Answer with just "running" still passes (mentions the key concept)
        let result = validate_answer_relevance("is systemd running", "Yes, it is running");
        assert_eq!(result, AnswerValidation::Ok);
    }

    #[test]
    fn test_evidence_summary_generators() {
        // CPU summary
        let cpu = EvidenceSummary::cpu("Intel i9-14900HX");
        assert!(cpu.contains("hardware inventory"));
        assert!(cpu.contains("i9-14900HX"));

        // Memory summary
        let mem = EvidenceSummary::memory(32.0, 45);
        assert!(mem.contains("32.0 GiB"));
        assert!(mem.contains("45%"));

        // Network summary
        let net = EvidenceSummary::network(2, true, "link up");
        assert!(net.contains("2 interface"));
        assert!(net.contains("manager present"));

        // Missing evidence
        let missing = EvidenceSummary::missing("audio");
        assert!(missing.contains("no audio evidence"));
    }
}

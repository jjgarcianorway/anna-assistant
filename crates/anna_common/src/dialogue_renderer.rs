//! Dialogue Renderer v0.0.57 - Fly-on-the-Wall IT Department Transcript
//!
//! Renders CaseEvents as natural-language dialogue with consistent actor voices.
//! Tone is applied at render-time only - logic layer produces structured events.
//!
//! Actor voices:
//! - [you]: literal user input (never altered)
//! - [anna]: calm senior admin, concise, honest, dry humor, pushes for evidence
//! - [translator]: service desk triage, brisk, checklist-driven
//! - [junior]: skeptical QA, calls out missing evidence, disagrees when warranted
//! - [annad]: robotic/operational, terse, structured

use crate::case_engine::{CaseActor, CaseEvent, CaseEventType, CasePhase, IntentType};
use crate::case_file_v1::CaseFileV1;
use crate::doctor_registry::DoctorDomain;

/// Reliability threshold for "ship it" vs "needs work"
pub const RELIABILITY_SHIP_THRESHOLD: u8 = 75;

/// Confidence threshold for translator certainty
pub const CONFIDENCE_CERTAIN_THRESHOLD: u8 = 80;

// ============================================================================
// Dialogue Context
// ============================================================================

/// Context for rendering dialogue (passed through rendering)
#[derive(Debug, Clone, Default)]
pub struct DialogueContext {
    /// Intent confidence from translator
    pub intent_confidence: u8,
    /// Whether translator used fallback
    pub translator_fallback: bool,
    /// Reliability score from junior
    pub reliability_score: Option<u8>,
    /// Whether junior disagreed
    pub junior_disagreed: bool,
    /// Missing evidence critique from junior
    pub missing_evidence: Option<String>,
    /// Selected doctor domain
    pub doctor_domain: Option<DoctorDomain>,
    /// Selected doctor ID
    pub doctor_id: Option<String>,
    /// Evidence IDs collected
    pub evidence_ids: Vec<String>,
}

// ============================================================================
// Phase Separators
// ============================================================================

/// Render a quiet phase separator
pub fn phase_separator(phase: CasePhase) -> String {
    let name = match phase {
        CasePhase::Intake => "intake",
        CasePhase::Triage => "triage",
        CasePhase::DoctorSelect => "doctor-select",
        CasePhase::EvidencePlan => "evidence-plan",
        CasePhase::EvidenceGather => "evidence",
        CasePhase::SynthesisDraft => "synthesis",
        CasePhase::JuniorVerify => "verification",
        CasePhase::Respond => "response",
        CasePhase::RecordCase => "record",
        CasePhase::LearnRecipe => "learning",
    };
    format!("----- {} -----", name)
}

// ============================================================================
// Actor Voice Rendering
// ============================================================================

/// Get doctor actor name based on domain
pub fn doctor_actor_name(domain: &DoctorDomain) -> &'static str {
    match domain {
        DoctorDomain::Network => "networking-doctor",
        DoctorDomain::Storage => "storage-doctor",
        DoctorDomain::Audio => "audio-doctor",
        DoctorDomain::Boot => "boot-doctor",
        DoctorDomain::Graphics => "graphics-doctor",
        DoctorDomain::System => "system-doctor",
    }
}

/// Render translator's classification with appropriate tone
pub fn render_translator_classification(
    intent: IntentType,
    confidence: u8,
    fallback_used: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    if fallback_used {
        lines.push("[translator] to [anna]: I'm not fully confident here. Parse failed, using fallback.".to_string());
    } else if confidence < CONFIDENCE_CERTAIN_THRESHOLD {
        lines.push(format!(
            "[translator] to [anna]: Classifying as {} ({}% confidence). Not fully certain.",
            intent, confidence
        ));
    } else {
        lines.push(format!(
            "[translator] to [anna]: Clear {} request. {}% confidence.",
            intent, confidence
        ));
    }

    lines
}

/// Render Anna's response to translator classification
pub fn render_anna_translator_response(confidence: u8, fallback_used: bool) -> String {
    if fallback_used {
        "[anna] to [translator]: Noted. Taking a conservative route.".to_string()
    } else if confidence < CONFIDENCE_CERTAIN_THRESHOLD {
        "[anna] to [translator]: Acknowledged. I'll proceed carefully.".to_string()
    } else {
        "[anna] to [translator]: Acknowledged, I agree.".to_string()
    }
}

/// Render doctor selection handoff
pub fn render_doctor_handoff(domain: &DoctorDomain, doctor_id: &str) -> Vec<String> {
    let doctor_name = doctor_actor_name(domain);
    let evidence_hint = match domain {
        DoctorDomain::Network => "link state, routes, DNS, NetworkManager status",
        DoctorDomain::Storage => "mounts, BTRFS health, SMART data",
        DoctorDomain::Audio => "PipeWire status, sinks, devices",
        DoctorDomain::Boot => "boot timing, slow units, journal errors",
        DoctorDomain::Graphics => "GPU info, compositor, Wayland/X11 state",
        DoctorDomain::System => "system overview, services, recent changes",
    };

    vec![
        format!("[translator] to [anna]: This looks like a DIAGNOSE case for {}.", domain),
        format!("[anna] to [{}]: You're up. Collect {}.", doctor_name, evidence_hint),
        format!("[{}] to [anna]: Got it. Pulling evidence now.", doctor_name),
    ]
}

/// Render evidence collection summary
pub fn render_evidence_summary(evidence_ids: &[String]) -> String {
    if evidence_ids.is_empty() {
        "[annad] to [anna]: No evidence collected.".to_string()
    } else {
        format!(
            "[annad] to [anna]: Evidence collected: [{}]",
            evidence_ids.join(", ")
        )
    }
}

/// Render a single evidence item from annad
pub fn render_evidence_item(evidence_id: &str, tool_name: &str, summary: &str) -> String {
    format!(
        "[annad]: [{}] {} -> {}",
        evidence_id, tool_name, summary
    )
}

/// Render junior verification result with QA signoff tone
pub fn render_junior_verification(
    reliability: u8,
    missing_evidence: Option<&str>,
    uncited_claims: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    if reliability >= 90 {
        lines.push(format!(
            "[junior] to [anna]: Reliability {}%. Solid evidence. Ship it.",
            reliability
        ));
    } else if reliability >= RELIABILITY_SHIP_THRESHOLD {
        lines.push(format!(
            "[junior] to [anna]: Reliability {}%. Acceptable. Ship it.",
            reliability
        ));
    } else {
        lines.push(format!(
            "[junior] to [anna]: Reliability {}%. Not good enough.",
            reliability
        ));

        if let Some(missing) = missing_evidence {
            lines.push(format!(
                "[junior] to [anna]: Missing evidence for: {}. Don't guess.",
                missing
            ));
        }

        if uncited_claims {
            lines.push("[junior] to [anna]: I see uncited claims. Need evidence IDs.".to_string());
        }
    }

    lines
}

/// Render junior coverage check (v0.0.57)
pub fn render_junior_coverage_check(
    coverage_percent: u8,
    missing_fields: &[String],
    target: &str,
) -> Vec<String> {
    let mut lines = Vec::new();

    if coverage_percent < 50 {
        lines.push(format!(
            "[junior] to [anna]: Coverage {}%. Evidence doesn't include {}.",
            coverage_percent, target
        ));
        if !missing_fields.is_empty() {
            lines.push(format!(
                "[junior] to [anna]: Missing: {}",
                missing_fields.join(", ")
            ));
        }
    } else if coverage_percent < 90 {
        lines.push(format!(
            "[junior] to [anna]: Coverage {}%. Some fields missing: {}.",
            coverage_percent,
            missing_fields.join(", ")
        ));
    }

    lines
}

/// Render Anna's response to low coverage (triggers retry)
pub fn render_anna_coverage_retry(target: &str, tool: &str) -> String {
    format!(
        "[anna] to [annad]: Coverage too low. Pulling {} for {} evidence.",
        tool, target
    )
}

/// Render Anna's response to junior verification
pub fn render_anna_junior_response(reliability: u8, disagreed: bool) -> String {
    if disagreed || reliability < RELIABILITY_SHIP_THRESHOLD {
        "[anna] to [junior]: Agreed. I'll either collect more or say we can't conclude.".to_string()
    } else {
        "[anna] to [junior]: Good. Shipping response.".to_string()
    }
}

/// Render the final response block
pub fn render_final_response(answer: &str, width: usize) -> Vec<String> {
    let mut lines = vec!["[anna] to [you]:".to_string()];
    for line in wrap_text(answer, width.saturating_sub(2)).lines() {
        lines.push(format!("  {}", line));
    }
    lines
}

/// Render reliability footer as QA signoff
pub fn render_reliability_footer(reliability: u8, duration_ms: u64) -> Vec<String> {
    let verdict = if reliability >= 90 {
        "Verified."
    } else if reliability >= RELIABILITY_SHIP_THRESHOLD {
        "Acceptable."
    } else {
        "Low confidence."
    };

    vec![
        String::new(),
        format!("Reliability: {}% - {}", reliability, verdict),
        format!("({}ms)", duration_ms),
    ]
}

// ============================================================================
// Full Transcript Rendering
// ============================================================================

/// Render a complete fly-on-the-wall transcript from CaseFileV1
pub fn render_dialogue_transcript(case: &CaseFileV1, width: usize) -> String {
    let width = width.max(60);
    let mut lines = Vec::new();

    // Header
    lines.push(format!("=== Case: {} ===", case.case_id));
    lines.push(String::new());

    // Intake: User request (literal, never altered)
    lines.push(phase_separator(CasePhase::Intake));
    lines.push(format!("[you] to [anna]: {}", case.request));
    lines.push(String::new());

    // Triage: Translator classification
    lines.push(phase_separator(CasePhase::Triage));
    lines.push("[anna] to [translator]: What are we looking at?".to_string());

    let translator_lines = render_translator_classification(
        case.intent,
        case.intent_confidence,
        false, // TODO: track fallback in case file
    );
    lines.extend(translator_lines);

    lines.push(render_anna_translator_response(case.intent_confidence, false));
    lines.push(String::new());

    // DoctorSelect (if DIAGNOSE)
    if case.intent == IntentType::Diagnose {
        if let (Some(domain), Some(doctor_id)) = (&case.doctor_domain, &case.doctor_id) {
            lines.push(phase_separator(CasePhase::DoctorSelect));
            lines.extend(render_doctor_handoff(domain, doctor_id));
            lines.push(String::new());
        }
    }

    // EvidenceGather
    if !case.evidence.is_empty() {
        lines.push(phase_separator(CasePhase::EvidenceGather));

        // If we have a doctor, they initiate the probes
        if let Some(domain) = &case.doctor_domain {
            let doctor_name = doctor_actor_name(domain);
            lines.push(format!("[{}] to [annad]: Run the probes.", doctor_name));
        } else {
            lines.push("[anna] to [annad]: Run the probes.".to_string());
        }

        for e in &case.evidence {
            lines.push(render_evidence_item(&e.id, &e.tool_name, &e.summary));
        }

        let evidence_ids: Vec<String> = case.evidence.iter().map(|e| e.id.clone()).collect();
        lines.push(render_evidence_summary(&evidence_ids));
        lines.push(String::new());
    }

    // JuniorVerify
    lines.push(phase_separator(CasePhase::JuniorVerify));
    lines.push("[anna] to [junior]: Check this before I ship it.".to_string());

    let junior_lines = render_junior_verification(
        case.reliability_score,
        None, // TODO: track missing evidence critique
        false,
    );
    lines.extend(junior_lines);

    lines.push(render_anna_junior_response(
        case.reliability_score,
        case.reliability_score < RELIABILITY_SHIP_THRESHOLD,
    ));
    lines.push(String::new());

    // Respond
    lines.push(phase_separator(CasePhase::Respond));
    if let Some(answer) = &case.final_answer {
        lines.extend(render_final_response(answer, width));
    }

    // Footer
    lines.extend(render_reliability_footer(case.reliability_score, case.duration_ms));

    lines.join("\n")
}

// ============================================================================
// Text Utilities
// ============================================================================

/// Wrap text to fit width
fn wrap_text(text: &str, width: usize) -> String {
    let width = width.max(40);
    let mut result = Vec::new();

    for line in text.lines() {
        if line.len() <= width {
            result.push(line.to_string());
        } else {
            let mut current = String::new();
            for word in line.split_whitespace() {
                if current.is_empty() {
                    current = word.to_string();
                } else if current.len() + 1 + word.len() <= width {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    result.push(current);
                    current = word.to_string();
                }
            }
            if !current.is_empty() {
                result.push(current);
            }
        }
    }

    result.join("\n")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_separator() {
        assert_eq!(phase_separator(CasePhase::Triage), "----- triage -----");
        assert_eq!(phase_separator(CasePhase::EvidenceGather), "----- evidence -----");
    }

    #[test]
    fn test_doctor_actor_name() {
        assert_eq!(doctor_actor_name(&DoctorDomain::Network), "networking-doctor");
        assert_eq!(doctor_actor_name(&DoctorDomain::Audio), "audio-doctor");
    }

    #[test]
    fn test_translator_high_confidence() {
        let lines = render_translator_classification(IntentType::SystemQuery, 95, false);
        assert!(lines[0].contains("Clear SYSTEM_QUERY"));
        assert!(lines[0].contains("95%"));
    }

    #[test]
    fn test_translator_low_confidence() {
        let lines = render_translator_classification(IntentType::Diagnose, 65, false);
        assert!(lines[0].contains("Not fully certain"));
    }

    #[test]
    fn test_translator_fallback() {
        let lines = render_translator_classification(IntentType::SystemQuery, 50, true);
        assert!(lines[0].contains("not fully confident"));
        assert!(lines[0].contains("fallback"));
    }

    #[test]
    fn test_anna_agrees() {
        let response = render_anna_translator_response(90, false);
        assert!(response.contains("Acknowledged, I agree"));
    }

    #[test]
    fn test_anna_conservative() {
        let response = render_anna_translator_response(60, true);
        assert!(response.contains("conservative"));
    }

    #[test]
    fn test_doctor_handoff() {
        let lines = render_doctor_handoff(&DoctorDomain::Network, "networking_doctor");
        assert!(lines[0].contains("DIAGNOSE"));
        assert!(lines[0].contains("Network"));
        assert!(lines[1].contains("[networking-doctor]"));
        assert!(lines[2].contains("Pulling evidence"));
    }

    #[test]
    fn test_junior_ship_it() {
        let lines = render_junior_verification(92, None, false);
        assert!(lines[0].contains("92%"));
        assert!(lines[0].contains("Ship it"));
    }

    #[test]
    fn test_junior_not_good_enough() {
        let lines = render_junior_verification(65, Some("disk free"), false);
        assert!(lines[0].contains("Not good enough"));
        assert!(lines[1].contains("disk free"));
        assert!(lines[1].contains("Don't guess"));
    }

    #[test]
    fn test_evidence_summary() {
        let ids = vec!["E1".to_string(), "E2".to_string(), "E3".to_string()];
        let summary = render_evidence_summary(&ids);
        assert!(summary.contains("[E1, E2, E3]"));
    }

    #[test]
    fn test_reliability_footer() {
        let lines = render_reliability_footer(92, 150);
        assert!(lines[1].contains("92%"));
        assert!(lines[1].contains("Verified"));
        assert!(lines[2].contains("150ms"));
    }
}

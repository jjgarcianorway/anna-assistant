//! Narrator v0.0.64 - IT Department Voice Layer
//!
//! v0.0.64: Enhanced with Service Desk ticketing and Doctor lifecycle
//! - Ticket opening messages with ID, category, severity
//! - Routing messages showing which team handles the request
//! - Doctor lifecycle stage updates
//! - IT department feel: like a fly-on-the-wall conversation
//!
//! Converts internal events into human-readable IT department dialogue.
//! The Narrator is the "voice" that describes what's happening without
//! exposing internal implementation details.
//!
//! In Human Mode, the Narrator:
//! - Never shows tool names like "hw_snapshot_summary"
//! - Never shows evidence IDs like "[E1]"
//! - Never shows raw prompts or JSON
//! - Describes what's being checked, not how
//! - Honestly narrates fallbacks and retries
//!
//! In Debug Mode, raw details are shown instead.

use crate::doctor_lifecycle::DoctorLifecycleStage;
use crate::evidence_topic::EvidenceTopic;
use crate::human_labels::{tool_evidence_desc, department_working_msg};
use crate::service_desk::{TicketCategory, TicketSeverity};
use crate::transcript_events::TranscriptMode;
use owo_colors::OwoColorize;
use std::io::{self, Write};

// ============================================================================
// Narrator Configuration
// ============================================================================

/// Get the effective output mode, checking CLI flag and env var
pub fn get_output_mode() -> TranscriptMode {
    // 1. Check ANNA_DEBUG env var (highest priority for debug)
    if std::env::var("ANNA_DEBUG").map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false) {
        return TranscriptMode::Debug;
    }

    // 2. Check ANNA_UI_TRANSCRIPT_MODE env var
    if let Ok(mode) = std::env::var("ANNA_UI_TRANSCRIPT_MODE") {
        return TranscriptMode::from_str(&mode);
    }

    // 3. Default to Human mode
    TranscriptMode::Human
}

/// Check if we're in debug mode
pub fn is_debug_mode() -> bool {
    get_output_mode().show_internals()
}

// ============================================================================
// Actor Voices (IT Department Tone)
// ============================================================================

/// Actor voice styles for Human Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorVoice {
    /// Anna: calm, competent coordinator
    Anna,
    /// Translator: brisk intake clerk
    Translator,
    /// Junior: skeptical QA reviewer
    Junior,
    /// Annad: mostly silent operations daemon
    Annad,
    /// Doctor: domain specialist
    Doctor(&'static str),
    /// User: the person asking
    You,
    /// v0.0.66: Service Desk (terse, structured)
    ServiceDesk,
    /// v0.0.66: Networking department (investigative, skeptical)
    Networking,
    /// v0.0.66: Storage department (cautious, rollback-minded)
    Storage,
    /// v0.0.66: Audio department (practical, stack-focused)
    Audio,
    /// v0.0.66: Graphics department (driver-aware, config-sensitive)
    Graphics,
    /// v0.0.66: Boot department (timeline-focused, service-aware)
    Boot,
    /// v0.0.66: Info Desk (general queries)
    InfoDesk,
}

impl ActorVoice {
    pub fn tag(&self) -> String {
        match self {
            ActorVoice::Anna => "[anna]".cyan().to_string(),
            ActorVoice::Translator => "[translator]".yellow().to_string(),
            ActorVoice::Junior => "[junior]".magenta().to_string(),
            ActorVoice::Annad => "[annad]".dimmed().to_string(),
            ActorVoice::Doctor(name) => format!("[{}]", name).green().to_string(),
            ActorVoice::You => "[you]".white().to_string(),
            ActorVoice::ServiceDesk => "[service desk]".cyan().to_string(),
            ActorVoice::Networking => "[networking]".blue().to_string(),
            ActorVoice::Storage => "[storage]".yellow().to_string(),
            ActorVoice::Audio => "[audio]".magenta().to_string(),
            ActorVoice::Graphics => "[graphics]".green().to_string(),
            ActorVoice::Boot => "[boot]".red().to_string(),
            ActorVoice::InfoDesk => "[info desk]".white().to_string(),
        }
    }

    /// v0.0.66: Get tone-appropriate prefix for department messages
    pub fn tone_prefix(&self) -> &'static str {
        match self {
            // Service Desk: calm, structured, slightly terse
            ActorVoice::ServiceDesk | ActorVoice::Anna => "",
            // Networking: investigative, skeptical, "prove it"
            ActorVoice::Networking => "",
            // Storage: cautious, rollback-minded
            ActorVoice::Storage => "",
            // Audio: practical, checks stack/services/devices
            ActorVoice::Audio => "",
            // Graphics: driver-aware, configuration-sensitive
            ActorVoice::Graphics => "",
            // Boot: timeline-focused, service-aware
            ActorVoice::Boot => "",
            // Others
            _ => "",
        }
    }
}

// ============================================================================
// v0.0.66: Department Tone Messages
// ============================================================================

/// Get department-specific investigation message
pub fn get_investigation_message(voice: ActorVoice) -> &'static str {
    match voice {
        ActorVoice::ServiceDesk | ActorVoice::Anna => "Opening a case and routing.",
        ActorVoice::Networking => "Checking link state, IP, route, and DNS.",
        ActorVoice::Storage => "Checking mount points, usage, and filesystem health.",
        ActorVoice::Audio => "Checking audio stack, services, and device configuration.",
        ActorVoice::Graphics => "Checking display configuration and driver status.",
        ActorVoice::Boot => "Analyzing boot timeline and service dependencies.",
        ActorVoice::InfoDesk => "Gathering relevant information.",
        _ => "Investigating.",
    }
}

/// Get department-specific summary prefix
pub fn get_summary_prefix(voice: ActorVoice) -> &'static str {
    match voice {
        ActorVoice::ServiceDesk | ActorVoice::Anna => "Summary:",
        ActorVoice::Networking => "Network status:",
        ActorVoice::Storage => "Storage status:",
        ActorVoice::Audio => "Audio status:",
        ActorVoice::Graphics => "Display status:",
        ActorVoice::Boot => "Boot analysis:",
        ActorVoice::InfoDesk => "Result:",
        _ => "Finding:",
    }
}

/// Format a department message with appropriate tone
pub fn format_department_message(voice: ActorVoice, message: &str) -> String {
    let tag = voice.tag();
    format!("  {} {}", tag, message)
}

// ============================================================================
// Narration Events
// ============================================================================

/// Events that the Narrator can describe
#[derive(Debug, Clone)]
pub enum NarratorEvent {
    /// Request received from user
    RequestReceived { request: String },

    // =========================================================================
    // v0.0.64: Service Desk Ticketing Events
    // =========================================================================

    /// Ticket opened by Service Desk
    TicketOpened {
        ticket_id: String,
        category: TicketCategory,
        severity: TicketSeverity,
    },

    /// Routing decision made
    RoutingDecision {
        /// Name of doctor/team handling this
        team_name: Option<String>,
        /// Whether using doctor flow
        use_doctor_flow: bool,
        /// Brief reason
        reason: String,
    },

    /// Doctor lifecycle stage update
    DoctorStage {
        doctor_name: String,
        stage: DoctorLifecycleStage,
        /// What the doctor is doing
        message: String,
    },

    // =========================================================================

    /// Topic identified (pre-LLM)
    TopicIdentified {
        topic: EvidenceTopic,
        confidence: u8,
    },

    /// Classification complete
    ClassificationComplete {
        intent: String,
        targets: Vec<String>,
        risk: String,
        confidence: u8,
        llm_backed: bool,
    },

    /// Translator fallback (LLM unavailable or failed)
    TranslatorFallback { reason: String },

    /// Doctor selected for problem
    DoctorSelected {
        doctor_name: String,
        reason: String,
    },

    /// Evidence gathering started
    EvidenceGathering { tools: Vec<String> },

    /// Single evidence item collected
    EvidenceCollected {
        tool_name: String,
        evidence_id: String,
        success: bool,
        duration_ms: u64,
    },

    /// Evidence summary (for Human Mode)
    EvidenceSummary { descriptions: Vec<String> },

    /// Junior verification result
    JuniorVerification {
        score: u8,
        critique: String,
        suggestions: String,
    },

    /// Junior disagreement/retry
    JuniorDisagrees { reason: String },

    /// Confirmation required
    ConfirmationRequired {
        risk_level: String,
        phrase: String,
        action_summary: String,
    },

    /// Final answer
    FinalAnswer {
        answer: String,
        reliability: u8,
        evidence_summary: String,
    },

    /// Rollback plan preview
    RollbackPreview { steps: Vec<String> },

    /// Warning or note
    Warning { message: String },

    /// Error occurred
    Error { message: String },

    /// Phase separator
    Phase { name: String },

    /// Working indicator
    Working { message: String },
}

// ============================================================================
// Narrator Output
// ============================================================================

/// Narrate an event based on current mode
pub fn narrate(event: &NarratorEvent) {
    let mode = get_output_mode();
    if mode.show_internals() {
        narrate_debug(event);
    } else {
        narrate_human(event);
    }
}

/// Narrate in Human Mode (professional IT dialogue)
fn narrate_human(event: &NarratorEvent) {
    match event {
        NarratorEvent::RequestReceived { request } => {
            println!();
            println!("  {} to {}: {}", ActorVoice::You.tag(), ActorVoice::Anna.tag(), request);
            println!();
        }

        // v0.0.64: Service Desk Ticketing Events
        NarratorEvent::TicketOpened { ticket_id, category, severity } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} Opening ticket #{}. Triage: {}. Severity: {}.",
                tag, ticket_id, category, severity);
        }

        NarratorEvent::RoutingDecision { team_name, use_doctor_flow, reason: _ } => {
            let tag = ActorVoice::Anna.tag();
            if *use_doctor_flow {
                if let Some(team) = team_name {
                    println!("  {} → {} Routing this to {} team.",
                        tag, ActorVoice::Anna.tag(), team);
                }
            }
            // For non-doctor flow, we don't mention routing explicitly
        }

        NarratorEvent::DoctorStage { doctor_name, stage: _, message } => {
            // Format doctor name into a tag
            let tag_str = format!("[{}]", doctor_name.to_lowercase().replace(' ', "_"));
            let doctor_tag = tag_str.green();
            println!("  {} {}", doctor_tag, message);
        }

        NarratorEvent::TopicIdentified { topic, confidence: _ } => {
            // In human mode, we don't show topic detection explicitly
            // It's implied by the evidence we gather
            let _ = topic; // Suppress unused warning
        }

        NarratorEvent::ClassificationComplete {
            intent,
            targets: _,
            risk,
            confidence,
            llm_backed,
        } => {
            let tag = ActorVoice::Translator.tag();
            // Keep it brief in human mode
            if *llm_backed {
                println!("  {} I've reviewed your request. {} query, {} risk. ({}% confident)",
                    tag, intent, risk, confidence);
            } else {
                println!("  {} Quick classification: {} query, {} risk.",
                    tag, intent, risk);
            }
            println!();
        }

        NarratorEvent::TranslatorFallback { reason } => {
            let tag = ActorVoice::Anna.tag();
            // Honest narration of fallback
            println!("  {} Note: {}, using quick classification instead.", tag, reason);
            println!();
        }

        NarratorEvent::DoctorSelected { doctor_name, reason } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} Routing to {} specialist: {}", tag, doctor_name, reason);
            println!();
        }

        NarratorEvent::EvidenceGathering { tools: _ } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} Let me gather the relevant information.", tag);
        }

        NarratorEvent::EvidenceCollected { tool_name, evidence_id: _, success, duration_ms: _ } => {
            // In human mode, we only mention failed evidence
            if !success {
                let tag = ActorVoice::Annad.tag();
                let desc = tool_evidence_desc(tool_name);
                println!("  {} Could not retrieve: {}", tag, desc);
            }
            // Successful evidence is summarized later
        }

        NarratorEvent::EvidenceSummary { descriptions } => {
            if !descriptions.is_empty() {
                let tag = ActorVoice::Annad.tag();
                let summary = descriptions.join("; ");
                println!("  {} Evidence: {}", tag, summary);
                println!();
            }
        }

        NarratorEvent::JuniorVerification { score, critique, suggestions: _ } => {
            let tag = ActorVoice::Junior.tag();
            let verdict = if *score >= 80 {
                "Verified.".green().to_string()
            } else if *score >= 50 {
                "Acceptable.".yellow().to_string()
            } else {
                "Low confidence.".red().to_string()
            };

            if critique.is_empty() || *score >= 80 {
                println!("  {} Reliability {}%. {}", tag, score, verdict);
            } else {
                println!("  {} Reliability {}%. {} {}", tag, score, verdict, critique);
            }
            println!();
        }

        NarratorEvent::JuniorDisagrees { reason } => {
            let tag = ActorVoice::Junior.tag();
            println!("  {} I disagree: {}", tag, reason.yellow());
            println!();
        }

        NarratorEvent::ConfirmationRequired { risk_level, phrase, action_summary } => {
            let tag = ActorVoice::Anna.tag();
            println!();
            println!("  {} This is a {} operation.", tag, risk_level.yellow());
            println!("  {}", action_summary);
            println!();
            println!("  To proceed, type: {}", phrase.bold());
            println!();
        }

        NarratorEvent::FinalAnswer { answer, reliability, evidence_summary } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} to {}: {}", tag, ActorVoice::You.tag(), answer);
            println!();

            // Show reliability with color
            let rel_str = format!("Reliability: {}%", reliability);
            if *reliability >= 80 {
                println!("  {}", rel_str.green());
            } else if *reliability >= 50 {
                println!("  {}", rel_str.yellow());
            } else {
                println!("  {}", rel_str.red());
            }

            // Evidence summary in human terms
            if !evidence_summary.is_empty() {
                println!("  Based on: {}", evidence_summary.dimmed());
            }
        }

        NarratorEvent::RollbackPreview { steps } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} If needed, I can undo this:", tag);
            for step in steps {
                println!("      - {}", step);
            }
            println!();
        }

        NarratorEvent::Warning { message } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} Note: {}", tag, message.yellow());
        }

        NarratorEvent::Error { message } => {
            let tag = ActorVoice::Anna.tag();
            println!("  {} Error: {}", tag, message.red());
        }

        NarratorEvent::Phase { name } => {
            let sep = format!("----- {} -----", name);
            println!("  {}", sep.dimmed());
        }

        NarratorEvent::Working { message } => {
            print!("\r  {} {}...", ActorVoice::Anna.tag(), message.dimmed());
            let _ = io::stdout().flush();
        }
    }
}

/// Narrate in Debug Mode (full internal details)
fn narrate_debug(event: &NarratorEvent) {
    let ts = chrono::Utc::now().format("%H:%M:%S%.3f");

    match event {
        NarratorEvent::RequestReceived { request } => {
            println!();
            println!("  {} {} [you] -> [anna]: {}", ts.to_string().dimmed(), "[request]".blue(), request);
            println!();
        }

        // v0.0.64: Service Desk Ticketing Events
        NarratorEvent::TicketOpened { ticket_id, category, severity } => {
            println!("  {} {} ticket_id={} category={:?} severity={:?}",
                ts.to_string().dimmed(), "[ticket]".blue(), ticket_id, category, severity);
        }

        NarratorEvent::RoutingDecision { team_name, use_doctor_flow, reason } => {
            let team_str = team_name.clone().unwrap_or_else(|| "(none)".to_string());
            println!("  {} {} team={} doctor_flow={} reason=\"{}\"",
                ts.to_string().dimmed(), "[routing]".blue(), team_str, use_doctor_flow, reason);
        }

        NarratorEvent::DoctorStage { doctor_name, stage, message } => {
            println!("  {} {} doctor={} stage={:?} message=\"{}\"",
                ts.to_string().dimmed(), "[doctor_stage]".green(), doctor_name, stage, message);
        }

        NarratorEvent::TopicIdentified { topic, confidence } => {
            println!("  {} {} topic={:?} confidence={}",
                ts.to_string().dimmed(), "[topic]".blue(), topic, confidence);
        }

        NarratorEvent::ClassificationComplete {
            intent,
            targets,
            risk,
            confidence,
            llm_backed,
        } => {
            let targets_str = if targets.is_empty() { "(none)".to_string() } else { targets.join(", ") };
            println!("  {} {} [translator]", ts.to_string().dimmed(), "[classification]".blue());
            println!("      INTENT: {}", intent);
            println!("      TARGETS: {}", targets_str);
            println!("      RISK: {}", risk);
            println!("      CONFIDENCE: {}%", confidence);
            println!("      LLM_BACKED: {}", llm_backed);
            println!();
        }

        NarratorEvent::TranslatorFallback { reason } => {
            println!("  {} {} reason={}",
                ts.to_string().dimmed(), "[fallback]".yellow(), reason);
        }

        NarratorEvent::DoctorSelected { doctor_name, reason } => {
            println!("  {} {} doctor={} reason=\"{}\"",
                ts.to_string().dimmed(), "[doctor]".green(), doctor_name, reason);
        }

        NarratorEvent::EvidenceGathering { tools } => {
            println!("  {} {} tools=[{}]",
                ts.to_string().dimmed(), "[evidence_start]".blue(), tools.join(", "));
        }

        NarratorEvent::EvidenceCollected { tool_name, evidence_id, success, duration_ms } => {
            let status = if *success { "OK".green().to_string() } else { "FAIL".red().to_string() };
            println!("  {} {} tool={} {} [{}] ({}ms)",
                ts.to_string().dimmed(), "[tool_result]".blue(),
                tool_name.cyan(), status, evidence_id.green(), duration_ms);
        }

        NarratorEvent::EvidenceSummary { descriptions } => {
            println!("  {} {} count={} items=[{}]",
                ts.to_string().dimmed(), "[evidence_summary]".blue(),
                descriptions.len(), descriptions.join("; "));
        }

        NarratorEvent::JuniorVerification { score, critique, suggestions } => {
            println!("  {} {} [junior]", ts.to_string().dimmed(), "[verification]".magenta());
            println!("      SCORE: {}%", score);
            if !critique.is_empty() {
                println!("      CRITIQUE: {}", critique);
            }
            if !suggestions.is_empty() {
                println!("      SUGGESTIONS: {}", suggestions);
            }
            println!();
        }

        NarratorEvent::JuniorDisagrees { reason } => {
            println!("  {} {} reason=\"{}\"",
                ts.to_string().dimmed(), "[junior_disagrees]".yellow(), reason);
        }

        NarratorEvent::ConfirmationRequired { risk_level, phrase, action_summary } => {
            println!("  {} {} risk={} phrase=\"{}\"",
                ts.to_string().dimmed(), "[confirmation]".yellow(), risk_level, phrase);
            println!("      ACTION: {}", action_summary);
        }

        NarratorEvent::FinalAnswer { answer, reliability, evidence_summary } => {
            println!("  {} {} [anna] -> [you]", ts.to_string().dimmed(), "[final]".green());
            println!("      ANSWER: {}", answer);
            println!("      RELIABILITY: {}%", reliability);
            println!("      EVIDENCE: {}", evidence_summary);
            println!();
        }

        NarratorEvent::RollbackPreview { steps } => {
            println!("  {} {} steps=[{}]",
                ts.to_string().dimmed(), "[rollback_plan]".blue(), steps.join("; "));
        }

        NarratorEvent::Warning { message } => {
            println!("  {} {} {}",
                ts.to_string().dimmed(), "[warning]".yellow(), message);
        }

        NarratorEvent::Error { message } => {
            println!("  {} {} {}",
                ts.to_string().dimmed(), "[error]".red(), message);
        }

        NarratorEvent::Phase { name } => {
            println!("  {} {} ----- {} -----",
                ts.to_string().dimmed(), "[phase]".blue(), name);
        }

        NarratorEvent::Working { message } => {
            println!("  {} {} {}",
                ts.to_string().dimmed(), "[working]".dimmed(), message);
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Clear working indicator line
pub fn clear_working() {
    print!("\r{}\r", " ".repeat(80));
    let _ = io::stdout().flush();
}

/// Show phase separator
pub fn phase(name: &str) {
    narrate(&NarratorEvent::Phase { name: name.to_string() });
}

/// Show working indicator
pub fn working(message: &str) {
    narrate(&NarratorEvent::Working { message: message.to_string() });
}

/// Get evidence topic description for human mode
pub fn topic_evidence_description(topic: EvidenceTopic) -> String {
    match topic {
        EvidenceTopic::CpuInfo => "CPU model and specifications".to_string(),
        EvidenceTopic::MemoryInfo => "memory usage and capacity".to_string(),
        EvidenceTopic::KernelVersion => "kernel version".to_string(),
        EvidenceTopic::DiskFree => "disk usage for mounted filesystems".to_string(),
        EvidenceTopic::NetworkStatus => "network connectivity status".to_string(),
        EvidenceTopic::AudioStatus => "audio device configuration".to_string(),
        EvidenceTopic::ServiceState => "service status check".to_string(),
        EvidenceTopic::RecentErrors => "recent system log entries".to_string(),
        EvidenceTopic::BootTime => "boot performance data".to_string(),
        EvidenceTopic::PackagesChanged => "recent package changes".to_string(),
        EvidenceTopic::GraphicsStatus => "GPU and display configuration".to_string(),
        EvidenceTopic::Alerts => "active system alerts".to_string(),
        EvidenceTopic::Unknown => "system information".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode_is_human() {
        // Clear any env vars that might affect this
        std::env::remove_var("ANNA_DEBUG");
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");

        let mode = get_output_mode();
        assert_eq!(mode, TranscriptMode::Human);
        assert!(!is_debug_mode());
    }

    #[test]
    fn test_anna_debug_env_enables_debug() {
        std::env::set_var("ANNA_DEBUG", "1");
        let mode = get_output_mode();
        assert_eq!(mode, TranscriptMode::Debug);
        assert!(is_debug_mode());
        std::env::remove_var("ANNA_DEBUG");
    }

    #[test]
    fn test_actor_tags() {
        // Just ensure they don't panic
        let _ = ActorVoice::Anna.tag();
        let _ = ActorVoice::Translator.tag();
        let _ = ActorVoice::Junior.tag();
        let _ = ActorVoice::Doctor("networking").tag();
    }

    #[test]
    fn test_topic_evidence_description() {
        let desc = topic_evidence_description(EvidenceTopic::DiskFree);
        assert!(desc.contains("disk"));

        let desc = topic_evidence_description(EvidenceTopic::KernelVersion);
        assert!(desc.contains("kernel"));
    }

    // v0.0.64: Test new narrator events
    #[test]
    fn test_narrator_event_ticket_opened() {
        use crate::service_desk::{TicketCategory, TicketSeverity};

        // Just verify it doesn't panic
        let event = NarratorEvent::TicketOpened {
            ticket_id: "A-20251204-0001".to_string(),
            category: TicketCategory::Networking,
            severity: TicketSeverity::High,
        };
        // Would print in real usage; just verify creation
        let _ = format!("{:?}", event);
    }

    #[test]
    fn test_narrator_event_routing_decision() {
        let event = NarratorEvent::RoutingDecision {
            team_name: Some("Network Doctor".to_string()),
            use_doctor_flow: true,
            reason: "Problem report routed to Network Doctor".to_string(),
        };
        let _ = format!("{:?}", event);
    }

    #[test]
    fn test_narrator_event_doctor_stage() {
        use crate::doctor_lifecycle::DoctorLifecycleStage;

        let event = NarratorEvent::DoctorStage {
            doctor_name: "Network Doctor".to_string(),
            stage: DoctorLifecycleStage::Intake,
            message: "Recording symptoms: connectivity issues".to_string(),
        };
        let _ = format!("{:?}", event);
    }
}

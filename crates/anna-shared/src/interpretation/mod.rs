//! Interpretation Mode - Close the feedback loop without adding power.
//!
//! v0.3.72: Interpretation Mode implementation.
//!
//! # Purpose
//!
//! Close the feedback loop between observation, external user actions, and future
//! behavior, without adding execution power, suggestions, or new capabilities.
//!
//! # Hard Constraints
//!
//! - No new commands
//! - No new execution paths
//! - No fixes, no suggestions, no "you could"
//! - No hallucinated causes or intent
//! - No user-visible gamification
//! - No proactive output unless explicitly triggered
//! - Anna remains a system component, not an assistant
//!
//! # Operates On Existing Data
//!
//! - Observation snapshots and hashes (SystemBaseline)
//! - Baseline diffs (BaselineChanges)
//! - Warning history (IssueStore)
//! - Session history (SessionStore)
//! - Outcome ledger (what was asked, what state changed later)
//!
//! # Responsibilities
//!
//! 1. **Recognition** - Detect when issues resolve
//! 2. **Attribution** - Determine actor (Anna/User/Unknown)
//! 3. **Learning** - Update silent competence record
//! 4. **Acknowledgment** - Only when explicitly asked
//!
//! # State Machine
//!
//! ```text
//! [Issue Detected] --> [Issue Active] --> [Issue Resolved]
//!                           |                    |
//!                           v                    v
//!                      [User Asked?]      [Attribute Actor]
//!                           |                    |
//!                           v                    v
//!                   [Track in Session]  [Anna/User/Unknown]
//!                                               |
//!                                               v
//!                                    [Update Competence Record]
//! ```
//!
//! # Explicit Non-Goals
//!
//! - Do not teach yet
//! - Do not reward yet
//! - Do not warn yet
//! - Do not optimize yet

mod recognition;
mod attribution;
mod competence;
mod acknowledgment;

pub use recognition::{detect_resolutions, Resolution, ResolutionEvent};
pub use attribution::{attribute_resolution, Actor, Attribution};
pub use competence::{CompetenceRecord, CompetenceEntry, load_competence, save_competence};
pub use acknowledgment::{is_resolution_inquiry, format_resolution_acknowledgment, format_no_resolution};

use crate::monitor::{IssueStore, IssueType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Interpretation Mode state - annotates, does not change behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterpretationState {
    /// Last time interpretation ran
    pub last_run: Option<DateTime<Utc>>,
    /// Resolutions detected since last run
    pub pending_resolutions: Vec<ResolutionEvent>,
    /// Whether interpretation mode is enabled
    pub enabled: bool,
}

impl InterpretationState {
    /// Run interpretation cycle on current state.
    /// Returns resolutions detected (if any).
    pub fn run_cycle(&mut self) -> Vec<ResolutionEvent> {
        if !self.enabled {
            return Vec::new();
        }

        // Detect resolutions
        let resolutions = detect_resolutions();

        // For each resolution, attribute and record
        for resolution in &resolutions {
            let attribution = attribute_resolution(resolution);

            // Update competence record (silent)
            if let Ok(mut record) = load_competence() {
                record.record(CompetenceEntry {
                    issue_type: resolution.issue_type.clone(),
                    resolution_observed: resolution.resolution.clone(),
                    actor: attribution.actor,
                    timestamp: Utc::now(),
                });
                let _ = save_competence(&record);
            }
        }

        self.last_run = Some(Utc::now());
        self.pending_resolutions = resolutions.clone();

        resolutions
    }
}

//------------------------------------------------------------------------------
// INTERNAL SPEC: Attribution Logic Pseudocode
//------------------------------------------------------------------------------
//
// function attribute_resolution(resolution: ResolutionEvent) -> Attribution:
//     // Check if Anna performed any action on this issue
//     if outcome_ledger.has_anna_action_for(resolution.issue_id):
//         action = outcome_ledger.get_last_action(resolution.issue_id)
//         if action.outcome == Resolved AND action.timestamp < resolution.detected_at:
//             return Attribution { actor: Anna, confidence: High, evidence: action }
//
//     // Check session history for user asking about this issue
//     if session_history.has_question_about(resolution.issue_type):
//         question = session_history.get_last_question(resolution.issue_type)
//         // User asked, then issue resolved - likely user action
//         if question.timestamp < resolution.detected_at:
//             return Attribution { actor: User, confidence: Medium, evidence: question }
//
//     // No evidence of who resolved it
//     return Attribution { actor: Unknown, confidence: None, evidence: None }
//
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// EXAMPLES: Allowed and Forbidden Outputs
//------------------------------------------------------------------------------
//
// === ALLOWED (when user asks "what changed" or "why is this resolved") ===
//
// User: "what changed with the group warning?"
// Anna: "The /etc/group warning is no longer active.
//        File hash now matches baseline.
//        Resolution attributed to: external action (not Anna).
//        [End of observation]"
//
// User: "why did the ssh warning go away?"
// Anna: "The sshd_config warning resolved.
//        Current state: file unchanged from new baseline.
//        Attribution: unknown - insufficient evidence.
//        [End of observation]"
//
// === FORBIDDEN ===
//
// Anna: "The group warning resolved. You probably ran usermod." (hallucinated cause)
// Anna: "The ssh warning is gone. Good job!" (reward/gamification)
// Anna: "Warning resolved. You should also check..." (suggestion)
// Anna: "I noticed the warning cleared up." (proactive, not triggered)
// Anna: "This might have been caused by..." (speculation)
//
//------------------------------------------------------------------------------

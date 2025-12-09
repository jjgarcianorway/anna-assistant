//! LLM-powered dialogue generation for natural specialist chatter (v0.0.265).
//!
//! v0.0.255: Added personality quirks for unique character voices.
//! v0.0.265: DISABLED LLM generation - small models produce nonsense.
//!           All functions now return None immediately, using static fallbacks.

use anna_shared::roster::PersonProfile;
// use anna_shared::roster::personality_for;
// use crate::ollama;
use tracing::debug;

/// Context for dialogue generation
pub struct DialogueContext<'a> {
    pub query: &'a str,
    pub case_id: &'a str,
    pub stage: DialogueStage,
    pub probe_count: Option<usize>,
    pub probe_success: Option<usize>,
    pub confidence: Option<u8>,
}

/// Which stage of the request we're generating dialogue for
#[derive(Debug, Clone, Copy)]
pub enum DialogueStage {
    Dispatch,
    Acknowledge,
    StartProbing,
    ProbesDone,
    Reviewing,
    Escalate,
    SeniorResponse,
    Done,
    AnnaReturning,
}

/// Generate Anna's dispatch message
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_dispatch(
    _model: &str,
    _junior: &PersonProfile,
    _case_id: &str,
    _query: &str,
) -> Option<String> {
    debug!("LLM dialogue disabled, using static fallback");
    None
}

/// Generate junior's acknowledgment
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_junior_ack(
    _model: &str,
    _junior: &PersonProfile,
    _query: &str,
) -> Option<String> {
    None
}

/// Generate junior's probing message
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_junior_probing(
    _model: &str,
    _junior: &PersonProfile,
    _probe_count: usize,
) -> Option<String> {
    None
}

/// Generate junior's probes done message
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_junior_probes_done(
    _model: &str,
    _junior: &PersonProfile,
    _success_count: usize,
    _planned_count: usize,
) -> Option<String> {
    None
}

/// Generate junior's reviewing message
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_junior_reviewing(
    _model: &str,
    _junior: &PersonProfile,
) -> Option<String> {
    None
}

/// Generate junior's done message
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_junior_done(
    _model: &str,
    _junior: &PersonProfile,
    _confidence: u8,
) -> Option<String> {
    None
}

/// Generate Anna's returning message
/// v0.0.265: DISABLED - returns None, uses static fallback
pub async fn gen_anna_returning(
    _model: &str,
    _junior: &PersonProfile,
) -> Option<String> {
    None
}

// v0.0.265: All LLM dialogue generation disabled
// The small translator models (qwen2.5:0.5b) produce nonsense
// Static fallbacks in messages.rs are used instead

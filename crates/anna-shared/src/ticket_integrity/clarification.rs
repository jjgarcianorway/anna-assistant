//! Clarification Before Probes (Part 2) - v0.0.442.
//!
//! Enforce Fact-First Clarification Loop (FCL):
//! - For config-like intents, check required facts FIRST
//! - If facts missing → clarify BEFORE running probes
//! - No probes until facts are known
//!
//! WRONG ORDER: probes → clarification → answer
//! RIGHT ORDER: check facts → clarify if missing → probes → answer

pub mod clarification_decision;
pub mod clarification_facts;
pub mod clarification_types;

#[cfg(test)]
mod clarification_tests;

// Re-export all public types and functions
pub use clarification_decision::{check_clarification_needed, is_clarification_required_intent};
pub use clarification_facts::{FactSource, KnownFact, KnownFacts};
pub use clarification_types::{
    ClarificationDecision, ClarificationOption, ClarificationQuestion, ClarificationRequiredIntent,
};

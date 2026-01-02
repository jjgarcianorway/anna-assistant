//! Core fallback engine logic.

use std::collections::HashMap;

use crate::specialist_v2::answer::AnswerType;
use crate::specialist_v2::schema::{SpecialistResponseV2, SpecialistStatus};

use super::handlers::{
    try_boot_fallback, try_disk_fallback, try_memory_fallback, try_network_fallback,
    try_services_fallback, try_swap_fallback, try_uptime_fallback,
};

/// Result from fallback engine
#[derive(Debug, Clone)]
pub struct FallbackResult {
    /// The generated response
    pub response: SpecialistResponseV2,
    /// Whether fallback was successful
    pub success: bool,
    /// Reason for fallback
    pub reason: String,
}

/// Fallback engine for generating deterministic answers from probe data
pub struct FallbackEngine {
    /// Probe results available
    probes: HashMap<String, String>,
    /// Intent string
    intent: String,
    /// Original question (kept for future use in more complex fallbacks)
    #[allow(dead_code)]
    question: String,
}

impl FallbackEngine {
    /// Create a new fallback engine
    pub fn new(intent: &str, question: &str, probes: HashMap<String, String>) -> Self {
        Self {
            probes,
            intent: intent.to_string(),
            question: question.to_string(),
        }
    }

    /// Try to generate a fallback response
    pub fn try_fallback(&self, reason: &str) -> FallbackResult {
        let _answer_type = AnswerType::from_intent(&self.intent);

        // Try specific fallbacks based on intent
        let response = self
            .try_intent_specific_fallback()
            .unwrap_or_else(|| self.generic_fallback(reason));

        FallbackResult {
            success: response.status == SpecialistStatus::Ok,
            response,
            reason: reason.to_string(),
        }
    }

    /// Try fallback based on intent keywords
    fn try_intent_specific_fallback(&self) -> Option<SpecialistResponseV2> {
        if self.intent.contains("memory") || self.intent.contains("ram") {
            return try_memory_fallback(&self.probes);
        }

        if self.intent.contains("service") || self.intent.contains("failed") {
            return try_services_fallback(&self.probes);
        }

        if self.intent.contains("disk")
            || self.intent.contains("storage")
            || self.intent.contains("space")
        {
            return try_disk_fallback(&self.probes);
        }

        if self.intent.contains("network")
            || self.intent.contains("interface")
            || self.intent.contains("ip")
        {
            return try_network_fallback(&self.probes);
        }

        if self.intent.contains("swap") {
            return try_swap_fallback(&self.probes);
        }

        if self.intent.contains("uptime") || self.intent.contains("running") {
            return try_uptime_fallback(&self.probes);
        }

        if self.intent.contains("boot") {
            return try_boot_fallback(&self.probes);
        }

        None
    }

    /// Generic fallback when no specific handler matches
    fn generic_fallback(&self, reason: &str) -> SpecialistResponseV2 {
        // If we have probe data, build a generic summary
        if !self.probes.is_empty() {
            let probe_names: Vec<_> = self.probes.keys().collect();
            return SpecialistResponseV2::insufficient_evidence(&format!(
                "I had trouble processing the response. Available probe data: {}",
                probe_names
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .with_notes(reason);
        }

        SpecialistResponseV2::insufficient_evidence(
            "I couldn't collect enough data to answer this. Try again or run: anna status",
        )
        .with_notes(reason)
    }
}

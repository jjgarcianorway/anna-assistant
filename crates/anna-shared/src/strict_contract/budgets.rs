//! Time budgets for the specialist pipeline
//!
//! This module defines timeout configurations for different parts
//! of the specialist execution pipeline.

/// Time budgets for the pipeline (in milliseconds)
#[derive(Debug, Clone, Copy)]
pub struct TimeBudgets {
    /// Translator LLM call max time
    pub translator_ms: u64,
    /// Specialist LLM call max time
    pub specialist_ms: u64,
    /// Individual probe max time
    pub probe_ms: u64,
    /// Total probes combined max time
    pub probes_total_ms: u64,
    /// Knowledge query max time
    pub knowledge_ms: u64,
}

impl Default for TimeBudgets {
    fn default() -> Self {
        Self {
            translator_ms: 1500,   // 1.5s
            specialist_ms: 4000,   // 4s
            probe_ms: 3000,        // 3s per probe
            probes_total_ms: 5000, // 5s total for all probes
            knowledge_ms: 500,     // 500ms for knowledge queries
        }
    }
}

impl TimeBudgets {
    /// Aggressive budgets for fast responses
    pub fn fast() -> Self {
        Self {
            translator_ms: 1000,
            specialist_ms: 3000,
            probe_ms: 2000,
            probes_total_ms: 4000,
            knowledge_ms: 300,
        }
    }

    /// Relaxed budgets for complex queries
    pub fn thorough() -> Self {
        Self {
            translator_ms: 2000,
            specialist_ms: 6000,
            probe_ms: 4000,
            probes_total_ms: 8000,
            knowledge_ms: 1000,
        }
    }
}

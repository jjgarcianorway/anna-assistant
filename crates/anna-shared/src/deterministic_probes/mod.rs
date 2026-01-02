//! Deterministic Probe Mapping (v0.0.448).
//!
//! CORE PRINCIPLE: Common intents get deterministic probes, not LLM guesses.
//!
//! This module maps specific question intents to exact probe sets.
//! The translator should check this FIRST before falling back to domain-based selection.
//!
//! Why deterministic:
//! - "which service uses most CPU?" → must run top_cpu, NOT cpu_info
//! - "do I have swap?" → must run swap_files, NOT pacman -Q swap
//! - "what is my vim setup?" → must run vimrc_content + nvim_config, NOT memory_info

mod types;
mod rules_cpu;
mod rules_memory;
mod rules_system;
mod rules_hardware;
mod rules_storage;
mod rules_network;
mod rules_config;
mod rules_misc;
mod registry;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::ProbeRule;
pub use registry::{
    DeterministicProbeRegistry,
    deterministic_probes_for_query,
    is_concept_query,
};

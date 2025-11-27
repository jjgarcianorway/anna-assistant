//! System prompts for LLM-A and LLM-B
//!
//! v0.14.0: Aligned to Reality (6 real probes)
//! v0.15.0: Junior/Senior architecture with dynamic checks
//!
//! The v0.14.0 prompts use a static probe catalog.
//! The v0.15.0 prompts use dynamic checks with risk classification.

// v0.14.0 - Static probe catalog (current stable)
pub mod llm_a;
pub mod llm_b;

// v0.15.0 - Junior/Senior with dynamic checks (next iteration)
pub mod llm_a_v15;
pub mod llm_b_v15;

// Re-export v0.14.0 (current stable)
pub use llm_a::{generate_llm_a_prompt, generate_llm_a_prompt_with_iteration, LLM_A_SYSTEM_PROMPT};
pub use llm_b::{generate_llm_b_prompt, LLM_B_SYSTEM_PROMPT};

// Re-export v0.15.0 (next iteration)
pub use llm_a_v15::{build_llm_a_request, generate_llm_a_prompt_v15, LLM_A_SYSTEM_PROMPT_V15};
pub use llm_b_v15::{build_llm_b_request, generate_llm_b_prompt_v15, LLM_B_SYSTEM_PROMPT_V15};

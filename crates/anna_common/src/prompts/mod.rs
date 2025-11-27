//! System prompts for LLM-A and LLM-B
//!
//! v0.14.0: Aligned to Reality (6 real probes)
//! v0.15.0: Junior/Senior architecture with dynamic checks
//! v0.18.0: Step-by-step orchestration (one action per iteration)
//!
//! The v0.14.0 prompts use a static probe catalog.
//! The v0.15.0 prompts use dynamic checks with risk classification.
//! The v0.18.0 prompts use step-by-step protocol with clear actions.

// v0.14.0 - Static probe catalog (legacy)
pub mod llm_a;
pub mod llm_b;

// v0.15.0 - Junior/Senior with dynamic checks (legacy)
pub mod llm_a_v15;
pub mod llm_b_v15;

// v0.18.0 - Step-by-step orchestration (current)
pub mod llm_a_v18;
pub mod llm_b_v18;

// Re-export v0.14.0 (legacy)
pub use llm_a::{generate_llm_a_prompt, generate_llm_a_prompt_with_iteration, LLM_A_SYSTEM_PROMPT};
pub use llm_b::{generate_llm_b_prompt, LLM_B_SYSTEM_PROMPT};

// Re-export v0.15.0 (legacy)
pub use llm_a_v15::{build_llm_a_request, generate_llm_a_prompt_v15, LLM_A_SYSTEM_PROMPT_V15};
pub use llm_b_v15::{build_llm_b_request, generate_llm_b_prompt_v15, LLM_B_SYSTEM_PROMPT_V15};

// Re-export v0.18.0 (current)
pub use llm_a_v18::{generate_junior_prompt, LLM_A_SYSTEM_PROMPT_V18};
pub use llm_b_v18::{generate_senior_prompt, LLM_B_SYSTEM_PROMPT_V18};

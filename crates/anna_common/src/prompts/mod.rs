//! System prompts for LLM-A and LLM-B v0.13.0
//!
//! v0.13.0: Strict Evidence Discipline
//! - Hard rule: "If there is no probe, you do not know it"
//! - Mandatory fix_and_accept for contradictions
//! - Intent mapping for common questions

pub mod llm_a;
pub mod llm_b;

pub use llm_a::{generate_llm_a_prompt, generate_llm_a_prompt_with_iteration, LLM_A_SYSTEM_PROMPT};
pub use llm_b::{generate_llm_b_prompt, LLM_B_SYSTEM_PROMPT};

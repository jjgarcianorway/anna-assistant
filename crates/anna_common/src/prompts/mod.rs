//! System prompts for LLM-A and LLM-B v0.10.0

pub mod llm_a;
pub mod llm_b;

pub use llm_a::{generate_llm_a_prompt, LLM_A_SYSTEM_PROMPT};
pub use llm_b::{generate_llm_b_prompt, LLM_B_SYSTEM_PROMPT};

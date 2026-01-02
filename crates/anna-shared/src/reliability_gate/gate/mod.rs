//! Hard Reliability Gate (v0.0.447).
//!
//! The gate that decides: show answer or show failure.
//!
//! Checks (ALL must pass):
//! 1. No timeout occurred
//! 2. No parsing errors occurred
//! 3. Not a generic/fallback answer
//! 4. Confidence >= 0.85 (configurable)
//! 5. At least 1 claim exists
//! 6. Every claim has ≥1 evidence item
//! 7. Question match (answer matches question type)
//! 8. Domain consistency (domain matches probes)
//! 9. No hallucinated entities (all nouns in evidence)
//! 10. Contract validation (answer shape = question shape)

mod check;
mod evaluator;
mod input;
mod outcome;
mod result;

#[cfg(test)]
mod tests;

// Re-exports
pub use check::GateCheck;
pub use evaluator::ReliabilityGate;
pub use input::GateInput;
pub use outcome::GateOutcome;
pub use result::GateResult;

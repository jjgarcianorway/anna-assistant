//! Startup context awareness and welcome summary module (Beta.209)
//!
//! This module provides deterministic welcome reports with zero LLM usage.
//! It tracks session metadata and generates system state summaries based on
//! telemetry diffs between the last run and current state.

pub mod welcome;

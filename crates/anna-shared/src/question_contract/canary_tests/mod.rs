//! Canary Tests - v0.0.437.
//!
//! Fixed tests that MUST pass for any release.
//! Any regression here BLOCKS release.
//!
//! Tests are split into categories:
//! - memory_tests: RAM and zRAM tests
//! - boot_gpu_tests: Boot services and GPU driver tests
//! - diagnosis_tests: Diagnosis conclusion tests
//! - validation_tests: Intent validation, evidence, and quality tests

mod memory_tests;
mod boot_gpu_tests;
mod diagnosis_tests;
mod validation_tests;

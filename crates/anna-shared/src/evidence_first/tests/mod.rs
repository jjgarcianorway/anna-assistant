//! Acceptance Tests for Evidence-First Knowledge Engine (v0.0.435).
//!
//! Tests for the key acceptance criteria:
//! 1. Boot slow diagnosis with citations
//! 2. CPU temperature check
//! 3. Recipe promotion after N confirmations
//!
//! Tests are organized by category:
//! - workflow_tests: End-to-end workflows and acceptance tests
//! - citation_tests: Citation storage and evidence management
//! - recipe_tests: Recipe promotion, instantiation, and tracking
//! - primitive_tests: Primitive library and probe selection
//! - claim_tests: Claim extraction and validation
//! - wiki_tests: Wiki cache operations
//! - helpers: Integration test utilities

mod citation_tests;
mod claim_tests;
mod helpers;
mod primitive_tests;
mod recipe_tests;
mod wiki_tests;
mod workflow_tests;

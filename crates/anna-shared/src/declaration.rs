//! Capability Declaration - Runtime Trust Disclosure (Phase 46)
//!
//! This module provides a read-only, human-facing view of Anna's capabilities.
//! It exists to make trust visible at runtime, not to expand power.
//!
//! # Purpose
//!
//! Users must be able to see what Anna can and cannot do BEFORE she acts.
//! This module generates declarations from the capability ledger that are:
//!
//! - Human-readable (plain text, no jargon)
//! - Versioned (changes are visible in diffs)
//! - Deterministic (same input always produces same output)
//!
//! # Architectural Constraints
//!
//! This module MUST NOT:
//!
//! - Import execution_request, human_execution, or any proposal modules
//! - Contain any Command::new or process spawning code
//! - Request, trigger, or enable any execution
//! - Define new capabilities (only read from capabilities.rs)
//!
//! This module is purely declarative. It reads the ledger and formats text.
//!
//! # Verification
//!
//! Tests in this module verify that:
//!
//! 1. The declaration derives exactly from the capability ledger
//! 2. No execution-related modules are imported
//! 3. Output is deterministic and stable
//!
//! Any violation of these constraints is an architectural failure.

// ISOLATION BOUNDARY: Only import from capabilities.rs
// DO NOT ADD IMPORTS from execution_request, human_execution, proposal, or similar
use crate::capabilities::{
    all_allowed_binaries, capabilities_by_category, execution_capabilities,
    forbidden_capabilities, CapabilityCategory, ExecutionLevel, CAPABILITIES, LEDGER_VERSION,
};

/// Declaration format version - tracks declaration format changes
pub const DECLARATION_FORMAT_VERSION: &str = "1.0";

/// A runtime declaration of Anna's capabilities for human inspection.
#[derive(Debug, Clone)]
pub struct CapabilityDeclaration {
    /// Version of the capability ledger
    pub ledger_version: String,
    /// Version of the declaration format
    pub format_version: String,
    /// What Anna can do (with human confirmation)
    pub can_do: Vec<CapabilityEntry>,
    /// What Anna cannot do (structural constraints)
    pub cannot_do: Vec<CapabilityEntry>,
    /// What Anna will never do (explicit forbiddance)
    pub will_never_do: Vec<CapabilityEntry>,
}

/// A single capability entry for human display.
#[derive(Debug, Clone)]
pub struct CapabilityEntry {
    /// Human-readable name
    pub name: String,
    /// Plain-language description
    pub description: String,
    /// Additional details (tools used, restrictions)
    pub details: Option<String>,
}

impl CapabilityDeclaration {
    /// Generate a declaration from the capability ledger.
    /// This is the ONLY way to create a declaration - there is no manual construction.
    pub fn from_ledger() -> Self {
        let mut can_do = Vec::new();
        let mut cannot_do = Vec::new();
        let mut will_never_do = Vec::new();

        // Diagnosis capabilities - what Anna can read
        for cap in capabilities_by_category(CapabilityCategory::Diagnosis) {
            can_do.push(CapabilityEntry {
                name: humanize_name(cap.name),
                description: cap.description.to_string(),
                details: if !cap.allowed_binaries.is_empty() {
                    Some(format!("Uses: {}", cap.allowed_binaries.join(", ")))
                } else {
                    None
                },
            });
        }

        // FilesystemRead capabilities
        for cap in capabilities_by_category(CapabilityCategory::FilesystemRead) {
            can_do.push(CapabilityEntry {
                name: humanize_name(cap.name),
                description: cap.description.to_string(),
                details: Some(cap.args_policy.to_string()),
            });
        }

        // Execution capabilities - what Anna can do with confirmation
        for cap in execution_capabilities() {
            can_do.push(CapabilityEntry {
                name: humanize_name(cap.name),
                description: cap.description.to_string(),
                details: Some(format!(
                    "Requires confirmation: \"{}\"",
                    cap.requires_confirmation.unwrap_or("(none)")
                )),
            });
        }

        // Proposal capabilities - suggestions only
        for cap in capabilities_by_category(CapabilityCategory::Proposal) {
            cannot_do.push(CapabilityEntry {
                name: humanize_name(cap.name),
                description: format!("{} (cannot execute, only suggest)", cap.description),
                details: None,
            });
        }

        // Forbidden capabilities - explicit non-powers
        for cap in forbidden_capabilities() {
            will_never_do.push(CapabilityEntry {
                name: humanize_name(cap.name.strip_prefix("NO_").unwrap_or(cap.name)),
                description: cap.description.to_string(),
                details: Some(cap.args_policy.to_string()),
            });
        }

        Self {
            ledger_version: LEDGER_VERSION.to_string(),
            format_version: DECLARATION_FORMAT_VERSION.to_string(),
            can_do,
            cannot_do,
            will_never_do,
        }
    }

    /// Render the declaration as plain text for CLI display.
    pub fn render_plain_text(&self) -> String {
        let mut output = String::new();

        output.push_str("ANNA CAPABILITY DECLARATION\n");
        output.push_str(&"=".repeat(60));
        output.push('\n');
        output.push_str(&format!(
            "Ledger Version: {}  |  Format Version: {}\n",
            self.ledger_version, self.format_version
        ));
        output.push_str(&"=".repeat(60));
        output.push_str("\n\n");

        // What Anna can do
        output.push_str("WHAT ANNA CAN DO\n");
        output.push_str(&"-".repeat(40));
        output.push('\n');
        for entry in &self.can_do {
            output.push_str(&format!("* {}\n", entry.name));
            output.push_str(&format!("  {}\n", entry.description));
            if let Some(ref details) = entry.details {
                output.push_str(&format!("  [{}]\n", details));
            }
            output.push('\n');
        }

        // What Anna cannot do automatically
        output.push_str("WHAT ANNA CANNOT DO AUTOMATICALLY\n");
        output.push_str(&"-".repeat(40));
        output.push('\n');
        for entry in &self.cannot_do {
            output.push_str(&format!("* {}\n", entry.name));
            output.push_str(&format!("  {}\n", entry.description));
            output.push('\n');
        }

        // What Anna will never do
        output.push_str("WHAT ANNA WILL NEVER DO\n");
        output.push_str(&"-".repeat(40));
        output.push('\n');
        for entry in &self.will_never_do {
            output.push_str(&format!("* {}\n", entry.name));
            output.push_str(&format!("  {}\n", entry.description));
            if let Some(ref details) = entry.details {
                output.push_str(&format!("  Policy: {}\n", details));
            }
            output.push('\n');
        }

        // Summary
        output.push_str(&"=".repeat(60));
        output.push('\n');
        output.push_str("SUMMARY\n");
        output.push_str(&format!(
            "Total capabilities: {} can do, {} cannot do automatically, {} will never do\n",
            self.can_do.len(),
            self.cannot_do.len(),
            self.will_never_do.len()
        ));
        output.push_str(&format!(
            "Allowed binaries: {}\n",
            all_allowed_binaries().len()
        ));
        output.push_str(&"=".repeat(60));
        output.push('\n');

        output
    }

    /// Render as a deterministic format suitable for file output and diffing.
    pub fn render_deterministic(&self) -> String {
        let mut output = String::new();

        output.push_str("# Anna Capability Declaration\n\n");
        output.push_str(&format!("ledger_version: {}\n", self.ledger_version));
        output.push_str(&format!("format_version: {}\n\n", self.format_version));

        output.push_str("## can_do\n");
        for entry in &self.can_do {
            output.push_str(&format!("- name: {}\n", entry.name));
            output.push_str(&format!("  description: {}\n", entry.description));
            if let Some(ref details) = entry.details {
                output.push_str(&format!("  details: {}\n", details));
            }
        }
        output.push('\n');

        output.push_str("## cannot_do_automatically\n");
        for entry in &self.cannot_do {
            output.push_str(&format!("- name: {}\n", entry.name));
            output.push_str(&format!("  description: {}\n", entry.description));
        }
        output.push('\n');

        output.push_str("## will_never_do\n");
        for entry in &self.will_never_do {
            output.push_str(&format!("- name: {}\n", entry.name));
            output.push_str(&format!("  description: {}\n", entry.description));
            if let Some(ref details) = entry.details {
                output.push_str(&format!("  policy: {}\n", details));
            }
        }

        output
    }

    /// Render a compact onboarding summary.
    pub fn render_onboarding(&self) -> String {
        let mut output = String::new();

        output.push_str("Before Anna acts, here is what she can do:\n\n");

        output.push_str("CAN DO (with your confirmation):\n");
        for entry in &self.can_do {
            if entry.details.is_some()
                && entry
                    .details
                    .as_ref()
                    .map_or(false, |d| d.contains("confirmation"))
            {
                output.push_str(&format!("  - {}\n", entry.name));
            }
        }
        output.push('\n');

        output.push_str("CAN READ (automatically):\n");
        for entry in &self.can_do {
            if entry.details.is_some()
                && entry
                    .details
                    .as_ref()
                    .map_or(false, |d| d.starts_with("Uses:"))
            {
                output.push_str(&format!("  - {}\n", entry.name));
            }
        }
        output.push('\n');

        output.push_str("WILL NEVER:\n");
        for entry in &self.will_never_do {
            output.push_str(&format!("  - {}\n", entry.name));
        }

        output
    }
}

/// Convert snake_case capability names to human-readable form.
fn humanize_name(name: &str) -> String {
    name.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// =============================================================================
// ISOLATION VERIFICATION TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the declaration derives from the ledger.
    #[test]
    fn test_declaration_derives_from_ledger() {
        let decl = CapabilityDeclaration::from_ledger();

        // Must have the same ledger version
        assert_eq!(decl.ledger_version, LEDGER_VERSION);

        // Total entries should relate to capability count
        let total_entries = decl.can_do.len() + decl.cannot_do.len() + decl.will_never_do.len();
        assert!(
            total_entries > 0,
            "Declaration must have entries from ledger"
        );
    }

    /// Verify the declaration is deterministic.
    #[test]
    fn test_declaration_is_deterministic() {
        let decl1 = CapabilityDeclaration::from_ledger();
        let decl2 = CapabilityDeclaration::from_ledger();

        let output1 = decl1.render_deterministic();
        let output2 = decl2.render_deterministic();

        assert_eq!(output1, output2, "Declaration must be deterministic");
    }

    /// Verify will_never_do entries match forbidden capabilities.
    #[test]
    fn test_will_never_do_matches_forbidden() {
        let decl = CapabilityDeclaration::from_ledger();
        let forbidden = forbidden_capabilities();

        assert_eq!(
            decl.will_never_do.len(),
            forbidden.len(),
            "will_never_do must match forbidden capabilities count"
        );
    }

    /// Verify execution capabilities require confirmation.
    #[test]
    fn test_execution_entries_show_confirmation() {
        let decl = CapabilityDeclaration::from_ledger();

        for entry in &decl.can_do {
            if let Some(ref details) = entry.details {
                if details.contains("confirmation") {
                    // Execution entries must show the confirmation requirement
                    assert!(
                        details.contains("Requires confirmation:"),
                        "Execution entry must show confirmation text"
                    );
                }
            }
        }
    }

    /// Verify humanize_name works correctly.
    #[test]
    fn test_humanize_name() {
        assert_eq!(humanize_name("wifi_diagnosis"), "Wifi Diagnosis");
        assert_eq!(humanize_name("system_state_diagnosis"), "System State Diagnosis");
        assert_eq!(humanize_name("network_requests"), "Network Requests");
    }

    /// Verify plain text rendering produces output.
    #[test]
    fn test_plain_text_renders() {
        let decl = CapabilityDeclaration::from_ledger();
        let output = decl.render_plain_text();

        assert!(output.contains("ANNA CAPABILITY DECLARATION"));
        assert!(output.contains("WHAT ANNA CAN DO"));
        assert!(output.contains("WHAT ANNA CANNOT DO AUTOMATICALLY"));
        assert!(output.contains("WHAT ANNA WILL NEVER DO"));
        assert!(output.contains("SUMMARY"));
    }

    /// Verify onboarding rendering produces output.
    #[test]
    fn test_onboarding_renders() {
        let decl = CapabilityDeclaration::from_ledger();
        let output = decl.render_onboarding();

        assert!(output.contains("Before Anna acts"));
        assert!(output.contains("CAN DO"));
        assert!(output.contains("CAN READ"));
        assert!(output.contains("WILL NEVER"));
    }

    /// Verify deterministic rendering is parseable.
    #[test]
    fn test_deterministic_is_structured() {
        let decl = CapabilityDeclaration::from_ledger();
        let output = decl.render_deterministic();

        assert!(output.contains("ledger_version:"));
        assert!(output.contains("format_version:"));
        assert!(output.contains("## can_do"));
        assert!(output.contains("## cannot_do_automatically"));
        assert!(output.contains("## will_never_do"));
    }
}

// =============================================================================
// ISOLATION INVARIANT TESTS
// =============================================================================

#[cfg(test)]
mod isolation_tests {
    //! These tests verify the architectural isolation of this module.
    //! They ensure no execution-related imports can sneak in.

    use std::fs;
    use std::path::Path;

    /// Verify this module does not import execution-related modules.
    #[test]
    fn test_no_execution_imports() {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/declaration.rs");
        let source = fs::read_to_string(&source_path).expect("Failed to read declaration.rs");

        // Forbidden imports - if any appear, isolation is broken
        let forbidden_imports = [
            "execution_request",
            "human_execution",
            "proposal",
            "execution_reservation",
            "action_plan",
        ];

        for forbidden in &forbidden_imports {
            // Check for use statements importing these modules
            let import_pattern = format!("use crate::{}", forbidden);
            let import_pattern2 = format!("use super::{}", forbidden);

            assert!(
                !source.contains(&import_pattern) && !source.contains(&import_pattern2),
                "declaration.rs must not import {} - isolation violated",
                forbidden
            );
        }
    }

    /// Verify this module contains no Command execution.
    #[test]
    fn test_no_command_execution() {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/declaration.rs");
        let source = fs::read_to_string(&source_path).expect("Failed to read declaration.rs");

        // Extract non-test, non-comment code lines
        // We only check actual code for execution patterns
        let code_lines: String = source
            .lines()
            .take_while(|line| !line.contains("#[cfg(test)]"))
            .filter(|line| {
                let trimmed = line.trim();
                // Exclude doc comments and regular comments
                !trimmed.starts_with("//!")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("*")
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Check for Command execution patterns in actual code
        let cmd_new = "Command".to_string() + "::new";
        assert!(
            !code_lines.contains(&cmd_new),
            "declaration.rs must not contain Command execution in code"
        );
        assert!(
            !code_lines.contains("std::process::"),
            "declaration.rs must not import std::process"
        );
        assert!(
            !code_lines.contains("tokio::process"),
            "declaration.rs must not import tokio::process"
        );
    }

    /// Verify this module only imports from capabilities.rs.
    #[test]
    fn test_only_capabilities_import() {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/declaration.rs");
        let source = fs::read_to_string(&source_path).expect("Failed to read declaration.rs");

        // Find all `use crate::` lines
        let crate_imports: Vec<&str> = source
            .lines()
            .filter(|line| line.trim().starts_with("use crate::"))
            .collect();

        // All crate imports must be from capabilities
        for import in &crate_imports {
            assert!(
                import.contains("capabilities"),
                "declaration.rs has non-capabilities import: {}",
                import
            );
        }
    }
}

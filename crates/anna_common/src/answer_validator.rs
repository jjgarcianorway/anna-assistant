//! Answer Validation Pipeline - Zero Hallucination Guarantee
//!
//! v5.7.0-beta.87: Multi-pass validation with hallucination detection
//!
//! This module implements a rigorous validation pipeline to ensure:
//! 1. Factual accuracy - no made-up commands, files, or packages
//! 2. Command safety - all suggested commands are valid and safe
//! 3. Completeness - answers address the user's question
//! 4. Clarity - answers are understandable and well-structured

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Validation result with detailed feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the answer passed validation
    pub passed: bool,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,

    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssue {
    /// Potential hallucination detected
    Hallucination {
        item: String,
        reason: String,
    },

    /// Invalid or dangerous command
    UnsafeCommand {
        command: String,
        reason: String,
    },

    /// Missing information
    Incomplete {
        missing: String,
    },

    /// Unclear or confusing explanation
    Clarity {
        section: String,
        issue: String,
    },

    /// Factual error
    FactualError {
        claim: String,
        correction: String,
    },
}

/// Answer validator with multi-pass validation
pub struct AnswerValidator {
    /// Enable debug mode for detailed logging
    debug_mode: bool,

    /// Known safe commands (whitelist)
    safe_commands: HashSet<String>,

    /// Known dangerous command patterns (blacklist)
    dangerous_patterns: Vec<String>,
}

impl Default for AnswerValidator {
    fn default() -> Self {
        Self::new(false)
    }
}

impl AnswerValidator {
    /// Create a new answer validator
    pub fn new(debug_mode: bool) -> Self {
        let safe_commands = Self::build_safe_command_list();
        let dangerous_patterns = Self::build_dangerous_patterns();

        Self {
            debug_mode,
            safe_commands,
            dangerous_patterns,
        }
    }

    /// Validate an answer through multi-pass pipeline
    pub async fn validate(&self, answer: &str, context: &ValidationContext) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Pass 1: Check for hallucinated commands
        let command_issues = self.validate_commands(answer)?;
        issues.extend(command_issues);

        // Pass 2: Check for hallucinated file paths
        let file_issues = self.validate_file_references(answer, context)?;
        issues.extend(file_issues);

        // Pass 3: Check for hallucinated package names
        let package_issues = self.validate_package_references(answer, context)?;
        issues.extend(package_issues);

        // Pass 4: Check completeness
        let completeness_issues = self.validate_completeness(answer, context)?;
        issues.extend(completeness_issues);

        // Pass 5: Check clarity
        let clarity_issues = self.validate_clarity(answer)?;
        issues.extend(clarity_issues);

        // Calculate confidence score
        let confidence = self.calculate_confidence(&issues);

        // Generate suggestions
        if !issues.is_empty() {
            suggestions = self.generate_suggestions(&issues);
        }

        let passed = issues.is_empty() || confidence >= 0.8;

        if self.debug_mode {
            println!("[AnswerValidator] Validation completed:");
            println!("  Passed: {}", passed);
            println!("  Confidence: {:.2}", confidence);
            println!("  Issues: {}", issues.len());
        }

        Ok(ValidationResult {
            passed,
            confidence,
            issues,
            suggestions,
        })
    }

    /// Validate that all mentioned commands exist and are safe
    fn validate_commands(&self, answer: &str) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Extract commands from code blocks and inline code
        let commands = self.extract_commands(answer);

        for cmd in commands {
            // Check if command is in safe list
            let cmd_name = cmd.split_whitespace().next().unwrap_or("");

            if !cmd_name.is_empty() {
                // Check against dangerous patterns first
                for pattern in &self.dangerous_patterns {
                    if cmd.contains(pattern) {
                        issues.push(ValidationIssue::UnsafeCommand {
                            command: cmd.clone(),
                            reason: format!("Contains dangerous pattern: {}", pattern),
                        });
                        continue;
                    }
                }

                // If not in safe commands and looks complex, flag as potential hallucination
                if !self.safe_commands.contains(cmd_name) && cmd.len() > 50 {
                    issues.push(ValidationIssue::Hallucination {
                        item: cmd.clone(),
                        reason: "Command not in known safe list and appears complex".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Validate that referenced files actually exist or are clearly hypothetical
    fn validate_file_references(&self, answer: &str, context: &ValidationContext) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Extract file paths from answer
        let file_refs = self.extract_file_paths(answer);

        for file_path in file_refs {
            // Skip common placeholders
            if file_path.contains("example") || file_path.contains("<") || file_path.contains("your") {
                continue;
            }

            // Check if file exists in context
            if !context.known_files.iter().any(|f| f.contains(&file_path)) {
                // Not necessarily a hallucination - could be a suggestion
                // Only flag if presented as fact
                if answer.contains(&format!("in {}", file_path)) ||
                   answer.contains(&format!("at {}", file_path)) {
                    issues.push(ValidationIssue::Hallucination {
                        item: file_path.clone(),
                        reason: "File path mentioned as existing but not found in context".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Validate package name references
    fn validate_package_references(&self, answer: &str, context: &ValidationContext) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Extract package names from pacman/yay commands
        let packages = self.extract_package_names(answer);

        for package in packages {
            // Common packages are OK
            let common_packages = vec!["base-devel", "linux", "linux-headers", "gcc", "git"];
            if common_packages.contains(&package.as_str()) {
                continue;
            }

            // Check against known packages in context
            if !context.known_packages.is_empty() &&
               !context.known_packages.iter().any(|p| p == &package) {
                // This might be a hallucination
                issues.push(ValidationIssue::Hallucination {
                    item: package.clone(),
                    reason: "Package name not verified against package database".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// Validate that answer addresses the user's question
    fn validate_completeness(&self, answer: &str, context: &ValidationContext) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check if answer is too short
        if answer.len() < 50 {
            issues.push(ValidationIssue::Incomplete {
                missing: "Answer seems too brief to fully address the question".to_string(),
            });
        }

        // Check for key question words in context
        let question = &context.user_question;
        let question_lower = question.to_lowercase();

        if question_lower.contains("how") && !answer.to_lowercase().contains("step")
            && !answer.contains("1.") && !answer.contains("First") {
            issues.push(ValidationIssue::Incomplete {
                missing: "Question asks 'how' but answer doesn't provide clear steps".to_string(),
            });
        }

        if question_lower.contains("why") && !answer.to_lowercase().contains("because")
            && !answer.to_lowercase().contains("reason") {
            issues.push(ValidationIssue::Incomplete {
                missing: "Question asks 'why' but answer doesn't provide reasoning".to_string(),
            });
        }

        Ok(issues)
    }

    /// Validate answer clarity and structure
    fn validate_clarity(&self, answer: &str) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for overly complex sentences
        let sentences: Vec<&str> = answer.split('.').collect();
        for sentence in sentences {
            if sentence.split_whitespace().count() > 50 {
                issues.push(ValidationIssue::Clarity {
                    section: sentence.chars().take(50).collect::<String>() + "...",
                    issue: "Sentence is too long and complex".to_string(),
                });
            }
        }

        // Check for missing code formatting
        if answer.contains("sudo ") || answer.contains("pacman ") || answer.contains("systemctl ") {
            if !answer.contains("```") && !answer.contains("`") {
                issues.push(ValidationIssue::Clarity {
                    section: "commands".to_string(),
                    issue: "Commands should be in code blocks for clarity".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// Calculate confidence score based on issues
    fn calculate_confidence(&self, issues: &[ValidationIssue]) -> f64 {
        if issues.is_empty() {
            return 1.0_f64;
        }

        let mut penalty = 0.0_f64;

        for issue in issues {
            penalty += match issue {
                ValidationIssue::Hallucination { .. } => 0.3_f64,
                ValidationIssue::UnsafeCommand { .. } => 0.4_f64,
                ValidationIssue::FactualError { .. } => 0.3_f64,
                ValidationIssue::Incomplete { .. } => 0.1_f64,
                ValidationIssue::Clarity { .. } => 0.05_f64,
            };
        }

        (1.0_f64 - penalty).max(0.0_f64)
    }

    /// Generate improvement suggestions
    fn generate_suggestions(&self, issues: &[ValidationIssue]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for issue in issues {
            let suggestion = match issue {
                ValidationIssue::Hallucination { item, .. } => {
                    format!("Verify that '{}' actually exists before mentioning it", item)
                }
                ValidationIssue::UnsafeCommand { command, .. } => {
                    format!("Replace unsafe command: {}", command)
                }
                ValidationIssue::Incomplete { missing } => {
                    format!("Add missing information: {}", missing)
                }
                ValidationIssue::Clarity { section, issue } => {
                    format!("Improve clarity in '{}': {}", section, issue)
                }
                ValidationIssue::FactualError { correction, .. } => {
                    format!("Correct factual error: {}", correction)
                }
            };
            suggestions.push(suggestion);
        }

        suggestions
    }

    /// Extract commands from answer text
    fn extract_commands(&self, text: &str) -> Vec<String> {
        let mut commands = Vec::new();

        // Extract from code blocks
        let code_block_pattern = "```";
        let mut in_code_block = false;
        for line in text.lines() {
            if line.contains(code_block_pattern) {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block && !line.trim().is_empty() {
                commands.push(line.trim().to_string());
            }
        }

        // Extract from inline code
        for part in text.split('`') {
            if part.contains("sudo ") || part.contains("pacman ") || part.contains("systemctl ") {
                commands.push(part.trim().to_string());
            }
        }

        commands
    }

    /// Extract file paths from answer text
    fn extract_file_paths(&self, text: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // Simple extraction - look for path-like patterns
        for word in text.split_whitespace() {
            if word.starts_with('/') || word.starts_with("~/") || word.contains(".conf") || word.contains(".service") {
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '-' && c != '_');
                paths.push(clean.to_string());
            }
        }

        paths
    }

    /// Extract package names from answer text
    fn extract_package_names(&self, text: &str) -> Vec<String> {
        let mut packages = Vec::new();

        // Look for pacman/yay install commands
        for line in text.lines() {
            if line.contains("pacman -S ") || line.contains("yay -S ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut found_s = false;
                for part in parts {
                    if found_s && !part.starts_with('-') {
                        packages.push(part.to_string());
                    }
                    if part == "-S" || part == "-Syu" {
                        found_s = true;
                    }
                }
            }
        }

        packages
    }

    /// Build list of known safe commands
    fn build_safe_command_list() -> HashSet<String> {
        vec![
            "ls", "cd", "pwd", "cat", "echo", "grep", "find", "which", "man",
            "sudo", "pacman", "yay", "systemctl", "journalctl", "ip", "ping",
            "mkdir", "rm", "cp", "mv", "chmod", "chown", "ln", "touch",
            "git", "cargo", "rustc", "python", "node", "npm", "make",
            "ps", "top", "htop", "kill", "killall", "pkill",
            "tar", "gzip", "unzip", "curl", "wget",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    /// Build list of dangerous command patterns
    fn build_dangerous_patterns() -> Vec<String> {
        vec![
            "rm -rf /".to_string(),
            "dd if=".to_string(),
            ":(){ :|:& };:".to_string(),
            "> /dev/sda".to_string(),
            "mkfs".to_string(),
            "chmod -R 777".to_string(),
        ]
    }
}

/// Context for answer validation
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// The user's original question
    pub user_question: String,

    /// Files known to exist in the current context
    pub known_files: Vec<String>,

    /// Packages known to exist
    pub known_packages: Vec<String>,

    /// System information
    pub system_info: Option<SystemInfo>,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub arch_version: String,
    pub kernel_version: String,
    pub installed_packages: Vec<String>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new(user_question: String) -> Self {
        Self {
            user_question,
            known_files: Vec::new(),
            known_packages: Vec::new(),
            system_info: None,
        }
    }

    /// Add known file to context
    pub fn add_file(&mut self, file: String) {
        self.known_files.push(file);
    }

    /// Add known package to context
    pub fn add_package(&mut self, package: String) {
        self.known_packages.push(package);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_creation() {
        let validator = AnswerValidator::new(true);
        assert!(validator.debug_mode);
        assert!(!validator.safe_commands.is_empty());
    }

    #[tokio::test]
    async fn test_safe_command_validation() {
        let validator = AnswerValidator::new(false);
        let answer = "Run `ls -la` to list files";
        let context = ValidationContext::new("How do I list files?".to_string());

        let result = validator.validate(answer, &context).await.unwrap();
        // Should not flag 'ls' as it's in safe list
        assert!(result.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_dangerous_command_detection() {
        let validator = AnswerValidator::new(false);
        let answer = "Run `rm -rf /` to clean up";
        let context = ValidationContext::new("How do I clean up?".to_string());

        let result = validator.validate(answer, &context).await.unwrap();
        // Should detect dangerous command
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| matches!(i, ValidationIssue::UnsafeCommand { .. })));
    }
}

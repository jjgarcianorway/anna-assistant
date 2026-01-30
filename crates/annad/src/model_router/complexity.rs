//! Task complexity classification.
//!
//! Classifies questions into complexity tiers for model routing.

use anna_shared::agent::detect_domains;

/// Task complexity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Complexity {
    /// Simple: direct facts, single lookup (e.g., "what is my IP?")
    Simple,
    /// Standard: straightforward questions, single domain
    Standard,
    /// Complex: debugging, configuration, multi-step
    Complex,
    /// VeryComplex: multi-domain, optimization, security analysis
    VeryComplex,
}

/// Classifies task complexity.
pub struct ComplexityClassifier {
    /// Simple question patterns
    simple_patterns: Vec<&'static str>,
    /// Complex question indicators
    complex_indicators: Vec<&'static str>,
    /// Very complex indicators
    very_complex_indicators: Vec<&'static str>,
}

impl ComplexityClassifier {
    pub fn new() -> Self {
        Self {
            simple_patterns: vec![
                "what is",
                "what's",
                "how much",
                "how many",
                "show me",
                "list",
                "which",
                "where is",
                "who is",
                "tell me",
            ],
            complex_indicators: vec![
                "not working",
                "doesn't work",
                "won't",
                "can't",
                "cannot",
                "failed",
                "failing",
                "error",
                "fix",
                "repair",
                "configure",
                "setup",
                "set up",
                "install",
                "uninstall",
                "remove",
                "troubleshoot",
                "debug",
                "diagnose",
                "why",
                "how do i",
                "how can i",
                "help me",
            ],
            very_complex_indicators: vec![
                "optimize",
                "optimise",
                "performance",
                "slow",
                "secure",
                "security",
                "harden",
                "audit",
                "analyze",
                "compare",
                "migrate",
                "upgrade",
                "downgrade",
                "rollback",
                "backup",
                "restore",
                "automate",
                "script",
            ],
        }
    }

    /// Classify question complexity.
    pub fn classify(&self, question: &str) -> Complexity {
        let q_lower = question.to_lowercase();

        // Check domain count first
        let domains = detect_domains(question);
        let domain_count = domains.len();

        // Very complex: multi-domain or specific indicators
        if domain_count > 2 {
            return Complexity::VeryComplex;
        }

        // Count very_complex_indicators
        let very_complex_count = self.very_complex_indicators
            .iter()
            .filter(|ind| q_lower.contains(*ind))
            .count();

        // Multiple very_complex_indicators = VeryComplex regardless of domains
        if very_complex_count >= 2 {
            return Complexity::VeryComplex;
        }

        // Single very_complex_indicator: Complex or VeryComplex based on domains
        if very_complex_count == 1 {
            return if domain_count > 1 {
                Complexity::VeryComplex
            } else {
                Complexity::Complex
            };
        }

        // Complex: debugging, configuration
        for indicator in &self.complex_indicators {
            if q_lower.contains(indicator) {
                return Complexity::Complex;
            }
        }

        // Simple: direct questions with simple patterns
        for pattern in &self.simple_patterns {
            if q_lower.starts_with(pattern) {
                return Complexity::Simple;
            }
        }

        // Multi-domain defaults to complex
        if domain_count > 1 {
            return Complexity::Complex;
        }

        // Short questions are typically simple
        if question.split_whitespace().count() <= 5 {
            return Complexity::Simple;
        }

        // Default: standard
        Complexity::Standard
    }

    /// Get complexity score (0.0-1.0).
    pub fn score(&self, question: &str) -> f32 {
        match self.classify(question) {
            Complexity::Simple => 0.2,
            Complexity::Standard => 0.4,
            Complexity::Complex => 0.7,
            Complexity::VeryComplex => 1.0,
        }
    }
}

impl Default for ComplexityClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_questions() {
        let classifier = ComplexityClassifier::new();

        assert_eq!(classifier.classify("what is my IP?"), Complexity::Simple);
        assert_eq!(classifier.classify("how much RAM do I have?"), Complexity::Simple);
        assert_eq!(classifier.classify("show me disk usage"), Complexity::Simple);
        assert_eq!(classifier.classify("list services"), Complexity::Simple);
    }

    #[test]
    fn test_complex_questions() {
        let classifier = ComplexityClassifier::new();

        assert_eq!(classifier.classify("wifi is not working"), Complexity::Complex);
        assert_eq!(classifier.classify("fix my audio"), Complexity::Complex);
        assert_eq!(classifier.classify("configure ssh"), Complexity::Complex);
        assert_eq!(classifier.classify("why does my system freeze?"), Complexity::Complex);
    }

    #[test]
    fn test_very_complex_questions() {
        let classifier = ComplexityClassifier::new();

        // Multiple very_complex_indicators ("optimize" + "performance") = VeryComplex
        assert_eq!(
            classifier.classify("optimize my system for performance"),
            Complexity::VeryComplex
        );
        // Single indicator with single domain = Complex
        assert_eq!(
            classifier.classify("optimize my system"),
            Complexity::Complex
        );
        assert_eq!(
            classifier.classify("secure my ssh and firewall configuration"),
            Complexity::VeryComplex // Multi-domain (security + network)
        );
        assert_eq!(
            classifier.classify("analyze network and disk performance"),
            Complexity::VeryComplex // Multi-domain
        );
    }

    #[test]
    fn test_multi_domain_detection() {
        let classifier = ComplexityClassifier::new();

        // Multi-domain questions should be at least Complex
        let result = classifier.classify("check my wifi and disk space");
        assert!(matches!(result, Complexity::Complex | Complexity::VeryComplex));
    }
}

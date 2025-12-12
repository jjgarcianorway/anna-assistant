//! Regression Test Harness (v0.0.415).
//!
//! Tests answer SHAPE, not exact wording.
//! Catches: parse errors, timeouts, empty summaries, missing metrics.
//! Does NOT hardcode expected answers.

use crate::strict_contract::{StrictSpecialistResponse, StrictStatus};

/// A regression test case
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Test ID
    pub id: &'static str,
    /// User query
    pub query: &'static str,
    /// Expected domain
    pub expected_domain: &'static str,
    /// Expected intent
    pub expected_intent: &'static str,
    /// Required metric keys (if any)
    pub required_metrics: Vec<&'static str>,
    /// Whether status=ok is required
    pub require_ok: bool,
    /// Minimum required evidence items
    pub min_evidence: usize,
    /// Forbidden patterns in summary
    pub forbidden_patterns: Vec<&'static str>,
}

impl TestCase {
    /// Validate a response against this test case
    pub fn validate(&self, response: &StrictSpecialistResponse) -> TestResult {
        let mut issues = Vec::new();

        // Check summary is not empty
        if response.summary.trim().is_empty() {
            issues.push("summary is empty".to_string());
        }

        // Check for forbidden patterns
        let summary_lower = response.summary.to_lowercase();
        for pattern in &self.forbidden_patterns {
            if summary_lower.contains(&pattern.to_lowercase()) {
                issues.push(format!("summary contains forbidden pattern: '{}'", pattern));
            }
        }

        // Check status if required
        if self.require_ok && response.status != StrictStatus::Ok {
            issues.push(format!("expected status=ok, got {:?}", response.status));
        }

        // Check evidence count
        if response.evidence.len() < self.min_evidence {
            issues.push(format!(
                "expected at least {} evidence items, got {}",
                self.min_evidence,
                response.evidence.len()
            ));
        }

        // Check required metrics
        if !self.required_metrics.is_empty() {
            if let Some(metrics) = &response.metrics {
                if let Some(obj) = metrics.as_object() {
                    for key in &self.required_metrics {
                        if !obj.contains_key(*key) {
                            issues.push(format!("missing required metric: {}", key));
                        }
                    }
                } else {
                    issues.push("metrics is not an object".to_string());
                }
            } else {
                issues.push("metrics is missing".to_string());
            }
        }

        // Validate the response itself
        let validation_issues = response.validate();
        issues.extend(validation_issues);

        TestResult {
            test_id: self.id.to_string(),
            query: self.query.to_string(),
            passed: issues.is_empty(),
            issues,
            confidence: response.confidence,
            status: response.status,
        }
    }
}

/// Result of a test case
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub query: String,
    pub passed: bool,
    pub issues: Vec<String>,
    pub confidence: f32,
    pub status: StrictStatus,
}

impl TestResult {
    /// Format for display
    pub fn format(&self) -> String {
        if self.passed {
            format!(
                "[PASS] {} - {:.0}% confidence",
                self.test_id,
                self.confidence * 100.0
            )
        } else {
            format!(
                "[FAIL] {} - {}\n       Issues: {}",
                self.test_id,
                self.query,
                self.issues.join("; ")
            )
        }
    }
}

/// The regression test suite
pub fn regression_suite() -> Vec<TestCase> {
    vec![
        // System domain
        TestCase {
            id: "ram_free",
            query: "how much free RAM do I have right now?",
            expected_domain: "system",
            expected_intent: "query_metric",
            required_metrics: vec!["mem_available_gb"],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown", "error"],
        },
        TestCase {
            id: "swap_enabled",
            query: "do I have a swap file?",
            expected_domain: "system",
            expected_intent: "check_status",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown"],
        },
        TestCase {
            id: "uptime",
            query: "how long has my system been running?",
            expected_domain: "system",
            expected_intent: "query_metric",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec![],
        },
        TestCase {
            id: "top_memory_process",
            query: "which process is using the most memory?",
            expected_domain: "system",
            expected_intent: "query_metric",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown"],
        },
        // Boot domain
        TestCase {
            id: "boot_time",
            query: "why is my boot time so slow?",
            expected_domain: "boot",
            expected_intent: "diagnose",
            required_metrics: vec!["boot_total_seconds"],
            require_ok: false, // might be partial
            min_evidence: 1,
            forbidden_patterns: vec![],
        },
        // Services domain
        TestCase {
            id: "failed_services",
            query: "do I have any failed systemd services?",
            expected_domain: "services",
            expected_intent: "check_status",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown"],
        },
        // Storage domain
        TestCase {
            id: "disk_space",
            query: "what is filling my disk?",
            expected_domain: "storage",
            expected_intent: "diagnose",
            required_metrics: vec![],
            require_ok: false,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown"],
        },
        // Packages domain
        TestCase {
            id: "package_check",
            query: "do I have hyprland installed?",
            expected_domain: "packages",
            expected_intent: "check_status",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown is installed", "2 is installed"],
        },
        TestCase {
            id: "package_count",
            query: "how many packages do I have installed?",
            expected_domain: "packages",
            expected_intent: "query_metric",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown"],
        },
        // Desktop domain
        TestCase {
            id: "config_location",
            query: "where is my hyprland config?",
            expected_domain: "desktop",
            expected_intent: "query_metric",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 0, // might be from built-in knowledge
            forbidden_patterns: vec!["unknown"],
        },
    ]
}

/// Run all regression tests against responses
pub fn run_regression_tests(responses: &[(String, StrictSpecialistResponse)]) -> RegressionReport {
    let suite = regression_suite();
    let mut results = Vec::new();

    for (query, response) in responses {
        // Find matching test case
        if let Some(test_case) = suite.iter().find(|t| t.query == query) {
            let result = test_case.validate(response);
            results.push(result);
        }
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();

    RegressionReport {
        total: results.len(),
        passed,
        failed,
        results,
    }
}

/// Regression test report
#[derive(Debug, Clone)]
pub struct RegressionReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<TestResult>,
}

impl RegressionReport {
    /// Format for display
    pub fn format(&self) -> String {
        let mut output = format!(
            "Regression Tests: {} total, {} passed, {} failed\n",
            self.total, self.passed, self.failed
        );

        for result in &self.results {
            output.push_str(&format!("\n{}", result.format()));
        }

        output
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Validate a single response for common issues
pub fn validate_response_shape(response: &StrictSpecialistResponse) -> Vec<String> {
    let mut issues = response.validate();

    // Additional shape checks
    if response.summary.trim().is_empty() {
        issues.push("summary is empty".to_string());
    }

    // Check for common hallucination patterns
    let summary_lower = response.summary.to_lowercase();
    let hallucination_patterns = [
        "unknown is",
        "2 is installed",
        "1 is installed",
        "installed package is",
        "**unknown**",
    ];

    for pattern in hallucination_patterns {
        if summary_lower.contains(pattern) {
            issues.push(format!("hallucination detected: '{}'", pattern));
        }
    }

    // Check for generic non-answers
    let non_answer_patterns = [
        "run annactl status",
        "i cannot determine",
        "your system is healthy", // when not asked about health
    ];

    for pattern in non_answer_patterns {
        if summary_lower.contains(pattern) && response.status == StrictStatus::Ok {
            issues.push(format!("generic non-answer detected: '{}'", pattern));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suite_has_cases() {
        let suite = regression_suite();
        assert!(!suite.is_empty());
        assert!(suite.len() >= 10);
    }

    #[test]
    fn test_good_response_passes() {
        let test_case = TestCase {
            id: "test",
            query: "how much RAM?",
            expected_domain: "system",
            expected_intent: "query_metric",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown"],
        };

        let response =
            StrictSpecialistResponse::ok("DSK-001", "query_metric", "You have 16GB RAM", 0.95)
                .with_evidence("memory_info", "MemTotal: 16384000 kB");

        let result = test_case.validate(&response);
        assert!(result.passed, "Issues: {:?}", result.issues);
    }

    #[test]
    fn test_bad_response_fails() {
        let test_case = TestCase {
            id: "test",
            query: "is vim installed?",
            expected_domain: "packages",
            expected_intent: "check_status",
            required_metrics: vec![],
            require_ok: true,
            min_evidence: 1,
            forbidden_patterns: vec!["unknown is installed"],
        };

        let response =
            StrictSpecialistResponse::ok("DSK-001", "check_status", "unknown is installed", 0.95);

        let result = test_case.validate(&response);
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("forbidden")));
    }

    #[test]
    fn test_validate_response_shape() {
        let bad = StrictSpecialistResponse::ok("DSK-001", "check_status", "2 is installed", 0.95);
        let issues = validate_response_shape(&bad);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("hallucination")));
    }
}

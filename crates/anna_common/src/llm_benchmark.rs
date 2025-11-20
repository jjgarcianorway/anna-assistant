//! LLM Benchmarking Harness for Beta.68
//!
//! Simple benchmark system for:
//! - Model performance comparison
//! - Regression detection
//! - User guidance on model selection
//!
//! NOT a scientific benchmark - just enough to detect huge regressions
//! and help users understand very slow models.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A single benchmark prompt for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkPrompt {
    /// Short identifier
    pub id: String,
    /// Category (sysadmin, hardware, troubleshooting)
    pub category: String,
    /// The actual prompt
    pub prompt: String,
    /// Expected answer keywords (for quality check)
    pub expected_keywords: Vec<String>,
}

impl BenchmarkPrompt {
    /// Standard sysadmin benchmark prompts
    pub fn standard_suite() -> Vec<Self> {
        vec![
            BenchmarkPrompt {
                id: "simple_info".to_string(),
                category: "sysadmin".to_string(),
                prompt: "What does the 'systemctl status' command do?".to_string(),
                expected_keywords: vec![
                    "systemd".to_string(),
                    "status".to_string(),
                    "service".to_string(),
                ],
            },
            BenchmarkPrompt {
                id: "arch_specific".to_string(),
                category: "sysadmin".to_string(),
                prompt: "How do I update packages on Arch Linux?".to_string(),
                expected_keywords: vec!["pacman".to_string(), "Syu".to_string()],
            },
            BenchmarkPrompt {
                id: "hardware_query".to_string(),
                category: "hardware".to_string(),
                prompt: "What command shows CPU information on Linux?".to_string(),
                expected_keywords: vec!["lscpu".to_string()],
            },
            BenchmarkPrompt {
                id: "troubleshooting".to_string(),
                category: "troubleshooting".to_string(),
                prompt: "How do I check disk space usage?".to_string(),
                expected_keywords: vec!["df".to_string()],
            },
            BenchmarkPrompt {
                id: "log_analysis".to_string(),
                category: "troubleshooting".to_string(),
                prompt: "How do I view system logs with journalctl?".to_string(),
                expected_keywords: vec!["journalctl".to_string(), "logs".to_string()],
            },
        ]
    }
}

/// Result from running a single benchmark prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Prompt ID
    pub prompt_id: String,
    /// Time to first token (ms)
    pub time_to_first_token_ms: u64,
    /// Total duration (ms)
    pub total_duration_ms: u64,
    /// Tokens per second (total tokens / total seconds)
    pub tokens_per_second: f64,
    /// Total tokens generated
    pub total_tokens: usize,
    /// How many expected keywords were found
    pub keywords_found: usize,
    /// Total expected keywords
    pub keywords_expected: usize,
    /// Quality score (0.0 - 1.0)
    pub quality_score: f64,
    /// Response text (for manual inspection)
    pub response: String,
}

impl BenchmarkResult {
    /// Create from timing and response
    pub fn new(
        prompt: &BenchmarkPrompt,
        time_to_first_token: Duration,
        total_duration: Duration,
        total_tokens: usize,
        response: String,
    ) -> Self {
        let time_to_first_token_ms = time_to_first_token.as_millis() as u64;
        let total_duration_ms = total_duration.as_millis() as u64;
        let total_seconds = total_duration.as_secs_f64();

        let tokens_per_second = if total_seconds > 0.0 {
            total_tokens as f64 / total_seconds
        } else {
            0.0
        };

        // Check quality by keyword presence
        let response_lower = response.to_lowercase();
        let keywords_found = prompt
            .expected_keywords
            .iter()
            .filter(|kw| response_lower.contains(&kw.to_lowercase()))
            .count();

        let quality_score = if prompt.expected_keywords.is_empty() {
            1.0
        } else {
            keywords_found as f64 / prompt.expected_keywords.len() as f64
        };

        Self {
            prompt_id: prompt.id.clone(),
            time_to_first_token_ms,
            total_duration_ms,
            tokens_per_second,
            total_tokens,
            keywords_found,
            keywords_expected: prompt.expected_keywords.len(),
            quality_score,
            response,
        }
    }

    /// Is performance acceptable? (> 10 tokens/sec for good UX)
    pub fn is_performance_good(&self) -> bool {
        self.tokens_per_second >= 10.0
    }

    /// Is quality acceptable? (>= 80% keywords found)
    pub fn is_quality_good(&self) -> bool {
        self.quality_score >= 0.8
    }

    /// Overall passing grade
    pub fn is_passing(&self) -> bool {
        self.is_performance_good() && self.is_quality_good()
    }
}

/// Complete benchmark suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteResult {
    /// Model being tested
    pub model_name: String,
    /// Individual prompt results
    pub results: Vec<BenchmarkResult>,
    /// Total time for all prompts (ms)
    pub total_time_ms: u64,
    /// Average tokens per second
    pub avg_tokens_per_second: f64,
    /// Average quality score
    pub avg_quality_score: f64,
    /// Passed count
    pub passed: usize,
    /// Failed count
    pub failed: usize,
}

impl BenchmarkSuiteResult {
    /// Create from individual results
    pub fn new(model_name: String, results: Vec<BenchmarkResult>) -> Self {
        let total_time_ms = results.iter().map(|r| r.total_duration_ms).sum();

        let avg_tokens_per_second = if !results.is_empty() {
            results.iter().map(|r| r.tokens_per_second).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        let avg_quality_score = if !results.is_empty() {
            results.iter().map(|r| r.quality_score).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        let passed = results.iter().filter(|r| r.is_passing()).count();
        let failed = results.len() - passed;

        Self {
            model_name,
            results,
            total_time_ms,
            avg_tokens_per_second,
            avg_quality_score,
            passed,
            failed,
        }
    }

    /// Overall assessment
    pub fn assessment(&self) -> &'static str {
        if self.avg_tokens_per_second >= 20.0 && self.avg_quality_score >= 0.9 {
            "Excellent - Fast and accurate"
        } else if self.avg_tokens_per_second >= 10.0 && self.avg_quality_score >= 0.8 {
            "Good - Acceptable for interactive use"
        } else if self.avg_tokens_per_second >= 5.0 {
            "Slow - May feel sluggish in REPL"
        } else {
            "Very Slow - Not recommended for interactive use"
        }
    }

    /// Detailed summary for user
    pub fn summary(&self) -> String {
        format!(
            "Model: {}\n\
             Performance: {:.1} tokens/sec (avg)\n\
             Quality: {:.0}% accuracy\n\
             Prompts: {} passed, {} failed\n\
             Total time: {:.1}s\n\
             Assessment: {}",
            self.model_name,
            self.avg_tokens_per_second,
            self.avg_quality_score * 100.0,
            self.passed,
            self.failed,
            self.total_time_ms as f64 / 1000.0,
            self.assessment()
        )
    }
}

/// Benchmark runner trait (implemented by LLM backends)
pub trait BenchmarkRunner {
    /// Run a single benchmark prompt
    fn run_benchmark(&self, prompt: &BenchmarkPrompt) -> Result<BenchmarkResult>;

    /// Run full benchmark suite
    fn run_suite(&self, prompts: Vec<BenchmarkPrompt>) -> Result<BenchmarkSuiteResult> {
        let mut results = Vec::new();

        for prompt in prompts {
            let result = self.run_benchmark(&prompt)?;
            results.push(result);
        }

        Ok(BenchmarkSuiteResult::new(
            self.model_name().to_string(),
            results,
        ))
    }

    /// Get model name being tested
    fn model_name(&self) -> &str;
}

/// Mock benchmark runner for testing
pub struct MockBenchmarkRunner {
    pub model_name: String,
    pub tokens_per_second: f64,
    pub quality_score: f64,
}

impl MockBenchmarkRunner {
    pub fn fast_accurate() -> Self {
        Self {
            model_name: "llama3.1:8b".to_string(),
            tokens_per_second: 25.0,
            quality_score: 0.95,
        }
    }

    pub fn slow_accurate() -> Self {
        Self {
            model_name: "llama3.2:3b".to_string(),
            tokens_per_second: 8.0,
            quality_score: 0.85,
        }
    }

    pub fn fast_inaccurate() -> Self {
        Self {
            model_name: "llama3.2:1b".to_string(),
            tokens_per_second: 30.0,
            quality_score: 0.60,
        }
    }
}

impl BenchmarkRunner for MockBenchmarkRunner {
    fn run_benchmark(&self, prompt: &BenchmarkPrompt) -> Result<BenchmarkResult> {
        // Simulate response time based on tokens_per_second
        let total_tokens = 100; // Assume 100 token response
        let total_seconds = total_tokens as f64 / self.tokens_per_second;
        let total_duration = Duration::from_secs_f64(total_seconds);
        let time_to_first_token = Duration::from_millis(200);

        // Simulate response quality
        let response = if self.quality_score >= 0.8 {
            format!(
                "The answer involves: {}",
                prompt.expected_keywords.join(", ")
            )
        } else {
            "Vague answer that might not be very helpful.".to_string()
        };

        Ok(BenchmarkResult::new(
            prompt,
            time_to_first_token,
            total_duration,
            total_tokens,
            response,
        ))
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_performance_classification() {
        let prompt = BenchmarkPrompt::standard_suite()[0].clone();

        // Fast model (25 tokens/sec)
        let fast_result = BenchmarkResult::new(
            &prompt,
            Duration::from_millis(200),
            Duration::from_secs(4),
            100,
            "systemctl status shows systemd service status".to_string(),
        );
        assert!(fast_result.is_performance_good());
        assert!(fast_result.is_quality_good());
        assert!(fast_result.is_passing());

        // Slow model (5 tokens/sec)
        let slow_result = BenchmarkResult::new(
            &prompt,
            Duration::from_millis(500),
            Duration::from_secs(20),
            100,
            "systemctl status shows systemd service status".to_string(),
        );
        assert!(!slow_result.is_performance_good());
        assert!(slow_result.is_quality_good());
        assert!(!slow_result.is_passing()); // Fails due to slow performance
    }

    #[test]
    fn test_benchmark_result_quality_classification() {
        let prompt = BenchmarkPrompt::standard_suite()[0].clone();

        // High quality (all keywords found)
        let good_quality = BenchmarkResult::new(
            &prompt,
            Duration::from_millis(200),
            Duration::from_secs(4),
            100,
            "systemctl status shows systemd service status information".to_string(),
        );
        assert_eq!(good_quality.keywords_found, 3);
        assert_eq!(good_quality.quality_score, 1.0);
        assert!(good_quality.is_quality_good());

        // Low quality (missing keywords)
        let poor_quality = BenchmarkResult::new(
            &prompt,
            Duration::from_millis(200),
            Duration::from_secs(4),
            100,
            "It shows various things about your computer.".to_string(),
        );
        assert_eq!(poor_quality.keywords_found, 0);
        assert_eq!(poor_quality.quality_score, 0.0);
        assert!(!poor_quality.is_quality_good());
    }

    #[test]
    fn test_mock_benchmark_runner_fast_accurate() {
        let runner = MockBenchmarkRunner::fast_accurate();
        let prompts = BenchmarkPrompt::standard_suite();

        let suite_result = runner.run_suite(prompts).unwrap();

        assert_eq!(suite_result.model_name, "llama3.1:8b");
        assert!(suite_result.avg_tokens_per_second >= 20.0);
        assert!(suite_result.avg_quality_score >= 0.8);
        assert_eq!(suite_result.assessment(), "Excellent - Fast and accurate");
    }

    #[test]
    fn test_mock_benchmark_runner_slow_accurate() {
        let runner = MockBenchmarkRunner::slow_accurate();
        let prompts = BenchmarkPrompt::standard_suite();

        let suite_result = runner.run_suite(prompts).unwrap();

        assert_eq!(suite_result.model_name, "llama3.2:3b");
        assert!(suite_result.avg_tokens_per_second < 10.0);
        assert!(suite_result.avg_quality_score >= 0.8);
        assert!(suite_result.assessment().contains("Slow"));
    }

    #[test]
    fn test_mock_benchmark_runner_fast_inaccurate() {
        let runner = MockBenchmarkRunner::fast_inaccurate();
        let prompts = BenchmarkPrompt::standard_suite();

        let suite_result = runner.run_suite(prompts).unwrap();

        assert_eq!(suite_result.model_name, "llama3.2:1b");
        assert!(suite_result.avg_tokens_per_second >= 20.0);
        assert!(suite_result.avg_quality_score < 0.8);
        // Should have failures due to low quality
        assert!(suite_result.failed > 0);
    }

    #[test]
    fn test_benchmark_suite_summary() {
        let runner = MockBenchmarkRunner::fast_accurate();
        let prompts = vec![BenchmarkPrompt::standard_suite()[0].clone()];

        let suite_result = runner.run_suite(prompts).unwrap();
        let summary = suite_result.summary();

        assert!(summary.contains("llama3.1:8b"));
        assert!(summary.contains("tokens/sec"));
        assert!(summary.contains("accuracy"));
        assert!(summary.contains("Assessment:"));
    }
}

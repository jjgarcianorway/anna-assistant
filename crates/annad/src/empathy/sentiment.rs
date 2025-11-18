//! Sentiment analysis for empathy kernel
//!
//! Phase 1.2: Lightweight sentiment modeling via token entropy and anomaly detection
//! Citation: [archwiki:System_maintenance]

use super::types::SentimentAnalysis;
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

/// Sentiment analyzer - analyzes logs for emotional/stress patterns
pub struct SentimentAnalyzer {
    /// Baseline token frequencies for comparison
    baseline_frequencies: HashMap<String, f64>,
    /// Recent log samples for trend analysis
    recent_samples: Vec<String>,
}

impl SentimentAnalyzer {
    /// Create new sentiment analyzer
    pub fn new() -> Self {
        Self {
            baseline_frequencies: HashMap::new(),
            recent_samples: Vec::new(),
        }
    }

    /// Analyze sentiment from conscience journal and systemd logs
    pub async fn analyze(&mut self) -> Result<SentimentAnalysis> {
        debug!("Analyzing sentiment from system logs");

        // Load recent journal entries
        let journal_entries = self.load_conscience_journal().await?;
        let systemd_entries = self.load_systemd_logs().await?;

        // Combine for analysis
        let all_entries = [journal_entries, systemd_entries].concat();

        if all_entries.is_empty() {
            return Ok(SentimentAnalysis::default());
        }

        // Calculate metrics
        let sentiment_score = self.calculate_sentiment_score(&all_entries);
        let token_entropy = self.calculate_token_entropy(&all_entries);
        let anomaly_delta = self.calculate_anomaly_delta(&all_entries);
        let patterns = self.detect_patterns(&all_entries);

        // Update baseline
        self.update_baseline(&all_entries);

        Ok(SentimentAnalysis {
            sentiment_score,
            token_entropy,
            anomaly_delta,
            patterns,
        })
    }

    /// Load recent conscience journal entries
    async fn load_conscience_journal(&self) -> Result<Vec<String>> {
        let journal_path = "/var/log/anna/journal.jsonl";

        match tokio::fs::read_to_string(journal_path).await {
            Ok(content) => {
                // Parse JSONL and extract summaries
                let entries: Vec<String> = content
                    .lines()
                    .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                    .filter_map(|v| v.get("summary").and_then(|s| s.as_str().map(String::from)))
                    .collect();
                Ok(entries)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    /// Load recent systemd journal logs
    async fn load_systemd_logs(&self) -> Result<Vec<String>> {
        let output = tokio::process::Command::new("journalctl")
            .args(&[
                "--user-unit=annad",
                "--since=1 hour ago",
                "--no-pager",
                "-q",
                "-n",
                "100",
            ])
            .output()
            .await?;

        if output.status.success() {
            let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(String::from)
                .collect();
            Ok(lines)
        } else {
            Ok(Vec::new())
        }
    }

    /// Calculate sentiment score (-1.0 to 1.0)
    fn calculate_sentiment_score(&self, entries: &[String]) -> f64 {
        if entries.is_empty() {
            return 0.0;
        }

        // Simple keyword-based sentiment analysis
        let positive_keywords = [
            "success",
            "approved",
            "completed",
            "healthy",
            "normal",
            "good",
            "stable",
            "optimal",
            "recovered",
        ];

        let negative_keywords = [
            "error",
            "failed",
            "rejected",
            "critical",
            "warning",
            "degraded",
            "unsafe",
            "violation",
            "blocked",
            "strain",
        ];

        let mut score = 0.0;
        let mut count = 0;

        for entry in entries {
            let entry_lower = entry.to_lowercase();

            // Count positive keywords
            for keyword in &positive_keywords {
                if entry_lower.contains(keyword) {
                    score += 1.0;
                    count += 1;
                }
            }

            // Count negative keywords
            for keyword in &negative_keywords {
                if entry_lower.contains(keyword) {
                    score -= 1.0;
                    count += 1;
                }
            }
        }

        if count > 0 {
            (score / count as f64).max(-1.0).min(1.0)
        } else {
            0.0
        }
    }

    /// Calculate token entropy (complexity/chaos indicator)
    fn calculate_token_entropy(&self, entries: &[String]) -> f64 {
        if entries.is_empty() {
            return 0.0;
        }

        // Tokenize all entries
        let mut token_counts: HashMap<String, usize> = HashMap::new();
        let mut total_tokens = 0;

        for entry in entries {
            for token in entry.split_whitespace() {
                let token_clean = token.to_lowercase();
                *token_counts.entry(token_clean).or_insert(0) += 1;
                total_tokens += 1;
            }
        }

        if total_tokens == 0 {
            return 0.0;
        }

        // Calculate Shannon entropy
        let mut entropy = 0.0;
        for count in token_counts.values() {
            let p = *count as f64 / total_tokens as f64;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        // Normalize to 0.0-1.0 (max entropy ~10 for natural language)
        (entropy / 10.0).min(1.0)
    }

    /// Calculate anomaly delta from baseline
    fn calculate_anomaly_delta(&mut self, entries: &[String]) -> f64 {
        if entries.is_empty() || self.baseline_frequencies.is_empty() {
            return 0.0;
        }

        // Calculate current token frequencies
        let mut current_frequencies: HashMap<String, f64> = HashMap::new();
        let mut total_tokens = 0;

        for entry in entries {
            for token in entry.split_whitespace() {
                let token_clean = token.to_lowercase();
                *current_frequencies.entry(token_clean).or_insert(0.0) += 1.0;
                total_tokens += 1;
            }
        }

        // Normalize frequencies
        for freq in current_frequencies.values_mut() {
            *freq /= total_tokens as f64;
        }

        // Calculate KL divergence from baseline
        let mut divergence = 0.0;
        for (token, current_freq) in &current_frequencies {
            if let Some(&baseline_freq) = self.baseline_frequencies.get(token) {
                if baseline_freq > 0.0 && *current_freq > 0.0 {
                    divergence += current_freq * (current_freq / baseline_freq).ln();
                }
            }
        }

        // Normalize to 0.0-1.0
        (divergence.abs() / 2.0).min(1.0)
    }

    /// Detect patterns in log entries
    fn detect_patterns(&self, entries: &[String]) -> Vec<String> {
        let mut patterns = Vec::new();

        // Detect repeated errors
        let error_count = entries
            .iter()
            .filter(|e| e.to_lowercase().contains("error"))
            .count();
        if error_count > entries.len() / 4 {
            patterns.push(format!(
                "High error frequency: {}/{} entries",
                error_count,
                entries.len()
            ));
        }

        // Detect degradation trend
        let degraded_count = entries
            .iter()
            .filter(|e| {
                e.to_lowercase().contains("degraded") || e.to_lowercase().contains("failed")
            })
            .count();
        if degraded_count > 5 {
            patterns.push("System degradation trend detected".to_string());
        }

        // Detect strain indicators
        let strain_keywords = ["strain", "pressure", "overload", "throttle"];
        let strain_count = entries
            .iter()
            .filter(|e| {
                let entry_lower = e.to_lowercase();
                strain_keywords
                    .iter()
                    .any(|&keyword| entry_lower.contains(keyword))
            })
            .count();

        if strain_count > 0 {
            patterns.push(format!(
                "Strain indicators detected ({} occurrences)",
                strain_count
            ));
        }

        // Detect recovery patterns
        let recovery_keywords = ["recovered", "restored", "resolved", "fixed"];
        let recovery_count = entries
            .iter()
            .filter(|e| {
                let entry_lower = e.to_lowercase();
                recovery_keywords
                    .iter()
                    .any(|&keyword| entry_lower.contains(keyword))
            })
            .count();

        if recovery_count > 0 {
            patterns.push(format!(
                "Recovery activity detected ({} occurrences)",
                recovery_count
            ));
        }

        patterns
    }

    /// Update baseline frequencies from new samples
    fn update_baseline(&mut self, entries: &[String]) {
        // Keep recent samples
        self.recent_samples.extend(entries.iter().cloned());

        // Keep only last 1000 samples
        if self.recent_samples.len() > 1000 {
            self.recent_samples = self
                .recent_samples
                .split_off(self.recent_samples.len() - 1000);
        }

        // Recalculate baseline frequencies
        self.baseline_frequencies.clear();
        let mut total_tokens = 0;

        for entry in &self.recent_samples {
            for token in entry.split_whitespace() {
                let token_clean = token.to_lowercase();
                *self.baseline_frequencies.entry(token_clean).or_insert(0.0) += 1.0;
                total_tokens += 1;
            }
        }

        // Normalize
        for freq in self.baseline_frequencies.values_mut() {
            *freq /= total_tokens as f64;
        }
    }
}

impl Default for SentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentiment_score_positive() {
        let analyzer = SentimentAnalyzer::new();
        let entries = vec![
            "System healthy and running normally".to_string(),
            "All checks completed successfully".to_string(),
        ];

        let score = analyzer.calculate_sentiment_score(&entries);
        assert!(score > 0.0);
    }

    #[test]
    fn test_sentiment_score_negative() {
        let analyzer = SentimentAnalyzer::new();
        let entries = vec![
            "Critical error detected in system".to_string(),
            "Operation failed with warnings".to_string(),
        ];

        let score = analyzer.calculate_sentiment_score(&entries);
        assert!(score < 0.0);
    }

    #[test]
    fn test_token_entropy() {
        let analyzer = SentimentAnalyzer::new();
        let entries = vec![
            "test test test".to_string(), // Low entropy
        ];

        let entropy1 = analyzer.calculate_token_entropy(&entries);

        let entries2 = vec![
            "alpha beta gamma delta epsilon".to_string(), // High entropy
        ];

        let entropy2 = analyzer.calculate_token_entropy(&entries2);

        assert!(entropy2 > entropy1);
    }

    #[test]
    fn test_pattern_detection() {
        let analyzer = SentimentAnalyzer::new();
        let entries = vec![
            "error in module A".to_string(),
            "error in module B".to_string(),
            "error in module C".to_string(),
        ];

        let patterns = analyzer.detect_patterns(&entries);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("High error frequency")));
    }
}

//! Active Falsification - Adversarial testing of hypotheses.
//!
//! For each hypothesis:
//! - Generate at least one disconfirming test in sandbox
//!
//! For each skill candidate:
//! - Generate counterexample sandboxes with parameter variations
//! - Try to break it with different conditions
//!
//! Store "failed falsification attempts" as evidence that INCREASES confidence.
//! Store successful falsification as negative memory that BLOCKS promotion.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A hypothesis to be falsified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// Unique ID
    pub id: String,
    /// The claim being made
    pub claim: String,
    /// Evidence supporting this hypothesis
    pub supporting_evidence: Vec<String>,
    /// Current confidence (0.0-1.0)
    pub confidence: f32,
    /// Falsification attempts
    pub falsification_attempts: Vec<FalsificationTest>,
    /// Has this been successfully falsified?
    pub falsified: bool,
    /// Falsification evidence (if falsified)
    pub falsification_evidence: Option<String>,
}

/// A test designed to falsify a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationTest {
    /// Test ID
    pub id: String,
    /// What this test tries to disprove
    pub target: String,
    /// The test command/action
    pub test_action: String,
    /// Environment variations applied
    pub variations: Vec<EnvironmentVariation>,
    /// Expected outcome if hypothesis is FALSE
    pub falsification_outcome: String,
    /// Actual result
    pub result: Option<FalsificationResult>,
    /// When this test was run
    pub ran_at: Option<String>,
}

/// Environment variation for counterexample testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariation {
    /// What aspect is varied
    pub aspect: String,
    /// The variation applied
    pub variation: String,
    /// Description
    pub description: String,
}

/// Standard environment variations to test
pub fn standard_variations() -> Vec<EnvironmentVariation> {
    vec![
        EnvironmentVariation {
            aspect: "dns_resolver".to_string(),
            variation: "1.1.1.1".to_string(),
            description: "Different DNS resolver (Cloudflare)".to_string(),
        },
        EnvironmentVariation {
            aspect: "dns_resolver".to_string(),
            variation: "8.8.8.8".to_string(),
            description: "Different DNS resolver (Google)".to_string(),
        },
        EnvironmentVariation {
            aspect: "dns_resolver".to_string(),
            variation: "offline".to_string(),
            description: "No network connectivity".to_string(),
        },
        EnvironmentVariation {
            aspect: "initramfs".to_string(),
            variation: "minimal".to_string(),
            description: "Minimal initramfs (no extra hooks)".to_string(),
        },
        EnvironmentVariation {
            aspect: "initramfs".to_string(),
            variation: "fallback".to_string(),
            description: "Fallback initramfs".to_string(),
        },
        EnvironmentVariation {
            aspect: "network_topology".to_string(),
            variation: "nat".to_string(),
            description: "Behind NAT".to_string(),
        },
        EnvironmentVariation {
            aspect: "network_topology".to_string(),
            variation: "vpn".to_string(),
            description: "Through VPN tunnel".to_string(),
        },
        EnvironmentVariation {
            aspect: "package_version".to_string(),
            variation: "older".to_string(),
            description: "Older package version".to_string(),
        },
        EnvironmentVariation {
            aspect: "package_version".to_string(),
            variation: "testing".to_string(),
            description: "Testing repository version".to_string(),
        },
        EnvironmentVariation {
            aspect: "filesystem".to_string(),
            variation: "btrfs".to_string(),
            description: "Btrfs filesystem".to_string(),
        },
        EnvironmentVariation {
            aspect: "filesystem".to_string(),
            variation: "ext4".to_string(),
            description: "Ext4 filesystem".to_string(),
        },
        EnvironmentVariation {
            aspect: "filesystem".to_string(),
            variation: "readonly".to_string(),
            description: "Read-only filesystem".to_string(),
        },
    ]
}

/// Result of a falsification test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Did the test successfully falsify the hypothesis?
    pub falsified: bool,
    /// Output from the test
    pub output: String,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Was this in sandbox?
    pub sandbox: bool,
    /// Duration in ms
    pub duration_ms: u64,
}

impl FalsificationResult {
    /// Create a successful falsification result
    pub fn falsified(output: &str, exit_code: i32) -> Self {
        Self {
            falsified: true,
            output: output.to_string(),
            exit_code: Some(exit_code),
            sandbox: true,
            duration_ms: 0,
        }
    }

    /// Create a failed falsification result (hypothesis survives)
    pub fn survived(output: &str, exit_code: i32) -> Self {
        Self {
            falsified: false,
            output: output.to_string(),
            exit_code: Some(exit_code),
            sandbox: true,
            duration_ms: 0,
        }
    }
}

/// The falsification engine
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FalsificationEngine {
    /// Active hypotheses being tested
    pub hypotheses: HashMap<String, Hypothesis>,
    /// Falsification history
    pub history: Vec<FalsificationRecord>,
    /// Available variations
    pub variations: Vec<EnvironmentVariation>,
    /// Minimum falsification attempts required
    pub min_attempts: usize,
}

/// Record of a falsification session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationRecord {
    /// Hypothesis ID
    pub hypothesis_id: String,
    /// Claim that was tested
    pub claim: String,
    /// Tests run
    pub tests_run: usize,
    /// Was it falsified?
    pub falsified: bool,
    /// Final confidence
    pub final_confidence: f32,
    /// Timestamp
    pub timestamp: String,
}

impl FalsificationEngine {
    /// Create new engine
    pub fn new() -> Self {
        Self {
            hypotheses: HashMap::new(),
            history: Vec::new(),
            variations: standard_variations(),
            min_attempts: 1,
        }
    }

    /// Register a hypothesis for testing
    pub fn register_hypothesis(&mut self, claim: &str, evidence: Vec<String>, confidence: f32) -> String {
        let id = uuid::Uuid::new_v4().to_string();

        let hypothesis = Hypothesis {
            id: id.clone(),
            claim: claim.to_string(),
            supporting_evidence: evidence,
            confidence,
            falsification_attempts: Vec::new(),
            falsified: false,
            falsification_evidence: None,
        };

        self.hypotheses.insert(id.clone(), hypothesis);
        id
    }

    /// Generate falsification tests for a hypothesis
    pub fn generate_tests(&self, hypothesis_id: &str) -> Vec<FalsificationTest> {
        let hypothesis = match self.hypotheses.get(hypothesis_id) {
            Some(h) => h,
            None => return Vec::new(),
        };

        let mut tests = Vec::new();

        // Generate a test for each relevant variation
        for variation in &self.variations {
            if self.variation_relevant(&hypothesis.claim, variation) {
                let test = FalsificationTest {
                    id: uuid::Uuid::new_v4().to_string(),
                    target: hypothesis.claim.clone(),
                    test_action: self.generate_test_action(&hypothesis.claim, variation),
                    variations: vec![variation.clone()],
                    falsification_outcome: format!(
                        "Hypothesis fails with {}",
                        variation.description
                    ),
                    result: None,
                    ran_at: None,
                };
                tests.push(test);
            }
        }

        // Always generate at least one basic contradiction test
        if tests.is_empty() {
            tests.push(FalsificationTest {
                id: uuid::Uuid::new_v4().to_string(),
                target: hypothesis.claim.clone(),
                test_action: format!("Test negation of: {}", hypothesis.claim),
                variations: Vec::new(),
                falsification_outcome: "Negation of claim holds true".to_string(),
                result: None,
                ran_at: None,
            });
        }

        tests
    }

    /// Check if a variation is relevant to a claim
    fn variation_relevant(&self, claim: &str, variation: &EnvironmentVariation) -> bool {
        let claim_lower = claim.to_lowercase();

        match variation.aspect.as_str() {
            "dns_resolver" => {
                claim_lower.contains("network")
                    || claim_lower.contains("dns")
                    || claim_lower.contains("resolve")
                    || claim_lower.contains("connect")
            }
            "initramfs" => {
                claim_lower.contains("boot")
                    || claim_lower.contains("kernel")
                    || claim_lower.contains("initramfs")
            }
            "network_topology" => {
                claim_lower.contains("network")
                    || claim_lower.contains("firewall")
                    || claim_lower.contains("connect")
            }
            "package_version" => {
                claim_lower.contains("package")
                    || claim_lower.contains("install")
                    || claim_lower.contains("version")
            }
            "filesystem" => {
                claim_lower.contains("file")
                    || claim_lower.contains("write")
                    || claim_lower.contains("read")
                    || claim_lower.contains("mount")
            }
            _ => true, // Default to relevant
        }
    }

    /// Generate a test action for a claim with a variation
    fn generate_test_action(&self, claim: &str, variation: &EnvironmentVariation) -> String {
        format!(
            "In sandbox with {}: verify that '{}' holds",
            variation.description, claim
        )
    }

    /// Record a test result
    pub fn record_result(
        &mut self,
        hypothesis_id: &str,
        test_id: &str,
        result: FalsificationResult,
    ) -> Result<(), String> {
        let hypothesis = self
            .hypotheses
            .get_mut(hypothesis_id)
            .ok_or("Hypothesis not found")?;

        // Find the test
        let test = hypothesis
            .falsification_attempts
            .iter_mut()
            .find(|t| t.id == test_id);

        if let Some(test) = test {
            test.result = Some(result.clone());
            test.ran_at = Some(chrono::Utc::now().to_rfc3339());
        }

        // Update hypothesis based on result
        if result.falsified {
            hypothesis.falsified = true;
            hypothesis.falsification_evidence = Some(result.output.clone());
            hypothesis.confidence = 0.0;
        } else {
            // Surviving falsification INCREASES confidence
            let attempts = hypothesis
                .falsification_attempts
                .iter()
                .filter(|t| t.result.is_some())
                .count();
            let survived = hypothesis
                .falsification_attempts
                .iter()
                .filter(|t| t.result.as_ref().map(|r| !r.falsified).unwrap_or(false))
                .count();

            if attempts > 0 {
                // Confidence boost for surviving falsification
                let survival_rate = survived as f32 / attempts as f32;
                hypothesis.confidence = (hypothesis.confidence + survival_rate * 0.1).min(0.95);
            }
        }

        Ok(())
    }

    /// Add a test to a hypothesis
    pub fn add_test(&mut self, hypothesis_id: &str, test: FalsificationTest) -> Result<(), String> {
        let hypothesis = self
            .hypotheses
            .get_mut(hypothesis_id)
            .ok_or("Hypothesis not found")?;
        hypothesis.falsification_attempts.push(test);
        Ok(())
    }

    /// Check if hypothesis has passed minimum falsification attempts
    pub fn has_minimum_attempts(&self, hypothesis_id: &str) -> bool {
        self.hypotheses
            .get(hypothesis_id)
            .map(|h| {
                h.falsification_attempts
                    .iter()
                    .filter(|t| t.result.is_some())
                    .count()
                    >= self.min_attempts
            })
            .unwrap_or(false)
    }

    /// Get confidence after falsification testing
    pub fn get_confidence(&self, hypothesis_id: &str) -> Option<f32> {
        self.hypotheses.get(hypothesis_id).map(|h| h.confidence)
    }

    /// Check if hypothesis was falsified
    pub fn is_falsified(&self, hypothesis_id: &str) -> bool {
        self.hypotheses
            .get(hypothesis_id)
            .map(|h| h.falsified)
            .unwrap_or(false)
    }

    /// Get falsification log for a hypothesis (for promotion evidence)
    pub fn get_falsification_log(&self, hypothesis_id: &str) -> Option<Vec<&FalsificationTest>> {
        self.hypotheses.get(hypothesis_id).map(|h| {
            h.falsification_attempts
                .iter()
                .filter(|t| t.result.is_some())
                .collect()
        })
    }
}

/// Acceptance test: at least one promoted skill must show a falsification log
/// with counterexample generation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_registration() {
        let mut engine = FalsificationEngine::new();
        let id = engine.register_hypothesis(
            "pacman -S works for all packages",
            vec!["worked for vim".to_string()],
            0.8,
        );

        assert!(engine.hypotheses.contains_key(&id));
    }

    #[test]
    fn test_falsification_generates_tests() {
        let mut engine = FalsificationEngine::new();
        let id = engine.register_hypothesis(
            "Network connection works",
            vec!["ping successful".to_string()],
            0.9,
        );

        let tests = engine.generate_tests(&id);
        assert!(!tests.is_empty());

        // Should have DNS-related tests
        assert!(tests.iter().any(|t| t
            .variations
            .iter()
            .any(|v| v.aspect == "dns_resolver")));
    }

    #[test]
    fn test_successful_falsification_drops_confidence() {
        let mut engine = FalsificationEngine::new();
        let id = engine.register_hypothesis("test claim", vec![], 0.8);

        let test = FalsificationTest {
            id: "test1".to_string(),
            target: "test claim".to_string(),
            test_action: "test".to_string(),
            variations: vec![],
            falsification_outcome: "fails".to_string(),
            result: None,
            ran_at: None,
        };

        engine.add_test(&id, test).unwrap();

        // Record successful falsification
        engine
            .record_result(&id, "test1", FalsificationResult::falsified("broke it", 1))
            .unwrap();

        assert!(engine.is_falsified(&id));
        assert_eq!(engine.get_confidence(&id), Some(0.0));
    }

    #[test]
    fn test_survived_falsification_increases_confidence() {
        let mut engine = FalsificationEngine::new();
        let id = engine.register_hypothesis("test claim", vec![], 0.5);

        let test = FalsificationTest {
            id: "test1".to_string(),
            target: "test claim".to_string(),
            test_action: "test".to_string(),
            variations: vec![],
            falsification_outcome: "fails".to_string(),
            result: None,
            ran_at: None,
        };

        engine.add_test(&id, test).unwrap();

        // Record FAILED falsification (hypothesis survives)
        engine
            .record_result(&id, "test1", FalsificationResult::survived("still works", 0))
            .unwrap();

        assert!(!engine.is_falsified(&id));
        // Confidence should increase
        assert!(engine.get_confidence(&id).unwrap() > 0.5);
    }

    #[test]
    fn test_standard_variations_exist() {
        let variations = standard_variations();
        assert!(variations.len() >= 5);

        // Should have DNS variations
        assert!(variations.iter().any(|v| v.aspect == "dns_resolver"));
        // Should have filesystem variations
        assert!(variations.iter().any(|v| v.aspect == "filesystem"));
    }
}

//! Verification probes for clarification answers.
//!
//! Safe, read-only probes to verify user clarifications against system reality.
//! No destructive commands - only existence checks.
//! v0.0.160: System verifiers extracted to separate module.

use anna_shared::facts::{FactKey, FactsStore};
use anna_shared::intake::{VerificationResult, VerifyPlan};
use anna_shared::rpc::ProbeResult;
use tracing::info;

// Re-export system verification functions for backwards compatibility
pub use crate::system_verifiers::{
    binary_exists, is_safe_name, verify_binary_exists, verify_directory_exists, verify_file_exists,
    verify_interface_exists, verify_mount_exists, verify_unit_exists,
};

/// Run a verification probe and return the result
pub fn run_verify_probe(plan: &VerifyPlan, user_answer: &str) -> VerificationResult {
    match plan {
        VerifyPlan::None => VerificationResult::success(user_answer.to_string(), "no_verification"),

        VerifyPlan::BinaryExists { binary } => {
            let target = if binary == "PLACEHOLDER" {
                user_answer
            } else {
                binary.as_str()
            };
            verify_binary_exists(target)
        }

        VerifyPlan::UnitExists { unit } => {
            let target = if unit == "PLACEHOLDER" {
                user_answer
            } else {
                unit.as_str()
            };
            verify_unit_exists(target)
        }

        VerifyPlan::MountExists { mount } => {
            let target = if mount == "PLACEHOLDER" {
                user_answer
            } else {
                mount.as_str()
            };
            verify_mount_exists(target)
        }

        VerifyPlan::InterfaceExists { iface } => {
            let target = if iface == "PLACEHOLDER" {
                user_answer
            } else {
                iface.as_str()
            };
            verify_interface_exists(target)
        }

        VerifyPlan::FileExists { path } => {
            let target = if path == "PLACEHOLDER" {
                user_answer
            } else {
                path.as_str()
            };
            verify_file_exists(target)
        }

        VerifyPlan::DirectoryExists { path } => {
            let target = if path == "PLACEHOLDER" {
                user_answer
            } else {
                path.as_str()
            };
            verify_directory_exists(target)
        }

        VerifyPlan::FromEvidence { key } => {
            // This is handled by checking existing probe evidence
            VerificationResult::success(user_answer.to_string(), &format!("evidence:{}", key))
        }
    }
}

/// Verify clarification answer and update facts store if successful
pub fn verify_and_store(
    plan: &VerifyPlan,
    user_answer: &str,
    fact_key: Option<&FactKey>,
    facts: &mut FactsStore,
) -> VerificationResult {
    let result = run_verify_probe(plan, user_answer);

    if result.verified {
        if let (Some(key), Some(ref value)) = (fact_key, &result.value) {
            // Store the verified fact
            facts.set_verified(key.clone(), value.clone(), result.source.clone());
            info!("Stored verified fact: {:?} = {}", key, value);

            // For binaries, also store the binary availability fact
            if let FactKey::PreferredEditor = key {
                let binary_key = FactKey::BinaryAvailable(user_answer.to_string());
                facts.set_verified(binary_key, value.clone(), result.source.clone());
            }
        }
    }

    result
}

/// Verify from existing probe evidence
pub fn verify_from_evidence(
    plan: &VerifyPlan,
    user_answer: &str,
    probe_results: &[ProbeResult],
) -> VerificationResult {
    match plan {
        VerifyPlan::FromEvidence { key } => {
            // Search probe results for matching evidence
            for probe in probe_results {
                if probe.exit_code == 0 {
                    // Check if answer matches evidence
                    let stdout_lower = probe.stdout.to_lowercase();
                    let answer_lower = user_answer.to_lowercase();

                    match key.as_str() {
                        "network_interfaces" => {
                            // For wifi/ethernet check
                            let has_wifi =
                                stdout_lower.contains("wlan") || stdout_lower.contains("wlp");
                            let has_eth =
                                stdout_lower.contains("eth") || stdout_lower.contains("enp");

                            match answer_lower.as_str() {
                                "wifi" if has_wifi => {
                                    return VerificationResult::success(
                                        user_answer.to_string(),
                                        "evidence:ip_link",
                                    );
                                }
                                "ethernet" if has_eth => {
                                    return VerificationResult::success(
                                        user_answer.to_string(),
                                        "evidence:ip_link",
                                    );
                                }
                                "both" if has_wifi && has_eth => {
                                    return VerificationResult::success(
                                        user_answer.to_string(),
                                        "evidence:ip_link",
                                    );
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            // Generic contains check
                            if stdout_lower.contains(&answer_lower) {
                                return VerificationResult::success(
                                    user_answer.to_string(),
                                    &format!("evidence:{}", key),
                                );
                            }
                        }
                    }
                }
            }

            VerificationResult::failed(
                &format!("Could not verify {} from evidence", user_answer),
                "evidence_check",
            )
        }
        _ => VerificationResult::failed("Not an evidence verification plan", "invalid_plan"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_name() {
        assert!(is_safe_name("vim"));
        assert!(is_safe_name("nginx.service"));
        assert!(is_safe_name("my-app"));
        assert!(is_safe_name("app_name"));
        assert!(!is_safe_name(""));
        assert!(!is_safe_name("cmd; rm -rf"));
        assert!(!is_safe_name("$(malicious)"));
    }

    #[test]
    fn test_verify_plan_none() {
        let result = run_verify_probe(&VerifyPlan::None, "anything");
        assert!(result.verified);
        assert_eq!(result.value, Some("anything".to_string()));
    }

    #[test]
    fn test_verification_result_success() {
        let result = VerificationResult::success("/usr/bin/vim".to_string(), "test");
        assert!(result.verified);
        assert!(result.alternatives.is_empty());
    }

    #[test]
    fn test_verification_result_failed_with_alternatives() {
        let result = VerificationResult::failed_with_alternatives(
            "not found",
            vec!["alt1".to_string(), "alt2".to_string()],
            "test",
        );
        assert!(!result.verified);
        assert_eq!(result.alternatives.len(), 2);
    }
}

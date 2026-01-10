//! Contradiction detection between LLM answers and command output.
//!
//! v0.0.890: Detects status, numeric, presence, and boolean contradictions

use anna_shared::rpc::{ValidationIssueType, ValidationWarning};
use regex::Regex;

use super::{RE_CONTEXT, RE_MEM};

/// Check for contradictions between answer text and command output
pub fn check_contradiction(text: &str, command_output: &str) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();
    let output_lower = command_output.to_lowercase();

    if let Some(warning) = check_status_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    if let Some(warning) = check_numeric_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    if let Some(warning) = check_presence_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    if let Some(warning) = check_boolean_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    None
}

/// Check for status contradictions (running vs stopped)
pub fn check_status_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    let status_pairs = [
        ("running", "stopped"),
        ("running", "dead"),
        ("running", "not running"),
        ("active", "inactive"),
        ("active", "failed"),
        ("enabled", "disabled"),
        ("up", "down"),
        ("online", "offline"),
        ("started", "stopped"),
        ("healthy", "unhealthy"),
        ("connected", "disconnected"),
        ("mounted", "unmounted"),
        ("loaded", "not loaded"),
    ];

    for (positive, negative) in status_pairs {
        if answer.contains(positive) && output.contains(negative) && !output.contains(positive) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Contradiction,
                message: format!(
                    "Answer says '{}' but command output shows '{}'",
                    positive, negative
                ),
                severity: "high".to_string(),
            });
        }
        if answer.contains(negative) && output.contains(positive) && !output.contains(negative) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Contradiction,
                message: format!(
                    "Answer says '{}' but command output shows '{}'",
                    negative, positive
                ),
                severity: "high".to_string(),
            });
        }
    }

    None
}

/// Check for numeric contradictions with tolerance for units
pub fn check_numeric_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    for cap in RE_MEM.captures_iter(answer) {
        let answer_num: f64 = cap.get(1)?.as_str().parse().ok()?;
        let answer_unit = cap.get(2)?.as_str();
        let answer_gb = normalize_to_gb(answer_num, answer_unit);

        for out_cap in RE_MEM.captures_iter(output) {
            let output_num: f64 = out_cap.get(1)?.as_str().parse().ok()?;
            let output_unit = out_cap.get(2)?.as_str();
            let output_gb = normalize_to_gb(output_num, output_unit);

            if answer_gb > 0.5 && output_gb > 0.5 {
                let ratio = answer_gb / output_gb;
                if ratio < 0.5 || ratio > 2.0 {
                    return Some(ValidationWarning {
                        issue_type: ValidationIssueType::Contradiction,
                        message: format!(
                            "Answer states '{:.1}{}' but output shows '{:.1}{}'",
                            answer_num, answer_unit, output_num, output_unit
                        ),
                        severity: "high".to_string(),
                    });
                }
            }
        }
    }

    None
}

/// Normalize memory value to GB for comparison
pub fn normalize_to_gb(value: f64, unit: &str) -> f64 {
    match unit.to_lowercase().as_str() {
        "tb" | "tib" | "ti" | "t" => value * 1024.0,
        "gb" | "gib" | "gi" | "g" => value,
        "mb" | "mib" | "mi" | "m" => value / 1024.0,
        "kb" | "kib" | "ki" | "k" => value / (1024.0 * 1024.0),
        _ => value,
    }
}

/// Check for presence contradictions (installed vs not found)
pub fn check_presence_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    let presence_positive = ["installed", "available", "found", "exists", "present"];
    let presence_negative = [
        "not installed",
        "not found",
        "not available",
        "does not exist",
        "no such",
        "error: target not found",
        "package not found",
    ];

    for positive in presence_positive {
        if answer.contains(positive) {
            for negative in presence_negative {
                if output.contains(negative) {
                    return Some(ValidationWarning {
                        issue_type: ValidationIssueType::Contradiction,
                        message: format!(
                            "Answer says '{}' but command output shows '{}'",
                            positive, negative
                        ),
                        severity: "high".to_string(),
                    });
                }
            }
        }
    }

    for negative in presence_negative {
        let neg_parts: Vec<&str> = negative.split_whitespace().collect();
        if neg_parts.iter().any(|&n| answer.contains(n)) {
            if presence_positive.iter().any(|&p| output.contains(p)) && !output.contains("not") {
                return Some(ValidationWarning {
                    issue_type: ValidationIssueType::Contradiction,
                    message: "Answer claims something is not present but command output suggests it exists".to_string(),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    None
}

/// Check for boolean contradictions (yes/no, enabled/disabled)
pub fn check_boolean_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    let bool_pairs = [
        ("yes", "no"),
        ("true", "false"),
        ("on", "off"),
        ("1", "0"),
        ("success", "failed"),
        ("passed", "failed"),
        ("ok", "error"),
    ];

    for cap in RE_CONTEXT.captures_iter(answer) {
        let answer_val = cap.get(1)?.as_str().to_lowercase();

        for (positive, negative) in bool_pairs {
            if answer_val == positive {
                for out_cap in RE_CONTEXT.captures_iter(output) {
                    let output_val = out_cap.get(1)?.as_str().to_lowercase();
                    if output_val == negative {
                        return Some(ValidationWarning {
                            issue_type: ValidationIssueType::Contradiction,
                            message: format!(
                                "Answer shows '{}' but output shows '{}'",
                                positive, negative
                            ),
                            severity: "high".to_string(),
                        });
                    }
                }
            }
        }
    }

    None
}

/// Check if answer assumes something exists but output shows it doesn't
pub fn check_existence_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    let output_lower = output.to_lowercase();

    let nonexist_patterns = [
        "does not exist",
        "no such file",
        "not found",
        "unit .* could not be found",
        "no packages found",
        "command not found",
    ];

    for pattern in nonexist_patterns {
        if output_lower.contains(pattern)
            || (pattern.contains(".*") && {
                let re = Regex::new(&format!("(?i){}", pattern)).ok();
                re.map(|r| r.is_match(&output_lower)).unwrap_or(false)
            })
        {
            let answer_lower = answer.to_lowercase();
            let property_claims = ["is ", "has ", "uses ", "runs ", "contains "];

            for claim in property_claims {
                if answer_lower.contains(claim)
                    && !answer_lower.contains("does not exist")
                    && !answer_lower.contains("not found")
                    && !answer_lower.contains("doesn't exist")
                {
                    return Some(ValidationWarning {
                        issue_type: ValidationIssueType::Contradiction,
                        message: format!(
                            "Answer describes properties of something that doesn't exist (output shows: {})",
                            pattern.replace(".*", "...")
                        ),
                        severity: "high".to_string(),
                    });
                }
            }
        }
    }

    None
}

/// Check for arithmetic errors in derived values
pub fn check_arithmetic_error(answer: &str, output: &str) -> Option<ValidationWarning> {
    let output_values: Vec<f64> = RE_MEM
        .captures_iter(output)
        .filter_map(|cap| {
            let num: f64 = cap.get(1)?.as_str().parse().ok()?;
            let unit = cap.get(2)?.as_str();
            Some(normalize_to_gb(num, unit))
        })
        .collect();

    if output_values.len() < 2 {
        return None;
    }

    for cap in RE_MEM.captures_iter(answer) {
        let answer_num: f64 = cap.get(1)?.as_str().parse().ok()?;
        let answer_unit = cap.get(2)?.as_str();
        let answer_gb = normalize_to_gb(answer_num, answer_unit);

        let matches_single = output_values.iter().any(|&v| {
            let ratio = answer_gb / v;
            ratio > 0.85 && ratio < 1.15
        });

        if matches_single {
            continue;
        }

        let total: f64 = output_values.iter().sum();
        let sum_ratio = answer_gb / total;
        let is_reasonable_sum = sum_ratio > 0.85 && sum_ratio < 1.15;

        let max_output = output_values.iter().cloned().fold(f64::MIN, f64::max);
        if answer_gb > max_output * 2.5 && !is_reasonable_sum {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Hallucination,
                message: format!(
                    "Answer claims {:.1}{} but output values don't support this (max: {:.1}GB, sum: {:.1}GB)",
                    answer_num, answer_unit, max_output, total
                ),
                severity: "high".to_string(),
            });
        }
    }

    None
}

//! Pending clarification verification (v0.0.227).

use super::types::VerifyResult;

/// Verify a clarification answer (e.g., check if editor is installed)
pub fn verify_answer(answer: &str, verify_cmd: Option<&str>) -> VerifyResult {
    let Some(cmd_template) = verify_cmd else {
        // No verification needed
        return VerifyResult::Verified {
            value: answer.to_string(),
        };
    };

    let cmd = cmd_template.replace("{}", answer);
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.is_empty() {
        return VerifyResult::Verified {
            value: answer.to_string(),
        };
    }

    // Run verification command
    let output = std::process::Command::new(parts[0])
        .args(&parts[1..])
        .output();

    match output {
        Ok(out) if out.status.success() => VerifyResult::Verified {
            value: answer.to_string(),
        },
        _ => {
            // Check for common alternatives
            if answer == "vim" {
                // Check if "vi" exists instead
                if let Ok(out) = std::process::Command::new("which").arg("vi").output() {
                    if out.status.success() {
                        return VerifyResult::AlternativeFound {
                            requested: "vim".to_string(),
                            available: "vi".to_string(),
                        };
                    }
                }
            }
            VerifyResult::NotVerified {
                value: answer.to_string(),
                reason: format!("{} not found", answer),
            }
        }
    }
}

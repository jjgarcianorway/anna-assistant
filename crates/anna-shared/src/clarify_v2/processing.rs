//! Clarification processing (v0.0.197).

use crate::inventory::InventoryCache;
use crate::verify::{run_verification, VerificationStep, VerifyExpectation};

use super::types::{ClarifyOption, ClarifyRequest, ClarifyResponse, ClarifyResult};

/// Process clarification response with verification
pub fn process_response(
    request: &ClarifyRequest,
    response: &ClarifyResponse,
    cache: &InventoryCache,
) -> ClarifyResult {
    if response.cancelled {
        return ClarifyResult::Cancelled;
    }

    // Handle free text (other)
    if let Some(text) = &response.free_text {
        if text.is_empty() {
            return ClarifyResult::Cancelled;
        }

        // Verify the free text input
        let step = VerificationStep::editor_installed(text);
        let result = run_verification(&step);

        if result.passed {
            return ClarifyResult::Verified {
                value: text.clone(),
                fact_key: Some("preferred_editor".to_string()),
            };
        } else {
            let alts = find_installed_alternatives(text, cache);
            return ClarifyResult::VerificationFailed {
                value: text.clone(),
                error: result.error.unwrap_or_else(|| "not installed".to_string()),
                alternatives: alts,
            };
        }
    }

    // Handle numeric selection
    if let Some(key) = response.selected {
        if let Some(opt) = request.get_option(key) {
            if let Some(verify_exp) = &opt.verify {
                let step = VerificationStep::new(
                    format!("verify_{}", opt.value),
                    format!("Verify {}", opt.label),
                    verify_exp.clone(),
                );
                let result = run_verification(&step);

                if result.passed {
                    return ClarifyResult::Verified {
                        value: opt.value.clone(),
                        fact_key: Some("preferred_editor".to_string()),
                    };
                } else {
                    let alts = find_installed_alternatives(&opt.value, cache);
                    return ClarifyResult::VerificationFailed {
                        value: opt.value.clone(),
                        error: result.error.unwrap_or_else(|| "failed".to_string()),
                        alternatives: alts,
                    };
                }
            } else {
                return ClarifyResult::Verified {
                    value: opt.value.clone(),
                    fact_key: None,
                };
            }
        }
    }

    ClarifyResult::Cancelled
}

/// Find installed alternatives for a tool
pub fn find_installed_alternatives(tool: &str, cache: &InventoryCache) -> Vec<String> {
    let alt_map: &[(&str, &[&str])] = &[
        ("vim", &["nvim", "vi", "nano", "micro"]),
        ("nvim", &["vim", "vi", "nano", "micro"]),
        ("emacs", &["vim", "nano", "code", "nvim"]),
        ("code", &["vim", "nano", "nvim", "emacs"]),
        ("nano", &["vim", "micro", "vi", "nvim"]),
        ("vi", &["vim", "nano", "nvim", "micro"]),
        ("micro", &["nano", "vim", "nvim", "vi"]),
    ];

    let mut alts = Vec::new();
    for (t, alternatives) in alt_map {
        if *t == tool {
            for alt in *alternatives {
                if cache.is_installed(alt).unwrap_or(false) {
                    alts.push(alt.to_string());
                }
            }
            break;
        }
    }
    alts
}

/// Generate installed-only editor request (v0.45.x: shows only installed editors)
pub fn editor_request(cache: &InventoryCache) -> ClarifyRequest {
    use super::types::KEY_OTHER;

    let editors = [
        ("vim", "Vim"),
        ("nvim", "Neovim"),
        ("nano", "Nano"),
        ("emacs", "Emacs"),
        ("code", "VS Code"),
        ("micro", "Micro"),
        ("vi", "Vi"),
    ];

    let mut opts = Vec::new();
    let mut key: u8 = 1;

    for (cmd, label) in &editors {
        if cache.is_installed(cmd).unwrap_or(false) && key < KEY_OTHER {
            // Use friendly label for display, command for value
            opts.push(ClarifyOption::new(key, *label, *cmd).with_verify(
                VerifyExpectation::CommandExists {
                    name: cmd.to_string(),
                },
            ));
            key += 1;
        }
    }

    ClarifyRequest::new("editor_select", "Which editor do you prefer?")
        .with_options(opts)
        .with_reason("Options shown are installed on your system")
}

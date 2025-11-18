//! LLM Upgrade Detection - Step 3
//!
//! Detects when hardware improves and suggests upgrading to a better model

use crate::context::db::ContextDb;
use crate::hardware_capability::{HardwareAssessment, LlmCapability};
// LlmConfig imported but not used - removed
use crate::model_profiles::find_upgrade_profile;
use anyhow::Result;

/// Check if a brain upgrade is available and store suggestion if found
///
/// This should be called by the daemon on startup to detect hardware improvements
pub async fn check_for_brain_upgrade(db: &ContextDb) -> Result<()> {
    // Load current LLM config
    let config = db.load_llm_config().await?;

    // Only check if we have a local model configured
    if let Some(profile_id) = &config.model_profile_id {
        // Assess current hardware
        let current_hw = HardwareAssessment::assess();

        // Load initial capability from when model was first set up
        let initial_capability = load_initial_capability(db).await?;

        if let Some(initial_cap) = initial_capability {
            // Check if hardware improved
            if current_hw.llm_capability > initial_cap {
                // Hardware got better! Check if better model is available
                let ram_gb = current_hw.total_ram_mb as f64 / 1024.0;
                let cores = current_hw.cpu_cores;

                if let Some(upgrade) = find_upgrade_profile(profile_id, ram_gb, cores) {
                    // Better model is available!
                    // Store upgrade suggestion
                    let suggestion = format!(
                        "{}|{}|{:.1}",
                        upgrade.id, upgrade.model_name, upgrade.size_gb
                    );
                    db.save_preference("pending_brain_upgrade", &suggestion)
                        .await?;

                    eprintln!(
                        "Debug: Brain upgrade available: {} → {}",
                        profile_id, upgrade.id
                    );
                }
            }
        }
    }

    Ok(())
}

/// Load initial capability from DB
async fn load_initial_capability(db: &ContextDb) -> Result<Option<LlmCapability>> {
    let tier_str = db.load_preference("llm_initial_capability").await?;

    Ok(tier_str.and_then(|s| match s.as_str() {
        "high" => Some(LlmCapability::High),
        "medium" => Some(LlmCapability::Medium),
        "low" => Some(LlmCapability::Low),
        _ => None,
    }))
}

/// Check if there's a pending brain upgrade suggestion
pub async fn has_pending_upgrade(db: &ContextDb) -> Result<bool> {
    Ok(db.load_preference("pending_brain_upgrade").await?.is_some())
}

/// Get and clear pending brain upgrade suggestion
///
/// Returns (new_profile_id, model_name, size_gb) if upgrade is pending
pub async fn get_and_clear_pending_upgrade(
    db: &ContextDb,
) -> Result<Option<(String, String, f64)>> {
    if let Some(suggestion) = db.load_preference("pending_brain_upgrade").await? {
        // Clear the flag
        db.save_preference("pending_brain_upgrade", "").await?;

        // Parse: "profile_id|model_name|size_gb"
        let parts: Vec<&str> = suggestion.split('|').collect();
        if parts.len() == 3 {
            if let Ok(size_gb) = parts[2].parse::<f64>() {
                return Ok(Some((parts[0].to_string(), parts[1].to_string(), size_gb)));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_profiles::find_upgrade_profile;

    #[test]
    fn test_capability_ordering() {
        assert!(LlmCapability::Medium > LlmCapability::Low);
        assert!(LlmCapability::High > LlmCapability::Medium);
        assert!(LlmCapability::High > LlmCapability::Low);
    }

    #[test]
    fn test_upgrade_detected_when_capability_improves() {
        // Start with Low capability (1B model)
        let initial_cap = LlmCapability::Low;
        let current_cap = LlmCapability::High;

        // Should detect improvement
        assert!(current_cap > initial_cap);

        // Should find an upgrade for the 1B model
        let upgrade = find_upgrade_profile("ollama-llama3.2-1b", 16.0, 8);
        assert!(upgrade.is_some());
    }

    #[test]
    fn test_no_upgrade_when_capability_unchanged() {
        // Capability stays the same
        let initial_cap = LlmCapability::Medium;
        let current_cap = LlmCapability::Medium;

        // No improvement
        assert!(!(current_cap > initial_cap));
    }

    #[test]
    fn test_no_upgrade_when_capability_degrades() {
        // Capability gets worse (user downgrades RAM)
        let initial_cap = LlmCapability::High;
        let current_cap = LlmCapability::Low;

        // Degraded
        assert!(!(current_cap > initial_cap));
    }

    #[test]
    fn test_no_upgrade_when_already_best_model() {
        // Already using the best available model
        let upgrade = find_upgrade_profile("ollama-llama3.1-8b", 32.0, 16);

        // No better model available
        assert!(upgrade.is_none());
    }

    #[test]
    fn test_pending_suggestion_format() {
        // Test the suggestion format parsing
        let suggestion = "ollama-llama3.2-3b|llama3.2:3b|2.0";
        let parts: Vec<&str> = suggestion.split('|').collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "ollama-llama3.2-3b");
        assert_eq!(parts[1], "llama3.2:3b");
        assert_eq!(parts[2].parse::<f64>().unwrap(), 2.0);
    }
}

//! Fact management for clarifications (v0.0.197).

use crate::facts::{FactKey, FactSource, FactValue, FactsStore};
use crate::inventory::InventoryCache;

/// Check if clarification can be skipped
/// - Skip if fact is fresh and verified
/// - Skip if only one option (auto-select)
pub fn should_skip(
    fact_key: &FactKey,
    facts: &FactsStore,
    cache: &InventoryCache,
) -> Option<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if fact_key == &FactKey::PreferredEditor {
        // Check if we have a verified editor fact
        if let Some(fact) = facts.get_fresh(&FactKey::PreferredEditor, now) {
            // Verify it's still installed
            if cache.is_installed(&fact.value).unwrap_or(false) {
                return Some(fact.value.clone());
            }
        }

        // Check if only one editor installed (auto-select)
        let installed = cache.installed_editors();
        if installed.len() == 1 {
            return Some(installed[0].to_string());
        }
    }

    None
}

/// Store verified fact from clarification
pub fn store_fact(fact_key: FactKey, value: &str, facts: &mut FactsStore, transcript_id: &str) {
    facts.upsert_verified(
        fact_key,
        FactValue::String(value.to_string()),
        FactSource::UserConfirmed {
            transcript_id: transcript_id.to_string(),
        },
        90, // User-confirmed confidence
    );
}

/// Invalidate fact when tool is uninstalled
pub fn invalidate_on_uninstall(tool: &str, facts: &mut FactsStore, cache: &InventoryCache) -> bool {
    // Check if tool still exists
    if cache.is_installed(tool).unwrap_or(false) {
        return false; // Still installed
    }

    // Mark related facts as stale
    if let Some(value) = facts.get_verified(&FactKey::PreferredEditor) {
        if value == tool {
            facts.invalidate(&FactKey::PreferredEditor);
            return true;
        }
    }

    false
}

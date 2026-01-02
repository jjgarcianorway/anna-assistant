// v0.0.564: Settings Sync - Utilities (Phase 140)
// Helper functions for settings sync

use super::manager::SyncManager;

/// Format sync status for display
pub fn format_sync_status(manager: &SyncManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Sync ===\n\n");
    output.push_str(&format!("Provider: {}\n", manager.config.provider));
    output.push_str(&format!("Status: {}\n", manager.status()));

    if let Some(location) = &manager.config.location {
        output.push_str(&format!("Location: {}\n", location));
    }

    if let Some(last) = manager.config.last_sync {
        output.push_str(&format!("Last sync: {}\n", last.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    output.push_str(&format!("Auto-sync: {}\n", manager.config.auto_sync));

    if manager.config.is_sync_due() {
        output.push_str("\nSync is due!\n");
    }

    output
}

/// Fun fact about settings sync
pub fn settings_sync_fun_fact() -> &'static str {
    "Anna can sync your settings across machines using a shared folder or git repository!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_sync_status() {
        let manager = SyncManager::new();
        let output = format_sync_status(&manager);
        assert!(output.contains("Sync"));
        assert!(output.contains("Not configured"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_sync_fun_fact();
        assert!(fact.contains("sync"));
    }
}

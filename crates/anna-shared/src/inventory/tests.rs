//! Tests for inventory module (v0.0.188).

#[cfg(test)]
mod tests {
    use crate::inventory::{InventoryCache, InventoryItem, InventoryState};

    #[test]
    fn test_inventory_cache_new() {
        let cache = InventoryCache::new();
        assert!(cache.items.is_empty());
        assert_eq!(cache.version, 1);
    }

    #[test]
    fn test_inventory_item_installed() {
        let item = InventoryItem::installed("vim", "/usr/bin/vim");
        assert_eq!(item.state, InventoryState::Installed);
        assert_eq!(item.path, Some("/usr/bin/vim".to_string()));
        assert!(!item.is_stale());
    }

    #[test]
    fn test_inventory_item_stale() {
        let mut item = InventoryItem::installed("vim", "/usr/bin/vim");
        item.verified_at = 0; // Very old
        assert!(item.is_stale());
    }

    #[test]
    fn test_inventory_verify_tool() {
        let mut cache = InventoryCache::new();
        // Test with a tool that's almost certainly installed
        let has_sh = cache.verify_tool("sh");
        assert!(has_sh); // sh should be on any Unix system
        assert_eq!(cache.is_installed("sh"), Some(true));
    }

    #[test]
    fn test_installed_editors_filter() {
        let mut cache = InventoryCache::new();
        cache.items.insert(
            "vim".to_string(),
            InventoryItem::installed("vim", "/usr/bin/vim"),
        );
        cache
            .items
            .insert("nano".to_string(), InventoryItem::not_installed("nano"));

        let editors = cache.installed_editors();
        assert!(editors.contains(&"vim"));
        assert!(!editors.contains(&"nano"));
    }
}

//! Recipe store with persistence and indexing (v0.0.423).
//!
//! Stores recipes in JSON files with indexes by:
//! - ID (unique lookup)
//! - Domain (category filtering)
//! - Tags (search)

pub use super::store_types::{RecipeStore, StoreError};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_v3::{RecipeDomain, RecipeV3};
    use tempfile::TempDir;

    fn test_recipe(id: &str) -> RecipeV3 {
        RecipeV3::new(id, &format!("Test Recipe {}", id))
            .with_description("A test recipe")
            .with_tag("test")
    }

    #[test]
    fn test_store_init() {
        let tmp = TempDir::new().unwrap();
        let store = RecipeStore::with_base_dir(tmp.path());
        assert!(store.init().is_ok());
        assert!(tmp.path().join("global").exists());
        assert!(tmp.path().join("user").exists());
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        let recipe = test_recipe("test-1");
        store.save(recipe).unwrap();

        // Reload
        let mut store2 = RecipeStore::with_base_dir(tmp.path());
        let count = store2.load().unwrap();
        assert_eq!(count, 1);
        assert!(store2.get("test-1").is_some());
    }

    #[test]
    fn test_index_by_domain() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        let r1 = RecipeV3::new("r1", "Test 1")
            .with_matcher(crate::recipe_v3::RecipeMatcher::new(RecipeDomain::Systemd));
        let r2 = RecipeV3::new("r2", "Test 2")
            .with_matcher(crate::recipe_v3::RecipeMatcher::new(RecipeDomain::Package));

        store.save(r1).unwrap();
        store.save(r2).unwrap();

        let systemd = store.by_domain(RecipeDomain::Systemd);
        assert_eq!(systemd.len(), 1);
        assert_eq!(systemd[0].id, "r1");
    }

    #[test]
    fn test_search() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        store.save(test_recipe("nginx-restart")).unwrap();
        store.save(test_recipe("vim-config")).unwrap();

        let results = store.search("nginx");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_delete() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        store.save(test_recipe("to-delete")).unwrap();
        assert!(store.get("to-delete").is_some());

        store.delete("to-delete").unwrap();
        assert!(store.get("to-delete").is_none());
    }
}

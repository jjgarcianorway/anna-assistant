//! Recipe statistics and listing module.
//!
//! Handles getting recipe store statistics and listing recipes.

use crate::recipe_engine_v2::store::get_store;

/// Get recipe store stats
pub fn get_recipe_stats() -> String {
    let store = get_store();
    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return "Failed to read recipe store".to_string(),
    };

    format!("{}", store.stats())
}

/// List all recipes (for annactl)
pub fn list_recipes() -> Vec<RecipeSummary> {
    let store = get_store();
    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    store
        .recipes
        .values()
        .map(|r| RecipeSummary {
            id: r.id.clone(),
            name: r.name.clone(),
            domain: r.domain.clone(),
            kind: r.kind.to_string(),
            use_count: r.use_count,
            success_rate: r.success_rate(),
            deprecated: r.deprecated,
            doc_sources: r.doc_sources.clone(),
        })
        .collect()
}

/// Recipe summary for listing
#[derive(Debug, Clone)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub kind: String,
    pub use_count: u32,
    pub success_rate: f32,
    pub deprecated: bool,
    pub doc_sources: Vec<String>,
}

impl std::fmt::Display for RecipeSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.deprecated { "[DEP]" } else { "[ACT]" };
        writeln!(
            f,
            "{} {} ({}) - {}",
            status, self.name, self.id, self.domain
        )?;
        writeln!(
            f,
            "    Uses: {}, Success: {:.0}%, Kind: {}",
            self.use_count,
            self.success_rate * 100.0,
            self.kind
        )?;
        if !self.doc_sources.is_empty() {
            writeln!(f, "    Sources: {}", self.doc_sources.join(", "))?;
        }
        Ok(())
    }
}

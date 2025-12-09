//! Recipe signature for unique identification (v0.0.177).

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Signature that uniquely identifies a query pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeSignature {
    pub domain: String,
    pub intent: String,
    pub route_class: String,
    pub query_pattern: String,
}

impl RecipeSignature {
    pub fn new(
        domain: impl Into<String>,
        intent: impl Into<String>,
        route_class: impl Into<String>,
        query: &str,
    ) -> Self {
        Self {
            domain: domain.into(),
            intent: intent.into(),
            route_class: route_class.into(),
            query_pattern: query.to_lowercase().trim().to_string(),
        }
    }

    pub fn hash_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

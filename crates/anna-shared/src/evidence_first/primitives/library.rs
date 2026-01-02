//! Primitive library containing all probe definitions.

use super::default_primitives::default_primitives;
use super::domain::Domain;
use super::probe_primitive::ProbePrimitive;

/// The primitive library.
pub struct PrimitiveLibrary {
    primitives: Vec<ProbePrimitive>,
}

impl PrimitiveLibrary {
    /// Create a new library with default primitives.
    pub fn new() -> Self {
        Self::default_library()
    }

    /// Create the default library.
    pub fn default_library() -> Self {
        Self {
            primitives: default_primitives(),
        }
    }

    /// Get primitive by ID.
    pub fn get(&self, id: &str) -> Option<&ProbePrimitive> {
        self.primitives.iter().find(|p| p.id == id)
    }

    /// Get primitives for a domain.
    pub fn for_domain(&self, domain: Domain) -> Vec<&ProbePrimitive> {
        self.primitives
            .iter()
            .filter(|p| p.domain == domain)
            .collect()
    }

    /// Find primitives matching keywords.
    pub fn find_by_keywords(&self, keywords: &[&str]) -> Vec<&ProbePrimitive> {
        self.primitives
            .iter()
            .filter(|p| p.matches_keywords(keywords))
            .collect()
    }

    /// Find primitives matching a single keyword.
    pub fn find_by_keyword(&self, keyword: &str) -> Vec<&ProbePrimitive> {
        self.find_by_keywords(&[keyword])
    }

    /// Get all primitive IDs.
    pub fn all_ids(&self) -> Vec<&str> {
        self.primitives.iter().map(|p| p.id).collect()
    }

    /// Check if ID exists.
    pub fn exists(&self, id: &str) -> bool {
        self.primitives.iter().any(|p| p.id == id)
    }

    /// Count primitives.
    pub fn count(&self) -> usize {
        self.primitives.len()
    }
}

impl Default for PrimitiveLibrary {
    fn default() -> Self {
        Self::default_library()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_library() {
        let lib = PrimitiveLibrary::default_library();

        assert!(lib.count() > 0);
        assert!(lib.exists("sys.boot.analyze"));
        assert!(lib.exists("sys.mem.free"));
        assert!(!lib.exists("nonexistent"));
    }

    #[test]
    fn test_find_by_keywords() {
        let lib = PrimitiveLibrary::default_library();

        let boot_probes = lib.find_by_keywords(&["boot", "slow"]);
        assert!(!boot_probes.is_empty());
        assert!(boot_probes.iter().any(|p| p.id == "sys.boot.analyze"));

        let memory_probes = lib.find_by_keywords(&["memory", "ram"]);
        assert!(!memory_probes.is_empty());
    }

    #[test]
    fn test_for_domain() {
        let lib = PrimitiveLibrary::default_library();

        let boot_probes = lib.for_domain(Domain::Boot);
        assert!(!boot_probes.is_empty());
        assert!(boot_probes.iter().all(|p| p.domain == Domain::Boot));
    }
}

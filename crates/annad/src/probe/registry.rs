//! Probe registry - loads and manages probe definitions

use anna_common::ProbeDefinition;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

/// Registry of available probes
pub struct ProbeRegistry {
    probes: HashMap<String, ProbeDefinition>,
}

impl ProbeRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self {
            probes: HashMap::new(),
        }
    }

    /// Load probes from directory
    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut registry = Self::new();
        let dir_path = path.as_ref();

        if !dir_path.exists() {
            warn!("  Probes directory not found: {:?}", dir_path);
            return Ok(registry);
        }

        for entry in fs::read_dir(dir_path).context("Failed to read probes directory")? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().is_some_and(|ext| ext == "json") {
                match registry.load_probe_file(&file_path) {
                    Ok(()) => debug!("  Loaded probe: {:?}", file_path),
                    Err(e) => warn!("  Failed to load probe {:?}: {}", file_path, e),
                }
            }
        }

        Ok(registry)
    }

    /// Load a single probe file
    fn load_probe_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = fs::read_to_string(&path)?;
        let probe: ProbeDefinition = serde_json::from_str(&content)?;
        self.probes.insert(probe.id.clone(), probe);
        Ok(())
    }

    /// Get probe by ID
    pub fn get(&self, id: &str) -> Option<&ProbeDefinition> {
        self.probes.get(id)
    }

    /// List all probes
    pub fn list(&self) -> Vec<&ProbeDefinition> {
        self.probes.values().collect()
    }

    /// Count probes
    pub fn count(&self) -> usize {
        self.probes.len()
    }

    /// Register a probe programmatically
    pub fn register(&mut self, probe: ProbeDefinition) {
        self.probes.insert(probe.id.clone(), probe);
    }
}

impl Default for ProbeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::CachePolicy;

    #[test]
    fn test_empty_registry() {
        let registry = ProbeRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_probe() {
        let mut registry = ProbeRegistry::new();
        let probe = ProbeDefinition {
            id: "test.probe".to_string(),
            cmd: vec!["echo".to_string(), "test".to_string()],
            parser: "echo_v1".to_string(),
            cache_policy: CachePolicy::Volatile,
            ttl: 5,
        };
        registry.register(probe);
        assert_eq!(registry.count(), 1);
        assert!(registry.get("test.probe").is_some());
    }
}

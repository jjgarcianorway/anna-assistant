// v0.0.741: Settings Sphere Core (Phase 317)
// Core SettingsSphere implementation

use super::types::{SphereConfig, SphereEntity, SphereInterest, SphereStats};

/// Settings sphere
#[derive(Debug, Clone, Default)]
pub struct SettingsSphere {
    /// Config
    config: SphereConfig,
    /// Interests
    interests: Vec<SphereInterest>,
    /// Entities
    entities: Vec<SphereEntity>,
    /// Stats
    stats: SphereStats,
}

impl SettingsSphere {
    /// Create new sphere system
    pub fn new(config: SphereConfig) -> Self {
        Self {
            config,
            interests: Vec::new(),
            entities: Vec::new(),
            stats: SphereStats::default(),
        }
    }

    /// Add interest
    pub fn add_interest(&mut self, interest: SphereInterest) -> bool {
        if self.interests.len() >= self.config.max_interests {
            return false;
        }
        self.interests.push(interest);
        self.update_stats();
        true
    }

    /// Get interest
    pub fn get_interest(&self, id: &str) -> Option<&SphereInterest> {
        self.interests.iter().find(|i| i.id == id)
    }

    /// Get interest mut
    pub fn get_interest_mut(&mut self, id: &str) -> Option<&mut SphereInterest> {
        self.interests.iter_mut().find(|i| i.id == id)
    }

    /// Add entity
    pub fn add_entity(&mut self, entity: SphereEntity) {
        self.entities.push(entity);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.interests, self.config.sphere_type);
    }

    /// Get stats
    pub fn stats(&self) -> &SphereStats {
        &self.stats
    }

    /// Interest count
    pub fn interest_count(&self) -> usize {
        self.interests.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_sphere::types::SphereConfig;

    #[test]
    fn test_sphere_new() {
        let sp = SettingsSphere::new(SphereConfig::default());
        assert_eq!(sp.interest_count(), 0);
    }

    #[test]
    fn test_sphere_add_interest() {
        let mut sp = SettingsSphere::new(SphereConfig::default());
        sp.add_interest(SphereInterest::new("i1", "Title", "Content"));
        assert_eq!(sp.interest_count(), 1);
    }
}

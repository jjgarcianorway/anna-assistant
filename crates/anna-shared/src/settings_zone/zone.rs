// v0.0.742: Settings Zone (Phase 318)

use super::config::ZoneConfig;
use super::regulation::ZoneRegulation;
use super::participant::ZoneParticipant;
use super::stats::ZoneStats;

/// Settings zone
#[derive(Debug, Clone, Default)]
pub struct SettingsZone {
    /// Config
    config: ZoneConfig,
    /// Regulations
    regulations: Vec<ZoneRegulation>,
    /// Participants
    participants: Vec<ZoneParticipant>,
    /// Stats
    stats: ZoneStats,
}

impl SettingsZone {
    /// Create new zone system
    pub fn new(config: ZoneConfig) -> Self {
        Self {
            config,
            regulations: Vec::new(),
            participants: Vec::new(),
            stats: ZoneStats::default(),
        }
    }

    /// Add regulation
    pub fn add_regulation(&mut self, regulation: ZoneRegulation) -> bool {
        if self.regulations.len() >= self.config.max_regulations {
            return false;
        }
        self.regulations.push(regulation);
        self.update_stats();
        true
    }

    /// Get regulation
    pub fn get_regulation(&self, id: &str) -> Option<&ZoneRegulation> {
        self.regulations.iter().find(|r| r.id == id)
    }

    /// Get regulation mut
    pub fn get_regulation_mut(&mut self, id: &str) -> Option<&mut ZoneRegulation> {
        self.regulations.iter_mut().find(|r| r.id == id)
    }

    /// Add participant
    pub fn add_participant(&mut self, participant: ZoneParticipant) {
        self.participants.push(participant);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.regulations, self.config.zone_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ZoneStats {
        &self.stats
    }

    /// Regulation count
    pub fn regulation_count(&self) -> usize {
        self.regulations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_new() {
        let z = SettingsZone::new(ZoneConfig::default());
        assert_eq!(z.regulation_count(), 0);
    }

    #[test]
    fn test_zone_add_regulation() {
        let mut z = SettingsZone::new(ZoneConfig::default());
        z.add_regulation(ZoneRegulation::new("r1", "Title", "Content"));
        assert_eq!(z.regulation_count(), 1);
    }
}

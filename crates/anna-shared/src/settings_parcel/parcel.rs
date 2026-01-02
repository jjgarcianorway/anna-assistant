// v0.0.757: Settings Parcel - Parcel (Phase 333)

use super::config::ParcelConfig;
use super::title::ParcelTitle;
use super::examiner::ParcelExaminer;
use super::stats::ParcelStats;

/// Settings parcel
#[derive(Debug, Clone, Default)]
pub struct SettingsParcel {
    /// Config
    config: ParcelConfig,
    /// Titles
    titles: Vec<ParcelTitle>,
    /// Examiners
    examiners: Vec<ParcelExaminer>,
    /// Stats
    stats: ParcelStats,
}

impl SettingsParcel {
    /// Create new parcel system
    pub fn new(config: ParcelConfig) -> Self {
        Self {
            config,
            titles: Vec::new(),
            examiners: Vec::new(),
            stats: ParcelStats::default(),
        }
    }

    /// Add title
    pub fn add_title(&mut self, title: ParcelTitle) -> bool {
        if self.titles.len() >= self.config.max_titles {
            return false;
        }
        self.titles.push(title);
        self.update_stats();
        true
    }

    /// Get title
    pub fn get_title(&self, id: &str) -> Option<&ParcelTitle> {
        self.titles.iter().find(|t| t.id == id)
    }

    /// Get title mut
    pub fn get_title_mut(&mut self, id: &str) -> Option<&mut ParcelTitle> {
        self.titles.iter_mut().find(|t| t.id == id)
    }

    /// Add examiner
    pub fn add_examiner(&mut self, examiner: ParcelExaminer) {
        self.examiners.push(examiner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.titles, self.config.parcel_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ParcelStats {
        &self.stats
    }

    /// Title count
    pub fn title_count(&self) -> usize {
        self.titles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parcel_new() {
        let p = SettingsParcel::new(ParcelConfig::default());
        assert_eq!(p.title_count(), 0);
    }

    #[test]
    fn test_parcel_add_title() {
        let mut p = SettingsParcel::new(ParcelConfig::default());
        p.add_title(ParcelTitle::new("t1", "Title", "Content"));
        assert_eq!(p.title_count(), 1);
    }
}

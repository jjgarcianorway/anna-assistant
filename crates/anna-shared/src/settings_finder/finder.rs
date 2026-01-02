// v0.0.685: Settings Finder Implementation (Phase 261)
// Core finder implementation

use std::collections::HashMap;
use super::config::FinderConfig;
use super::stats::FinderStats;
use super::types::{FindMode, FindLimit, FindResult, FoundItem};

/// Settings finder
#[derive(Debug, Clone, Default)]
pub struct SettingsFinder {
    /// Config
    config: FinderConfig,
    /// Stats
    stats: FinderStats,
}

impl SettingsFinder {
    /// Create new finder
    pub fn new(config: FinderConfig) -> Self {
        Self {
            config,
            stats: FinderStats::default(),
        }
    }

    /// Calculate match score
    fn calc_score(&self, target: &str, pattern: &str) -> f64 {
        let (t, p) = if self.config.case_insensitive {
            (target.to_lowercase(), pattern.to_lowercase())
        } else {
            (target.to_string(), pattern.to_string())
        };

        if t == p {
            1.0
        } else if t.contains(&p) {
            p.len() as f64 / t.len() as f64
        } else {
            0.0
        }
    }

    /// Apply limit
    fn apply_limit(&self, mut items: Vec<FoundItem>) -> Vec<FoundItem> {
        match self.config.limit {
            FindLimit::First => items.into_iter().take(1).collect(),
            FindLimit::All => items,
            FindLimit::Max(n) => {
                items.truncate(n);
                items
            }
        }
    }

    /// Find by key pattern
    pub fn find_by_key(&mut self, settings: &HashMap<String, String>, pattern: &str) -> FindResult {
        let mut items = Vec::new();

        for (key, value) in settings {
            let score = self.calc_score(key, pattern);
            if score > 0.0 || (self.config.partial_match && score == 0.0) {
                if score > 0.0 {
                    items.push(FoundItem::new(key.clone(), value.clone(), score, FindMode::KeyPattern));
                }
            }
        }

        // Sort by score descending
        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let items = self.apply_limit(items);

        let result = FindResult::new(items, settings.len(), FindMode::KeyPattern);
        self.stats.record(&result);
        result
    }

    /// Find by value pattern
    pub fn find_by_value(&mut self, settings: &HashMap<String, String>, pattern: &str) -> FindResult {
        let mut items = Vec::new();

        for (key, value) in settings {
            let score = self.calc_score(value, pattern);
            if score > 0.0 {
                items.push(FoundItem::new(key.clone(), value.clone(), score, FindMode::ValuePattern));
            }
        }

        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let items = self.apply_limit(items);

        let result = FindResult::new(items, settings.len(), FindMode::ValuePattern);
        self.stats.record(&result);
        result
    }

    /// Find exact key
    pub fn find_exact(&mut self, settings: &HashMap<String, String>, key: &str) -> FindResult {
        let items = if let Some(value) = settings.get(key) {
            vec![FoundItem::new(key.to_string(), value.clone(), 1.0, FindMode::ExactKey)]
        } else {
            Vec::new()
        };

        let result = FindResult::new(items, settings.len(), FindMode::ExactKey);
        self.stats.record(&result);
        result
    }

    /// Find by exact value
    pub fn find_by_exact_value(&mut self, settings: &HashMap<String, String>, target_value: &str) -> FindResult {
        let mut items = Vec::new();

        let target = if self.config.case_insensitive {
            target_value.to_lowercase()
        } else {
            target_value.to_string()
        };

        for (key, value) in settings {
            let v = if self.config.case_insensitive {
                value.to_lowercase()
            } else {
                value.clone()
            };

            if v == target {
                items.push(FoundItem::new(key.clone(), value.clone(), 1.0, FindMode::ByValue));
            }
        }

        let items = self.apply_limit(items);
        let result = FindResult::new(items, settings.len(), FindMode::ByValue);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &FinderStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finder_new() {
        let f = SettingsFinder::new(FinderConfig::default());
        assert_eq!(f.stats().total_finds, 0);
    }

    #[test]
    fn test_finder_find_exact() {
        let mut f = SettingsFinder::new(FinderConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());

        let result = f.find_exact(&settings, "app.name");
        assert!(result.has_results());
        assert_eq!(result.first().unwrap().value, "test");
    }

    #[test]
    fn test_finder_find_by_key() {
        let mut f = SettingsFinder::new(FinderConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = f.find_by_key(&settings, "app");
        assert_eq!(result.total_found, 2);
    }

    #[test]
    fn test_finder_find_by_value() {
        let mut f = SettingsFinder::new(FinderConfig::default());
        let mut settings = HashMap::new();
        settings.insert("host".to_string(), "localhost".to_string());
        settings.insert("port".to_string(), "8080".to_string());

        let result = f.find_by_value(&settings, "local");
        assert_eq!(result.total_found, 1);
    }

    #[test]
    fn test_finder_with_limit() {
        let mut f = SettingsFinder::new(FinderConfig::default().limit(FindLimit::Max(1)));
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("ab".to_string(), "2".to_string());
        settings.insert("abc".to_string(), "3".to_string());

        let result = f.find_by_key(&settings, "a");
        assert_eq!(result.total_found, 1);
    }
}

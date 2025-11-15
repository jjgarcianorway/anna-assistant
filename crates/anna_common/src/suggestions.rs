//! Suggestion Engine - Prioritized, actionable system improvements
//!
//! Phase 5.1: Conversational UX
//! Generates 2-5 prioritized suggestions with Arch Wiki grounding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Knowledge source for a suggestion (Task 10: Arch Wiki backing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    /// Short human-readable label (e.g. "Arch Wiki: Pacman cache")
    pub label: String,

    /// Canonical URL (prefer Arch Wiki or official docs linked from Arch Wiki)
    pub url: String,
}

/// A single system improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique key for this suggestion type
    pub key: String,

    /// Short title
    pub title: String,

    /// Detailed explanation in plain English ("What is going on")
    pub explanation: String,

    /// Why the user should care (Task 10: explicit "why")
    /// 1-3 sentences of plain language explaining user-visible effects
    pub why_it_matters: String,

    /// Impact description
    pub impact: String,

    /// Knowledge sources (Arch Wiki or official docs) (Task 10)
    /// Must have at least one source for config/performance/security suggestions
    pub knowledge_sources: Vec<KnowledgeSource>,

    /// Priority level
    pub priority: SuggestionPriority,

    /// Category
    pub category: SuggestionCategory,

    /// Can Anna fix this automatically?
    pub auto_fixable: bool,

    /// Fix description if auto-fixable
    pub fix_description: Option<String>,

    /// Commands that would be executed (for transparency)
    pub fix_commands: Vec<String>,

    /// Estimated impact on metrics
    pub estimated_impact: Option<EstimatedImpact>,

    /// Dependencies - keys of suggestions that should be addressed first (Task 9)
    pub depends_on: Vec<String>,
}

/// Priority of a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SuggestionPriority {
    /// Critical - immediate attention needed
    Critical = 4,

    /// High - should be addressed soon
    High = 3,

    /// Medium - good to address
    Medium = 2,

    /// Low - nice to have
    Low = 1,

    /// Info - FYI only
    Info = 0,
}

/// Category of suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// Disk space and storage
    Disk,

    /// Memory usage
    Memory,

    /// CPU and performance
    Performance,

    /// Package management
    Packages,

    /// Service health
    Services,

    /// Security hardening
    Security,

    /// Network configuration
    Network,

    /// Desktop environment
    Desktop,

    /// Power management
    Power,

    /// Boot performance
    Boot,

    /// General configuration
    Configuration,
}

/// Estimated impact of applying a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedImpact {
    /// Space saved in MB
    pub space_saved_mb: Option<f64>,

    /// Memory freed in MB
    pub memory_freed_mb: Option<f64>,

    /// Boot time improvement in seconds
    pub boot_time_saved_secs: Option<f64>,

    /// Descriptive impacts
    pub descriptions: Vec<String>,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        priority: SuggestionPriority,
        category: SuggestionCategory,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            explanation: String::new(),
            why_it_matters: String::new(),
            impact: String::new(),
            knowledge_sources: Vec::new(),
            priority,
            category,
            auto_fixable: false,
            fix_description: None,
            fix_commands: Vec::new(),
            estimated_impact: None,
            depends_on: Vec::new(),
        }
    }

    /// Add explanation ("What is going on")
    pub fn explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Add "why this matters" (Task 10: explicit user-focused reasoning)
    pub fn why_it_matters(mut self, why: impl Into<String>) -> Self {
        self.why_it_matters = why.into();
        self
    }

    /// Add impact description
    pub fn impact(mut self, impact: impl Into<String>) -> Self {
        self.impact = impact.into();
        self
    }

    /// Add a knowledge source (Task 10: Arch Wiki backing)
    pub fn add_source(mut self, label: impl Into<String>, url: impl Into<String>) -> Self {
        self.knowledge_sources.push(KnowledgeSource {
            label: label.into(),
            url: url.into(),
        });
        self
    }

    /// Add a knowledge source object directly
    pub fn add_knowledge_source(mut self, source: KnowledgeSource) -> Self {
        self.knowledge_sources.push(source);
        self
    }

    /// Mark as auto-fixable with description and commands
    pub fn auto_fixable(
        mut self,
        fix_description: impl Into<String>,
        fix_commands: Vec<String>,
    ) -> Self {
        self.auto_fixable = true;
        self.fix_description = Some(fix_description.into());
        self.fix_commands = fix_commands;
        self
    }

    /// Add estimated impact
    pub fn estimated_impact(mut self, impact: EstimatedImpact) -> Self {
        self.estimated_impact = Some(impact);
        self
    }

    /// Add dependencies (Task 9: dependency-aware suggestions)
    pub fn depends_on(mut self, keys: Vec<String>) -> Self {
        self.depends_on = keys;
        self
    }

    /// Add a single dependency
    pub fn add_dependency(mut self, key: impl Into<String>) -> Self {
        self.depends_on.push(key.into());
        self
    }
}

impl KnowledgeSource {
    /// Create an Arch Wiki knowledge source (Task 10)
    pub fn arch_wiki(section: &str, description: impl Into<String>) -> Self {
        Self {
            label: format!("Arch Wiki: {}", description.into()),
            url: format!("https://wiki.archlinux.org/title/{}", section),
        }
    }

    /// Create an official docs knowledge source
    pub fn official(url: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: format!("Official docs: {}", description.into()),
            url: url.into(),
        }
    }
}

/// Suggestion engine state
pub struct SuggestionEngine {
    /// All available suggestions
    suggestions: Vec<Suggestion>,

    /// User preferences (discarded/acknowledged suggestions)
    user_preferences: HashMap<String, UserPreference>,
}

/// User preference for a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserPreference {
    /// User doesn't want to see this
    Discarded,

    /// User acknowledged but won't fix
    Acknowledged,

    /// Snoozed until a specific time
    Snoozed,
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SuggestionEngine {
    /// Create a new suggestion engine
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            user_preferences: HashMap::new(),
        }
    }

    /// Add a suggestion
    pub fn add_suggestion(&mut self, suggestion: Suggestion) {
        self.suggestions.push(suggestion);
    }

    /// Get top N prioritized suggestions (respecting user preferences)
    pub fn get_top_suggestions(&self, max_count: usize) -> Vec<&Suggestion> {
        let mut suggestions: Vec<_> = self.suggestions
            .iter()
            .filter(|s| {
                // Filter out discarded suggestions
                !matches!(
                    self.user_preferences.get(&s.key),
                    Some(UserPreference::Discarded)
                )
            })
            .collect();

        // Sort by priority (highest first)
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Return top N, but between 2-5 as per spec
        let count = max_count.min(5).max(2).min(suggestions.len());
        suggestions.into_iter().take(count).collect()
    }

    /// Mark a suggestion as discarded
    pub fn discard_suggestion(&mut self, key: &str) {
        self.user_preferences.insert(key.to_string(), UserPreference::Discarded);
    }

    /// Mark a suggestion as acknowledged
    pub fn acknowledge_suggestion(&mut self, key: &str) {
        self.user_preferences.insert(key.to_string(), UserPreference::Acknowledged);
    }
}

/// Common suggestions based on system state
pub mod common_suggestions {
    use super::*;

    /// Pacman cache cleanup suggestion
    pub fn pacman_cache_cleanup(cache_size_mb: f64) -> Suggestion {
        Suggestion::new(
            "pacman-cache-cleanup",
            "Clean up old package cache",
            if cache_size_mb > 5000.0 {
                SuggestionPriority::High
            } else {
                SuggestionPriority::Medium
            },
            SuggestionCategory::Disk,
        )
        .explanation(format!(
            "Your pacman cache is using {:.1} GB of disk space. \
             Pacman keeps all downloaded packages by default, which helps with downgrades \
             but uses significant disk space over time.",
            cache_size_mb / 1024.0
        ))
        .why_it_matters(
            "This can cause your disk to fill up and make package updates fail. \
             A full disk can prevent your system from booting or cause crashes."
        )
        .impact(format!(
            "Cleaning the cache will free up approximately {:.1} GB of disk space.",
            cache_size_mb / 1024.0
        ))
        .add_knowledge_source(
            KnowledgeSource::arch_wiki(
                "Pacman#Cleaning_the_package_cache",
                "Pacman cache management"
            )
        )
        .auto_fixable(
            "Remove all cached packages except the most recent 2 versions using paccache",
            vec!["paccache -rk2".to_string()],
        )
        .estimated_impact(EstimatedImpact {
            space_saved_mb: Some(cache_size_mb * 0.8), // Estimate 80% can be cleaned
            memory_freed_mb: None,
            boot_time_saved_secs: None,
            descriptions: vec![format!("Free up ~{:.1} GB of disk space", cache_size_mb * 0.8 / 1024.0)],
        })
    }

    /// Orphaned packages suggestion
    pub fn orphaned_packages(count: usize) -> Suggestion {
        Suggestion::new(
            "orphaned-packages",
            format!("Remove {} orphaned packages", count),
            SuggestionPriority::Low,
            SuggestionCategory::Packages,
        )
        .explanation(format!(
            "There are {} packages installed as dependencies that are no longer needed \
             by any explicitly installed package. These are safe to remove.",
            count
        ))
        .why_it_matters(
            "Orphaned packages waste disk space and increase the time needed for system updates. \
             Removing them keeps your system lean and reduces maintenance overhead."
        )
        .impact("Frees up disk space and reduces package update overhead.")
        .add_knowledge_source(
            KnowledgeSource::arch_wiki(
                "Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)",
                "Removing orphaned packages"
            )
        )
        .auto_fixable(
            format!("List and review {} orphaned packages, then remove them if safe", count),
            vec!["pacman -Qtdq | pacman -Rns -".to_string()],
        )
    }

    /// Failed systemd services
    pub fn failed_services(services: Vec<String>) -> Suggestion {
        let count = services.len();
        Suggestion::new(
            "failed-services",
            format!("{} failed systemd services", count),
            SuggestionPriority::High,
            SuggestionCategory::Services,
        )
        .explanation(format!(
            "The following systemd services have failed: {}. \
             Failed services may indicate configuration issues or missing dependencies.",
            services.join(", ")
        ))
        .why_it_matters(
            "Failed services can cause applications to malfunction, prevent features from working, \
             or indicate deeper system problems that could lead to instability or data loss."
        )
        .impact("Failed services may affect system functionality or stability.")
        .add_knowledge_source(
            KnowledgeSource::arch_wiki(
                "Systemd#Investigating_failed_services",
                "Investigating systemd failures"
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_priority_ordering() {
        let mut engine = SuggestionEngine::new();

        engine.add_suggestion(
            Suggestion::new("low", "Low priority", SuggestionPriority::Low, SuggestionCategory::Configuration)
        );
        engine.add_suggestion(
            Suggestion::new("critical", "Critical", SuggestionPriority::Critical, SuggestionCategory::Security)
        );
        engine.add_suggestion(
            Suggestion::new("medium", "Medium", SuggestionPriority::Medium, SuggestionCategory::Performance)
        );

        let top = engine.get_top_suggestions(3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].key, "critical");
        assert_eq!(top[1].key, "medium");
        assert_eq!(top[2].key, "low");
    }

    #[test]
    fn test_discard_suggestion() {
        let mut engine = SuggestionEngine::new();

        engine.add_suggestion(
            Suggestion::new("test", "Test", SuggestionPriority::High, SuggestionCategory::Configuration)
        );

        let top = engine.get_top_suggestions(5);
        assert_eq!(top.len(), 1);

        engine.discard_suggestion("test");

        let top = engine.get_top_suggestions(5);
        assert_eq!(top.len(), 0);
    }

    #[test]
    fn test_max_suggestions_limit() {
        let mut engine = SuggestionEngine::new();

        for i in 0..10 {
            engine.add_suggestion(
                Suggestion::new(
                    format!("test-{}", i),
                    format!("Test {}", i),
                    SuggestionPriority::Medium,
                    SuggestionCategory::Configuration
                )
            );
        }

        let top = engine.get_top_suggestions(10);
        assert!(top.len() <= 5, "Should cap at 5 suggestions");
        assert!(top.len() >= 2, "Should show at least 2 suggestions");
    }

    // Task 10: Knowledge source and why_it_matters tests

    #[test]
    fn test_knowledge_source_creation() {
        let source = KnowledgeSource::arch_wiki("Pacman#Cache", "Pacman cache management");

        assert_eq!(source.label, "Arch Wiki: Pacman cache management");
        assert!(source.url.contains("wiki.archlinux.org"));
        assert!(source.url.contains("Pacman#Cache"));
    }

    #[test]
    fn test_suggestion_has_knowledge_sources() {
        let suggestion = Suggestion::new(
            "test-suggestion",
            "Test Suggestion",
            SuggestionPriority::High,
            SuggestionCategory::Configuration,
        )
        .add_source("Arch Wiki: Test", "https://wiki.archlinux.org/title/Test");

        assert_eq!(suggestion.knowledge_sources.len(), 1);
        assert_eq!(suggestion.knowledge_sources[0].label, "Arch Wiki: Test");
        assert_eq!(suggestion.knowledge_sources[0].url, "https://wiki.archlinux.org/title/Test");
    }

    #[test]
    fn test_suggestion_has_why_it_matters() {
        let why = "This prevents crashes and data loss by ensuring proper configuration.";
        let suggestion = Suggestion::new(
            "test-suggestion",
            "Test Suggestion",
            SuggestionPriority::High,
            SuggestionCategory::Security,
        )
        .why_it_matters(why);

        assert_eq!(suggestion.why_it_matters, why);
        assert!(!suggestion.why_it_matters.is_empty());
    }

    #[test]
    fn test_common_suggestions_have_sources_and_why() {
        let pacman_suggestion = common_suggestions::pacman_cache_cleanup(5000.0);

        // Must have at least one knowledge source
        assert!(
            !pacman_suggestion.knowledge_sources.is_empty(),
            "Pacman cache suggestion must have knowledge sources"
        );

        // Must have why_it_matters
        assert!(
            !pacman_suggestion.why_it_matters.trim().is_empty(),
            "Pacman cache suggestion must have why_it_matters"
        );

        // Should link to Arch Wiki
        assert!(
            pacman_suggestion.knowledge_sources.iter().any(|s| s.url.contains("wiki.archlinux.org")),
            "Should have Arch Wiki source"
        );
    }

    #[test]
    fn test_orphaned_packages_has_sources_and_why() {
        let orphan_suggestion = common_suggestions::orphaned_packages(20);

        assert!(!orphan_suggestion.knowledge_sources.is_empty());
        assert!(!orphan_suggestion.why_it_matters.trim().is_empty());
        assert!(orphan_suggestion.knowledge_sources[0].url.contains("Pacman"));
    }

    #[test]
    fn test_failed_services_has_sources_and_why() {
        let services = vec!["test.service".to_string()];
        let service_suggestion = common_suggestions::failed_services(services);

        assert!(!service_suggestion.knowledge_sources.is_empty());
        assert!(!service_suggestion.why_it_matters.trim().is_empty());
        assert!(service_suggestion.knowledge_sources[0].url.contains("Systemd"));
    }
}

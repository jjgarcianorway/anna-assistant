//! Suggestion Engine - Prioritized, actionable system improvements
//!
//! Phase 5.1: Conversational UX
//! Generates 2-5 prioritized suggestions with Arch Wiki grounding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single system improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique key for this suggestion type
    pub key: String,

    /// Short title
    pub title: String,

    /// Detailed explanation in plain English
    pub explanation: String,

    /// Why this matters (impact)
    pub impact: String,

    /// Documentation URLs (Arch Wiki preferred)
    pub docs: Vec<DocumentationLink>,

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

/// Documentation link with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLink {
    /// URL
    pub url: String,

    /// Description of what this link covers
    pub description: String,

    /// Source type
    pub source: DocSource,
}

/// Source of documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocSource {
    /// Arch Wiki (preferred)
    ArchWiki,

    /// Official project documentation
    OfficialDocs,

    /// Manual pages
    ManPage,
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
            impact: String::new(),
            docs: Vec::new(),
            priority,
            category,
            auto_fixable: false,
            fix_description: None,
            fix_commands: Vec::new(),
            estimated_impact: None,
        }
    }

    /// Add explanation
    pub fn explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Add impact description
    pub fn impact(mut self, impact: impl Into<String>) -> Self {
        self.impact = impact.into();
        self
    }

    /// Add a documentation link
    pub fn add_doc(mut self, url: impl Into<String>, description: impl Into<String>, source: DocSource) -> Self {
        self.docs.push(DocumentationLink {
            url: url.into(),
            description: description.into(),
            source,
        });
        self
    }

    /// Add a documentation link object directly
    pub fn add_doc_link(mut self, link: DocumentationLink) -> Self {
        self.docs.push(link);
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
}

impl DocumentationLink {
    /// Create an Arch Wiki link
    pub fn arch_wiki(section: &str, description: impl Into<String>) -> Self {
        Self {
            url: format!("https://wiki.archlinux.org/title/{}", section),
            description: description.into(),
            source: DocSource::ArchWiki,
        }
    }

    /// Create an official docs link
    pub fn official(url: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            description: description.into(),
            source: DocSource::OfficialDocs,
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
        .impact(format!(
            "Cleaning the cache will free up approximately {:.1} GB of disk space.",
            cache_size_mb / 1024.0
        ))
        .add_doc_link(
            DocumentationLink::arch_wiki(
                "Pacman#Cleaning_the_package_cache",
                "Arch Wiki guide on cleaning pacman cache"
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
        .impact("Frees up disk space and reduces package update overhead.")
        .add_doc_link(
            DocumentationLink::arch_wiki(
                "Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)",
                "Arch Wiki guide on orphaned packages"
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
        .impact("Failed services may affect system functionality or stability.")
        .add_doc_link(
            DocumentationLink::arch_wiki(
                "Systemd#Investigating_failed_services",
                "Arch Wiki guide on investigating service failures"
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
}

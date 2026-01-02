// v0.0.566: Category-Specific Search Methods
// Search implementations for each settings category

use crate::unified_settings::{SettingsCategory, UnifiedSettings};
use super::types::{MatchType, SearchResult, SearchResults};

/// Category search implementation
pub struct CategorySearcher;

impl CategorySearcher {
    /// Check if text matches query
    pub fn matches(text: &str, query: &str, case_sensitive: bool) -> Option<f32> {
        let text_normalized = if case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        if text_normalized == query {
            return Some(1.0);
        }
        if text_normalized.contains(query) {
            return Some(0.7);
        }
        if text_normalized.starts_with(query) {
            return Some(0.8);
        }

        None
    }

    pub fn search_personality(
        settings: &UnifiedSettings,
        query: &str,
        results: &mut SearchResults,
        case_sensitive: bool,
        search_names: bool,
        search_values: bool,
    ) {
        let p = &settings.personality;

        if search_names {
            if let Some(score) = Self::matches("formality", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "formality",
                    format!("{:?}", p.formality), MatchType::FieldName, score
                ));
            }
            if let Some(score) = Self::matches("friendliness", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "friendliness",
                    format!("{:?}", p.friendliness), MatchType::FieldName, score
                ));
            }
            if let Some(score) = Self::matches("humor", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "humor",
                    format!("{:?}", p.humor), MatchType::FieldName, score
                ));
            }
        }

        if search_values {
            let formality_str = format!("{:?}", p.formality).to_lowercase();
            if let Some(score) = Self::matches(&formality_str, query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "formality", formality_str, MatchType::FieldValue, score
                ));
            }
        }
    }

    pub fn search_risk(
        settings: &UnifiedSettings,
        query: &str,
        results: &mut SearchResults,
        case_sensitive: bool,
        search_names: bool,
        _search_values: bool,
    ) {
        let r = &settings.risk;

        if search_names {
            if let Some(score) = Self::matches("risk", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Risk, "auto_approve_up_to",
                    format!("{:?}", r.auto_approve_up_to), MatchType::FieldName, score
                ));
            }
            if let Some(score) = Self::matches("confirmation", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Risk, "confirmation_mode",
                    format!("{:?}", r.confirmation_mode), MatchType::FieldName, score
                ));
            }
        }
    }

    pub fn search_learning(
        settings: &UnifiedSettings,
        query: &str,
        results: &mut SearchResults,
        case_sensitive: bool,
        search_names: bool,
        _search_values: bool,
    ) {
        let l = &settings.learning;

        if search_names {
            if let Some(score) = Self::matches("learning", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Learning, "level",
                    format!("{:?}", l.level), MatchType::FieldName, score
                ));
            }
            if let Some(score) = Self::matches("explain", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Learning, "explain_commands",
                    l.explain_commands.to_string(), MatchType::FieldName, score
                ));
            }
        }
    }

    pub fn search_verbosity(
        settings: &UnifiedSettings,
        query: &str,
        results: &mut SearchResults,
        case_sensitive: bool,
        search_names: bool,
        _search_values: bool,
    ) {
        let v = &settings.verbosity;

        if search_names {
            if let Some(score) = Self::matches("verbosity", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Verbosity, "level",
                    format!("{:?}", v.level), MatchType::FieldName, score
                ));
            }
            if let Some(score) = Self::matches("detail", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Verbosity, "answer_detail",
                    format!("{:?}", v.answer_detail), MatchType::FieldName, score
                ));
            }
        }
    }

    pub fn search_timeout(
        settings: &UnifiedSettings,
        query: &str,
        results: &mut SearchResults,
        case_sensitive: bool,
        search_names: bool,
        _search_values: bool,
    ) {
        let t = &settings.timeout;

        if search_names {
            if let Some(score) = Self::matches("timeout", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Timeout, "command_timeout_ms",
                    t.command_timeout_ms.to_string(), MatchType::FieldName, score
                ));
            }
        }
    }

    pub fn search_privacy(
        settings: &UnifiedSettings,
        query: &str,
        results: &mut SearchResults,
        case_sensitive: bool,
        search_names: bool,
        _search_values: bool,
    ) {
        let p = &settings.privacy;

        if search_names {
            if let Some(score) = Self::matches("privacy", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Privacy, "data_collection",
                    format!("{:?}", p.data_collection), MatchType::FieldName, score
                ));
            }
            if let Some(score) = Self::matches("log", query, case_sensitive) {
                results.add(SearchResult::new(
                    SettingsCategory::Privacy, "log_retention",
                    format!("{:?}", p.log_retention), MatchType::FieldName, score
                ));
            }
        }
    }
}

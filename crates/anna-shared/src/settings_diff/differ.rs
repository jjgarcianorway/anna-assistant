// Settings differ implementation

use crate::unified_settings::{SettingsCategory, UnifiedSettings};
use super::types::{DiffEntry, DiffType, SettingsDiff};

/// Settings differ
#[derive(Debug, Clone, Default)]
pub struct SettingsDiffer {
    /// Include unchanged fields
    pub include_unchanged: bool,
    /// Categories to compare (None = all)
    pub categories: Option<Vec<SettingsCategory>>,
}

impl SettingsDiffer {
    /// Create new differ
    pub fn new() -> Self {
        Self::default()
    }

    /// Include unchanged fields in output
    pub fn with_unchanged(mut self) -> Self {
        self.include_unchanged = true;
        self
    }

    /// Only compare specific categories
    pub fn only_categories(mut self, categories: Vec<SettingsCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Compare two settings objects
    pub fn diff(&self, old: &UnifiedSettings, new: &UnifiedSettings) -> SettingsDiff {
        let mut result = SettingsDiff::new();

        // Compare each category
        if self.should_compare(SettingsCategory::Personality) {
            self.diff_personality(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Risk) {
            self.diff_risk(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Learning) {
            self.diff_learning(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Escalation) {
            self.diff_escalation(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Verbosity) {
            self.diff_verbosity(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Confirmation) {
            self.diff_confirmation(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Timeout) {
            self.diff_timeout(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::OutputStyle) {
            self.diff_output_style(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Privacy) {
            self.diff_privacy(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Backup) {
            self.diff_backup(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Update) {
            self.diff_update(old, new, &mut result);
        }
        if self.should_compare(SettingsCategory::Model) {
            self.diff_model(old, new, &mut result);
        }

        result
    }

    /// Should compare this category?
    fn should_compare(&self, category: SettingsCategory) -> bool {
        self.categories
            .as_ref()
            .map(|cats| cats.contains(&category))
            .unwrap_or(true)
    }

    /// Add diff entry if changed (or if include_unchanged)
    fn add_if_different<T: PartialEq + std::fmt::Debug>(
        &self,
        result: &mut SettingsDiff,
        category: SettingsCategory,
        field: &str,
        old: &T,
        new: &T,
    ) {
        if old != new {
            result.add(
                DiffEntry::new(category, field, DiffType::Changed)
                    .old(format!("{:?}", old))
                    .new_val(format!("{:?}", new)),
            );
        } else if self.include_unchanged {
            result.add(DiffEntry::new(category, field, DiffType::Unchanged));
        }
    }

    fn diff_personality(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Personality, "formality",
            &old.personality.formality, &new.personality.formality);
        self.add_if_different(result, SettingsCategory::Personality, "friendliness",
            &old.personality.friendliness, &new.personality.friendliness);
        self.add_if_different(result, SettingsCategory::Personality, "humor",
            &old.personality.humor, &new.personality.humor);
    }

    fn diff_risk(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Risk, "confirmation_mode",
            &old.risk.confirmation_mode, &new.risk.confirmation_mode);
        self.add_if_different(result, SettingsCategory::Risk, "auto_approve_up_to",
            &old.risk.auto_approve_up_to, &new.risk.auto_approve_up_to);
    }

    fn diff_learning(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Learning, "level",
            &old.learning.level, &new.learning.level);
        self.add_if_different(result, SettingsCategory::Learning, "explain_commands",
            &old.learning.explain_commands, &new.learning.explain_commands);
    }

    fn diff_escalation(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Escalation, "mode",
            &old.escalation.mode, &new.escalation.mode);
        self.add_if_different(result, SettingsCategory::Escalation, "notify",
            &old.escalation.notify, &new.escalation.notify);
    }

    fn diff_verbosity(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Verbosity, "level",
            &old.verbosity.level, &new.verbosity.level);
        self.add_if_different(result, SettingsCategory::Verbosity, "show_citations",
            &old.verbosity.show_citations, &new.verbosity.show_citations);
    }

    fn diff_confirmation(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Confirmation, "style",
            &old.confirmation.style, &new.confirmation.style);
        self.add_if_different(result, SettingsCategory::Confirmation, "timeout_behavior",
            &old.confirmation.timeout_behavior, &new.confirmation.timeout_behavior);
    }

    fn diff_timeout(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Timeout, "profile",
            &old.timeout.profile, &new.timeout.profile);
        self.add_if_different(result, SettingsCategory::Timeout, "command_timeout_ms",
            &old.timeout.command_timeout_ms, &new.timeout.command_timeout_ms);
    }

    fn diff_output_style(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::OutputStyle, "theme",
            &old.output_style.theme, &new.output_style.theme);
        self.add_if_different(result, SettingsCategory::OutputStyle, "color_scheme",
            &old.output_style.color_scheme, &new.output_style.color_scheme);
    }

    fn diff_privacy(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Privacy, "data_collection",
            &old.privacy.data_collection, &new.privacy.data_collection);
        self.add_if_different(result, SettingsCategory::Privacy, "log_retention",
            &old.privacy.log_retention, &new.privacy.log_retention);
    }

    fn diff_backup(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Backup, "frequency",
            &old.backup.frequency, &new.backup.frequency);
        self.add_if_different(result, SettingsCategory::Backup, "encrypt_backups",
            &old.backup.encrypt_backups, &new.backup.encrypt_backups);
    }

    fn diff_update(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Update, "check_frequency",
            &old.update.check_frequency, &new.update.check_frequency);
        self.add_if_different(result, SettingsCategory::Update, "channel",
            &old.update.channel, &new.update.channel);
    }

    fn diff_model(&self, old: &UnifiedSettings, new: &UnifiedSettings, result: &mut SettingsDiff) {
        self.add_if_different(result, SettingsCategory::Model, "size_preference",
            &old.model.size_preference, &new.model.size_preference);
        self.add_if_different(result, SettingsCategory::Model, "quality_speed",
            &old.model.quality_speed, &new.model.quality_speed);
    }
}

/// Quick diff helper
pub fn diff_settings(old: &UnifiedSettings, new: &UnifiedSettings) -> SettingsDiff {
    SettingsDiffer::new().diff(old, new)
}

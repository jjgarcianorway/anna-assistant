// v0.0.561: Settings Diff (Phase 137)
// Compares two settings objects and reports differences

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Type of change in a diff
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// Value was added
    Added,
    /// Value was removed
    Removed,
    /// Value was changed
    Changed,
    /// No change
    Unchanged,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "+"),
            Self::Removed => write!(f, "-"),
            Self::Changed => write!(f, "~"),
            Self::Unchanged => write!(f, " "),
        }
    }
}

/// A single difference entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Category affected
    pub category: SettingsCategory,
    /// Field path within category
    pub field: String,
    /// Type of change
    pub diff_type: DiffType,
    /// Old value (serialized)
    pub old_value: Option<String>,
    /// New value (serialized)
    pub new_value: Option<String>,
}

impl DiffEntry {
    /// Create a new diff entry
    pub fn new(
        category: SettingsCategory,
        field: impl Into<String>,
        diff_type: DiffType,
    ) -> Self {
        Self {
            category,
            field: field.into(),
            diff_type,
            old_value: None,
            new_value: None,
        }
    }

    /// Set old value
    pub fn old(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn new_val(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Is this a change?
    pub fn is_changed(&self) -> bool {
        self.diff_type != DiffType::Unchanged
    }
}

/// Result of comparing two settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsDiff {
    /// All differences found
    pub entries: Vec<DiffEntry>,
    /// Categories that changed
    pub changed_categories: Vec<SettingsCategory>,
}

impl SettingsDiff {
    /// Create empty diff result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diff entry
    pub fn add(&mut self, entry: DiffEntry) {
        if entry.is_changed() && !self.changed_categories.contains(&entry.category) {
            self.changed_categories.push(entry.category);
        }
        self.entries.push(entry);
    }

    /// Are the settings identical?
    pub fn is_identical(&self) -> bool {
        self.entries.iter().all(|e| !e.is_changed())
    }

    /// Has any changes?
    pub fn has_changes(&self) -> bool {
        self.entries.iter().any(|e| e.is_changed())
    }

    /// Get only changes (filter out unchanged)
    pub fn changes_only(&self) -> Vec<&DiffEntry> {
        self.entries.iter().filter(|e| e.is_changed()).collect()
    }

    /// Count changes
    pub fn change_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_changed()).count()
    }

    /// Get changes for a specific category
    pub fn category_changes(&self, category: SettingsCategory) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category && e.is_changed())
            .collect()
    }
}

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

/// Format diff for display
pub fn format_diff(diff: &SettingsDiff) -> String {
    let mut output = String::new();

    if diff.is_identical() {
        output.push_str("Settings are identical.\n");
        return output;
    }

    output.push_str(&format!("=== Settings Diff ({} changes) ===\n\n", diff.change_count()));

    for entry in diff.changes_only() {
        output.push_str(&format!(
            "[{}] {}.{}\n",
            entry.diff_type, entry.category, entry.field
        ));
        if let Some(old) = &entry.old_value {
            output.push_str(&format!("  - {}\n", old));
        }
        if let Some(new) = &entry.new_value {
            output.push_str(&format!("  + {}\n", new));
        }
    }

    output
}

/// Fun fact about settings diff
pub fn settings_diff_fun_fact() -> &'static str {
    "Anna can show you exactly what changed between any two configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_type_display() {
        assert_eq!(format!("{}", DiffType::Added), "+");
        assert_eq!(format!("{}", DiffType::Removed), "-");
        assert_eq!(format!("{}", DiffType::Changed), "~");
        assert_eq!(format!("{}", DiffType::Unchanged), " ");
    }

    #[test]
    fn test_diff_entry_new() {
        let entry = DiffEntry::new(SettingsCategory::Privacy, "test", DiffType::Changed);
        assert!(entry.is_changed());
        assert_eq!(entry.category, SettingsCategory::Privacy);
    }

    #[test]
    fn test_diff_entry_values() {
        let entry = DiffEntry::new(SettingsCategory::Privacy, "test", DiffType::Changed)
            .old("old_val")
            .new_val("new_val");
        assert_eq!(entry.old_value, Some("old_val".to_string()));
        assert_eq!(entry.new_value, Some("new_val".to_string()));
    }

    #[test]
    fn test_settings_diff_empty() {
        let diff = SettingsDiff::new();
        assert!(diff.is_identical());
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_settings_diff_with_change() {
        let mut diff = SettingsDiff::new();
        diff.add(DiffEntry::new(
            SettingsCategory::Privacy,
            "test",
            DiffType::Changed,
        ));
        assert!(!diff.is_identical());
        assert!(diff.has_changes());
        assert_eq!(diff.change_count(), 1);
    }

    #[test]
    fn test_settings_diff_category_changes() {
        let mut diff = SettingsDiff::new();
        diff.add(DiffEntry::new(
            SettingsCategory::Privacy,
            "test",
            DiffType::Changed,
        ));
        diff.add(DiffEntry::new(
            SettingsCategory::Model,
            "test2",
            DiffType::Changed,
        ));

        let privacy_changes = diff.category_changes(SettingsCategory::Privacy);
        assert_eq!(privacy_changes.len(), 1);
    }

    #[test]
    fn test_differ_identical_settings() {
        let settings = UnifiedSettings::default();
        let diff = diff_settings(&settings, &settings);
        assert!(diff.is_identical());
    }

    #[test]
    fn test_differ_with_change() {
        let old = UnifiedSettings::default();
        let mut new = UnifiedSettings::default();
        new.learning.enable();

        let diff = diff_settings(&old, &new);
        assert!(diff.has_changes());
        assert!(diff.changed_categories.contains(&SettingsCategory::Learning));
    }

    #[test]
    fn test_differ_only_categories() {
        let old = UnifiedSettings::default();
        let mut new = UnifiedSettings::default();
        new.learning.enable();

        let diff = SettingsDiffer::new()
            .only_categories(vec![SettingsCategory::Privacy])
            .diff(&old, &new);

        // Should not see Learning changes since we only asked for Privacy
        assert!(diff.is_identical());
    }

    #[test]
    fn test_format_diff_identical() {
        let diff = SettingsDiff::new();
        let output = format_diff(&diff);
        assert!(output.contains("identical"));
    }

    #[test]
    fn test_format_diff_with_changes() {
        let mut diff = SettingsDiff::new();
        diff.add(
            DiffEntry::new(SettingsCategory::Privacy, "data_collection", DiffType::Changed)
                .old("Full")
                .new_val("Minimal"),
        );
        let output = format_diff(&diff);
        assert!(output.contains("1 changes"));
        assert!(output.contains("Privacy"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_diff_fun_fact();
        assert!(fact.contains("changed"));
    }
}

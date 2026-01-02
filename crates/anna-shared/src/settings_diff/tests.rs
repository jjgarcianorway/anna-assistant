// Tests for settings diff module

use super::*;
use crate::unified_settings::{SettingsCategory, UnifiedSettings};

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

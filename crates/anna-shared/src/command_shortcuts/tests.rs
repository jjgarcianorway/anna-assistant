//! Tests for command shortcuts functionality.

#[cfg(test)]
mod tests {
    use crate::command_shortcuts::*;

    #[test]
    fn test_builtin_shortcuts() {
        let shortcuts = builtin_shortcuts();
        assert!(shortcuts.len() >= 20);

        // Check each shortcut is valid
        for shortcut in &shortcuts {
            assert!(!shortcut.short.is_empty());
            assert!(!shortcut.expanded.is_empty());
            assert!(!shortcut.description.is_empty());
        }
    }

    #[test]
    fn test_expand_shortcut() {
        assert_eq!(
            expand_shortcut("mem"),
            Some("show memory usage".to_string())
        );
        assert_eq!(
            expand_shortcut("du"),
            Some("show disk usage".to_string())
        );
        assert_eq!(expand_shortcut("unknown"), None);
    }

    #[test]
    fn test_expand_shortcut_case_insensitive() {
        assert_eq!(
            expand_shortcut("MEM"),
            Some("show memory usage".to_string())
        );
        assert_eq!(
            expand_shortcut("Du"),
            Some("show disk usage".to_string())
        );
    }

    #[test]
    fn test_is_shortcut() {
        assert!(is_shortcut("mem"));
        assert!(is_shortcut("du"));
        assert!(is_shortcut("dps"));
        assert!(!is_shortcut("not a shortcut"));
    }

    #[test]
    fn test_shortcuts_by_category() {
        let system = shortcuts_by_category(ShortcutCategory::System);
        assert!(!system.is_empty());
        assert!(system.iter().all(|s| s.category == ShortcutCategory::System));

        let docker = shortcuts_by_category(ShortcutCategory::Docker);
        assert!(!docker.is_empty());
        assert!(docker.iter().all(|s| s.category == ShortcutCategory::Docker));
    }

    #[test]
    fn test_format_shortcuts() {
        let output = format_shortcuts();
        assert!(output.contains("Command Shortcuts"));
        assert!(output.contains("System"));
        assert!(output.contains("mem"));
        assert!(output.contains("show memory usage"));
    }

    #[test]
    fn test_format_category_shortcuts() {
        let output = format_category_shortcuts(ShortcutCategory::Docker);
        assert!(output.contains("Docker Shortcuts"));
        assert!(output.contains("dps"));
        assert!(output.contains("list docker containers"));
    }

    #[test]
    fn test_is_shortcuts_query() {
        assert!(is_shortcuts_query("show shortcuts"));
        assert!(is_shortcuts_query("what shortcuts are available?"));
        assert!(is_shortcuts_query("list all aliases"));

        assert!(!is_shortcuts_query("show disk usage"));
        assert!(!is_shortcuts_query("restart docker"));
    }

    #[test]
    fn test_build_shortcut_map() {
        let map = build_shortcut_map();
        assert_eq!(map.get("mem"), Some(&"show memory usage"));
        assert_eq!(map.get("du"), Some(&"show disk usage"));
        assert!(map.len() >= 20);
    }

    #[test]
    fn test_all_categories_have_shortcuts() {
        for category in ShortcutCategory::all() {
            let shortcuts = shortcuts_by_category(*category);
            assert!(
                !shortcuts.is_empty(),
                "{:?} should have shortcuts",
                category
            );
        }
    }
}

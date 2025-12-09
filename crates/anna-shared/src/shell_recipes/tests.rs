//! Shell recipe tests (v0.0.231).

#[cfg(test)]
mod tests {
    use crate::shell_recipes::{
        builtin_recipes, detect_feature, find_recipe, Shell, ShellFeature,
    };

    #[test]
    fn test_shell_config_path() {
        // Just test that paths end with expected filenames
        assert!(Shell::Bash
            .config_path()
            .to_string_lossy()
            .ends_with(".bashrc"));
        assert!(Shell::Zsh
            .config_path()
            .to_string_lossy()
            .ends_with(".zshrc"));
    }

    #[test]
    fn test_find_recipe() {
        let recipe = find_recipe(Shell::Bash, ShellFeature::ColoredPrompt);
        assert!(recipe.is_some());
        assert!(recipe.unwrap().lines.iter().any(|l| l.contains("PS1")));
    }

    #[test]
    fn test_detect_feature() {
        assert_eq!(
            detect_feature("show git branch in prompt"),
            Some(ShellFeature::GitPrompt)
        );
        assert_eq!(
            detect_feature("enable syntax highlighting"),
            Some(ShellFeature::SyntaxHighlighting)
        );
        assert_eq!(detect_feature("colored ls"), Some(ShellFeature::ColoredLs));
    }

    #[test]
    fn test_builtin_recipes_count() {
        let recipes = builtin_recipes();
        assert!(recipes.len() >= 5);
    }
}

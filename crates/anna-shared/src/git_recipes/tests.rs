//! Git recipes tests (v0.0.224).

#[cfg(test)]
mod tests {
    use crate::git_recipes::{builtin_recipes, detect_feature, find_recipe, GitFeature};

    #[test]
    fn test_find_and_detect() {
        let recipes = find_recipe(GitFeature::UserIdentity);
        assert!(!recipes.is_empty());
        assert!(recipes[0].needs_parameters());
        assert_eq!(
            detect_feature("set git name"),
            Some(GitFeature::UserIdentity)
        );
        assert_eq!(
            detect_feature("change default branch"),
            Some(GitFeature::DefaultBranch)
        );
        assert!(builtin_recipes().len() >= 10);
    }

    #[test]
    fn test_apply_params() {
        let recipe = find_recipe(GitFeature::UserIdentity)
            .into_iter()
            .next()
            .unwrap();
        let commands = recipe.apply_params(&[
            ("name".to_string(), "John".to_string()),
            ("email".to_string(), "j@x.com".to_string()),
        ]);
        assert!(commands[0].contains("John"));
    }
}

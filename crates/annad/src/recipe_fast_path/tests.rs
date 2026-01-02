//! Tests for recipe fast path functionality.

#[cfg(test)]
mod tests {
    use super::super::checker::check_recipe_fast_path;
    use super::super::converter::build_recipe_result;
    use super::super::learned::can_answer_directly;
    use anna_shared::recipe_index::RecipeIndex;
    use anna_shared::transcript::Transcript;

    #[test]
    fn test_no_match_empty_index() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("random query", &index);
        assert!(!result.matched);
    }

    #[test]
    fn test_shell_recipe_match() {
        let index = RecipeIndex::new();
        // Use "zsh" because syntax highlighting recipe exists only for zsh
        let result = check_recipe_fast_path("enable syntax highlighting in zsh", &index);
        // Should match built-in shell recipe
        assert!(result.matched);
        assert!(result.skip_llm);
    }

    #[test]
    fn test_shell_recipe_match_bash_color() {
        let index = RecipeIndex::new();
        // Bash has colored prompt recipe
        let result = check_recipe_fast_path("enable colored prompt in bash", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
    }

    #[test]
    fn test_git_recipe_match() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("configure git aliases", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
    }

    #[test]
    fn test_git_recipe_no_match_without_git() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("configure aliases", &index);
        // Should not match git recipes without "git" in query
        // (might match other recipes though)
        if result.matched {
            assert!(result
                .recipe
                .as_ref()
                .map(|r| !r.id.starts_with("git"))
                .unwrap_or(true));
        }
    }

    #[test]
    fn test_can_answer_directly_with_template() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("enable syntax highlighting in zsh", &index);
        // Should be able to answer directly (has answer_template)
        assert!(can_answer_directly(&result));
    }

    #[test]
    fn test_build_recipe_result() {
        let index = RecipeIndex::new();
        let query = "enable colored prompt in bash";
        let result = check_recipe_fast_path(query, &index);
        assert!(result.matched);

        let recipe = result.recipe.as_ref().unwrap();
        let transcript = Transcript::new();
        let service_result = build_recipe_result(
            "test-123".to_string(),
            recipe,
            &result.matched_tokens,
            transcript,
            query,
        );

        // Verify the result
        assert_eq!(service_result.request_id, "test-123");
        assert!(service_result.answer.contains("PS1")); // Colored prompt has PS1
        assert!(service_result.reliability_score >= 90);
        assert!(service_result.execution_trace.is_some());
        // Recipe answers are deterministic
        assert!(
            service_result
                .execution_trace
                .as_ref()
                .unwrap()
                .answer_is_deterministic
        );
    }

    #[test]
    fn test_ssh_recipe_match_generate_key() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("how do I generate an ssh key", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
        assert!(result
            .recipe
            .as_ref()
            .unwrap()
            .answer_template
            .contains("ssh-keygen"));
    }

    #[test]
    fn test_ssh_recipe_match_github() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("setup ssh for github", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
        assert!(result
            .recipe
            .as_ref()
            .unwrap()
            .answer_template
            .contains("github"));
    }

    #[test]
    fn test_ssh_recipe_match_copy_key() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("ssh copy key to server", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
        assert!(result
            .recipe
            .as_ref()
            .unwrap()
            .answer_template
            .contains("ssh-copy-id"));
    }
}

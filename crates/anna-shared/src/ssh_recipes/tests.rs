//! Tests for ssh_recipes module (v0.0.196).

#[cfg(test)]
mod tests {
    use crate::ssh_recipes::{match_query, SshFeature, SshKeyType};

    #[test]
    fn test_keygen_command() {
        let cmd = SshKeyType::Ed25519.keygen_command("test@example.com");
        assert!(cmd.contains("ed25519"));
        assert!(cmd.contains("test@example.com"));
    }

    #[test]
    fn test_match_generate_key() {
        let recipe = match_query("how do I generate an ssh key");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SshFeature::GenerateKey);
    }

    #[test]
    fn test_match_copy_key() {
        let recipe = match_query("ssh copy key to server");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SshFeature::CopyKey);
    }

    #[test]
    fn test_match_github() {
        let recipe = match_query("setup ssh for github");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SshFeature::GitHubSsh);
    }

    #[test]
    fn test_no_match_unrelated() {
        let recipe = match_query("what is the weather");
        assert!(recipe.is_none());
    }
}

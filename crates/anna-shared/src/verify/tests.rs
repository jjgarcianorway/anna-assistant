//! Tests for verify module (v0.0.198).

#[cfg(test)]
mod tests {
    use crate::verify::{
        expand_path, run_verification, PreActionVerify, ServiceExpectedState, VerificationStep,
    };

    #[test]
    fn test_verify_command_exists_sh() {
        let step = VerificationStep::editor_installed("sh");
        let result = run_verification(&step);
        assert!(result.passed, "sh should exist on Unix systems");
    }

    #[test]
    fn test_verify_command_not_exists() {
        let step = VerificationStep::editor_installed("definitely_not_a_real_command_xyz");
        let result = run_verification(&step);
        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_pre_action_verify_batch() {
        let verify = PreActionVerify::new()
            .add(VerificationStep::editor_installed("sh"))
            .add(VerificationStep::editor_installed("nonexistent_xyz").optional())
            .run();

        assert!(verify.all_passed); // sh passes, nonexistent is optional
    }

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("~/.vimrc");
        assert!(!expanded.starts_with("~"));
    }

    #[test]
    fn test_verification_step_constructors() {
        let step = VerificationStep::editor_installed("vim");
        assert!(step.id.contains("vim"));
        assert!(step.mandatory);

        let step = VerificationStep::file_has_line("/etc/hosts", "localhost");
        assert!(step.description.contains("localhost"));

        let step = VerificationStep::service_is("sshd", ServiceExpectedState::Active);
        assert!(step.description.contains("sshd"));
    }
}

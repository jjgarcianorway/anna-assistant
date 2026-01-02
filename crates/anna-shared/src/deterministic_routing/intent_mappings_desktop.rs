//! Desktop department intent mappings.

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_desktop_mappings(mappings: &mut HashMap<CanonicalIntent, IntentMapping>) {
    mappings.insert(
        CanonicalIntent::SessionDesktop,
        IntentMapping {
            intent: CanonicalIntent::SessionDesktop,
            department: Department::Desktop,
            required_probes: vec!["echo_xdg_session", "loginctl_session"],
            optional_probes: vec!["echo_desktop_session"],
            can_answer_from_probes: true,
            description: "Desktop session info",
        },
    );

    mappings.insert(
        CanonicalIntent::EditorConfig,
        IntentMapping {
            intent: CanonicalIntent::EditorConfig,
            department: Department::Desktop,
            required_probes: vec![], // No probes, config lookup
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Editor configuration",
        },
    );

    mappings.insert(
        CanonicalIntent::ShellConfig,
        IntentMapping {
            intent: CanonicalIntent::ShellConfig,
            department: Department::Desktop,
            required_probes: vec!["echo_shell"],
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Shell configuration",
        },
    );

    mappings.insert(
        CanonicalIntent::ThemeConfig,
        IntentMapping {
            intent: CanonicalIntent::ThemeConfig,
            department: Department::Desktop,
            required_probes: vec![],
            optional_probes: vec!["gsettings_theme"],
            can_answer_from_probes: false,
            description: "Theme configuration",
        },
    );
}

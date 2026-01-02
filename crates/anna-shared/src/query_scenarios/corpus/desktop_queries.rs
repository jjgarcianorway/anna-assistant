//! Desktop team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== DESKTOP TEAM (15 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "enable syntax highlighting in vim".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("turn on vim colors".into()),
        tags: vec!["vim".into(), "config".into(), "recipe".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "edit my vimrc".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("configure vim".into()),
        tags: vec!["vim".into(), "edit".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "set nano as default editor".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("change default text editor".into()),
        tags: vec!["nano".into(), "default".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "configure hyprland.conf".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("edit hyprland config".into()),
        tags: vec!["hyprland".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to install gnome extensions".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["gnome".into(), "extension".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "fix screen tearing in KDE".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["kde".into(), "graphics".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "change keyboard shortcut for screenshot".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["shortcut".into(), "screenshot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "enable dark mode in gtk apps".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("dark theme for gtk".into()),
        tags: vec!["gtk".into(), "theme".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "wayland vs x11 which am I running".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("check display server".into()),
        tags: vec!["wayland".into(), "x11".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "fix blurry fonts on hidpi display".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["hidpi".into(), "fonts".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "configure neovim with lua".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["neovim".into(), "lua".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup helix editor".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["helix".into(), "editor".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "show line numbers in vim".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("vim display line numbers".into()),
        tags: vec!["vim".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "customize bash prompt".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("change PS1".into()),
        tags: vec!["bash".into(), "prompt".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "configure tmux keybindings".into(),
        expected_team: Team::Desktop,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["tmux".into(), "config".into()],
    });
}

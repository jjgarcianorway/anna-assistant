//! User-related answer functions (v0.0.187).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer logged in users query using who command
pub fn answer_logged_in_users(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "who")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No users currently logged in.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let sessions: Vec<&str> = output.lines().collect();
    let user_count = sessions.len();

    let unique_users: std::collections::HashSet<&str> = sessions
        .iter()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    let answer = if unique_users.len() == 1 && user_count == 1 {
        format!("1 user logged in: {}", unique_users.iter().next().unwrap_or(&"unknown"))
    } else if unique_users.len() == 1 {
        format!("{} sessions for user: {}", user_count, unique_users.iter().next().unwrap_or(&"unknown"))
    } else {
        format!(
            "{} users logged in ({} sessions): {}",
            unique_users.len(),
            user_count,
            unique_users.into_iter().collect::<Vec<_>>().join(", ")
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: user_count,
        route_class: route_class.to_string(),
    })
}

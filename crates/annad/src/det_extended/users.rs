//! User-related answer functions (v0.0.175).
//!
//! Users, groups, environment, shells, desktops.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer logged in users query using who command
pub fn answer_logged_in_users(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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

/// Answer current user query using id
pub fn answer_current_user(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "current_user")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let mut username = String::new();
    let mut uid = String::new();
    let mut groups = Vec::new();

    for part in output.split_whitespace() {
        if part.starts_with("uid=") {
            if let Some(name) = part.split('(').nth(1) {
                username = name.trim_end_matches(')').to_string();
            }
            if let Some(id) = part.strip_prefix("uid=") {
                uid = id.split('(').next().unwrap_or("").to_string();
            }
        } else if part.starts_with("groups=") {
            let grp = part.strip_prefix("groups=").unwrap_or("");
            for g in grp.split(',') {
                if let Some(name) = g.split('(').nth(1) {
                    groups.push(name.trim_end_matches(')').to_string());
                }
            }
        }
    }

    Some(DeterministicResult {
        answer: format!("User: {} (uid={})\nGroups: {}", username, uid, groups.join(", ")),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer environment variables query
pub fn answer_environment_vars(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "env_vars")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No environment variables found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let var_count = output.lines().count();
    let important_vars = ["PATH", "HOME", "USER", "SHELL", "TERM", "DISPLAY", "XDG_SESSION_TYPE"];
    let mut key_vars = Vec::new();
    let mut other_count = 0;

    for line in output.lines() {
        let key = line.split('=').next().unwrap_or("");
        if important_vars.contains(&key) {
            key_vars.push(line);
        } else {
            other_count += 1;
        }
    }

    let answer = if !key_vars.is_empty() {
        format!(
            "Environment variables ({}):\n  {}\n  ...and {} others",
            var_count, key_vars.join("\n  "), other_count
        )
    } else {
        format!("Environment variables ({}):\n{}", var_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: var_count,
        route_class: route_class.to_string(),
    })
}

/// Answer user groups query
pub fn answer_user_groups(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "user_groups")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "Unable to determine user groups.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("User group membership:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer available shells query
pub fn answer_available_shells(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "available_shells")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Shell list not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let shells: Vec<&str> = output.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).collect();
    Some(DeterministicResult {
        answer: format!("Available shells ({}):\n{}", shells.len(), shells.join("\n")),
        grounded: true,
        parsed_data_count: shells.len(),
        route_class: route_class.to_string(),
    })
}

/// Answer installed desktops query
pub fn answer_installed_desktops(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "installed_desktops")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No desktop environments detected.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let de_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Installed desktop environments ({}):\n{}", de_count, output),
        grounded: true,
        parsed_data_count: de_count,
        route_class: route_class.to_string(),
    })
}

/// Answer environment variables query (duplicate handler)
pub fn answer_environment_variables(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "environment_variables")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No environment variables found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (format!("Environment variables ({} shown):\n```\n{}\n```", count, output), count)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

//! Docker-related answer functions (v0.0.175).
//!
//! Handles Docker containers and images.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer Docker containers query
pub fn answer_docker_containers(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "docker_containers")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Docker is not installed or not running.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let container_count = output.lines().count().saturating_sub(1);
    let answer = if container_count == 0 {
        "No running containers.".to_string()
    } else {
        format!(
            "Docker containers ({}):\n```\n{}\n```",
            container_count, output
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: container_count,
        route_class: route_class.to_string(),
    })
}

/// Answer Docker images query
pub fn answer_docker_images(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "docker_images")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Docker is not installed or not running.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let image_count = output.lines().count().saturating_sub(1);
    let answer = if image_count == 0 {
        "No Docker images found.".to_string()
    } else {
        format!("Docker images ({}):\n```\n{}\n```", image_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: image_count,
        route_class: route_class.to_string(),
    })
}

//! Kernel answer functions (v0.0.175).
//!
//! Kernel modules, dmesg, cmdline, firmware, version.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer kernel version query using uname probe
pub fn answer_kernel_version(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "uname")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let parts: Vec<&str> = output.split_whitespace().collect();
    let answer = if parts.len() >= 3 {
        format!("Kernel version: {} ({})", parts[2], parts[0])
    } else {
        format!("Kernel: {}", output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer kernel modules query
pub fn answer_kernel_modules(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "kernel_modules")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No kernel modules information available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let module_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!(
            "Loaded kernel modules ({}):\n```\n{}\n```",
            module_count, output
        ),
        grounded: true,
        parsed_data_count: module_count,
        route_class: route_class.to_string(),
    })
}

/// Answer dmesg errors query
pub fn answer_dmesg_errors(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "dmesg_errors")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No kernel errors or warnings found in dmesg.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let error_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!(
            "Kernel errors/warnings ({} messages):\n```\n{}\n```",
            error_count, output
        ),
        grounded: true,
        parsed_data_count: error_count,
        route_class: route_class.to_string(),
    })
}

/// Answer kernel command line query
pub fn answer_kernel_cmdline(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "kernel_cmdline")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("Kernel command line not available.".to_string(), 0)
    } else {
        (format!("Kernel command line:\n```\n{}\n```", output), 1)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer module parameters query
pub fn answer_module_params(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "module_params")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No kernel module information available.".to_string(), 0)
    } else {
        let module_count = output.matches("===").count();
        (
            format!(
                "Kernel module parameters ({} modules):\n```\n{}\n```",
                module_count, output
            ),
            module_count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer loaded firmware query using dmesg
pub fn answer_loaded_firmware(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "loaded_firmware")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.contains("not available") || output.is_empty() {
        ("No firmware loading information available.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (
            format!(
                "Firmware/microcode log ({} entries):\n```\n{}\n```",
                count, output
            ),
            count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer Xorg log query
pub fn answer_xorg_log(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "xorg_log")?;

    let output = probe.stdout.trim();
    if output.contains("not found") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Xorg log not found. X11 may not be installed or using Wayland.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let error_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!(
            "Xorg log errors/warnings ({}):\n```\n{}\n```",
            error_count, output
        ),
        grounded: true,
        parsed_data_count: error_count,
        route_class: route_class.to_string(),
    })
}

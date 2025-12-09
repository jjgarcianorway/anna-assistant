//! Peripheral answer functions (v0.0.175).
//!
//! Bluetooth, audio devices, printers.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer bluetooth devices query
pub fn answer_bluetooth_devices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "bluetooth_devices")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Bluetooth not available or no devices paired.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Bluetooth devices ({}):\n```\n{}\n```", device_count, output),
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

/// Answer printer status query
pub fn answer_printer_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "printer_status")?;

    let output = probe.stdout.trim();
    if output.contains("No printers") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No printers configured on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Printer status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer audio devices query
pub fn answer_audio_devices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "audio_devices")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No audio devices found or audio system not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Audio devices ({}):\n```\n{}\n```", device_count, output),
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

//! Audio device answer handler (v0.0.176).

use anna_shared::rpc::ProbeResult;

use super::DeterministicResult;

/// Answer hardware audio query using typed AudioDevices evidence (v0.0.66)
/// v0.0.66: Clean output format, no markdown in debug OFF mode.
pub fn answer_hardware_audio(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};

    // Parse all probes to typed evidence
    let parsed: Vec<ParsedProbeData> = probes.iter().map(parse_probe_result).collect();

    // Count how many audio evidence sources we have
    let audio_evidence_count = parsed.iter().filter(|p| p.as_audio().is_some()).count();

    // Find merged audio evidence from parsed probes
    if let Some(audio) = find_audio_evidence(&parsed) {
        if audio.devices.is_empty() {
            // v0.0.66: Grounded negative evidence - only say "No audio" when evidenced
            let source_msg = if audio.source.contains('+') {
                "lspci and pactl"
            } else {
                &audio.source
            };
            return Some(DeterministicResult {
                answer: format!("No audio devices detected (checked {}).", source_msg),
                grounded: true,
                parsed_data_count: audio_evidence_count.max(1),
                route_class: route_class.to_string(),
            });
        }

        // v0.0.66: Clean format for debug OFF - "Detected audio hardware: <desc> (PCI <slot>)"
        let answer = if audio.devices.len() == 1 {
            let dev = &audio.devices[0];
            let pci_info = dev
                .pci_slot
                .as_ref()
                .map(|s| format!(" (PCI {})", s))
                .unwrap_or_default();
            format!("Detected audio hardware: {}{}", dev.description, pci_info)
        } else {
            // Multiple devices: one per line, no markdown
            let devices_list: Vec<String> = audio
                .devices
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let pci_info = d
                        .pci_slot
                        .as_ref()
                        .map(|s| format!(" (PCI {})", s))
                        .unwrap_or_default();
                    format!("  {}. {}{}", i + 1, d.description, pci_info)
                })
                .collect();
            format!(
                "Detected {} audio devices:\n{}",
                audio.devices.len(),
                devices_list.join("\n")
            )
        };

        return Some(DeterministicResult {
            answer,
            grounded: true,
            parsed_data_count: audio.devices.len().max(audio_evidence_count),
            route_class: route_class.to_string(),
        });
    }

    // Fallback: no audio evidence found - should not happen with proper probes
    None
}

//! Helper functions for finding and querying evidence (v0.0.173).

use super::audio::merge_audio_devices;
use super::evidence::{AudioDevices, PackageInstalled, ToolExists};
use super::parsed_data::ParsedProbeData;

/// Find tool existence evidence for a given tool name (v0.45.7).
pub fn find_tool_evidence<'a>(parsed: &'a [ParsedProbeData], name: &str) -> Option<&'a ToolExists> {
    parsed
        .iter()
        .filter_map(|p| p.as_tool())
        .find(|t| t.name.to_lowercase() == name.to_lowercase())
}

/// Find package installation evidence for a given package name (v0.45.7).
pub fn find_package_evidence<'a>(
    parsed: &'a [ParsedProbeData],
    name: &str,
) -> Option<&'a PackageInstalled> {
    parsed
        .iter()
        .filter_map(|p| p.as_package())
        .find(|p| p.name.to_lowercase() == name.to_lowercase())
}

/// Check if any tool/package evidence exists (positive or negative) for a name.
pub fn has_evidence_for(parsed: &[ParsedProbeData], name: &str) -> bool {
    find_tool_evidence(parsed, name).is_some() || find_package_evidence(parsed, name).is_some()
}

/// Find audio devices evidence (v0.45.8, v0.0.56, v0.0.60 merged).
/// If multiple sources exist (lspci + pactl), merge them:
/// - Use lspci for hardware identity (PCI slot, device name)
/// - Use pactl for card names/profiles if lspci found nothing
/// Never return "No audio" if either source has devices.
/// v0.0.60: Improved merging with deduplication by (pci_slot, description).
pub fn find_audio_evidence(parsed: &[ParsedProbeData]) -> Option<AudioDevices> {
    let all_audio: Vec<&AudioDevices> = parsed.iter().filter_map(|p| p.as_audio()).collect();

    if all_audio.is_empty() {
        return None;
    }

    // If only one source, return it
    if all_audio.len() == 1 {
        return Some(all_audio[0].clone());
    }

    // v0.0.60: Merge all sources with deduplication
    let lspci_audio = all_audio.iter().find(|a| a.source == "lspci");
    let pactl_audio = all_audio.iter().find(|a| a.source == "pactl");

    match (lspci_audio, pactl_audio) {
        (Some(lspci), Some(pactl)) => {
            // v0.0.60: Merge devices from both sources, deduplicate
            let merged = merge_audio_devices(&lspci.devices, &pactl.devices);
            if merged.is_empty() {
                // Both empty - return lspci (grounded negative evidence)
                Some(AudioDevices {
                    devices: vec![],
                    source: "lspci+pactl".to_string(),
                })
            } else {
                Some(AudioDevices {
                    devices: merged,
                    source: "lspci+pactl".to_string(),
                })
            }
        }
        (Some(lspci), None) => Some((*lspci).clone()),
        (None, Some(pactl)) => Some((*pactl).clone()),
        (None, None) => {
            // Unknown sources - return first with devices, or first
            all_audio
                .iter()
                .find(|a| !a.devices.is_empty())
                .or(all_audio.first())
                .map(|a| (*a).clone())
        }
    }
}

/// Find audio devices evidence returning a reference (v0.45.8 legacy)
pub fn find_audio_evidence_ref(parsed: &[ParsedProbeData]) -> Option<&AudioDevices> {
    parsed.iter().filter_map(|p| p.as_audio()).next()
}

/// Get all tool evidence from parsed probes (v0.45.8, v0.0.56 fix).
/// Returns both positive (exists=true) and negative (exists=false) evidence.
/// Caller should filter by `.exists` if only installed tools are needed.
pub fn get_installed_tools(parsed: &[ParsedProbeData]) -> Vec<&ToolExists> {
    parsed.iter().filter_map(|p| p.as_tool()).collect()
}

/// v0.0.59: Extract installed editor names from parsed probe evidence.
/// Only returns editors that exist (exists=true) in current probe results.
/// Maps tool names to canonical editor identifiers.
/// Returns sorted, deduplicated list for stable output.
pub fn installed_editors_from_parsed(parsed: &[ParsedProbeData]) -> Vec<String> {
    // Supported editor mappings: tool_name -> canonical_name
    const EDITOR_MAP: &[(&str, &str)] = &[
        ("vim", "vim"),
        ("nvim", "nvim"),
        ("nano", "nano"),
        ("emacs", "emacs"),
        ("micro", "micro"),
        ("hx", "helix"),
        ("helix", "helix"),
        ("code", "code"),
        ("kate", "kate"),
        ("gedit", "gedit"),
    ];

    let tools = get_installed_tools(parsed);
    let mut editors: Vec<String> = tools
        .iter()
        .filter(|t| t.exists)
        .filter_map(|t| {
            EDITOR_MAP
                .iter()
                .find(|(tool, _)| *tool == t.name.as_str())
                .map(|(_, canonical)| canonical.to_string())
        })
        .collect();

    // Deduplicate (in case hx and helix both map to helix)
    editors.sort();
    editors.dedup();
    editors
}

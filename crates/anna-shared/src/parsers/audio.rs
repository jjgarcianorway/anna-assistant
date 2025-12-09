//! Audio device parsing from lspci and pactl (v0.0.173).

use crate::rpc::ProbeResult;

use super::evidence::{AudioDevice, AudioDevices};
use super::parsed_data::ParsedProbeData;

/// v0.0.60, v0.0.61: Check if a command is an lspci audio probe.
/// Matches:
/// - "lspci | grep -i audio"
/// - "lspci_audio" probe ID
/// - Raw lspci output when context suggests audio
/// v0.0.61: Also check stdout for audio controller patterns
pub fn is_lspci_audio_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    // Explicit audio grep pattern
    if cmd_lower.contains("lspci") && cmd_lower.contains("audio") {
        return true;
    }
    // Probe ID form
    if cmd_lower == "lspci_audio" {
        return true;
    }
    false
}

/// v0.0.61: Check if stdout contains lspci audio device output.
/// This catches cases where the command doesn't match but output is clearly lspci audio.
pub fn stdout_contains_audio_device(stdout: &str) -> bool {
    let lower = stdout.to_lowercase();
    // Check for common lspci audio device class patterns
    // Note: lspci may show "[0403]" PCI class code between name and colon
    lower.contains("audio device")
        || lower.contains("multimedia audio controller")
        || lower.contains("audio controller")
        || (lower.contains("multimedia controller") && lower.contains("audio"))
}

/// Try to parse audio devices from lspci or pactl (v0.45.8, v0.0.60, v0.0.61 expanded).
/// Handles `lspci | grep -i audio` and `pactl list cards` commands.
/// v0.0.60: Improved to handle more lspci variants and grep exit codes.
/// v0.0.61: Also detect audio device output in stdout (fallback for command mismatch).
pub fn try_parse_audio_devices(probe: &ProbeResult, cmd_lower: &str) -> Option<ParsedProbeData> {
    // v0.0.61: First check if stdout contains audio device output
    // This catches cases where command pattern doesn't match exactly
    let has_lspci_audio_output = stdout_contains_audio_device(&probe.stdout);

    // Pattern: lspci audio probe (by command or by output content)
    if is_lspci_audio_command(&probe.command) || has_lspci_audio_output {
        // v0.0.60: Handle grep exit codes correctly
        // exit_code 0 = matches found (devices present)
        // exit_code 1 = no matches (valid empty evidence for grep)
        // exit_code 2+ = grep error

        // v0.0.61: If we detected audio output, always try to parse it
        if has_lspci_audio_output && probe.exit_code == 0 {
            let devices = parse_lspci_audio_output(&probe.stdout);
            if !devices.is_empty() {
                return Some(ParsedProbeData::Audio(AudioDevices {
                    devices,
                    source: "lspci".to_string(),
                }));
            }
        }

        if probe.exit_code == 1 && probe.stdout.trim().is_empty() {
            // grep found no matches - valid negative evidence
            return Some(ParsedProbeData::Audio(AudioDevices {
                devices: vec![],
                source: "lspci".to_string(),
            }));
        }

        if probe.exit_code != 0 && probe.exit_code != 1 {
            // Real error (exit code 2+)
            return Some(ParsedProbeData::Audio(AudioDevices {
                devices: vec![],
                source: "lspci".to_string(),
            }));
        }

        let devices = parse_lspci_audio_output(&probe.stdout);
        return Some(ParsedProbeData::Audio(AudioDevices {
            devices,
            source: "lspci".to_string(),
        }));
    }

    // Pattern: "pactl list cards"
    if cmd_lower.contains("pactl") && cmd_lower.contains("cards") {
        // pactl may return empty or error if no pulseaudio - still valid evidence
        if probe.exit_code != 0 || probe.stdout.trim().is_empty() {
            return Some(ParsedProbeData::Audio(AudioDevices {
                devices: vec![],
                source: "pactl".to_string(),
            }));
        }

        let devices = parse_pactl_cards_output(&probe.stdout);
        return Some(ParsedProbeData::Audio(AudioDevices {
            devices,
            source: "pactl".to_string(),
        }));
    }

    // v0.0.61: Also detect pactl output by content (Card # blocks)
    if probe.stdout.contains("Card #") && probe.exit_code == 0 {
        let devices = parse_pactl_cards_output(&probe.stdout);
        return Some(ParsedProbeData::Audio(AudioDevices {
            devices,
            source: "pactl".to_string(),
        }));
    }

    None
}

/// Parse lspci audio output (v0.0.55 fix: handles PCI class codes in brackets).
/// Handles multiple lspci formats:
/// - "00:1f.3 Audio device: Intel Corporation Cannon Lake PCH cAVS (rev 10)"
/// - "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS"
/// - "00:1f.3 Audio controller [0403]: Some Device" (with PCI class code)
/// - "00:1f.3 Multimedia audio controller [0403]: Intel Corporation..." (common format)
/// v0.0.55: Fixed to handle PCI class codes like [0403] between device class and colon.
pub fn parse_lspci_audio_output(stdout: &str) -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let line_lower = line.to_lowercase();

        // v0.0.55: Check for audio-related device class markers (case-insensitive)
        let is_audio_line = line_lower.contains("audio device")
            || line_lower.contains("multimedia audio controller")
            || line_lower.contains("audio controller")
            || (line_lower.contains("multimedia controller") && line_lower.contains("audio"));

        // v0.0.55: If the line contains "audio" and has PCI slot format, trust it
        let has_pci_slot = line.len() > 7 && line.chars().nth(2) == Some(':');
        let is_grep_match = line_lower.contains("audio") && has_pci_slot;

        if !is_audio_line && !is_grep_match {
            continue;
        }

        // Parse format: "XX:XX.X <device_class> [XXXX]: <description>"
        let pci_slot = extract_pci_slot(line);

        // v0.0.55: Extract description after device class marker (handles [XXXX] codes)
        let description = extract_lspci_description_v055(line);

        // Extract vendor from description (usually first word)
        let vendor = extract_vendor_from_description(&description);

        if !description.is_empty() {
            devices.push(AudioDevice {
                description,
                pci_slot,
                vendor,
            });
        }
    }

    devices
}

/// v0.0.60: Extract PCI slot from lspci line.
/// Expects format like "00:1f.3" at the beginning of the line.
pub fn extract_pci_slot(line: &str) -> Option<String> {
    let first_token = line.split_whitespace().next()?;
    // PCI slot format: XX:XX.X (e.g., 00:1f.3)
    if first_token.contains(':') && first_token.contains('.') {
        Some(first_token.to_string())
    } else {
        None
    }
}

/// v0.0.55: Extract description from lspci line, handling PCI class codes [XXXX].
/// Handles formats like:
/// - "00:1f.3 Audio device: Intel..."
/// - "00:1f.3 Multimedia audio controller [0403]: Intel..."
pub fn extract_lspci_description_v055(line: &str) -> String {
    // v0.0.55: Find the LAST colon before the description
    // This handles PCI class codes like [0403] that appear before the colon
    // Format: "XX:XX.X Device Type [XXXX]: Description"

    // Skip the PCI slot colon (first colon at position ~2)
    let after_slot = if line.len() > 8 && line.chars().nth(2) == Some(':') {
        &line[7..] // Skip "XX:XX.X"
    } else {
        line
    };

    // Find the colon that separates device class from description
    if let Some(colon_pos) = after_slot.find(':') {
        let description = after_slot[colon_pos + 1..].trim();
        if !description.is_empty() {
            return description.to_string();
        }
    }

    // Fallback: try to extract after common device class patterns
    let patterns = [
        "audio device",
        "multimedia audio controller",
        "audio controller",
    ];

    let line_lower = line.to_lowercase();
    for pattern in patterns {
        if let Some(pos) = line_lower.find(pattern) {
            let after_pattern = &line[pos + pattern.len()..];
            // Skip any [XXXX] class code and colon
            if let Some(colon_pos) = after_pattern.find(':') {
                let desc = after_pattern[colon_pos + 1..].trim();
                if !desc.is_empty() {
                    return desc.to_string();
                }
            }
        }
    }

    // Last resort: return the whole line (minus PCI slot)
    if line.len() > 8 {
        line[8..].trim().to_string()
    } else {
        line.to_string()
    }
}

/// Parse pactl list cards output (v0.45.8, v0.0.60 expanded).
/// v0.0.60: Also looks for driver, card.name, and other properties.
pub fn parse_pactl_cards_output(stdout: &str) -> Vec<AudioDevice> {
    let mut devices = Vec::new();
    let mut current_card_name: Option<String> = None;
    let mut current_card_description: Option<String> = None;
    let mut in_card_block = false;

    for line in stdout.lines() {
        let line = line.trim();

        // Detect card block start
        if line.starts_with("Card #") {
            // Save previous card if any
            if in_card_block {
                if let Some(desc) = current_card_description.take().or(current_card_name.take()) {
                    let vendor = extract_vendor_from_description(&desc);
                    devices.push(AudioDevice {
                        description: desc,
                        pci_slot: None,
                        vendor,
                    });
                }
            }
            in_card_block = true;
            current_card_name = None;
            current_card_description = None;
        }

        // Look for "Name:" lines
        if line.starts_with("Name:") {
            current_card_name = Some(line.trim_start_matches("Name:").trim().to_string());
        }
        // Look for card description properties
        else if line.contains("alsa.card_name")
            || line.contains("device.description")
            || line.contains("card.name")
            || line.contains("device.product.name")
        {
            if let Some(pos) = line.find('=') {
                let value = line[pos + 1..].trim().trim_matches('"').to_string();
                if !value.is_empty() && current_card_description.is_none() {
                    current_card_description = Some(value);
                }
            }
        }
    }

    // Save last card
    if in_card_block {
        if let Some(desc) = current_card_description.take().or(current_card_name.take()) {
            let vendor = extract_vendor_from_description(&desc);
            devices.push(AudioDevice {
                description: desc,
                pci_slot: None,
                vendor,
            });
        }
    }

    // Fallback: if we found a name but no description anywhere
    if devices.is_empty() {
        if let Some(name) = current_card_name {
            devices.push(AudioDevice {
                description: name,
                pci_slot: None,
                vendor: None,
            });
        }
    }

    devices
}

/// Extract vendor name from audio device description.
pub fn extract_vendor_from_description(description: &str) -> Option<String> {
    let known_vendors = [
        "Intel",
        "NVIDIA",
        "AMD",
        "Realtek",
        "Creative",
        "C-Media",
        "VIA",
        "SoundBlaster",
        "Logitech",
        "Corsair",
        "HyperX",
    ];

    for vendor in known_vendors {
        if description.to_lowercase().contains(&vendor.to_lowercase()) {
            return Some(vendor.to_string());
        }
    }

    // Try to extract first word if it looks like a vendor (capitalized)
    let first_word = description.split_whitespace().next()?;
    if first_word.chars().next()?.is_uppercase() && first_word.len() > 2 {
        return Some(first_word.to_string());
    }

    None
}

/// v0.0.60: Merge audio devices from lspci and pactl, deduplicating by description.
/// Prefers lspci devices (have PCI slot) over pactl (no PCI slot).
pub fn merge_audio_devices(lspci: &[AudioDevice], pactl: &[AudioDevice]) -> Vec<AudioDevice> {
    let mut merged: Vec<AudioDevice> = Vec::new();

    // Add all lspci devices first (preferred source)
    for dev in lspci {
        merged.push(dev.clone());
    }

    // Add pactl devices that aren't duplicates
    for pactl_dev in pactl {
        // Check if this pactl device is a duplicate of an lspci device
        // Compare by normalized description (case-insensitive, trim whitespace)
        let is_duplicate = merged.iter().any(|existing| {
            // Check if descriptions overlap (one contains the other)
            let existing_lower = existing.description.to_lowercase();
            let pactl_lower = pactl_dev.description.to_lowercase();
            existing_lower.contains(&pactl_lower) || pactl_lower.contains(&existing_lower)
        });

        if !is_duplicate {
            merged.push(pactl_dev.clone());
        }
    }

    merged
}

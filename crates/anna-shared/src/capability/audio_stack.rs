//! Audio Stack: PipeWire vs PulseAudio Detection
//!
//! Capability: audio.stack.detect (ReadOnly)
//!
//! Phase 33: Deterministic probing of audio subsystem.
//!
//! What this does:
//! - Detect which audio server is running (PipeWire, PulseAudio, JACK)
//! - Check if PipeWire is running with pipewire-pulse compatibility
//! - List active sinks and sources
//! - Report sample rate and buffer configuration
//!
//! What this does NOT do:
//! - Does not switch between audio servers
//! - Does not configure ALSA directly
//! - Does not handle Bluetooth audio (that's audio.bluetooth)

use super::response::{CapabilityExecutionResult, ResponseArtifact};
use std::process::Command;

// =============================================================================
// PROBE TYPES
// =============================================================================

/// Detected audio stack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioStack {
    PipeWire,
    PipeWireWithPulse,  // PipeWire running pipewire-pulse
    PulseAudio,
    Jack,
    Alsa,  // Raw ALSA, no server
    None,
}

impl AudioStack {
    pub fn name(&self) -> &'static str {
        match self {
            AudioStack::PipeWire => "PipeWire",
            AudioStack::PipeWireWithPulse => "PipeWire (with PulseAudio compat)",
            AudioStack::PulseAudio => "PulseAudio",
            AudioStack::Jack => "JACK",
            AudioStack::Alsa => "ALSA (no server)",
            AudioStack::None => "None detected",
        }
    }
}

/// Audio sink (output device).
#[derive(Debug, Clone)]
pub struct AudioSink {
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub state: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}

/// Audio source (input device).
#[derive(Debug, Clone)]
pub struct AudioSource {
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub state: String,
}

/// Complete audio probe results.
#[derive(Debug, Clone)]
pub struct AudioProbes {
    pub stack: AudioStack,
    pub pipewire_running: bool,
    pub pulseaudio_running: bool,
    pub jack_running: bool,
    pub pipewire_version: Option<String>,
    pub pulse_version: Option<String>,
    pub sinks: Vec<AudioSink>,
    pub sources: Vec<AudioSource>,
    pub default_sink: Option<String>,
    pub default_source: Option<String>,
    pub sample_rate: Option<u32>,
    pub buffer_size: Option<u32>,
}

impl AudioProbes {
    /// Phase 35: Convert probes to evidence - CAPPED AT 3 LINES.
    pub fn to_evidence(&self) -> Vec<ResponseArtifact> {
        let mut evidence = vec![];

        // Line 1: Stack with version and pipewire-pulse status
        let stack_info = match (&self.pipewire_version, &self.pulse_version) {
            (Some(pw_ver), _) if self.pipewire_running => format!("{} ({})", self.stack.name(), pw_ver),
            (_, Some(pa_ver)) if self.pulseaudio_running => format!("{} ({})", self.stack.name(), pa_ver),
            _ => self.stack.name().to_string(),
        };
        evidence.push(ResponseArtifact::evidence("Stack:", &stack_info));

        // Line 2: Default output
        if let Some(ref sink) = self.default_sink {
            let short_sink = sink.split('.').last().unwrap_or(sink);
            evidence.push(ResponseArtifact::evidence("Output:", short_sink));
        }

        // Line 3: Sample rate (if available)
        if let Some(rate) = self.sample_rate {
            evidence.push(ResponseArtifact::evidence("Rate:", &format!("{} Hz", rate)));
        } else if !self.sinks.is_empty() {
            evidence.push(ResponseArtifact::evidence("Devices:", &format!("{} outputs", self.sinks.len())));
        }

        evidence
    }

    /// Phase 35: Deterministic single-line explanation.
    pub fn format_explanation(&self) -> String {
        let compat_info = if self.stack == AudioStack::PipeWireWithPulse {
            " pipewire-pulse active."
        } else {
            ""
        };
        let rate_info = self.sample_rate
            .map(|r| format!(" {}Hz.", r))
            .unwrap_or_default();
        format!("{}.{}{}", self.stack.name(), compat_info, rate_info)
    }
}

// =============================================================================
// PROBE IMPLEMENTATION
// =============================================================================

/// Run all probes for audio stack.
pub fn gather_probes() -> AudioProbes {
    let pipewire_running = is_process_running("pipewire");
    let pulseaudio_running = is_process_running("pulseaudio");
    let pipewire_pulse_running = is_process_running("pipewire-pulse");
    let jack_running = is_process_running("jackd") || is_process_running("jackdbus");

    let pipewire_version = get_pipewire_version();
    let pulse_version = get_pulseaudio_version();

    // Determine stack
    let stack = if pipewire_running && pipewire_pulse_running {
        AudioStack::PipeWireWithPulse
    } else if pipewire_running {
        AudioStack::PipeWire
    } else if pulseaudio_running {
        AudioStack::PulseAudio
    } else if jack_running {
        AudioStack::Jack
    } else if has_alsa_devices() {
        AudioStack::Alsa
    } else {
        AudioStack::None
    };

    // Get sinks and sources based on stack
    let (sinks, sources, default_sink, default_source) = if pipewire_running || pulseaudio_running {
        probe_pulse_devices()
    } else {
        (vec![], vec![], None, None)
    };

    let (sample_rate, buffer_size) = if pipewire_running {
        probe_pipewire_config()
    } else {
        (None, None)
    };

    AudioProbes {
        stack,
        pipewire_running,
        pulseaudio_running: pulseaudio_running || pipewire_pulse_running,
        jack_running,
        pipewire_version,
        pulse_version,
        sinks,
        sources,
        default_sink,
        default_source,
        sample_rate,
        buffer_size,
    }
}

fn is_process_running(name: &str) -> bool {
    Command::new("pgrep").args(["-x", name]).output().map(|o| o.status.success()).unwrap_or(false)
}

fn get_pipewire_version() -> Option<String> {
    Command::new("pipewire").arg("--version").output().ok().and_then(|o| {
        if o.status.success() {
            String::from_utf8_lossy(&o.stdout).lines()
                .find(|l| l.contains("libpipewire"))
                .and_then(|l| l.split_whitespace().last()).map(|s| s.to_string())
        } else { None }
    })
}

fn get_pulseaudio_version() -> Option<String> {
    Command::new("pulseaudio").arg("--version").output().ok().and_then(|o| {
        if o.status.success() {
            String::from_utf8_lossy(&o.stdout).split_whitespace().last().map(|s| s.to_string())
        } else { None }
    })
}

fn has_alsa_devices() -> bool {
    Command::new("aplay").args(["-l"]).output().map(|o| o.status.success() && !o.stdout.is_empty()).unwrap_or(false)
}

fn probe_pulse_devices() -> (Vec<AudioSink>, Vec<AudioSource>, Option<String>, Option<String>) {
    let mut sinks = Vec::new();
    let mut sources = Vec::new();
    let mut default_sink = None;
    let mut default_source = None;

    // Get default sink
    if let Ok(output) = Command::new("pactl").args(["get-default-sink"]).output() {
        if output.status.success() {
            default_sink = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Get default source
    if let Ok(output) = Command::new("pactl").args(["get-default-source"]).output() {
        if output.status.success() {
            default_source = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // List sinks
    if let Ok(output) = Command::new("pactl")
        .args(["list", "sinks", "short"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let name = parts[1].to_string();
                    let is_default = default_sink.as_ref().map_or(false, |d| d == &name);
                    sinks.push(AudioSink {
                        name: name.clone(),
                        description: name,  // pactl short doesn't give description
                        is_default,
                        state: parts.get(4).unwrap_or(&"unknown").to_string(),
                        sample_rate: None,
                        channels: None,
                    });
                }
            }
        }
    }

    // List sources
    if let Ok(output) = Command::new("pactl")
        .args(["list", "sources", "short"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let name = parts[1].to_string();
                    let is_default = default_source.as_ref().map_or(false, |d| d == &name);
                    sources.push(AudioSource {
                        name: name.clone(),
                        description: name,
                        is_default,
                        state: parts.get(4).unwrap_or(&"unknown").to_string(),
                    });
                }
            }
        }
    }

    (sinks, sources, default_sink, default_source)
}

fn probe_pipewire_config() -> (Option<u32>, Option<u32>) {
    let mut sample_rate = None;
    let mut buffer_size = None;

    // Try pw-metadata
    if let Ok(output) = Command::new("pw-metadata")
        .args(["-n", "settings"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("clock.rate") {
                    // Parse: key:'clock.rate' value:'48000'
                    if let Some(val) = extract_pw_metadata_value(line) {
                        sample_rate = val.parse().ok();
                    }
                }
                if line.contains("clock.quantum") {
                    if let Some(val) = extract_pw_metadata_value(line) {
                        buffer_size = val.parse().ok();
                    }
                }
            }
        }
    }

    (sample_rate, buffer_size)
}

fn extract_pw_metadata_value(line: &str) -> Option<&str> {
    // Format: key:'clock.rate' value:'48000'
    line.split("value:'")
        .nth(1)
        .and_then(|s| s.split('\'').next())
}

// =============================================================================
// CAPABILITY HANDLER
// =============================================================================

/// Execute the audio.stack.detect capability.
/// Phase 33: ReadOnly capability - returns facts, no mutations.
pub fn execute_audio_stack_detect() -> CapabilityExecutionResult {
    let probes = gather_probes();

    // No audio detected
    if probes.stack == AudioStack::None {
        return CapabilityExecutionResult::with_explanation(
            probes.to_evidence(),
            "No audio server detected. ALSA may be available for direct hardware access. \
            To use PipeWire, ensure it's installed and started: systemctl --user start pipewire pipewire-pulse",
        );
    }

    let explanation = probes.format_explanation();
    CapabilityExecutionResult::with_explanation(probes.to_evidence(), &explanation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_returns_resolved() {
        let result = execute_audio_stack_detect();
        assert!(!result.wants_abstain(), "ReadOnly audio stack should not abstain");
    }

    #[test]
    fn test_evidence_capped_at_three() {
        let probes = AudioProbes {
            stack: AudioStack::PipeWireWithPulse, pipewire_running: true, pulseaudio_running: true,
            jack_running: false, pipewire_version: Some("1.0.3".to_string()), pulse_version: None,
            sinks: vec![AudioSink {
                name: "sink1".to_string(), description: "Built-in".to_string(),
                is_default: true, state: "RUNNING".to_string(), sample_rate: Some(48000), channels: Some(2),
            }],
            sources: vec![], default_sink: Some("sink1".to_string()), default_source: None,
            sample_rate: Some(48000), buffer_size: Some(1024),
        };
        let evidence = probes.to_evidence();
        assert!(evidence.len() <= 3, "Phase 35: Evidence must be capped at 3 lines");
    }

    #[test]
    fn test_deterministic_stack_names() {
        assert_eq!(AudioStack::PipeWire.name(), "PipeWire");
        assert_eq!(AudioStack::PipeWireWithPulse.name(), "PipeWire (with PulseAudio compat)");
        assert_eq!(AudioStack::PulseAudio.name(), "PulseAudio");
        assert_eq!(AudioStack::Jack.name(), "JACK");
        assert_eq!(AudioStack::None.name(), "None detected");
    }

    #[test]
    fn test_pipewire_pulse_in_explanation() {
        let probes = AudioProbes {
            stack: AudioStack::PipeWireWithPulse, pipewire_running: true, pulseaudio_running: true,
            jack_running: false, pipewire_version: None, pulse_version: None,
            sinks: vec![], sources: vec![], default_sink: None, default_source: None,
            sample_rate: Some(48000), buffer_size: None,
        };
        let explanation = probes.format_explanation();
        assert!(explanation.contains("pipewire-pulse active"), "Must show pipewire-pulse compat");
    }
}

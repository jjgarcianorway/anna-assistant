//! Audio system detection
//!
//! Detects audio configuration:
//! - Audio server (PipeWire, PulseAudio, ALSA)
//! - JACK presence
//! - Audio devices
//! - Default sinks/sources

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Audio server type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioServer {
    /// PipeWire audio server
    PipeWire,
    /// PulseAudio server
    PulseAudio,
    /// ALSA only (no server)
    AlsaOnly,
    /// Unknown audio system
    Unknown,
}

/// Audio system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    /// Primary audio server
    pub server: AudioServer,
    /// JACK is available
    pub jack_available: bool,
    /// Server is running
    pub server_running: bool,
    /// Default sink (output device)
    pub default_sink: Option<String>,
    /// Default source (input device)
    pub default_source: Option<String>,
    /// Number of sinks
    pub sink_count: usize,
    /// Number of sources
    pub source_count: usize,
}

impl AudioInfo {
    /// Detect audio system information
    pub fn detect() -> Self {
        let server = detect_audio_server();
        let jack_available = detect_jack();
        let server_running = is_server_running(&server);
        let (default_sink, default_source, sink_count, source_count) = get_audio_devices(&server);

        Self {
            server,
            jack_available,
            server_running,
            default_sink,
            default_source,
            sink_count,
            source_count,
        }
    }
}

/// Detect which audio server is in use
fn detect_audio_server() -> AudioServer {
    // Method 1: Check if PipeWire is running
    if let Ok(output) = Command::new("pw-cli").arg("info").output() {
        if output.status.success() {
            return AudioServer::PipeWire;
        }
    }

    // Method 2: Check pactl for PipeWire or PulseAudio
    if let Ok(output) = Command::new("pactl").arg("info").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("PipeWire") {
                return AudioServer::PipeWire;
            } else if stdout.contains("Server Name: pulseaudio") || stdout.contains("PulseAudio") {
                return AudioServer::PulseAudio;
            }
        }
    }

    // Method 3: Check running processes
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("pipewire").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return AudioServer::PipeWire;
        }
    }

    if let Ok(output) = Command::new("pgrep").arg("-x").arg("pulseaudio").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return AudioServer::PulseAudio;
        }
    }

    // Method 4: Check for ALSA only
    if let Ok(output) = Command::new("aplay").arg("-l").output() {
        if output.status.success() {
            return AudioServer::AlsaOnly;
        }
    }

    AudioServer::Unknown
}

/// Check if JACK is available
fn detect_jack() -> bool {
    // Check if jackd binary exists
    if let Ok(output) = Command::new("command").arg("-v").arg("jackd").output() {
        if output.status.success() {
            return true;
        }
    }

    // Check if JACK is running
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("jackd").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return true;
        }
    }

    false
}

/// Check if audio server is running
fn is_server_running(server: &AudioServer) -> bool {
    match server {
        AudioServer::PipeWire => {
            // Check if PipeWire daemon is running
            if let Ok(output) = Command::new("pgrep").arg("-x").arg("pipewire").output() {
                return output.status.success() && !output.stdout.is_empty();
            }
            false
        }
        AudioServer::PulseAudio => {
            // Check if PulseAudio is running
            if let Ok(output) = Command::new("pgrep").arg("-x").arg("pulseaudio").output() {
                return output.status.success() && !output.stdout.is_empty();
            }
            false
        }
        AudioServer::AlsaOnly => {
            // ALSA is kernel-level, always "running" if present
            true
        }
        AudioServer::Unknown => false,
    }
}

/// Get audio device information
fn get_audio_devices(server: &AudioServer) -> (Option<String>, Option<String>, usize, usize) {
    match server {
        AudioServer::PipeWire | AudioServer::PulseAudio => get_pulse_devices(),
        AudioServer::AlsaOnly => get_alsa_devices(),
        AudioServer::Unknown => (None, None, 0, 0),
    }
}

/// Get audio devices using pactl (PipeWire/PulseAudio)
fn get_pulse_devices() -> (Option<String>, Option<String>, usize, usize) {
    let mut default_sink = None;
    let mut default_source = None;
    let mut sink_count = 0;
    let mut source_count = 0;

    // Get default sink
    if let Ok(output) = Command::new("pactl").arg("get-default-sink").output() {
        if output.status.success() {
            default_sink = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Get default source
    if let Ok(output) = Command::new("pactl").arg("get-default-source").output() {
        if output.status.success() {
            default_source = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Count sinks
    if let Ok(output) = Command::new("pactl").arg("list").arg("short").arg("sinks").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            sink_count = stdout.lines().count();
        }
    }

    // Count sources
    if let Ok(output) = Command::new("pactl").arg("list").arg("short").arg("sources").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            source_count = stdout.lines().filter(|line| !line.contains(".monitor")).count();
        }
    }

    (default_sink, default_source, sink_count, source_count)
}

/// Get audio devices using ALSA
fn get_alsa_devices() -> (Option<String>, Option<String>, usize, usize) {
    let mut playback_count = 0;
    let mut capture_count = 0;

    // List playback devices
    if let Ok(output) = Command::new("aplay").arg("-l").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            playback_count = stdout.lines().filter(|line| line.contains("card")).count();
        }
    }

    // List capture devices
    if let Ok(output) = Command::new("arecord").arg("-l").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            capture_count = stdout.lines().filter(|line| line.contains("card")).count();
        }
    }

    // ALSA doesn't have "default" in the same way, use card 0
    let default_sink = if playback_count > 0 {
        Some("hw:0,0".to_string())
    } else {
        None
    };

    let default_source = if capture_count > 0 {
        Some("hw:0,0".to_string())
    } else {
        None
    };

    (default_sink, default_source, playback_count, capture_count)
}

impl std::fmt::Display for AudioServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioServer::PipeWire => write!(f, "PipeWire"),
            AudioServer::PulseAudio => write!(f, "PulseAudio"),
            AudioServer::AlsaOnly => write!(f, "ALSA Only"),
            AudioServer::Unknown => write!(f, "Unknown"),
        }
    }
}

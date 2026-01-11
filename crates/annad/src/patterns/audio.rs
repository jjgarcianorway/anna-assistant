//! Audio patterns for PipeWire, PulseAudio, ALSA troubleshooting.
//! v0.0.959: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an audio-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type AudioPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match audio-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_general_audio(q)
        .or_else(|| match_pipewire(q))
        .or_else(|| match_pulseaudio(q))
        .or_else(|| match_alsa(q))
        .or_else(|| match_bluetooth_audio(q))
}

/// General audio patterns
fn match_general_audio(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // Sound status
        (&["sound", "working"], "check if sound is working", "audio",
         &["pactl info 2>/dev/null || pipewire --version", "aplay -l"]),
        (&["no", "sound"], "troubleshoot no sound", "audio",
         &["pactl info", "aplay -l", "amixer sget Master"]),
        (&["audio", "not", "working"], "troubleshoot audio issues", "audio",
         &["pactl info", "aplay -l", "systemctl --user status pipewire"]),
        (&["no", "audio"], "troubleshoot no audio", "audio",
         &["pactl info", "aplay -l", "amixer"]),
        // Volume
        (&["volume", "level"], "show volume level", "audio",
         &["pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null || amixer sget Master"]),
        (&["current", "volume"], "show current volume", "audio",
         &["pactl get-sink-volume @DEFAULT_SINK@", "amixer sget Master"]),
        (&["muted"], "check if muted", "audio",
         &["pactl get-sink-mute @DEFAULT_SINK@", "amixer sget Master | grep -i mute"]),
        // Devices
        (&["audio", "devices"], "list audio devices", "audio",
         &["pactl list sinks short", "aplay -l"]),
        (&["sound", "cards"], "list sound cards", "audio",
         &["aplay -l", "cat /proc/asound/cards"]),
        (&["output", "devices"], "list audio output devices", "audio",
         &["pactl list sinks short"]),
        (&["input", "devices"], "list audio input devices", "audio",
         &["pactl list sources short", "arecord -l"]),
        // Default device
        (&["default", "sink"], "show default audio output", "audio",
         &["pactl get-default-sink"]),
        (&["default", "source"], "show default audio input", "audio",
         &["pactl get-default-source"]),
        // Audio system
        (&["audio", "system"], "check audio system", "audio",
         &["pactl info | head -5", "pipewire --version 2>/dev/null", "pulseaudio --version 2>/dev/null"]),
        (&["sound", "server"], "show sound server info", "audio",
         &["pactl info"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// PipeWire patterns
fn match_pipewire(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // Status
        (&["pipewire", "status"], "show PipeWire status", "audio",
         &["systemctl --user status pipewire pipewire-pulse"]),
        (&["pipewire", "running"], "check if PipeWire is running", "audio",
         &["systemctl --user is-active pipewire"]),
        (&["pipewire", "version"], "show PipeWire version", "audio",
         &["pipewire --version"]),
        // Nodes
        (&["pipewire", "nodes"], "list PipeWire nodes", "audio",
         &["pw-cli list-objects Node"]),
        (&["pipewire", "devices"], "list PipeWire devices", "audio",
         &["pw-cli list-objects Device"]),
        (&["pipewire", "links"], "list PipeWire links", "audio",
         &["pw-link -l"]),
        // Restart
        (&["restart", "pipewire"], "restart PipeWire info", "audio",
         &["echo 'Run: systemctl --user restart pipewire pipewire-pulse wireplumber'"]),
        // WirePlumber
        (&["wireplumber", "status"], "show WirePlumber status", "audio",
         &["systemctl --user status wireplumber"]),
        (&["wireplumber", "logs"], "show WirePlumber logs", "audio",
         &["journalctl --user -u wireplumber -n 30"]),
        // PipeWire config
        (&["pipewire", "config"], "show PipeWire config location", "audio",
         &["ls ~/.config/pipewire/ 2>/dev/null || echo 'No user config, using /usr/share/pipewire/'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// PulseAudio patterns
fn match_pulseaudio(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // Status
        (&["pulseaudio", "status"], "show PulseAudio status", "audio",
         &["pactl info", "systemctl --user status pulseaudio 2>/dev/null"]),
        (&["pulseaudio", "running"], "check if PulseAudio is running", "audio",
         &["pactl info >/dev/null && echo 'Running' || echo 'Not running'"]),
        (&["pulseaudio", "version"], "show PulseAudio version", "audio",
         &["pulseaudio --version"]),
        // Sinks/Sources
        (&["pactl", "sinks"], "list PulseAudio sinks", "audio",
         &["pactl list sinks short"]),
        (&["pactl", "sources"], "list PulseAudio sources", "audio",
         &["pactl list sources short"]),
        // Restart
        (&["restart", "pulseaudio"], "restart PulseAudio info", "audio",
         &["echo 'Run: systemctl --user restart pulseaudio' or 'pulseaudio -k && pulseaudio --start'"]),
        // Modules
        (&["pulseaudio", "modules"], "list PulseAudio modules", "audio",
         &["pactl list modules short"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// ALSA patterns
fn match_alsa(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // Devices
        (&["alsa", "devices"], "list ALSA devices", "audio",
         &["aplay -l"]),
        (&["alsa", "cards"], "list ALSA cards", "audio",
         &["cat /proc/asound/cards"]),
        // Mixer
        (&["alsa", "mixer"], "show ALSA mixer settings", "audio",
         &["amixer"]),
        (&["amixer"], "show ALSA mixer", "audio",
         &["amixer"]),
        (&["alsa", "volume"], "show ALSA volume", "audio",
         &["amixer sget Master"]),
        // Playback test
        (&["alsa", "test"], "test ALSA playback", "audio",
         &["echo 'Run: speaker-test -c 2 -t wav'"]),
        // ALSA info
        (&["alsa", "info"], "show ALSA info", "audio",
         &["cat /proc/asound/cards", "aplay -l"]),
        // Modules
        (&["sound", "modules"], "list sound kernel modules", "audio",
         &["lsmod | grep -E 'snd|sound'"]),
        (&["audio", "modules"], "list audio kernel modules", "audio",
         &["lsmod | grep -E 'snd|sound'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Bluetooth audio patterns
fn match_bluetooth_audio(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // Bluetooth audio
        (&["bluetooth", "audio"], "check Bluetooth audio", "audio",
         &["pactl list sinks short | grep -i blue", "bluetoothctl devices"]),
        (&["bluetooth", "headphones"], "check Bluetooth headphones", "audio",
         &["bluetoothctl devices Connected", "pactl list sinks short"]),
        (&["bluetooth", "speaker"], "check Bluetooth speaker", "audio",
         &["bluetoothctl devices Connected", "pactl list sinks short"]),
        (&["bluetooth", "codec"], "check Bluetooth audio codec", "audio",
         &["pactl list sinks | grep -A5 -i bluetooth | grep -i codec"]),
        // A2DP
        (&["a2dp", "sink"], "check A2DP profile", "audio",
         &["pactl list cards | grep -A10 -i bluetooth"]),
        (&["bluetooth", "profile"], "check Bluetooth audio profile", "audio",
         &["pactl list cards | grep -A10 -i bluetooth"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_audio() {
        assert!(match_patterns("sound working").is_some());
        assert!(match_patterns("no sound").is_some());
        assert!(match_patterns("audio devices").is_some());
        assert!(match_patterns("volume level").is_some());
    }

    #[test]
    fn test_pipewire() {
        assert!(match_patterns("pipewire status").is_some());
        assert!(match_patterns("pipewire version").is_some());
        assert!(match_patterns("wireplumber status").is_some());
    }

    #[test]
    fn test_pulseaudio() {
        assert!(match_patterns("pulseaudio status").is_some());
        assert!(match_patterns("pactl sinks").is_some());
    }

    #[test]
    fn test_alsa() {
        assert!(match_patterns("alsa devices").is_some());
        assert!(match_patterns("alsa mixer").is_some());
        assert!(match_patterns("sound modules").is_some());
    }

    #[test]
    fn test_bluetooth_audio() {
        assert!(match_patterns("bluetooth audio").is_some());
        assert!(match_patterns("bluetooth headphones").is_some());
    }
}

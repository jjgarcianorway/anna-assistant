//! Audio patterns for PipeWire, PulseAudio, ALSA troubleshooting.
//! v0.0.959: Initial implementation.
//! v0.0.989: Expanded patterns for JACK, MIDI, latency, routing.

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
        .or_else(|| match_jack(q))
        .or_else(|| match_midi(q))
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
        // Latency and sample rate
        (&["audio", "latency"], "check audio latency", "audio",
         &["pw-top 2>/dev/null || echo 'Use pw-top or check JACK for latency'", "cat /proc/asound/card*/pcm*/sub*/hw_params 2>/dev/null | head -20"]),
        (&["sample", "rate"], "check audio sample rate", "audio",
         &["pactl info | grep 'Default Sample'", "pw-cli info all 2>/dev/null | grep -i rate | head -5"]),
        (&["audio", "routing"], "show audio routing", "audio",
         &["pw-link -l 2>/dev/null || pactl list sinks short", "pactl list sink-inputs short"]),
        // Speaker test
        (&["speaker", "test"], "test speakers", "audio",
         &["echo 'Run: speaker-test -c 2 -t wav (stereo test)'", "echo 'Or: speaker-test -D default -c 2 (specific device)'"]),
        // Headphone detection
        (&["headphone", "detection"], "check headphone detection", "audio",
         &["pactl list sinks | grep -A3 'Active Port'", "cat /proc/asound/card*/codec* 2>/dev/null | grep -i 'jack\\|headphone' | head -10"]),
        (&["headphones", "detected"], "check if headphones detected", "audio",
         &["pactl list sinks | grep -A3 'Active Port'", "dmesg | grep -i 'headphone\\|jack' | tail -5"]),
        // Audio codecs
        (&["audio", "codecs"], "show audio codecs", "audio",
         &["cat /proc/asound/card*/codec* 2>/dev/null | head -30", "pactl list sinks | grep -i codec"]),
        // Equalizer
        (&["equalizer", "settings"], "check equalizer settings", "audio",
         &["pactl list modules | grep -i equalizer", "pw-cli ls Module 2>/dev/null | grep -i eq"]),
        (&["audio", "equalizer"], "show audio equalizer", "audio",
         &["echo 'PulseAudio: pacmd list-modules | grep equalizer'", "echo 'EasyEffects: flatpak list | grep easyeffects'"]),
        // Audio profiles
        (&["audio", "profiles"], "list audio profiles", "audio",
         &["pactl list cards | grep -A5 'Profiles:'", "pactl list cards | grep 'Active Profile'"]),
        (&["sound", "profiles"], "show sound profiles", "audio",
         &["pactl list cards | grep -A5 'Profiles:'"]),
        // Default device
        (&["default", "audio"], "show default audio device", "audio",
         &["pactl get-default-sink", "pactl get-default-source"]),
        // Audio troubleshooting
        (&["audio", "troubleshoot"], "troubleshoot audio issues", "audio",
         &["pactl info", "aplay -l", "amixer sget Master", "systemctl --user status pipewire pipewire-pulse 2>/dev/null"]),
        (&["sound", "troubleshoot"], "troubleshoot sound problems", "audio",
         &["pactl info", "aplay -l", "journalctl --user -u pipewire -n 20"]),
        // Audio mixing
        (&["audio", "mixing"], "show audio mixer info", "audio",
         &["pactl list sink-inputs", "pw-top 2>/dev/null || pavucontrol --help 2>&1 | head -1"]),
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

/// JACK audio patterns
fn match_jack(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // JACK status
        (&["jack", "status"], "show JACK audio status", "audio",
         &["jack_lsp 2>/dev/null || echo 'JACK not running'", "ps aux | grep jackd | grep -v grep"]),
        (&["jack", "audio"], "check JACK audio", "audio",
         &["jack_lsp 2>/dev/null || pw-jack --version 2>/dev/null || echo 'JACK not available'"]),
        (&["jack", "running"], "check if JACK is running", "audio",
         &["jack_wait -c 2>/dev/null || ps aux | grep jackd"]),
        // JACK connections
        (&["jack", "connections"], "show JACK connections", "audio",
         &["jack_lsp -c 2>/dev/null"]),
        (&["jack", "ports"], "list JACK ports", "audio",
         &["jack_lsp 2>/dev/null || pw-jack jack_lsp 2>/dev/null"]),
        // JACK latency
        (&["jack", "latency"], "check JACK latency", "audio",
         &["jack_bufsize 2>/dev/null", "jack_samplerate 2>/dev/null", "echo 'Latency = buffer_size / sample_rate'"]),
        // JACK settings
        (&["jack", "settings"], "show JACK settings", "audio",
         &["jack_samplerate 2>/dev/null", "jack_bufsize 2>/dev/null", "cat ~/.jackdrc 2>/dev/null"]),
        // JACK start
        (&["start", "jack"], "how to start JACK", "audio",
         &["echo 'jackd -d alsa -r 48000 -p 1024'", "echo 'Or with PipeWire: pw-jack'"]),
        // JACK transport
        (&["jack", "transport"], "check JACK transport", "audio",
         &["jack_transport 2>/dev/null || echo 'JACK transport not available'"]),
        // PipeWire JACK
        (&["pipewire", "jack"], "check PipeWire JACK emulation", "audio",
         &["pw-jack --version 2>/dev/null", "systemctl --user status pipewire-jack"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// MIDI patterns
fn match_midi(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[AudioPattern] = &[
        // MIDI devices
        (&["midi", "devices"], "list MIDI devices", "audio",
         &["aconnect -l", "cat /proc/asound/seq/clients 2>/dev/null"]),
        (&["midi", "ports"], "list MIDI ports", "audio",
         &["aconnect -i", "aconnect -o"]),
        // MIDI connections
        (&["midi", "connections"], "show MIDI connections", "audio",
         &["aconnect -l"]),
        (&["connect", "midi"], "how to connect MIDI devices", "audio",
         &["echo 'List: aconnect -l'", "echo 'Connect: aconnect sender:port receiver:port'"]),
        // MIDI through
        (&["midi", "through"], "MIDI through info", "audio",
         &["modprobe snd-virmidi 2>/dev/null; aconnect -l | grep -i 'through\\|virtual'"]),
        // MIDI monitor
        (&["midi", "monitor"], "monitor MIDI input", "audio",
         &["echo 'Use: aseqdump -p CLIENT:PORT'", "echo 'Or: amidi -l to list, then amidi -d -p hw:X,X,X'"]),
        // USB MIDI
        (&["usb", "midi"], "check USB MIDI devices", "audio",
         &["lsusb | grep -i midi", "aconnect -l | grep -i usb"]),
        // MIDI keyboard
        (&["midi", "keyboard"], "check MIDI keyboard", "audio",
         &["aconnect -l", "lsusb | grep -i 'midi\\|keyboard'"]),
        // Virtual MIDI
        (&["virtual", "midi"], "setup virtual MIDI", "audio",
         &["echo 'Load module: sudo modprobe snd-virmidi'", "aconnect -l | grep -i virtual"]),
        // ALSA MIDI
        (&["alsa", "midi"], "check ALSA MIDI", "audio",
         &["amidi -l", "aconnect -l"]),
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
    fn test_audio_expanded() {
        assert!(match_patterns("audio latency").is_some());
        assert!(match_patterns("sample rate").is_some());
        assert!(match_patterns("audio routing").is_some());
        assert!(match_patterns("speaker test").is_some());
        assert!(match_patterns("headphone detection").is_some());
        assert!(match_patterns("audio codecs").is_some());
        assert!(match_patterns("equalizer settings").is_some());
        assert!(match_patterns("audio profiles").is_some());
        assert!(match_patterns("audio troubleshoot").is_some());
        assert!(match_patterns("audio mixing").is_some());
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
    fn test_jack() {
        assert!(match_patterns("jack status").is_some());
        assert!(match_patterns("jack audio").is_some());
        assert!(match_patterns("jack latency").is_some());
        assert!(match_patterns("jack connections").is_some());
        assert!(match_patterns("pipewire jack").is_some());
    }

    #[test]
    fn test_midi() {
        assert!(match_patterns("midi devices").is_some());
        assert!(match_patterns("midi ports").is_some());
        assert!(match_patterns("midi connections").is_some());
        assert!(match_patterns("usb midi").is_some());
        assert!(match_patterns("alsa midi").is_some());
    }

    #[test]
    fn test_bluetooth_audio() {
        assert!(match_patterns("bluetooth audio").is_some());
        assert!(match_patterns("bluetooth headphones").is_some());
    }
}

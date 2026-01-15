//! Audio, logs, time, environment, and user patterns.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use super::super::contains_word;
use super::FactualPattern;

/// v0.0.937: Audio and sound queries
pub fn match_audio(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Audio status
        (&["audio", "device"], "audio devices query", "audio", &["pactl list sinks short 2>/dev/null || wpctl status 2>/dev/null | head -30"]),
        (&["sound", "device"], "sound devices query", "audio", &["pactl list sinks short 2>/dev/null || wpctl status 2>/dev/null | head -30"]),
        (&["audio", "output"], "audio output query", "audio", &["pactl get-default-sink 2>/dev/null || wpctl status 2>/dev/null | head -20"]),
        // Volume
        (&["volume"], "volume level query", "audio", &["pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null || wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null"]),
        (&["audio", "level"], "audio level query", "audio", &["pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null"]),
        // Muted
        (&["muted"], "mute status query", "audio", &["pactl get-sink-mute @DEFAULT_SINK@ 2>/dev/null"]),
        // Microphone
        (&["microphone"], "microphone query", "audio", &["pactl list sources short 2>/dev/null | grep -v monitor"]),
        (&["mic"], "mic query", "audio", &["pactl list sources short 2>/dev/null | grep -v monitor"]),
        // Pipewire/Pulse
        (&["pipewire", "status"], "Pipewire status query", "audio", &["systemctl --user status pipewire", "wpctl status | head -30"]),
        (&["pulseaudio", "status"], "PulseAudio status query", "audio", &["pactl info | head -15"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.937: Boot and log queries
pub fn match_logs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Boot time
        (&["boot", "time"], "boot time query", "logs", &["systemd-analyze"]),
        (&["startup", "time"], "startup time query", "logs", &["systemd-analyze"]),
        (&["boot", "slow"], "slow boot analysis", "logs", &["systemd-analyze blame | head -15"]),
        // Boot logs
        (&["boot", "log"], "boot log query", "logs", &["journalctl -b -p err..warning | head -30"]),
        (&["boot", "error"], "boot errors query", "logs", &["journalctl -b -p err | head -20"]),
        (&["dmesg"], "kernel messages query", "logs", &["dmesg | tail -30"]),
        // v0.1.0: "Any errors" patterns - show actual errors, not counts
        (&["any", "errors"], "check for errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'No errors found'"]),
        (&["any", "log", "errors"], "check log errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'No errors found'"]),
        (&["there", "errors"], "check for errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'No errors in logs'"]),
        (&["no", "errors"], "verify no errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'Confirmed: No errors found'"]),
        (&["log", "errors"], "show log errors", "logs", &["journalctl -b -p err --no-pager | head -30"]),
        // System logs
        (&["system", "log"], "system log query", "logs", &["journalctl -p err..warning --since '1 hour ago' | head -30"]),
        (&["error", "log"], "error log query", "logs", &["journalctl -p err --since '1 hour ago' | head -30"]),
        (&["recent", "error"], "recent errors query", "logs", &["journalctl -p err --since '1 hour ago' | head -20"]),
        (&["recent", "errors"], "recent errors query", "logs", &["journalctl -p err --since '1 hour ago' | head -20"]),
        // Journal
        (&["journal"], "journal query", "logs", &["journalctl --since '1 hour ago' | tail -30"]),
        (&["journalctl"], "journalctl query", "logs", &["journalctl --since '1 hour ago' | tail -30"]),
        // Kernel messages
        (&["kernel", "log"], "kernel log query", "logs", &["dmesg | tail -30"]),
        (&["kernel", "error"], "kernel errors query", "logs", &["dmesg --level=err,warn | tail -20"]),
        (&["kernel", "errors"], "kernel errors query", "logs", &["dmesg --level=err,warn | tail -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.945: Time and date queries
pub fn match_time(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Current time
        (&["what", "time"], "current time query", "time", &["date +%H:%M:%S"]),
        (&["current", "time"], "current time query", "time", &["date +%H:%M:%S"]),
        (&["show", "time"], "current time query", "time", &["date +%H:%M:%S"]),
        // Current date
        (&["what", "date"], "current date query", "time", &["date +%Y-%m-%d"]),
        (&["current", "date"], "current date query", "time", &["date +%Y-%m-%d"]),
        (&["today", "date"], "current date query", "time", &["date +%Y-%m-%d"]),
        // Full datetime
        (&["date", "time"], "datetime query", "time", &["date"]),
        // Timezone
        (&["timezone"], "timezone query", "time", &["timedatectl | grep 'Time zone'"]),
        (&["time", "zone"], "timezone query", "time", &["timedatectl | grep 'Time zone'"]),
        (&["what", "tz"], "timezone query", "time", &["timedatectl | grep 'Time zone'"]),
        // Calendar
        (&["calendar"], "calendar query", "time", &["cal"]),
        (&["what", "day"], "day of week query", "time", &["date +%A"]),
        (&["what", "month"], "month query", "time", &["date +%B"]),
        (&["what", "year"], "year query", "time", &["date +%Y"]),
        // NTP status
        (&["ntp", "status"], "NTP status query", "time", &["timedatectl | grep -E 'NTP|synchronized'"]),
        (&["time", "sync"], "time sync status query", "time", &["timedatectl | grep -E 'NTP|synchronized'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.945: Environment and shell queries
/// v0.1.0: Added exclusion check for size-related queries
pub fn match_environment(q: &str) -> Option<DeepUnderstanding> {
    // v0.1.0: Skip environment patterns if query is about size/space/largest
    // These should go to filesystem patterns instead
    let size_keywords = ["largest", "biggest", "size", "space", "usage", "top", "du ", "how big", "how much"];
    if size_keywords.iter().any(|kw| contains_word(q, kw)) {
        return None;
    }

    let patterns: &[FactualPattern] = &[
        // Shell
        (&["what", "shell"], "shell query", "environment", &["echo $SHELL", "basename $SHELL"]),
        (&["which", "shell"], "shell query", "environment", &["echo $SHELL"]),
        (&["my", "shell"], "shell query", "environment", &["echo $SHELL"]),
        (&["default", "shell"], "default shell query", "environment", &["getent passwd $USER | cut -d: -f7"]),
        // Home directory
        (&["home", "directory"], "home directory query", "environment", &["echo $HOME"]),
        (&["home", "folder"], "home directory query", "environment", &["echo $HOME"]),
        // PATH
        (&["show", "path"], "PATH query", "environment", &["echo $PATH | tr ':' '\\n'"]),
        (&["what", "path"], "PATH query", "environment", &["echo $PATH | tr ':' '\\n'"]),
        // Environment variables
        (&["environment", "variable"], "env vars query", "environment", &["env | head -30"]),
        (&["env", "var"], "env vars query", "environment", &["env | head -30"]),
        (&["list", "env"], "env vars query", "environment", &["env | head -30"]),
        // Editor
        (&["default", "editor"], "editor query", "environment", &["echo $EDITOR", "echo $VISUAL"]),
        (&["what", "editor"], "editor query", "environment", &["echo $EDITOR", "echo $VISUAL"]),
        // Display
        (&["display", "variable"], "display query", "environment", &["echo $DISPLAY", "echo $WAYLAND_DISPLAY"]),
        (&["session", "type"], "session type query", "environment", &["echo $XDG_SESSION_TYPE"]),
        // Locale
        (&["locale"], "locale query", "environment", &["locale"]),
        (&["language", "setting"], "locale query", "environment", &["echo $LANG", "locale"]),
        // Terminal
        (&["terminal", "emulator"], "terminal query", "environment", &["echo $TERM"]),
        (&["what", "term"], "terminal query", "environment", &["echo $TERM"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.945: User and group queries
pub fn match_users(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Current user
        (&["my", "username"], "username query", "users", &["whoami"]),
        (&["my", "user"], "user info query", "users", &["id"]),
        (&["what", "user", "am"], "username query", "users", &["whoami", "id"]),
        // User groups
        (&["my", "group"], "groups query", "users", &["groups"]),
        (&["what", "group"], "groups query", "users", &["groups"]),
        (&["list", "group"], "all groups query", "users", &["cat /etc/group | cut -d: -f1 | head -20"]),
        // All users
        (&["list", "user"], "all users query", "users", &["cat /etc/passwd | cut -d: -f1 | head -20"]),
        (&["system", "user"], "system users query", "users", &["cat /etc/passwd | awk -F: '$3 < 1000 {print $1}'"]),
        // User info
        (&["user", "id"], "user ID query", "users", &["id"]),
        (&["uid"], "UID query", "users", &["id -u"]),
        (&["gid"], "GID query", "users", &["id -g"]),
        // Sudo
        (&["sudo", "access"], "sudo access query", "users", &["sudo -l 2>&1 | head -10"]),
        (&["can", "sudo"], "sudo capability query", "users", &["groups | grep -q sudo && echo 'Yes' || echo 'No'"]),
        (&["sudoer"], "sudoers query", "users", &["getent group sudo wheel | cut -d: -f4"]),
        // Login history
        (&["login", "history"], "login history query", "users", &["last | head -10"]),
        (&["last", "login"], "last login query", "users", &["lastlog -u $USER"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

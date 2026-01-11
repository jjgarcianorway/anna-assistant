//! Locale, keyboard, and language patterns.
//! v0.0.969: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a locale-related DeepUnderstanding
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

type LocalePattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match locale-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_locale(q)
        .or_else(|| match_keyboard(q))
        .or_else(|| match_language(q))
        .or_else(|| match_fonts(q))
}

/// Locale patterns
fn match_locale(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LocalePattern] = &[
        // Current locale
        (&["current", "locale"], "show current locale", "locale",
         &["locale", "localectl"]),
        (&["my", "locale"], "show my locale", "locale",
         &["locale"]),
        (&["system", "locale"], "show system locale", "locale",
         &["localectl status"]),
        // Locale settings
        (&["locale", "settings"], "show locale settings", "locale",
         &["locale", "cat /etc/locale.conf 2>/dev/null"]),
        (&["locale", "config"], "show locale configuration", "locale",
         &["cat /etc/locale.conf 2>/dev/null", "locale"]),
        // Available locales
        (&["available", "locales"], "list available locales", "locale",
         &["locale -a | head -50"]),
        (&["list", "locales"], "list locales", "locale",
         &["locale -a | head -50"]),
        (&["installed", "locales"], "show installed locales", "locale",
         &["locale -a"]),
        // Generated locales
        (&["generated", "locales"], "show generated locales", "locale",
         &["cat /etc/locale.gen | grep -v '^#' | grep -v '^$'"]),
        // Locale environment
        (&["lang", "variable"], "show LANG variable", "locale",
         &["echo $LANG", "locale | grep LANG"]),
        (&["lc_all"], "show LC_ALL variable", "locale",
         &["echo $LC_ALL", "locale | grep LC_ALL"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Keyboard patterns
fn match_keyboard(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LocalePattern] = &[
        // Current keyboard layout
        (&["keyboard", "layout"], "show keyboard layout", "locale",
         &["localectl status | grep 'Keymap\\|Layout'", "setxkbmap -query 2>/dev/null"]),
        (&["current", "keyboard"], "show current keyboard layout", "locale",
         &["localectl status | grep -i key", "setxkbmap -query 2>/dev/null"]),
        (&["my", "keyboard"], "show my keyboard layout", "locale",
         &["localectl status | grep -i key"]),
        // Console keymap
        (&["console", "keymap"], "show console keymap", "locale",
         &["localectl status | grep 'VC Keymap'", "cat /etc/vconsole.conf 2>/dev/null"]),
        (&["vconsole"], "show vconsole config", "locale",
         &["cat /etc/vconsole.conf 2>/dev/null"]),
        // X11 layout
        (&["x11", "keyboard"], "show X11 keyboard layout", "locale",
         &["setxkbmap -query 2>/dev/null", "localectl status | grep X11"]),
        (&["xkb", "layout"], "show XKB layout", "locale",
         &["setxkbmap -query 2>/dev/null"]),
        // Available layouts
        (&["available", "layouts"], "list available keyboard layouts", "locale",
         &["localectl list-keymaps | head -50"]),
        (&["list", "keymaps"], "list keymaps", "locale",
         &["localectl list-keymaps | head -50"]),
        (&["available", "keymaps"], "list available keymaps", "locale",
         &["localectl list-keymaps | head -50"]),
        // X11 layouts
        (&["x11", "layouts"], "list X11 layouts", "locale",
         &["localectl list-x11-keymap-layouts | head -50"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Language patterns
fn match_language(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LocalePattern] = &[
        // System language
        (&["system", "language"], "show system language", "locale",
         &["echo $LANG", "localectl status | grep LANG"]),
        (&["current", "language"], "show current language", "locale",
         &["echo $LANG"]),
        // Available languages
        (&["available", "languages"], "list available languages", "locale",
         &["locale -a | cut -d_ -f1 | sort -u"]),
        // Language packs
        (&["language", "packs"], "list installed language packs", "locale",
         &["pacman -Qs 'lang\\|l10n' 2>/dev/null | head -30"]),
        // Input methods
        (&["input", "methods"], "show input methods", "locale",
         &["echo 'Common: fcitx5, ibus, kcim'", "pacman -Qs 'fcitx\\|ibus' 2>/dev/null | head -20"]),
        (&["input", "method"], "show input method status", "locale",
         &["echo $GTK_IM_MODULE $QT_IM_MODULE $XMODIFIERS"]),
        // Fcitx
        (&["fcitx", "status"], "show fcitx status", "locale",
         &["fcitx5-diagnose 2>/dev/null | head -30 || echo 'fcitx5 not installed'"]),
        // IBus
        (&["ibus", "status"], "show ibus status", "locale",
         &["ibus list-engine 2>/dev/null | head -20 || echo 'ibus not installed'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Font patterns
fn match_fonts(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LocalePattern] = &[
        // Installed fonts
        (&["installed", "fonts"], "list installed fonts", "locale",
         &["fc-list | head -50"]),
        (&["list", "fonts"], "list fonts", "locale",
         &["fc-list | cut -d: -f2 | sort -u | head -50"]),
        (&["available", "fonts"], "list available fonts", "locale",
         &["fc-list | cut -d: -f2 | sort -u | head -50"]),
        // Font families
        (&["font", "families"], "list font families", "locale",
         &["fc-list : family | sort -u | head -50"]),
        // System fonts
        (&["system", "fonts"], "show system fonts", "locale",
         &["ls /usr/share/fonts/", "fc-list | wc -l"]),
        // User fonts
        (&["user", "fonts"], "show user fonts", "locale",
         &["ls ~/.local/share/fonts/ 2>/dev/null || echo 'No user fonts directory'"]),
        // Font cache
        (&["font", "cache"], "show font cache status", "locale",
         &["fc-cache -v 2>&1 | tail -5"]),
        // Monospace fonts
        (&["monospace", "fonts"], "list monospace fonts", "locale",
         &["fc-list :spacing=mono family | sort -u"]),
        // Font config
        (&["fontconfig"], "show fontconfig", "locale",
         &["fc-match", "cat /etc/fonts/fonts.conf 2>/dev/null | head -30"]),
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
    fn test_locale() {
        assert!(match_patterns("current locale").is_some());
        assert!(match_patterns("system locale").is_some());
        assert!(match_patterns("available locales").is_some());
    }

    #[test]
    fn test_keyboard() {
        assert!(match_patterns("keyboard layout").is_some());
        assert!(match_patterns("console keymap").is_some());
        assert!(match_patterns("available layouts").is_some());
    }

    #[test]
    fn test_language() {
        assert!(match_patterns("system language").is_some());
        assert!(match_patterns("input methods").is_some());
    }

    #[test]
    fn test_fonts() {
        assert!(match_patterns("installed fonts").is_some());
        assert!(match_patterns("font families").is_some());
        assert!(match_patterns("monospace fonts").is_some());
    }
}

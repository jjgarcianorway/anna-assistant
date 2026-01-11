//! Time, date, and timezone patterns.
//! v0.0.966: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a time-related DeepUnderstanding
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

type TimePattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match time/date-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_current_time(q)
        .or_else(|| match_timezone(q))
        .or_else(|| match_ntp(q))
        .or_else(|| match_hardware_clock(q))
        .or_else(|| match_calendar(q))
}

/// Current time/date patterns
fn match_current_time(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[TimePattern] = &[
        // Current time
        (&["current", "time"], "show current time", "time",
         &["date", "timedatectl"]),
        (&["what", "time"], "show what time it is", "time",
         &["date +%H:%M:%S", "date"]),
        (&["system", "time"], "show system time", "time",
         &["timedatectl", "date"]),
        // Current date
        (&["current", "date"], "show current date", "time",
         &["date +%Y-%m-%d", "date"]),
        (&["what", "date"], "show today's date", "time",
         &["date +%Y-%m-%d", "date"]),
        (&["today", "date"], "show today's date", "time",
         &["date +%Y-%m-%d"]),
        // Full datetime
        (&["date", "time"], "show date and time", "time",
         &["date", "timedatectl"]),
        (&["datetime"], "show datetime", "time",
         &["date", "timedatectl"]),
        // Uptime
        (&["system", "uptime"], "show system uptime", "time",
         &["uptime", "cat /proc/uptime"]),
        (&["how", "long", "running"], "show how long system running", "time",
         &["uptime -p", "uptime"]),
        (&["since", "boot"], "show time since boot", "time",
         &["uptime -s", "who -b"]),
        // ISO format
        (&["iso", "date"], "show ISO format date", "time",
         &["date -Iseconds", "date --iso-8601=seconds"]),
        (&["unix", "timestamp"], "show Unix timestamp", "time",
         &["date +%s"]),
        (&["epoch", "time"], "show epoch time", "time",
         &["date +%s"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Timezone patterns
fn match_timezone(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[TimePattern] = &[
        // Current timezone
        (&["current", "timezone"], "show current timezone", "time",
         &["timedatectl | grep 'Time zone'", "cat /etc/timezone 2>/dev/null || timedatectl"]),
        (&["my", "timezone"], "show my timezone", "time",
         &["timedatectl | grep 'Time zone'"]),
        (&["what", "timezone"], "show what timezone", "time",
         &["timedatectl | grep 'Time zone'"]),
        (&["system", "timezone"], "show system timezone", "time",
         &["timedatectl | grep 'Time zone'"]),
        // List timezones
        (&["list", "timezones"], "list available timezones", "time",
         &["timedatectl list-timezones | head -50"]),
        (&["available", "timezones"], "show available timezones", "time",
         &["timedatectl list-timezones | head -50"]),
        // UTC offset
        (&["utc", "offset"], "show UTC offset", "time",
         &["date +%z", "date +%:z"]),
        // Time in other timezone
        (&["time", "utc"], "show time in UTC", "time",
         &["date -u", "TZ=UTC date"]),
        // Timezone file
        (&["localtime"], "show localtime symlink", "time",
         &["ls -la /etc/localtime", "readlink /etc/localtime"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// NTP patterns
fn match_ntp(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[TimePattern] = &[
        // NTP status
        (&["ntp", "status"], "show NTP status", "time",
         &["timedatectl | grep -i ntp", "systemctl status systemd-timesyncd"]),
        (&["ntp", "sync"], "show NTP sync status", "time",
         &["timedatectl show-timesync 2>/dev/null || timedatectl"]),
        (&["time", "sync"], "show time sync status", "time",
         &["timedatectl | grep -E 'synchronized|NTP'"]),
        // Time synchronized
        (&["time", "synchronized"], "check if time is synchronized", "time",
         &["timedatectl | grep -i synchronized"]),
        (&["clock", "synchronized"], "check if clock is synchronized", "time",
         &["timedatectl | grep -i synchronized"]),
        // Timesyncd
        (&["timesyncd", "status"], "show timesyncd status", "time",
         &["systemctl status systemd-timesyncd", "timedatectl timesync-status 2>/dev/null"]),
        (&["timesyncd", "config"], "show timesyncd config", "time",
         &["cat /etc/systemd/timesyncd.conf"]),
        // Chrony
        (&["chrony", "status"], "show chrony status", "time",
         &["chronyc tracking 2>/dev/null || echo 'chrony not installed'"]),
        (&["chrony", "sources"], "show chrony sources", "time",
         &["chronyc sources 2>/dev/null || echo 'chrony not installed'"]),
        // NTP servers
        (&["ntp", "servers"], "show NTP servers", "time",
         &["cat /etc/systemd/timesyncd.conf | grep -v '^#'", "chronyc sources 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Hardware clock patterns
fn match_hardware_clock(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[TimePattern] = &[
        // Hardware clock
        (&["hardware", "clock"], "show hardware clock", "time",
         &["sudo hwclock --show 2>/dev/null || timedatectl | grep 'RTC'"]),
        (&["rtc", "time"], "show RTC time", "time",
         &["timedatectl | grep 'RTC time'", "cat /sys/class/rtc/rtc0/time 2>/dev/null"]),
        (&["bios", "time"], "show BIOS time", "time",
         &["sudo hwclock --show 2>/dev/null || timedatectl | grep 'RTC'"]),
        // Local vs UTC
        (&["rtc", "utc"], "check if RTC is UTC", "time",
         &["timedatectl | grep 'RTC in local'"]),
        (&["rtc", "local"], "check if RTC is local time", "time",
         &["timedatectl | grep 'RTC in local'"]),
        // Clock drift
        (&["clock", "drift"], "check clock drift", "time",
         &["chronyc tracking 2>/dev/null | grep -E 'System time|Root delay' || echo 'Install chrony for drift info'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Calendar patterns
fn match_calendar(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[TimePattern] = &[
        // Calendar
        (&["this", "month"], "show this month calendar", "time",
         &["cal"]),
        (&["calendar"], "show calendar", "time",
         &["cal", "cal -3"]),
        (&["this", "year"], "show this year calendar", "time",
         &["cal -y"]),
        // Week number
        (&["week", "number"], "show week number", "time",
         &["date +%V", "date +'Week %V'"]),
        (&["which", "week"], "show which week", "time",
         &["date +%V"]),
        // Day of year
        (&["day", "year"], "show day of year", "time",
         &["date +%j", "date +'Day %j of %Y'"]),
        // Leap year
        (&["leap", "year"], "check if leap year", "time",
         &["python3 -c 'import calendar; y=__import__(\"datetime\").date.today().year; print(f\"{y} is leap year: {calendar.isleap(y)}\")' 2>/dev/null || date +'%Y'"]),
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
    fn test_current_time() {
        assert!(match_patterns("current time").is_some());
        assert!(match_patterns("what time").is_some());
        assert!(match_patterns("system uptime").is_some());
        assert!(match_patterns("unix timestamp").is_some());
    }

    #[test]
    fn test_timezone() {
        assert!(match_patterns("current timezone").is_some());
        assert!(match_patterns("my timezone").is_some());
        assert!(match_patterns("list timezones").is_some());
        assert!(match_patterns("time utc").is_some());
    }

    #[test]
    fn test_ntp() {
        assert!(match_patterns("ntp status").is_some());
        assert!(match_patterns("time sync").is_some());
        assert!(match_patterns("ntp servers").is_some());
        assert!(match_patterns("chrony status").is_some());
    }

    #[test]
    fn test_hardware_clock() {
        assert!(match_patterns("hardware clock").is_some());
        assert!(match_patterns("rtc time").is_some());
        assert!(match_patterns("bios time").is_some());
    }

    #[test]
    fn test_calendar() {
        assert!(match_patterns("calendar").is_some());
        assert!(match_patterns("this month").is_some());
        assert!(match_patterns("week number").is_some());
    }
}

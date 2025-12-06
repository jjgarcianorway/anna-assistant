//! Time formatting utilities for annactl (v0.0.85).
//!
//! Provides human-readable date and tenure formatting.

/// Format a unix timestamp as a readable date (e.g., "Dec 6, 2025")
pub fn format_date(ts: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let datetime = UNIX_EPOCH + Duration::from_secs(ts);
    if let Ok(since_epoch) = datetime.duration_since(UNIX_EPOCH) {
        let days_since_epoch = since_epoch.as_secs() / 86400;
        let (year, month, day) = days_to_date(days_since_epoch);
        let month_name = match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "???",
        };
        format!("{} {}, {}", month_name, day, year)
    } else {
        "unknown".to_string()
    }
}

/// Convert days since epoch to (year, month, day)
fn days_to_date(days: u64) -> (u32, u32, u32) {
    let mut remaining = days as i64;
    let mut year = 1970u32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let days_in_months: [i64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for days_in_month in days_in_months.iter() {
        if remaining < *days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    (year, month, (remaining + 1) as u32)
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Format tenure as human-readable duration from a timestamp
pub fn format_tenure(first_ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let elapsed = now.saturating_sub(first_ts);
    format_duration_days(elapsed / 86400)
}

/// Format a duration given in days
pub fn format_duration_days(days: u64) -> String {
    if days == 0 {
        "today".to_string()
    } else if days == 1 {
        "1 day".to_string()
    } else if days < 7 {
        format!("{} days", days)
    } else if days < 30 {
        let weeks = days / 7;
        if weeks == 1 {
            "1 week".to_string()
        } else {
            format!("{} weeks", weeks)
        }
    } else if days < 365 {
        let months = days / 30;
        if months == 1 {
            "1 month".to_string()
        } else {
            format!("{} months", months)
        }
    } else {
        let years = days / 365;
        let remaining_months = (days % 365) / 30;
        if years == 1 {
            if remaining_months > 0 {
                format!("1 year, {} months", remaining_months)
            } else {
                "1 year".to_string()
            }
        } else if remaining_months > 0 {
            format!("{} years, {} months", years, remaining_months)
        } else {
            format!("{} years", years)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration_days(0), "today");
        assert_eq!(format_duration_days(1), "1 day");
        assert_eq!(format_duration_days(5), "5 days");
        assert_eq!(format_duration_days(7), "1 week");
        assert_eq!(format_duration_days(14), "2 weeks");
        assert_eq!(format_duration_days(30), "1 month");
        assert_eq!(format_duration_days(60), "2 months");
        assert_eq!(format_duration_days(365), "1 year");
        assert_eq!(format_duration_days(400), "1 year, 1 months");
        assert_eq!(format_duration_days(730), "2 years");
    }

    #[test]
    fn test_days_to_date() {
        // Jan 1, 1970
        assert_eq!(days_to_date(0), (1970, 1, 1));
        // Feb 1, 1970
        assert_eq!(days_to_date(31), (1970, 2, 1));
        // Jan 1, 1971
        assert_eq!(days_to_date(365), (1971, 1, 1));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(1970));
        assert!(is_leap_year(1972));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
    }
}

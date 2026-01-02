//! Natural language parsing for alarm requests.

use super::types::{AlarmCondition, AlarmSchedule, UserAlarm, Weekday};

/// Parse natural language alarm request
pub fn parse_alarm_request(input: &str) -> Option<UserAlarm> {
    let lower = input.to_lowercase();

    // Extract topic (after "about")
    let topic = if let Some(pos) = lower.find("about ") {
        input[pos + 6..].trim().to_string()
    } else {
        "system status".to_string()
    };

    // Try to parse schedule
    let schedule = if lower.contains("every monday") {
        Some(parse_weekly(&lower, Weekday::Monday))
    } else if lower.contains("every tuesday") {
        Some(parse_weekly(&lower, Weekday::Tuesday))
    } else if lower.contains("every wednesday") {
        Some(parse_weekly(&lower, Weekday::Wednesday))
    } else if lower.contains("every thursday") {
        Some(parse_weekly(&lower, Weekday::Thursday))
    } else if lower.contains("every friday") {
        Some(parse_weekly(&lower, Weekday::Friday))
    } else if lower.contains("every saturday") {
        Some(parse_weekly(&lower, Weekday::Saturday))
    } else if lower.contains("every sunday") {
        Some(parse_weekly(&lower, Weekday::Sunday))
    } else if lower.contains("daily") || lower.contains("every day") {
        Some(parse_daily(&lower))
    } else if lower.contains("disk") && (lower.contains("above") || lower.contains(">")) {
        Some(parse_disk_condition(&lower))
    } else if lower.contains("memory") && (lower.contains("above") || lower.contains(">")) {
        Some(parse_memory_condition(&lower))
    } else if lower.contains("service") && lower.contains("fail") {
        Some(parse_service_condition(&lower))
    } else {
        None
    };

    schedule.map(|s| UserAlarm::new(&format!("Alarm: {}", &topic), &topic, s))
}

fn parse_weekly(input: &str, day: Weekday) -> AlarmSchedule {
    let (hour, minute) = parse_time_from_input(input);
    AlarmSchedule::Weekly { day, hour, minute }
}

fn parse_daily(input: &str) -> AlarmSchedule {
    let (hour, minute) = parse_time_from_input(input);
    AlarmSchedule::Daily { hour, minute }
}

pub fn parse_time_from_input(input: &str) -> (u8, u8) {
    // Look for "at X" pattern
    if let Some(pos) = input.find("at ") {
        let after_at = &input[pos + 3..];
        // Try to parse time like "9", "9:00", "09:00"
        let time_part: String = after_at
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ':')
            .collect();

        if let Some(colon_pos) = time_part.find(':') {
            let hour: u8 = time_part[..colon_pos].parse().unwrap_or(9);
            let minute: u8 = time_part[colon_pos + 1..].parse().unwrap_or(0);
            return (hour, minute);
        } else if let Ok(hour) = time_part.parse::<u8>() {
            return (hour, 0);
        }
    }
    (9, 0) // Default to 9:00
}

fn parse_disk_condition(input: &str) -> AlarmSchedule {
    let threshold = extract_percent(input).unwrap_or(90);
    AlarmSchedule::Conditional {
        condition: AlarmCondition::DiskAbove {
            threshold_percent: threshold,
            path: None,
        },
    }
}

fn parse_memory_condition(input: &str) -> AlarmSchedule {
    let threshold = extract_percent(input).unwrap_or(90);
    AlarmSchedule::Conditional {
        condition: AlarmCondition::MemoryAbove {
            threshold_percent: threshold,
        },
    }
}

fn parse_service_condition(input: &str) -> AlarmSchedule {
    // Check if specific service mentioned
    if input.contains("any ") {
        AlarmSchedule::Conditional {
            condition: AlarmCondition::AnyServiceFailed,
        }
    } else {
        // Try to extract service name (after "service")
        let service = "".to_string();
        AlarmSchedule::Conditional {
            condition: AlarmCondition::ServiceFailed { service },
        }
    }
}

fn extract_percent(input: &str) -> Option<u8> {
    // Find patterns like "90%", "90 %", "> 90"
    let re_patterns = [
        r"(\d+)\s*%",
        r">\s*(\d+)",
        r"above\s*(\d+)",
    ];

    for pattern in re_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(input) {
                if let Some(num) = caps.get(1) {
                    if let Ok(n) = num.as_str().parse::<u8>() {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

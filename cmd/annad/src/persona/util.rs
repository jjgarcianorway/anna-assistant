use anyhow::Result;
use time::{Date, OffsetDateTime, UtcOffset};

pub fn today_local() -> Result<Date> {
    match OffsetDateTime::now_local() {
        Ok(now) => Ok(now.date()),
        Err(_) => {
            let now = OffsetDateTime::now_utc();
            if let Ok(offset) = UtcOffset::current_local_offset() {
                Ok(now.to_offset(offset).date())
            } else {
                Ok(now.date())
            }
        }
    }
}

pub fn format_date(date: &Date) -> String {
    date.to_string()
}

pub fn previous_local_date(date: &Date) -> Option<Date> {
    date.previous_day()
}

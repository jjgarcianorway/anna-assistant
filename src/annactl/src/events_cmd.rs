//! Events command for annactl v0.12.9
//!
//! View system events from daemon log

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Event from daemon
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    Error {
        code: i32,
        msg: String,
        timestamp: i64,
    },
    Warning {
        msg: String,
        timestamp: i64,
    },
    Change {
        key: String,
        old: JsonValue,
        new: JsonValue,
        timestamp: i64,
    },
    Advice {
        key: String,
        msg: String,
        timestamp: i64,
    },
}

impl Event {
    fn timestamp(&self) -> i64 {
        match self {
            Event::Error { timestamp, .. } => *timestamp,
            Event::Warning { timestamp, .. } => *timestamp,
            Event::Change { timestamp, .. } => *timestamp,
            Event::Advice { timestamp, .. } => *timestamp,
        }
    }

    fn event_type(&self) -> &str {
        match self {
            Event::Error { .. } => "error",
            Event::Warning { .. } => "warning",
            Event::Change { .. } => "change",
            Event::Advice { .. } => "advice",
        }
    }
}

/// Display events in human-friendly format
pub fn display_events(events: &[Event]) -> Result<()> {
    if events.is_empty() {
        println!("No events found");
        return Ok(());
    }

    let red = "\x1b[31m";
    let yellow = "\x1b[33m";
    let blue = "\x1b[34m";
    let green = "\x1b[32m";
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";

    for event in events {
        let (emoji, color) = match event.event_type() {
            "error" => ("❌", red),
            "warning" => ("⚠️", yellow),
            "change" => ("↻", blue),
            "advice" => ("ℹ️", green),
            _ => ("•", dim),
        };

        // Format timestamp
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(event.timestamp(), 0)
            .unwrap_or_default();
        let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();

        print!("{}{} {} {}", color, emoji, time_str, reset);

        match event {
            Event::Error { code, msg, .. } => {
                println!(" {}Error {}{}: {}", red, code, reset, msg);
            }
            Event::Warning { msg, .. } => {
                println!(" {}Warning{}: {}", yellow, reset, msg);
            }
            Event::Change { key, old, new, .. } => {
                println!(" {}Changed{}: {} = {} → {}", blue, reset, key, old, new);
            }
            Event::Advice { key, msg, .. } => {
                println!(" {}Advice{} [{}]: {}", green, reset, key, msg);
            }
        }
    }

    println!();

    Ok(())
}

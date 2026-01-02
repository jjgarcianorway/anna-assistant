//! Alarm scheduler implementation

use super::types::{AlarmFrequency, AlarmRecord, AlarmScheduler, AlarmStatus, DayOfWeek};

impl AlarmScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an alarm
    pub fn add(&mut self, alarm: AlarmRecord) {
        *self.by_frequency.entry(alarm.frequency.name().to_string()).or_insert(0) += 1;
        *self.by_topic.entry(alarm.topic.clone()).or_insert(0) += 1;
        self.alarms.push(alarm);
    }

    /// Get alarm by ID
    pub fn get(&self, id: &str) -> Option<&AlarmRecord> {
        self.alarms.iter().find(|a| a.id == id)
    }

    /// Trigger an alarm
    pub fn trigger(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].last_triggered = Some(timestamp);
            self.alarms[idx].trigger_count += 1;
            self.alarms[idx].status = AlarmStatus::Triggered;
            self.total_triggers += 1;

            // Reset status to Active for recurring
            if self.alarms[idx].frequency != AlarmFrequency::Once {
                self.alarms[idx].status = AlarmStatus::Active;
            } else {
                self.alarms[idx].status = AlarmStatus::Expired;
            }
            true
        } else {
            false
        }
    }

    /// Pause an alarm
    pub fn pause(&mut self, id: &str) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].status = AlarmStatus::Paused;
            true
        } else {
            false
        }
    }

    /// Resume an alarm
    pub fn resume(&mut self, id: &str) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].status = AlarmStatus::Active;
            true
        } else {
            false
        }
    }

    /// Cancel an alarm
    pub fn cancel(&mut self, id: &str) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].status = AlarmStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// Get active alarms
    pub fn active(&self) -> Vec<&AlarmRecord> {
        self.alarms.iter().filter(|a| a.status == AlarmStatus::Active).collect()
    }

    /// Get alarms due at given time
    pub fn due_at(&self, hour: u8, minute: u8, day: Option<DayOfWeek>) -> Vec<&AlarmRecord> {
        self.alarms
            .iter()
            .filter(|a| {
                a.status == AlarmStatus::Active
                    && a.hour == hour
                    && a.minute == minute
                    && (a.day_of_week.is_none() || a.day_of_week == day)
            })
            .collect()
    }

    /// Get alarms by topic
    pub fn by_alarm_topic(&self, topic: &str) -> Vec<&AlarmRecord> {
        self.alarms.iter().filter(|a| a.topic == topic).collect()
    }

    /// Get alarms by frequency
    pub fn by_alarm_frequency(&self, freq: AlarmFrequency) -> Vec<&AlarmRecord> {
        self.alarms.iter().filter(|a| a.frequency == freq).collect()
    }

    /// Total alarm count
    pub fn total_count(&self) -> usize {
        self.alarms.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.alarms.iter().filter(|a| a.status == AlarmStatus::Active).count()
    }

    /// Most common topic
    pub fn most_common_topic(&self) -> Option<(&str, u64)> {
        self.by_topic
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

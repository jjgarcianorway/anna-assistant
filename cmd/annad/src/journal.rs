use anyhow::*;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct JRec {
    MESSAGE: Option<String>,
    SYSLOG_IDENTIFIER: Option<String>,
    UNIT: Option<String>,
    _PID: Option<i32>,
    _SOURCE_REALTIME_TIMESTAMP: Option<String>,
}

/// Sliding-window counter for sshd failures.
pub struct FailedCounter {
    times: std::collections::VecDeque<i64>, // unix seconds
    threshold: usize,
    window: i64,
}
impl FailedCounter {
    pub fn new(threshold: usize, window_secs: i64) -> Self {
        Self { times: Default::default(), threshold, window: window_secs }
    }
    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs() as i64
    }
    pub fn register_failure(&mut self) -> bool {
        let t = Self::now();
        self.times.push_back(t);
        while let Some(&old) = self.times.front() {
            if t - old > self.window { self.times.pop_front(); } else { break; }
        }
        self.times.len() >= self.threshold
    }
}

/// Follow journald and call `on_failure` when sshd brute-force is detected.
pub fn follow_journal(mut on_failure: impl FnMut() -> ()) -> Result<()> {
    let mut child = Command::new("journalctl")
        .args(["-f", "-o", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn journalctl")?;

    let out = child.stdout.take().context("take stdout")?;
    let reader = BufReader::new(out);

    let mut counter = FailedCounter::new(5, 10 * 60); // 5 fails in 10 minutes

    for line_res in reader.lines() {
        let line = match line_res.ok() {
            Some(l) => l,
            None => break,
        };

        if let Some(rec) = serde_json::from_str::<JRec>(&line).ok() {
            let id = rec.SYSLOG_IDENTIFIER.as_deref().unwrap_or("");
            if id == "sshd" {
                if let Some(msg) = rec.MESSAGE.as_deref() {
                    if msg.contains("Failed password for") && counter.register_failure() {
                        on_failure();
                    }
                }
            }
        }
    }
    Ok(())
}

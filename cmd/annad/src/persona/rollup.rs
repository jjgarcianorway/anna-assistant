use crate::persona::fs;
use crate::persona::util;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use time::{format_description::well_known::Rfc3339, Date, OffsetDateTime};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct RollupResult {
    pub date: String,
    pub total: u64,
    pub by_cat: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct SampleRecord {
    cat: String,
    exe: String,
}

#[derive(Default)]
struct Aggregation {
    total: u64,
    by_cat: HashMap<String, u64>,
    by_exec: HashMap<String, u64>,
}

impl Aggregation {
    fn add(&mut self, cat: String, exe: String) {
        self.total += 1;
        *self.by_cat.entry(cat).or_insert(0) += 1;
        *self.by_exec.entry(exe).or_insert(0) += 1;
    }

    fn top_execs(&self) -> Vec<(String, u64)> {
        let mut pairs: Vec<(String, u64)> =
            self.by_exec.iter().map(|(k, v)| (k.clone(), *v)).collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        pairs.truncate(10);
        pairs
    }
}

#[derive(Serialize)]
struct RollupFile {
    date: String,
    total: u64,
    by_cat: HashMap<String, u64>,
    top_execs: Vec<(String, u64)>,
    generated_at: String,
}

pub fn catch_up() -> Result<()> {
    let today = util::today_local()?;
    if let Some(yesterday) = util::previous_local_date(&today) {
        let date_str = util::format_date(&yesterday);
        let rollup_path = fs::rollup_path(&date_str);
        if !rollup_path.exists() {
            let samples = fs::samples_path(&date_str);
            if samples.exists() {
                let _ = generate_for_date(&yesterday)?;
            }
        }
    }
    Ok(())
}

pub fn generate_for_date(date: &Date) -> Result<Option<RollupResult>> {
    let date_str = util::format_date(date);
    let rollup_path = fs::rollup_path(&date_str);
    if rollup_path.exists() {
        return Ok(None);
    }
    let sample_path = fs::samples_path(&date_str);
    if !sample_path.exists() {
        return Ok(None);
    }
    let file = File::open(&sample_path)
        .with_context(|| format!("open samples {}", sample_path.display()))?;
    let reader = BufReader::new(file);
    let mut agg = Aggregation::default();
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                debug!(target: "annad", "persona rollup skip line read error: {e}");
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<SampleRecord>(&line) {
            Ok(record) => {
                let cat = record.cat.to_lowercase();
                let exe = record.exe.to_lowercase();
                agg.add(cat, exe);
            }
            Err(e) => debug!(target: "annad", "persona rollup skip bad json: {e}"),
        }
    }

    let generated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let rollup_file = RollupFile {
        date: date_str.clone(),
        total: agg.total,
        by_cat: agg.by_cat.clone(),
        top_execs: agg.top_execs(),
        generated_at,
    };
    let payload = serde_json::to_vec_pretty(&rollup_file)?;
    fs::write_atomic(&rollup_path, &payload)?;

    let result = RollupResult {
        date: date_str.clone(),
        total: agg.total,
        by_cat: agg.by_cat,
    };
    log_rollup(&result);
    Ok(Some(result))
}

fn log_rollup(res: &RollupResult) {
    let editor = res.by_cat.get("editor").copied().unwrap_or(0);
    let browser = res.by_cat.get("browser").copied().unwrap_or(0);
    info!(
        target: "annad",
        "persona rollup {} total={} editor={} browser={}",
        res.date,
        res.total,
        editor,
        browser
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregation_top_execs() {
        let mut agg = Aggregation::default();
        agg.add("editor".into(), "nvim".into());
        agg.add("editor".into(), "nvim".into());
        agg.add("browser".into(), "firefox".into());
        agg.add("browser".into(), "firefox".into());
        agg.add("browser".into(), "firefox".into());
        agg.add("terminal".into(), "alacritty".into());
        let tops = agg.top_execs();
        assert_eq!(tops[0], ("firefox".into(), 3));
        assert_eq!(tops[1], ("nvim".into(), 2));
        assert_eq!(tops[2], ("alacritty".into(), 1));
    }
}

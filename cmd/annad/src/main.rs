mod journal;
mod plan;
mod sysinfo;

use anyhow::Result;
use std::{env, fs, path::PathBuf, thread, time::Duration};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

fn ensure_dirs() -> Result<(PathBuf, PathBuf)> {
    let root = PathBuf::from("/var/lib/anna");
    let plans = root.join("plans");
    fs::create_dir_all(&plans)?;
    Ok((root, plans))
}

fn hb_interval() -> Duration {
    let secs = env::var("ANNA_HEARTBEAT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    Duration::from_secs(secs)
}

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_level(true)
        .without_time()
        .init();
    let (root, plans_root) = ensure_dirs()?;

    // Collect and persist system snapshot at startup
    let snapshot = sysinfo::collect();
    let system_json_path = root.join("system.json");
    if let Ok(js) = serde_json::to_string_pretty(&snapshot) {
        if let Err(e) = fs::write(&system_json_path, js) {
            error!(target: "annad", "failed writing system snapshot: {e:?}");
        } else {
            info!(target: "annad", "system snapshot written: {}", system_json_path.display());
        }
    }

    info!(target: "annad", "starting {}", env!("CARGO_PKG_VERSION"));

    // Heartbeat loop task
    let hb = tokio::spawn(async move {
        loop {
            tokio::time::sleep(hb_interval()).await;
            info!(target: "annad", "heartbeat");
        }
    });

    // Journald follower in a blocking thread
    let plans_root_clone = plans_root.clone();
    let jf = thread::spawn(move || {
        let _ = journal::follow_journal(|| {
            match plan::suggest_harden_ssh(&plans_root_clone) {
                Ok(pp) => info!(target: "annad", "suggested plan at {}", pp.dir.display()),
                Err(e) => error!(target: "annad", "plan write failed: {e:?}"),
            }
        });
    });

    // Keep daemon alive: await the heartbeat (never returns), keep join handles tidy if it ever does.
    let _ = hb.await;
    let _ = jf.join();

    Ok(())
}

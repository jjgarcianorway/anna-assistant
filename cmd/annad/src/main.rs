mod advice;
mod config;
mod journal;
mod notifications;
mod paths;
mod persona;
mod plan;
mod policy;
mod quickscan;
mod rpc;
mod signals;
mod sysinfo;

use anyhow::Result;
use std::{env, fs, sync::Arc, thread, time::Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

// Removed - now handled by paths::Paths::ensure_dirs()

fn hb_interval() -> Duration {
    let secs = env::var("ANNA_HEARTBEAT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    Duration::from_secs(secs)
}

// Removed - now handled by paths module

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

    // Detect mode FIRST, before any file operations
    let mode = paths::Mode::detect();
    info!(target: "annad", "starting {} mode={}", env!("CARGO_PKG_VERSION"), mode.as_str());

    // Initialize paths for the detected mode
    let paths = Arc::new(paths::Paths::new(mode)?);
    info!(target: "annad", "data_root={}", paths.data_root().display());
    info!(target: "annad", "socket={}", paths.socket().display());

    // Ensure all required directories exist
    paths.ensure_dirs()?;

    // Load config (mode-aware)
    let cfg = Arc::new(config::load(paths.config_file())?);

    // Initialize subsystems with mode-aware paths
    advice::fs::set_advice_dir(paths.advice_dir());
    advice::init(&cfg.advice)?;
    persona::fs::set_persona_dir(paths.persona_dir());
    persona::store::set_persona_paths(&paths.persona_dir(), paths.config_root());
    signals::set_signals_dir(paths.signals_dir());
    let persona_state = persona::init(cfg.as_ref())?;
    if cfg.persona.enabled {
        info!(
            target: "annad",
            "persona={} source={} confidence={:.2}",
            persona_state.persona.as_str(),
            persona_state.source.as_str(),
            persona_state.confidence
        );
        persona::start_background_tasks(cfg.as_ref())?;
        let _ = persona::maybe_update_current(cfg.as_ref())?;
    } else {
        info!(
            target: "annad",
            "persona subsystem disabled via config; current persona={} source={} confidence={:.2}",
            persona_state.persona.as_str(),
            persona_state.source.as_str(),
            persona_state.confidence
        );
    }

    advice::start_background(cfg.advice.clone())?;

    quickscan::set_quickscan_dir(paths.quickscan_dir());
    if cfg.quickscan.enable {
        if let Err(err) = quickscan::prune_report_cache(cfg.quickscan.retain_reports) {
            warn!(target: "annad", "quickscan cache prune failed: {err:?}");
        }
        let cfg_clone = Arc::clone(&cfg);
        tokio::spawn(async move {
            match quickscan::run_initial_if_needed(cfg_clone).await {
                Ok(Some(_)) => {}
                Ok(None) => {}
                Err(err) => {
                    warn!(target: "annad", "quickscan initial run failed: {err:?}");
                }
            }
        });
    }

    // Collect and persist system snapshot at startup
    let snapshot = sysinfo::collect();
    let system_json_path = paths.system_snapshot();
    if let Ok(js) = serde_json::to_string_pretty(&snapshot) {
        if let Err(e) = fs::write(&system_json_path, js) {
            error!(target: "annad", "failed writing system snapshot: {e:?}");
        } else {
            info!(target: "annad", "system snapshot written: {}", system_json_path.display());
        }
    }

    // Start RPC server in background thread
    let rpc_config = Arc::clone(&cfg);
    let socket_path = paths.socket();
    let rpc_server = rpc::RpcServer::new(socket_path, rpc_config);
    thread::spawn(move || {
        if let Err(e) = rpc_server.start() {
            error!(target: "annad", "rpc server error: {}", e);
        }
    });

    // Heartbeat loop task
    let hb = tokio::spawn(async move {
        loop {
            tokio::time::sleep(hb_interval()).await;
            info!(target: "annad", "heartbeat");
        }
    });

    // Journald follower in a blocking thread
    let plans_dir = paths.plans_dir();
    let jf = thread::spawn(move || {
        let _ = journal::follow_journal(|| match plan::suggest_harden_ssh(&plans_dir) {
            Ok(pp) => info!(target: "annad", "suggested plan at {}", pp.dir.display()),
            Err(e) => error!(target: "annad", "plan write failed: {e:?}"),
        });
    });

    // Keep daemon alive: await the heartbeat (never returns), keep join handles tidy if it ever does.
    let _ = hb.await;
    let _ = jf.join();

    Ok(())
}

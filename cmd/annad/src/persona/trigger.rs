use super::store::{Store, TriggerSnapshot};
use crate::config::{Config, TriggerConfig};
use crate::signals::{self, SignalDelta, SignalsCtx};
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use time::{Duration as TimeDuration, OffsetDateTime};
use tokio::time::{self as tokio_time, Duration};
use tracing::{info, warn};

#[derive(Default)]
struct TriggerState {
    last_fire: Option<OffsetDateTime>,
    last_log: Option<OffsetDateTime>,
}

struct TriggerOutcome {
    should_log: bool,
    should_recompute: bool,
    debounced: bool,
}

pub fn spawn(cfg: Config) -> Result<()> {
    if !cfg.persona.trigger.enable {
        return Ok(());
    }
    let trigger_cfg = cfg.persona.trigger.clone();
    let signals_cfg = cfg.signals.clone();
    let persona_cfg = Arc::new(cfg);
    let store = Arc::new(Store::new()?);
    let signals_ctx = Arc::new(SignalsCtx::new()?);
    let state = Arc::new(Mutex::new(TriggerState::default()));

    tokio::spawn(async move {
        let mut interval = tokio_time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let ctx = Arc::clone(&signals_ctx);
            let signals_cfg_clone = signals_cfg.clone();
            let delta =
                tokio::task::spawn_blocking(move || signals::collect(&ctx, &signals_cfg_clone))
                    .await
                    .unwrap_or_else(|err| Err(anyhow!("signal collection join error: {err}")));
            let delta = match delta {
                Ok(delta) => delta,
                Err(err) => {
                    warn!(target: "annad", "signal collection failed: {err:?}");
                    continue;
                }
            };
            if delta.is_empty() {
                continue;
            }
            if !exceeds_threshold(&delta, &trigger_cfg) {
                continue;
            }
            handle_trigger(&state, &trigger_cfg, &persona_cfg, &store, &delta).await;
        }
    });

    Ok(())
}

async fn handle_trigger(
    state: &Arc<Mutex<TriggerState>>,
    cfg: &TriggerConfig,
    persona_cfg: &Arc<Config>,
    store: &Arc<Store>,
    delta: &SignalDelta,
) {
    let now = OffsetDateTime::now_utc();
    let debounce = TimeDuration::seconds(cfg.debounce_secs as i64);
    let mut guard = state.lock().expect("trigger state poisoned");
    let outcome = evaluate_state(&mut guard, now, debounce);
    drop(guard);

    if outcome.should_log {
        let snapshot = TriggerSnapshot {
            time: now
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into()),
            pkg_churn: delta.pkg_churn,
            shell_lines: delta.shell_lines,
            browser_navs: delta.browser_navs,
            debounced: outcome.debounced,
        };
        if let Err(err) = store.write_last_trigger(&snapshot) {
            warn!(target: "annad", "failed to record trigger snapshot: {err:?}");
        }
        if outcome.debounced {
            info!(
                target: "annad",
                "persona trigger: pkg_churn={} shell={} browser={} (debounced {}s)",
                delta.pkg_churn,
                delta.shell_lines,
                delta.browser_navs,
                cfg.debounce_secs
            );
        } else {
            info!(
                target: "annad",
                "persona trigger: pkg_churn={} shell={} browser={}",
                delta.pkg_churn,
                delta.shell_lines,
                delta.browser_navs
            );
        }
    }

    if outcome.should_recompute {
        if let Err(err) = super::infer::maybe_update_current(persona_cfg.as_ref()) {
            warn!(target: "annad", "persona inference after trigger failed: {err:?}");
        }
    }
}

fn evaluate_state(
    state: &mut TriggerState,
    now: OffsetDateTime,
    debounce: TimeDuration,
) -> TriggerOutcome {
    let mut outcome = TriggerOutcome {
        should_log: false,
        should_recompute: false,
        debounced: false,
    };
    match state.last_fire {
        None => {
            state.last_fire = Some(now);
            state.last_log = Some(now);
            outcome.should_recompute = true;
            outcome.should_log = true;
        }
        Some(last_fire) => {
            if now - last_fire >= debounce {
                state.last_fire = Some(now);
                state.last_log = Some(now);
                outcome.should_recompute = true;
                outcome.should_log = true;
            } else {
                outcome.debounced = true;
                let should_log = match state.last_log {
                    None => true,
                    Some(last_log) => now - last_log >= debounce,
                };
                if should_log {
                    state.last_log = Some(now);
                    outcome.should_log = true;
                }
            }
        }
    }
    outcome
}

fn exceeds_threshold(delta: &SignalDelta, cfg: &TriggerConfig) -> bool {
    delta.pkg_churn >= cfg.pkg_churn_threshold
        || delta.shell_lines >= cfg.shell_hist_threshold
        || delta.browser_navs >= cfg.browser_nav_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_coalesces_events() {
        let cfg = TriggerConfig {
            enable: true,
            debounce_secs: 300,
            pkg_churn_threshold: 1,
            shell_hist_threshold: 1,
            browser_nav_threshold: 1,
        };
        let now = OffsetDateTime::now_utc();
        let debounce = TimeDuration::seconds(cfg.debounce_secs as i64);
        let mut state = TriggerState::default();

        let first = evaluate_state(&mut state, now, debounce);
        assert!(first.should_log);
        assert!(first.should_recompute);
        assert!(!first.debounced);

        let second = evaluate_state(&mut state, now + TimeDuration::seconds(60), debounce);
        assert!(second.debounced);
        assert!(!second.should_log);
        assert!(!second.should_recompute);

        let third = evaluate_state(&mut state, now + TimeDuration::seconds(360), debounce);
        assert!(third.should_recompute);
        assert!(third.should_log);
        assert!(!third.debounced);
    }
}

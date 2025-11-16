use anyhow::Result;
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;

use super::db::ContextDb;

fn get_db() -> Result<Arc<ContextDb>> {
    super::db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))
}

pub async fn record_timeline_event(
    ts: Option<String>,
    kind: &str,
    from_version: Option<&str>,
    to_version: Option<&str>,
    details: Option<&str>,
    outcome: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let kind = kind.to_string();
    let ts = ts.clone();
    let from_version = from_version.map(|s| s.to_string());
    let to_version = to_version.map(|s| s.to_string());
    let details = details.map(|s| s.to_string());
    let outcome = outcome.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO timeline_events (ts, kind, from_version, to_version, details, outcome)
             VALUES (COALESCE(?1, CURRENT_TIMESTAMP), ?2, ?3, ?4, ?5, ?6)",
            params![ts, kind, from_version, to_version, details, outcome],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_boot_session(
    boot_id: &str,
    ts_start: &str,
    ts_end: Option<&str>,
    goal: Option<&str>,
    time_to_goal_ms: Option<i64>,
    degraded: Option<bool>,
    fsck_ran: Option<bool>,
    fsck_duration_ms: Option<i64>,
    shutdown_duration_ms: Option<i64>,
    early_kernel_errors_count: Option<i64>,
    boot_health_score: Option<i64>,
) -> Result<i64> {
    let db = get_db()?;
    let boot_id = boot_id.to_string();
    let ts_start = ts_start.to_string();
    let ts_end = ts_end.map(|s| s.to_string());
    let goal = goal.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT OR REPLACE INTO boot_sessions
             (boot_id, ts_start, ts_end, goal, time_to_goal_ms, degraded, fsck_ran, fsck_duration_ms,
              shutdown_duration_ms, early_kernel_errors_count, boot_health_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                boot_id,
                ts_start,
                ts_end,
                goal,
                time_to_goal_ms,
                degraded.map(|b| if b { 1 } else { 0 }),
                fsck_ran.map(|b| if b { 1 } else { 0 }),
                fsck_duration_ms,
                shutdown_duration_ms,
                early_kernel_errors_count,
                boot_health_score
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_boot_unit(
    boot_id: &str,
    unit: &str,
    duration_ms: Option<i64>,
    state: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let boot_id = boot_id.to_string();
    let unit = unit.to_string();
    let state = state.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO boot_unit_slowlog (boot_id, unit, duration_ms, state)
             VALUES (?1, ?2, ?3, ?4)",
            params![boot_id, unit, duration_ms, state],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_cpu_window(
    window_start: &str,
    window_end: &str,
    avg_util_per_core_json: Option<&str>,
    peak_util_per_core_json: Option<&str>,
    idle_background_load: Option<f64>,
    throttling_events: Option<i64>,
    spikes_over_100pct: Option<i64>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let window_end = window_end.to_string();
    let avg_util = avg_util_per_core_json.map(|s| s.to_string());
    let peak_util = peak_util_per_core_json.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO cpu_windows (window_start, window_end, avg_util_per_core, peak_util_per_core,
                                      idle_background_load, throttling_events, spikes_over_100pct)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![window_start, window_end, avg_util, peak_util, idle_background_load, throttling_events, spikes_over_100pct],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_cpu_top_process(
    window_start: &str,
    process_name: &str,
    cpu_time_seconds: f64,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let process_name = process_name.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO cpu_top_processes (window_start, process_name, cpu_time_seconds)
             VALUES (?1, ?2, ?3)",
            params![window_start, process_name, cpu_time_seconds],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_mem_window(
    window_start: &str,
    avg_ram_mb: Option<f64>,
    peak_ram_mb: Option<f64>,
    swap_used_mb_avg: Option<f64>,
    swap_used_mb_peak: Option<f64>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO mem_windows (window_start, avg_ram_mb, peak_ram_mb, swap_used_mb_avg, swap_used_mb_peak)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![window_start, avg_ram_mb, peak_ram_mb, swap_used_mb_avg, swap_used_mb_peak],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_oom_event(
    ts: &str,
    process_name: Option<&str>,
    victim: Option<bool>,
    rss_mb: Option<f64>,
) -> Result<i64> {
    let db = get_db()?;
    let process_name = process_name.map(|s| s.to_string());
    let ts = ts.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO oom_events (ts, process_name, victim, rss_mb)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                ts,
                process_name,
                victim.map(|b| if b { 1 } else { 0 }),
                rss_mb
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_fs_capacity(
    ts: &str,
    mountpoint: &str,
    total_gb: Option<f64>,
    free_gb: Option<f64>,
) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();
    let mountpoint = mountpoint.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO fs_capacity_daily (ts, mountpoint, total_gb, free_gb)
             VALUES (?1, ?2, ?3, ?4)",
            params![ts, mountpoint, total_gb, free_gb],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_fs_growth(
    window_start: &str,
    mountpoint: &str,
    path_prefix: Option<&str>,
    delta_gb: Option<f64>,
    contributors_json: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let mountpoint = mountpoint.to_string();
    let path_prefix = path_prefix.map(|s| s.to_string());
    let contributors = contributors_json.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO fs_growth (window_start, mountpoint, path_prefix, delta_gb, contributors)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                window_start,
                mountpoint,
                path_prefix,
                delta_gb,
                contributors
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_fs_io_window(
    window_start: &str,
    mountpoint: &str,
    read_mb_s_avg: Option<f64>,
    write_mb_s_avg: Option<f64>,
    latency_ms_p50: Option<f64>,
    latency_ms_p95: Option<f64>,
    queue_depth_avg: Option<f64>,
    io_errors: Option<i64>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let mountpoint = mountpoint.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO fs_io_windows (window_start, mountpoint, read_mb_s_avg, write_mb_s_avg,
                                        latency_ms_p50, latency_ms_p95, queue_depth_avg, io_errors)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                window_start,
                mountpoint,
                read_mb_s_avg,
                write_mb_s_avg,
                latency_ms_p50,
                latency_ms_p95,
                queue_depth_avg,
                io_errors
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_net_window(
    window_start: &str,
    iface: Option<&str>,
    target: Option<&str>,
    latency_ms_avg: Option<f64>,
    latency_ms_p95: Option<f64>,
    packet_loss_pct: Option<f64>,
    dns_failures: Option<i64>,
    dhcp_failures: Option<i64>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let iface = iface.map(|s| s.to_string());
    let target = target.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO net_windows (window_start, iface, target, latency_ms_avg, latency_ms_p95,
                                      packet_loss_pct, dns_failures, dhcp_failures)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                window_start,
                iface,
                target,
                latency_ms_avg,
                latency_ms_p95,
                packet_loss_pct,
                dns_failures,
                dhcp_failures
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_net_event(ts: &str, iface: Option<&str>, event: &str) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();
    let iface = iface.map(|s| s.to_string());
    let event = event.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO net_events (ts, iface, event)
             VALUES (?1, ?2, ?3)",
            params![ts, iface, event],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_service_health(
    ts: &str,
    service: &str,
    state: Option<&str>,
    time_in_failed_ms: Option<i64>,
    avg_start_time_ms: Option<i64>,
    config_change_ts: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();
    let service = service.to_string();
    let state = state.map(|s| s.to_string());
    let config_change_ts = config_change_ts.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO service_health (ts, service, state, time_in_failed_ms, avg_start_time_ms, config_change_ts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![ts, service, state, time_in_failed_ms, avg_start_time_ms, config_change_ts],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_service_restart(ts: &str, service: &str, reason: Option<&str>) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();
    let service = service.to_string();
    let reason = reason.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO service_restarts (ts, service, reason)
             VALUES (?1, ?2, ?3)",
            params![ts, service, reason],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_log_window_counts(
    window_start: &str,
    errors: Option<i64>,
    warnings: Option<i64>,
    criticals: Option<i64>,
    source: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let source = source.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO log_window_counts (window_start, errors, warnings, criticals, source)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![window_start, errors, warnings, criticals, source],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn upsert_log_signature(
    signature_hash: &str,
    first_seen: &str,
    last_seen: &str,
    count: i64,
    source: Option<&str>,
    sample_message: Option<&str>,
    status: Option<&str>,
) -> Result<()> {
    let db = get_db()?;
    let signature_hash = signature_hash.to_string();
    let first_seen = first_seen.to_string();
    let last_seen = last_seen.to_string();
    let source = source.map(|s| s.to_string());
    let sample_message = sample_message.map(|s| s.to_string());
    let status = status.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO log_signatures (signature_hash, first_seen, last_seen, count, source, sample_message, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(signature_hash) DO UPDATE SET
                last_seen = excluded.last_seen,
                count = excluded.count,
                source = excluded.source,
                sample_message = excluded.sample_message,
                status = excluded.status",
            params![
                signature_hash,
                first_seen,
                last_seen,
                count,
                source,
                sample_message,
                status
            ],
        )?;
        Ok(())
    })
    .await
}

pub async fn record_baseline(
    label: &str,
    created_at: &str,
    metrics_json: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let label = label.to_string();
    let created_at = created_at.to_string();
    let metrics = metrics_json.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO baselines (label, created_at, metrics)
             VALUES (?1, ?2, ?3)",
            params![label, created_at, metrics],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_baseline_delta(
    ts: &str,
    baseline_id: i64,
    metric: &str,
    delta_pct: Option<f64>,
    context: Option<&str>,
    impact_score: Option<f64>,
) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();
    let metric = metric.to_string();
    let context = context.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO baseline_deltas (ts, baseline_id, metric, delta_pct, context, impact_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![ts, baseline_id, metric, delta_pct, context, impact_score],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_usage_pattern(
    window_start: &str,
    active_hours_detected: Option<i64>,
    heavy_load_minutes: Option<i64>,
    low_load_minutes: Option<i64>,
    package_updates_count: Option<i64>,
    anna_runs: Option<i64>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO usage_patterns (window_start, active_hours_detected, heavy_load_minutes, low_load_minutes,
                                         package_updates_count, anna_runs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                window_start,
                active_hours_detected,
                heavy_load_minutes,
                low_load_minutes,
                package_updates_count,
                anna_runs
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_app_usage(
    window_start: &str,
    app: &str,
    minutes_active: Option<i64>,
    category: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let app = app.to_string();
    let category = category.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO app_usage (window_start, app, minutes_active, category)
             VALUES (?1, ?2, ?3, ?4)",
            params![window_start, app, minutes_active, category],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_llm_usage_window(
    window_start: &str,
    model_name: Option<&str>,
    total_calls: Option<i64>,
    success_calls: Option<i64>,
    latency_ms_avg: Option<f64>,
    latency_ms_p95: Option<f64>,
    backend_rss_mb: Option<f64>,
    gpu_util_pct_avg: Option<f64>,
    cpu_util_pct_avg: Option<f64>,
    failed_calls: Option<i64>,
    cost_estimate: Option<f64>,
    delta_latency_ms_avg: Option<f64>,
    delta_latency_ms_p95: Option<f64>,
) -> Result<i64> {
    let db = get_db()?;
    let window_start = window_start.to_string();
    let model_name = model_name.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO llm_usage_windows (window_start, model_name, total_calls, success_calls, latency_ms_avg, latency_ms_p95, backend_rss_mb,
                                            gpu_util_pct_avg, cpu_util_pct_avg, failed_calls, cost_estimate, delta_latency_ms_avg, delta_latency_ms_p95)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                window_start,
                model_name,
                total_calls,
                success_calls,
                latency_ms_avg,
                latency_ms_p95,
                backend_rss_mb,
                gpu_util_pct_avg,
                cpu_util_pct_avg,
                failed_calls,
                cost_estimate,
                delta_latency_ms_avg,
                delta_latency_ms_p95
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_llm_model_change(
    ts: &str,
    model_name: &str,
    reason: Option<&str>,
    hw_requirements: Option<impl Serialize>,
    notes: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();
    let model_name = model_name.to_string();
    let reason = reason.map(|s| s.to_string());
    let notes = notes.map(|s| s.to_string());
    let hw_requirements = hw_requirements
        .map(|v| serde_json::to_string(&v))
        .transpose()?;

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO llm_model_changes (ts, model_name, reason, hw_requirements, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![ts, model_name, reason, hw_requirements, notes],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_repair_metric(
    repair_id: i64,
    metric: &str,
    before_value: Option<f64>,
    after_value: Option<f64>,
    units: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let metric = metric.to_string();
    let units = units.map(|s| s.to_string());

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO repair_metrics (repair_id, metric, before_value, after_value, units)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![repair_id, metric, before_value, after_value, units],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn record_health_score(
    ts: &str,
    stability_score: Option<i64>,
    performance_score: Option<i64>,
    noise_score: Option<i64>,
    trend_stability: Option<&str>,
    trend_performance: Option<&str>,
    trend_noise: Option<&str>,
    last_regression: Option<&str>,
    last_regression_cause: Option<&str>,
    last_improvement: Option<&str>,
    last_improvement_cause: Option<&str>,
) -> Result<i64> {
    let db = get_db()?;
    let ts = ts.to_string();

    db.execute(move |conn| {
        conn.execute(
            "INSERT INTO health_scores (
                ts, stability_score, performance_score, noise_score,
                trend_stability, trend_performance, trend_noise,
                last_regression, last_regression_cause, last_improvement, last_improvement_cause
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                ts,
                stability_score,
                performance_score,
                noise_score,
                trend_stability.map(|s| s.to_string()),
                trend_performance.map(|s| s.to_string()),
                trend_noise.map(|s| s.to_string()),
                last_regression.map(|s| s.to_string()),
                last_regression_cause.map(|s| s.to_string()),
                last_improvement.map(|s| s.to_string()),
                last_improvement_cause.map(|s| s.to_string())
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

# Historian Telemetry Schema

This document maps the detection scope to concrete telemetry datasets Anna must persist. Goal: the LLM always has structured, queriable history for trends, regressions, and impact analysis. All data is local-only.

---

## 1) Global Timeline
- **Table**: `timeline_events`
- **Fields**: id, ts, kind (`install|upgrade|rollback|partial-upgrade|kernel-change|config-migration|self-repair`), from_version, to_version, details, outcome (`success|failed|regressed|held`).
- **Derive**: “what changed before it broke”, count of self-repairs, config migrations performed by Anna.

## 2) Boot & Shutdown
- **Tables**: `boot_sessions`, `boot_unit_slowlog`
- **Fields (boot_sessions)**: boot_id, ts_start, ts_end, goal (`graphical|multi-user`), time_to_goal_ms, degraded (bool), fsck_ran (bool), fsck_duration_ms, shutdown_duration_ms, early_kernel_errors_count, boot_health_score (0–100).
- **Fields (boot_unit_slowlog)**: boot_id, unit, duration_ms, state (`ok|degraded|failed`).
- **Derive**: avg boot (7/30d), slowest recurring units, trends vs baseline, per-boot health score moving average.

## 3) CPU Usage
- **Tables**: `cpu_windows_hourly`, `cpu_top_processes`
- **Fields (cpu_windows_hourly)**: window_start, window_end, avg_util_per_core[], peak_util_per_core[], idle_background_load, throttling_events, spikes_over_100pct.
- **Fields (cpu_top_processes)**: window_start, process_name, cpu_time_seconds.
- **Derive**: idle pattern per hour-of-day, CPU usage trend, new top CPU processes.

## 4) Memory & Swap
- **Tables**: `mem_windows_hourly`, `oom_events`
- **Fields (mem_windows_hourly)**: window_start, avg_ram_mb, peak_ram_mb, swap_used_mb_avg, swap_used_mb_peak.
- **Fields (oom_events)**: ts, process_name, victim (bool), rss_mb.
- **Derive**: post-boot baseline vs now, swap dependency trend, chronic hogs, growth after updates.

## 5) Disk Space, I/O, Growth
- **Tables**: `fs_capacity_daily`, `fs_growth`, `fs_io_windows`
- **Fields (fs_capacity_daily)**: ts, mountpoint, total_gb, free_gb.
- **Fields (fs_growth)**: window_start, mountpoint, path_prefix, delta_gb, contributors (top dirs).
- **Fields (fs_io_windows)**: window_start, mountpoint, read_mb_s_avg, write_mb_s_avg, latency_ms_p50/p95, queue_depth_avg, io_errors.
- **Derive**: threshold crossings (80/90%), growth curves, log explosion/cache bloat, correlate I/O spikes to services/apps.

## 6) Network Quality
- **Tables**: `net_windows_hourly`, `net_events`
- **Fields (net_windows_hourly)**: window_start, iface, target (`gateway|8.8.8.8|arch-mirror`), latency_ms_avg, latency_ms_p95, packet_loss_pct, dns_failures, dhcp_failures.
- **Fields (net_events)**: ts, iface, event (`disconnect|reconnect|vpn-connect|vpn-disconnect`).
- **Derive**: baseline vs current latency, time-of-day badness, unstable interfaces, correlation to suspend/resume.

## 7) Service Reliability
- **Tables**: `service_health`, `service_restarts`
- **Fields (service_health)**: ts, service, state (`ok|failed|degraded`), time_in_failed_ms, avg_start_time_ms, config_change_ts.
- **Fields (service_restarts)**: ts, service, reason (`crash|manual|upgrade|unknown`).
- **Derive**: stability score per service, flaky units, time since last crash, trend.

## 8) Error & Warning Statistics
- **Tables**: `log_window_counts`, `log_signatures`
- **Fields (log_window_counts)**: window_start, errors, warnings, criticals, source (`kernel|service|app`).
- **Fields (log_signatures)**: signature_hash, first_seen, last_seen, count, source, sample_message, status (`active|resolved`).
- **Derive**: error rate trend, top recurring errors, first-seen, disappearance after change/repair.

## 9) Performance Baselines & Deltas
- **Tables**: `baselines`, `baseline_deltas`
- **Fields (baselines)**: baseline_id, label (`boot|idle|workflow-<name>`), created_at, metrics JSON (boot time, idle CPU/RAM/disk/net, workflow timings).
- **Fields (baseline_deltas)**: ts, baseline_id, metric, delta_pct, context (`gpu-driver-change|kernel-upgrade|repair-<id>`), impact_score.
- **Derive**: deviation vs baseline, before/after for major changes, impact of repairs/tuning.

## 10) User Behavior (Technical, Non-creepy)
- **Tables**: `usage_patterns`, `app_usage`
- **Fields (usage_patterns)**: window_start, active_hours_detected, heavy_load_minutes, low_load_minutes, package_updates_count, anna_runs.
- **Fields (app_usage)**: window_start, app, minutes_active, category (`browser|editor|game|dev|other`).
- **Derive**: routines vs anomalies, optimizations worth doing, whether suggestions were applied and improved metrics.

## 11) LLM Statistics
- **Tables**: `llm_usage_windows`, `llm_model_changes`
- **Fields (llm_usage_windows)**: window_start, latency_ms_avg/p95, backend_rss_mb, gpu_util_pct_avg, cpu_util_pct_avg, failed_calls.
- **Fields (llm_model_changes)**: ts, model_name, reason (`upgrade|downgrade|switch`), hw_requirements, notes.
- **Derive**: best-fit model for hardware, safety of heavier models, patterns of unavailability, thermal impact.

## 12) Self-Repair Effectiveness
- **Tables**: `repairs`, `repair_metrics`
- **Fields (repairs)**: repair_id, ts, trigger (`health-check|user-request|startup`), action, outcome (`success|partial|failed`), user_feedback (`helpful|worse|no-change|n/a`), recurred (bool).
- **Fields (repair_metrics)**: repair_id, metric, before_value, after_value, units.
- **Derive**: success rate, recurring problems, risky repairs to suggest but not auto-apply.

## 13) Synthesized Indicators
- **Table**: `health_scores`
- **Fields**: ts, stability_score_0_100, performance_score_0_100, noise_score_0_100, trend_stability (`up|down|flat`), trend_performance, trend_noise, last_regression, last_regression_cause, last_improvement, last_improvement_cause.
- **Derive**: simple numbers LLM can cite, with arrows and recent causes.

---

## Sampling & Retention Guidelines
- **Sampling**: hourly windows for load/resource metrics; daily snapshots for capacity; per-event for boots/reboots/errors/upgrades.
- **Retention**: at least 90 days rolling for windowed data; keep baselines, model changes, and timeline events indefinitely (small footprint).
- **Privacy**: user-behavior and command patterns remain opt-in; no raw command text is stored, only aggregates.


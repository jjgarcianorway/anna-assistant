//! Skill Handlers v1.6.0 - Generic skill execution
//!
//! This module implements handlers for each SkillType.
//! Each handler enforces strict invariants:
//! - Always returns non-empty message text
//! - Always returns valid reliability in [0.0, 1.0]
//! - Always sets origin appropriately
//! - Returns graceful fallback on any error
//!
//! ## Time Budgets
//! - Brain-only skills: <250ms
//! - LLM skills: <15s total (Junior + Senior)
//! - Long-running skills (benchmarks): <5min

use crate::skill_router::{SkillType, SkillAnswer, AnswerOrigin, SkillContext};
use crate::bench_snow_leopard::{
    SnowLeopardConfig, run_benchmark, BenchmarkMode,
    BenchmarkHistoryEntry, BenchmarkHistoryListItem,
    compare_last_two_benchmarks, format_benchmark_history,
};
use crate::brain_fast::{
    fast_cpu_answer, fast_ram_answer, fast_disk_answer, fast_health_answer,
    fast_debug_enable, fast_debug_disable, fast_debug_status,
    fast_reset_experience_confirm, fast_reset_factory_confirm,
    fast_gpu_answer, fast_os_answer, fast_uptime_answer,
    fast_network_answer, fast_logs_summary, fast_updates_check,
};
use std::time::Instant;

// ============================================================================
// MAIN DISPATCH FUNCTION
// ============================================================================

/// Handle a skill and return a guaranteed-valid SkillAnswer.
///
/// This function NEVER panics or returns an invalid result.
/// All errors are caught and converted to fallback answers.
pub async fn handle_skill(skill: SkillType, ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    let result = match skill {
        // Benchmark skills
        SkillType::BenchmarkRunFull => handle_benchmark_run(false, ctx).await,
        SkillType::BenchmarkRunQuick => handle_benchmark_run(true, ctx).await,
        SkillType::BenchmarkHistory => handle_benchmark_history(ctx),
        SkillType::BenchmarkCompareLastTwo => handle_benchmark_compare(ctx),

        // System info skills (Brain-only, deterministic - no LLM)
        SkillType::CpuInfo => handle_cpu_info(ctx),
        SkillType::RamInfo => handle_ram_info(ctx),
        SkillType::RootDiskInfo => handle_disk_info(ctx),
        SkillType::UptimeInfo => handle_uptime_info(ctx),
        SkillType::NetworkSummary => handle_network_summary(ctx),
        SkillType::GpuInfo => handle_gpu_info(ctx),
        SkillType::OsInfo => handle_os_info(ctx),

        // Service skills (Brain-only, deterministic - no LLM)
        SkillType::SelfHealth => handle_self_health(ctx),
        SkillType::LogsAnnadSummary => handle_logs_summary(ctx),
        SkillType::UpdatesPlan => handle_updates_check(ctx),

        // Debug control
        SkillType::DebugEnable => handle_debug_enable(ctx),
        SkillType::DebugDisable => handle_debug_disable(ctx),
        SkillType::DebugStatus => handle_debug_status(ctx),

        // Reset skills
        SkillType::ResetExperience => handle_reset_experience(ctx),
        SkillType::ResetFactory => handle_reset_factory(ctx),

        // Unsupported
        SkillType::Unsupported => {
            let duration = start.elapsed().as_millis() as u64;
            SkillAnswer::unsupported(&ctx.question, duration)
        }
    };

    // Ensure duration is set correctly
    let duration = start.elapsed().as_millis() as u64;

    // If result has zero duration, update it
    if result.duration_ms == 0 {
        SkillAnswer {
            duration_ms: duration,
            ..result
        }
    } else {
        result
    }
}

// ============================================================================
// BENCHMARK HANDLERS
// ============================================================================

async fn handle_benchmark_run(is_quick: bool, ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();
    let skill = if is_quick { SkillType::BenchmarkRunQuick } else { SkillType::BenchmarkRunFull };
    let mode = if is_quick { BenchmarkMode::Quick } else { BenchmarkMode::Full };

    // Check time budget
    if start.elapsed() > ctx.time_budget {
        return SkillAnswer::timeout(skill, None, start.elapsed().as_millis() as u64);
    }

    // Configure benchmark
    let mut config = SnowLeopardConfig::test_mode();
    config.phases_enabled = if is_quick {
        crate::bench_snow_leopard::PhaseId::quick()
    } else {
        crate::bench_snow_leopard::PhaseId::all()
    };

    // Run benchmark
    let result = run_benchmark(&config).await;

    // Save to history
    if let Err(e) = BenchmarkHistoryEntry::from_result(&result).save() {
        // Log but don't fail
        eprintln!("[WARN] Failed to save benchmark history: {}", e);
    }

    let duration = start.elapsed().as_millis() as u64;
    let success_rate = result.overall_success_rate();

    // Format result
    let message = if !result.ascii_summary.is_empty() {
        result.ascii_summary.clone()
    } else {
        format!(
            "SNOW LEOPARD BENCHMARK COMPLETE\n\
             ═══════════════════════════════════════════\n\
             Mode: {:?}\n\
             Total Questions: {}\n\
             Success Rate: {:.1}%\n\
             Average Latency: {}ms\n\
             Brain Usage: {:.1}%\n\
             Phases: {}\n\
             ═══════════════════════════════════════════",
            mode,
            result.total_questions,
            success_rate,
            result.overall_avg_latency(),
            result.brain_usage_pct() * 100.0,
            result.phases.len()
        )
    };

    SkillAnswer::success(
        skill,
        message,
        AnswerOrigin::Brain,
        0.99,
        duration,
    )
}

fn handle_benchmark_history(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    let entries = BenchmarkHistoryEntry::list_recent(10);
    let duration = start.elapsed().as_millis() as u64;

    if entries.is_empty() {
        return SkillAnswer::success(
            SkillType::BenchmarkHistory,
            "No benchmark history found.\n\n\
             Run a benchmark first with: \"run the snow leopard benchmark\"",
            AnswerOrigin::Brain,
            0.99,
            duration,
        );
    }

    // entries is already Vec<BenchmarkHistoryListItem> from list_recent()
    let message = format_benchmark_history(&entries);

    SkillAnswer::success(
        SkillType::BenchmarkHistory,
        message,
        AnswerOrigin::Brain,
        0.99,
        duration,
    )
}

fn handle_benchmark_compare(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    let comparison = compare_last_two_benchmarks();
    let duration = start.elapsed().as_millis() as u64;

    match comparison {
        Some(delta) => {
            SkillAnswer::success(
                SkillType::BenchmarkCompareLastTwo,
                delta.format_ascii(),
                AnswerOrigin::Brain,
                0.99,
                duration,
            )
        }
        None => {
            SkillAnswer::success(
                SkillType::BenchmarkCompareLastTwo,
                "Not enough benchmark runs to compare.\n\n\
                 Run at least two benchmarks to see a comparison.\n\
                 Use: \"run the snow leopard benchmark\"",
                AnswerOrigin::Brain,
                0.99,
                duration,
            )
        }
    }
}

// ============================================================================
// SYSTEM INFO HANDLERS (Brain-only)
// ============================================================================

fn handle_cpu_info(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_cpu_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::CpuInfo,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::CpuInfo,
                "Could not retrieve CPU information.\n\n\
                 This may happen if /proc/cpuinfo is not accessible.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "cpu info unavailable",
            )
        }
    }
}

fn handle_ram_info(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_ram_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::RamInfo,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::RamInfo,
                "Could not retrieve memory information.\n\n\
                 This may happen if /proc/meminfo is not accessible.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "memory info unavailable",
            )
        }
    }
}

fn handle_disk_info(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_disk_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::RootDiskInfo,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::RootDiskInfo,
                "Could not retrieve disk information.\n\n\
                 This may happen if the df command is not available.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "disk info unavailable",
            )
        }
    }
}

fn handle_uptime_info(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    // Read /proc/uptime directly
    match std::fs::read_to_string("/proc/uptime") {
        Ok(content) => {
            let uptime_secs: f64 = content
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            let days = (uptime_secs / 86400.0) as u64;
            let hours = ((uptime_secs % 86400.0) / 3600.0) as u64;
            let mins = ((uptime_secs % 3600.0) / 60.0) as u64;

            let message = if days > 0 {
                format!("System uptime: {} days, {} hours, {} minutes", days, hours, mins)
            } else if hours > 0 {
                format!("System uptime: {} hours, {} minutes", hours, mins)
            } else {
                format!("System uptime: {} minutes", mins)
            };

            SkillAnswer::success(
                SkillType::UptimeInfo,
                message,
                AnswerOrigin::Brain,
                0.99,
                start.elapsed().as_millis() as u64,
            )
        }
        Err(_) => {
            SkillAnswer::fallback(
                SkillType::UptimeInfo,
                "Could not retrieve uptime information.\n\n\
                 This may happen if /proc/uptime is not accessible.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "uptime unavailable",
            )
        }
    }
}

fn handle_network_summary(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_network_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::NetworkSummary,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::NetworkSummary,
                "Could not retrieve network information.\n\n\
                 The 'ip' command may not be available.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "network info unavailable",
            )
        }
    }
}

fn handle_gpu_info(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_gpu_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::GpuInfo,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::GpuInfo,
                "Could not retrieve GPU information.\n\n\
                 The 'lspci' command may not be available.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "gpu info unavailable",
            )
        }
    }
}

fn handle_os_info(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_os_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::OsInfo,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::OsInfo,
                "Could not retrieve OS information.\n\n\
                 The 'uname' command may not be available.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "os info unavailable",
            )
        }
    }
}

// ============================================================================
// SERVICE HANDLERS
// ============================================================================

fn handle_self_health(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_health_answer() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::SelfHealth,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::SelfHealth,
                "Could not perform health check.\n\n\
                 The health check system may be unavailable.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "health check unavailable",
            )
        }
    }
}

fn handle_logs_summary(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_logs_summary() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::LogsAnnadSummary,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::LogsAnnadSummary,
                "Could not retrieve annad logs.\n\n\
                 The journalctl command may not be available.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "logs unavailable",
            )
        }
    }
}

fn handle_updates_check(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_updates_check() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::UpdatesPlan,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            // Fallback: show version info
            let version = env!("CARGO_PKG_VERSION");
            SkillAnswer::success(
                SkillType::UpdatesPlan,
                format!(
                    "Anna v{}\n\n\
                     Could not check for package updates.\n\
                     Visit https://github.com/anna-assistant/releases for latest releases.",
                    version
                ),
                AnswerOrigin::Brain,
                0.7,
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

// ============================================================================
// DEBUG CONTROL HANDLERS
// ============================================================================

fn handle_debug_enable(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_debug_enable() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::DebugEnable,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::DebugEnable,
                "Could not enable debug mode.\n\n\
                 There may be an issue with the debug state file.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "debug enable failed",
            )
        }
    }
}

fn handle_debug_disable(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_debug_disable() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::DebugDisable,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::DebugDisable,
                "Could not disable debug mode.\n\n\
                 There may be an issue with the debug state file.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "debug disable failed",
            )
        }
    }
}

fn handle_debug_status(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_debug_status() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::DebugStatus,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::DebugStatus,
                "Could not check debug status.\n\n\
                 There may be an issue with the debug state file.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "debug status failed",
            )
        }
    }
}

// ============================================================================
// RESET HANDLERS
// ============================================================================

fn handle_reset_experience(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_reset_experience_confirm() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::ResetExperience,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::ResetExperience,
                "Could not initiate experience reset.\n\n\
                 There may be an issue with the reset system.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "reset experience failed",
            )
        }
    }
}

fn handle_reset_factory(_ctx: &SkillContext) -> SkillAnswer {
    let start = Instant::now();

    match fast_reset_factory_confirm() {
        Some(answer) => {
            SkillAnswer::success(
                SkillType::ResetFactory,
                answer.text,
                AnswerOrigin::Brain,
                answer.reliability,
                start.elapsed().as_millis() as u64,
            )
        }
        None => {
            SkillAnswer::fallback(
                SkillType::ResetFactory,
                "Could not initiate factory reset.\n\n\
                 There may be an issue with the reset system.",
                AnswerOrigin::Brain,
                0.3,
                start.elapsed().as_millis() as u64,
                "reset factory failed",
            )
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_unsupported() {
        let ctx = SkillContext::new("what is the meaning of life");
        let answer = handle_skill(SkillType::Unsupported, &ctx).await;

        assert!(!answer.message.is_empty());
        assert_eq!(answer.origin, AnswerOrigin::Unsupported);
        assert!(answer.reliability > 0.0);
    }

    #[tokio::test]
    async fn test_handle_benchmark_history_empty() {
        // This may or may not have history depending on test state
        let ctx = SkillContext::new("show benchmark history");
        let answer = handle_skill(SkillType::BenchmarkHistory, &ctx).await;

        // Should always return a valid answer
        assert!(!answer.message.is_empty());
        assert_eq!(answer.origin, AnswerOrigin::Brain);
        assert!(answer.reliability > 0.0);
    }

    #[tokio::test]
    async fn test_handle_uptime() {
        let ctx = SkillContext::new("what is the uptime");
        let answer = handle_skill(SkillType::UptimeInfo, &ctx).await;

        assert!(!answer.message.is_empty());
        assert_eq!(answer.origin, AnswerOrigin::Brain);
        // Either success or fallback, but always valid
        assert!(answer.reliability > 0.0);
    }

    #[tokio::test]
    async fn test_all_handlers_return_valid_answers() {
        let skills = [
            SkillType::CpuInfo,
            SkillType::RamInfo,
            SkillType::RootDiskInfo,
            SkillType::UptimeInfo,
            SkillType::SelfHealth,
            SkillType::DebugStatus,
            SkillType::BenchmarkHistory,
            SkillType::BenchmarkCompareLastTwo,
            SkillType::Unsupported,
        ];

        for skill in skills {
            let ctx = SkillContext::new("test question");
            let answer = handle_skill(skill, &ctx).await;

            assert!(
                !answer.message.is_empty(),
                "Skill {:?} returned empty message",
                skill
            );
            assert!(
                answer.reliability >= 0.0 && answer.reliability <= 1.0,
                "Skill {:?} returned invalid reliability: {}",
                skill,
                answer.reliability
            );
            assert!(
                answer.duration_ms < 10000,
                "Skill {:?} took too long: {}ms",
                skill,
                answer.duration_ms
            );
        }
    }

    #[test]
    fn test_skill_answer_invariants() {
        // Test that SkillAnswer enforces invariants

        // Success with valid message
        let answer = SkillAnswer::success(
            SkillType::CpuInfo,
            "Valid message",
            AnswerOrigin::Brain,
            0.95,
            100,
        );
        assert!(!answer.message.is_empty());
        assert!(!answer.is_fallback);

        // Fallback with empty message gets default
        let fallback = SkillAnswer::fallback(
            SkillType::CpuInfo,
            "",
            AnswerOrigin::Brain,
            0.3,
            100,
            "some error",
        );
        assert!(!fallback.message.is_empty());
        assert!(fallback.is_fallback);

        // Timeout
        let timeout = SkillAnswer::timeout(SkillType::LogsAnnadSummary, Some("partial"), 15000);
        assert!(!timeout.message.is_empty());
        assert!(timeout.is_fallback);
    }

    #[test]
    fn test_time_budget_constraints() {
        use std::time::Duration;

        // All deterministic skills (v1.7.0) must have <500ms budget
        let brain_skills = [
            SkillType::CpuInfo,
            SkillType::RamInfo,
            SkillType::RootDiskInfo,
            SkillType::UptimeInfo,
            SkillType::SelfHealth,
            SkillType::DebugEnable,
            SkillType::DebugDisable,
            SkillType::DebugStatus,
            // New in v1.7.0: these are now brain-only too
            SkillType::LogsAnnadSummary,
            SkillType::NetworkSummary,
            SkillType::UpdatesPlan,
            SkillType::GpuInfo,
            SkillType::OsInfo,
        ];

        for skill in brain_skills {
            assert!(
                skill.time_budget() < Duration::from_millis(500),
                "Brain-only skill {:?} has budget > 500ms: {:?}",
                skill,
                skill.time_budget()
            );
        }

        // NOTE: As of v1.7.0, no skills require LLM - all are deterministic

        // Long-running skills must have budget > 1 minute
        let long_skills = [
            SkillType::BenchmarkRunFull,
            SkillType::BenchmarkRunQuick,
        ];

        for skill in long_skills {
            assert!(
                skill.time_budget() >= Duration::from_secs(60),
                "Long-running skill {:?} has budget < 60s: {:?}",
                skill,
                skill.time_budget()
            );
        }
    }

    #[test]
    fn test_failure_policy_never_empty() {
        // Test that fallback answers never have empty messages
        let test_cases = [
            ("", "some error"),
            ("   ", "another error"),
            ("\n\t", "whitespace only"),
        ];

        for (msg, err) in test_cases {
            let answer = SkillAnswer::fallback(
                SkillType::CpuInfo,
                msg,
                AnswerOrigin::Brain,
                0.3,
                100,
                err,
            );

            assert!(
                !answer.message.trim().is_empty(),
                "Fallback with input '{}' produced empty message",
                msg
            );
        }
    }

    #[test]
    fn test_failure_policy_never_zero_reliability() {
        // Test that even fallback answers have non-zero reliability
        let answer = SkillAnswer::fallback(
            SkillType::CpuInfo,
            "Something went wrong",
            AnswerOrigin::Brain,
            0.0,  // Try to set zero
            100,
            "error",
        );

        // Should be clamped to at least 0.1
        assert!(
            answer.reliability >= 0.1,
            "Fallback has zero reliability: {}",
            answer.reliability
        );
    }

    #[test]
    fn test_timeout_includes_partial_work() {
        let timeout = SkillAnswer::timeout(
            SkillType::LogsAnnadSummary,
            Some("- Queried journalctl\n- Found 100 entries"),
            15000,
        );

        assert!(timeout.message.contains("journalctl"), "Timeout should include partial work");
        assert!(timeout.message.contains("checked"), "Timeout should mention what was checked");
        assert!(timeout.is_fallback);
    }

    #[tokio::test]
    async fn test_brain_skill_latency() {
        use std::time::Instant;

        // Brain-only skills should complete in <250ms (the user requirement)
        let brain_skills = [
            SkillType::CpuInfo,
            SkillType::RamInfo,
            SkillType::RootDiskInfo,
            SkillType::UptimeInfo,
        ];

        for skill in brain_skills {
            let ctx = SkillContext::new("test");
            let start = Instant::now();
            let _answer = handle_skill(skill, &ctx).await;
            let elapsed = start.elapsed();

            assert!(
                elapsed.as_millis() < 250,
                "Brain skill {:?} took {}ms (max 250ms)",
                skill,
                elapsed.as_millis()
            );
        }
    }
}

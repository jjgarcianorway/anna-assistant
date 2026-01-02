//! Display, stats, and UI module declarations
//! All modules for statistics, display, and user interface

// Stats modules
#[path = "stats.rs"]
pub mod stats;
#[path = "stats_store.rs"]
pub mod stats_store;
#[path = "person_stats.rs"]
pub mod person_stats;
#[path = "staff_stats/mod.rs"]
pub mod staff_stats; // v0.0.107: Staff performance tracking
#[path = "honest_stats.rs"]
pub mod honest_stats; // v0.0.415: Honest stats tracking
#[path = "ticket_stats.rs"]
pub mod ticket_stats; // v0.0.407: Truthful ticket statistics
#[path = "ticket_resolution_stats/mod.rs"]
pub mod ticket_resolution_stats; // v0.0.510: Ticket resolution stats (modularized)
#[path = "expert_stats/mod.rs"]
pub mod expert_stats; // v0.0.489: Expert ticket statistics

// Display modules
#[path = "xp_display.rs"]
pub mod xp_display; // v0.0.478: XP/Level RPG-style progression display
#[path = "fun_stats_display/mod.rs"]
pub mod fun_stats_display; // v0.0.479: Fun statistics display
#[path = "capabilities_display/mod.rs"]
pub mod capabilities_display; // v0.0.480: Capabilities display (modularized)
#[path = "quick_status/mod.rs"]
pub mod quick_status; // v0.0.484: Quick status summary
#[path = "session_display.rs"]
pub mod session_display; // v0.0.475: Session summary display
#[path = "settings_display.rs"]
pub mod settings_display; // v0.0.476: Unified settings display
#[path = "ticket_history_display/mod.rs"]
pub mod ticket_history_display; // v0.0.493: Ticket history display
#[path = "error_summary_display/mod.rs"]
pub mod error_summary_display; // v0.0.494: Error summary display
#[path = "team_performance_display/mod.rs"]
pub mod team_performance_display; // v0.0.495: Team performance display

// Dashboard and report modules
#[path = "stats_dashboard/mod.rs"]
pub mod stats_dashboard; // v0.0.491: Aggregated stats dashboard (modular)
#[path = "anna_progress_report/mod.rs"]
pub mod anna_progress_report; // v0.0.496: Anna progress report
#[path = "user_activity_summary/mod.rs"]
pub mod user_activity_summary; // v0.0.497: User activity summary
#[path = "anna_metrics_dashboard.rs"]
pub mod anna_metrics_dashboard; // v0.0.524: Anna metrics dashboard (Phase 100!)

// Rendering modules
#[path = "render/mod.rs"]
pub mod render;
#[path = "comms_render.rs"]
pub mod comms_render; // v0.0.407: Internal comms rendering from ticket state
#[path = "transcript_render/mod.rs"]
pub mod transcript_render; // v0.0.413: Cinematic/debug transcript renderer (modularized)
#[path = "dialogue_renderer/mod.rs"]
pub mod dialogue_renderer; // v0.0.513: Dialogue renderer (modularized directory)

// UI modules
#[path = "ui/mod.rs"]
pub mod ui;
#[path = "presentation.rs"]
pub mod presentation;
#[path = "hollywood_ux/mod.rs"]
pub mod hollywood_ux; // v0.0.431: Unified transcript and Hollywood terminal renderer
#[path = "theatre/mod.rs"]
pub mod theatre; // v0.0.81: Service Desk Theatre - cinematic narrative
#[path = "display_mode_manager.rs"]
pub mod display_mode_manager; // v0.0.536: Display mode manager

// Greeting modules
#[path = "greetings.rs"]
pub mod greetings; // v0.0.89: Personalized greetings and context-aware dialogue
#[path = "greeting_insights/mod.rs"]
pub mod greeting_insights; // v0.0.245
#[path = "greeting_context.rs"]
pub mod greeting_context; // v0.0.275: LLM-generated greeting context
#[path = "greeting_tips.rs"]
pub mod greeting_tips; // v0.0.468: Configuration tips in greetings
#[path = "repl_greeting/mod.rs"]
pub mod repl_greeting; // v0.0.413: Stats-based REPL greeting
#[path = "greeting_generator.rs"]
pub mod greeting_generator; // v0.0.535: Greeting generator

// Progress and status modules
#[path = "progress/mod.rs"]
pub mod progress;
#[path = "status.rs"]
pub mod status;
#[path = "status_snapshot/mod.rs"]
pub mod status_snapshot;
#[path = "system_health_score/mod.rs"]
pub mod system_health_score; // v0.0.498: System health score
#[path = "uptime_tracker/mod.rs"]
pub mod uptime_tracker; // v0.0.492: Uptime tracking

// Achievements and streaks
#[path = "achievements.rs"]
pub mod achievements; // v0.0.90: Achievement badges for stats/RPG
#[path = "streaks.rs"]
pub mod streaks; // v0.0.86: Streak calculations for stats/RPG

// Tips and hints
#[path = "idle_tips.rs"]
pub mod idle_tips; // v0.0.240
#[path = "health_tips/mod.rs"]
pub mod health_tips; // v0.0.244 (modularized directory)
#[path = "followup_hints/mod.rs"]
pub mod followup_hints; // v0.0.384: Context-aware follow-up suggestions
#[path = "contextual_tips/mod.rs"]
pub mod contextual_tips; // v0.0.482: Contextual tips system
#[path = "tips_system/mod.rs"]
pub mod tips_system; // v0.0.541: Tips system
#[path = "interesting_facts/mod.rs"]
pub mod interesting_facts; // v0.0.289: Interesting facts for greetings

// Health and monitoring
#[path = "health_brief/mod.rs"]
pub mod health_brief;
#[path = "health_delta/mod.rs"]
pub mod health_delta;
#[path = "health_view/mod.rs"]
pub mod health_view;
#[path = "health_alerts.rs"]
pub mod health_alerts; // v0.0.281: Proactive health alerts

// Dialogue modules
#[path = "dialogue.rs"]
pub mod dialogue; // v0.0.87: Dialogue variety for theatre

// Error output
#[path = "error_output/mod.rs"]
pub mod error_output; // v0.0.407: User-friendly error messages

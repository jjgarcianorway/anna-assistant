//! Error codes and exit status for annactl
//!
//! Phase 0.3a: Standard exit codes for different failure modes
//! Citation: [archwiki:system_maintenance]

/// Exit code when command is not available in current state
pub const EXIT_COMMAND_NOT_AVAILABLE: i32 = 64;

/// Exit code when daemon returns invalid JSON
pub const EXIT_INVALID_RESPONSE: i32 = 65;

/// Exit code when daemon is unavailable/unreachable
pub const EXIT_DAEMON_UNAVAILABLE: i32 = 70;

/// Exit code for success
pub const EXIT_SUCCESS: i32 = 0;

/// Exit code for general errors
pub const EXIT_GENERAL_ERROR: i32 = 1;

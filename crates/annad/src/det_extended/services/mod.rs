//! Services answer functions (v0.0.175).
//!
//! Systemd units, timers, sockets, scopes, paths, docker, crontabs.

mod docker;
mod other;
mod systemd;

// Re-export all functions for backwards compatibility
pub use docker::{answer_docker_containers, answer_docker_images};

pub use other::{answer_crontabs, answer_loginctl_sessions, answer_ntp_status};

pub use systemd::{
    answer_running_services, answer_systemctl_mask, answer_systemd_journal, answer_systemd_paths,
    answer_systemd_scopes, answer_systemd_slices, answer_systemd_sockets, answer_systemd_targets,
    answer_systemd_timers, answer_systemd_units,
};

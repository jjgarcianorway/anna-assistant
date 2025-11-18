// Phase 3.0.0-alpha.1: Adaptive Intelligence & Smart Profiling
// System profiling and capability detection module

pub mod detector;
pub mod types;

pub use detector::SystemProfiler;
pub use types::{MonitoringMode, SessionType, SystemProfile, VirtualizationInfo};

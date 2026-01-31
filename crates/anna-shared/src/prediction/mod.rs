//! Predictive ML module for Anna.
//!
//! Provides trend analysis and resource forecasting:
//! - Disk full prediction
//! - Boot time degradation detection
//! - Memory leak detection
//! - Capacity planning

mod trends;
mod forecaster;
mod alerts;

pub use trends::{TrendAnalysis, TrendDirection, analyze_trend, detect_memory_leak, detect_boot_degradation};
pub use forecaster::{ResourceForecast, Forecaster};
pub use alerts::{PredictiveAlert, AlertSeverity, AlertInput, generate_predictive_alerts};

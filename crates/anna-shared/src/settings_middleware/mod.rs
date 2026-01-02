// v0.0.590: Settings Middleware (Phase 166)
// Middleware pipeline for settings operations

mod pipeline;
mod types;
mod utils;

// Re-export all public types and functions to preserve API
pub use pipeline::MiddlewarePipeline;
pub use types::{
    Middleware, MiddlewareAction, MiddlewareContext, MiddlewarePriority, MiddlewareResult,
};
pub use utils::{format_pipeline, is_middleware_query, settings_middleware_fun_fact};

//! Probe result caching functionality.

use std::time::Instant;

use anna_shared::rpc::ProbeResult;

use super::types::DaemonStateInner;
use crate::state_types::CachedProbe;

impl DaemonStateInner {
    /// Get cached probe result if still valid
    pub fn get_cached_probe(&self, command: &str) -> Option<ProbeResult> {
        self.probe_cache.get(command).and_then(|cached| {
            if cached.is_valid() {
                Some(cached.result.clone())
            } else {
                None
            }
        })
    }

    /// Cache a probe result
    pub fn cache_probe(&mut self, result: ProbeResult) {
        self.probe_cache.insert(
            result.command.clone(),
            CachedProbe {
                result,
                cached_at: Instant::now(),
            },
        );
    }

    /// Clean expired probe cache entries
    pub fn clean_probe_cache(&mut self) {
        self.probe_cache.retain(|_, cached| cached.is_valid());
    }
}

//! Profile command implementations

use anyhow::Result;
use anna_common::anna_narrative;

use crate::profile::{ProfileCollector, run_checks, render_profile, render_checks};
use crate::profile::collector::ProfileDepth;

/// Show system profile
pub async fn profile_show() -> Result<()> {
    anna_narrative("Let me take a look at your system...");

    let collector = ProfileCollector::new(ProfileDepth::Standard);
    let data = collector.collect()?;

    render_profile(&data)?;

    Ok(())
}

/// Run and display system checks
pub async fn profile_checks(json: bool, status_filter: Option<&str>) -> Result<()> {
    if !json {
        anna_narrative("Running health checks...");
    }

    let mut checks = run_checks()?;

    // Filter by status if requested
    if let Some(status) = status_filter {
        let status_upper = status.to_uppercase();
        checks.retain(|c| format!("{:?}", c.status).to_uppercase() == status_upper);
    }

    render_checks(&checks, json)?;

    Ok(())
}

// Anna v0.11.0 - Devices Listener
//
// Watches for device hotplug events (USB, block devices, network interfaces).
// In full implementation, this would use udev. For now, it's a placeholder.

use crate::events::{create_event, EventDomain, SystemEvent};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{info, warn};

/// Spawn devices listener task
pub fn spawn_listener(_tx: mpsc::UnboundedSender<SystemEvent>) -> JoinHandle<()> {
    info!("Devices listener: placeholder (udev integration pending)");

    tokio::spawn(async move {
        // Placeholder: In full implementation, this would:
        // 1. use udev = "0.8" crate
        // 2. Monitor subsystems: usb, block, net, bluetooth
        // 3. Send events on add/remove/change
        //
        // For now, this is a no-op to maintain architecture

        warn!("Devices listener: not yet fully implemented");

        // Keep task alive but do nothing
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    })
}

/// Simulate a devices event (for testing)
#[cfg(test)]
pub fn simulate_event(cause: &str) -> SystemEvent {
    create_event(EventDomain::Devices, cause)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_event() {
        let event = simulate_event("usb_added /dev/sdb");
        assert_eq!(event.domain, EventDomain::Devices);
        assert!(event.cause.contains("usb_added"));
    }
}

/*
Full implementation would look like:

use udev::{MonitorBuilder, MonitorSocket};

async fn watch_udev(tx: mpsc::UnboundedSender<SystemEvent>) -> Result<()> {
    let socket = MonitorBuilder::new()?
        .match_subsystem("usb")?
        .match_subsystem("block")?
        .match_subsystem("net")?
        .match_subsystem("bluetooth")?
        .listen()?;

    for event in socket {
        let action = event.action().to_string();
        let devpath = event.devpath().display().to_string();
        let subsystem = event.subsystem().to_string();

        let cause = format!("{} {} ({})", action, devpath, subsystem);
        let event = create_event(EventDomain::Devices, cause);

        tx.send(event)?;
    }

    Ok(())
}
*/

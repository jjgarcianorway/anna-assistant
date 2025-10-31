// Anna v0.11.0 - Event Listeners Module
//
// Aggregates all event source listeners:
// - packages: Package manager changes (pacman database)
// - config: Configuration file drift (/etc)
// - devices: Device hotplug (USB, block, net) [placeholder]
// - network: Network interface changes [placeholder]
// - storage: Filesystem mount/unmount changes

pub mod config;
pub mod devices;
pub mod network;
pub mod packages;
pub mod storage;

use crate::events::SystemEvent;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::info;

/// Spawn all event listeners
pub fn spawn_all(tx: mpsc::UnboundedSender<SystemEvent>) -> Vec<JoinHandle<()>> {
    info!("Spawning all event listeners...");

    vec![
        packages::spawn_listener(tx.clone()),
        config::spawn_listener(tx.clone()),
        storage::spawn_listener(tx.clone()),
        devices::spawn_listener(tx.clone()),
        network::spawn_listener(tx),
    ]
}

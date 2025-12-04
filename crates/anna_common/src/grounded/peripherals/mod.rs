//! Peripheral Discovery v7.25.0
//!
//! Discovers USB, Bluetooth, Thunderbolt, FireWire, SD cards, cameras, input devices.
//! All data from real system commands: lsusb, rfkill, /sys, lspci, v4l2-ctl.
//!
//! Split into submodules for maintainability (< 400 lines each):
//! - types: All struct and enum definitions
//! - usb: USB device and controller discovery
//! - bluetooth: Bluetooth adapter discovery
//! - thunderbolt: Thunderbolt controller/device discovery
//! - sdcard: SD/MMC card reader discovery
//! - multimedia: Camera and audio discovery
//! - input: Input device discovery (keyboard, mouse, touchpad)
//! - overview: Hardware overview aggregation

pub mod bluetooth;
pub mod input;
pub mod multimedia;
pub mod overview;
pub mod sdcard;
pub mod thunderbolt;
pub mod types;
pub mod usb;

// Re-export all public types
pub use bluetooth::*;
pub use input::*;
pub use multimedia::*;
pub use overview::*;
pub use sdcard::*;
pub use thunderbolt::*;
pub use types::*;
pub use usb::*;

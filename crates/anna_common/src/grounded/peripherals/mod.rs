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

pub mod types;
pub mod usb;
pub mod bluetooth;
pub mod thunderbolt;
pub mod sdcard;
pub mod multimedia;
pub mod input;
pub mod overview;

// Re-export all public types
pub use types::*;
pub use usb::*;
pub use bluetooth::*;
pub use thunderbolt::*;
pub use sdcard::*;
pub use multimedia::*;
pub use input::*;
pub use overview::*;

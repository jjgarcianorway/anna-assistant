//! Peripheral type definitions v7.25.0

/// USB peripheral device with detailed info (distinct from drivers::UsbDevice)
#[derive(Debug, Clone)]
pub struct PeripheralUsbDevice {
    pub bus: u32,
    pub device: u32,
    pub vendor_id: String,
    pub product_id: String,
    pub vendor_name: String,
    pub product_name: String,
    pub speed: String,
    pub driver: Option<String>,
    pub path: String,
    pub device_class: String,
    pub power_ma: Option<u32>,
    pub is_hub: bool,
}

/// USB controller (root hub)
#[derive(Debug, Clone)]
pub struct UsbController {
    pub pci_address: String,
    pub name: String,
    pub driver: String,
    pub usb_version: String,
}

/// USB summary for overview
#[derive(Debug, Clone)]
pub struct UsbSummary {
    pub root_hubs: u32,
    pub device_count: u32,
    pub controllers: Vec<UsbController>,
    pub devices: Vec<PeripheralUsbDevice>,
    pub source: String,
}

/// Bluetooth adapter information
#[derive(Debug, Clone)]
pub struct BluetoothAdapter {
    pub name: String,
    pub address: String,
    pub manufacturer: String,
    pub driver: String,
    pub state: BluetoothState,
    pub powered: bool,
    pub discoverable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BluetoothState {
    Up,
    Down,
    Blocked,
    Unknown,
}

impl BluetoothState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Down => "DOWN",
            Self::Blocked => "BLOCKED",
            Self::Unknown => "unknown",
        }
    }
}

/// Bluetooth summary
#[derive(Debug, Clone)]
pub struct BluetoothSummary {
    pub adapter_count: u32,
    pub adapters: Vec<BluetoothAdapter>,
    pub source: String,
}

/// Thunderbolt controller
#[derive(Debug, Clone)]
pub struct ThunderboltController {
    pub name: String,
    pub pci_address: String,
    pub driver: String,
    pub generation: Option<u32>,
}

/// Thunderbolt device
#[derive(Debug, Clone)]
pub struct ThunderboltDevice {
    pub name: String,
    pub vendor: String,
    pub device_type: String,
    pub authorized: bool,
}

/// Thunderbolt summary
#[derive(Debug, Clone)]
pub struct ThunderboltSummary {
    pub controller_count: u32,
    pub device_count: u32,
    pub controllers: Vec<ThunderboltController>,
    pub devices: Vec<ThunderboltDevice>,
    pub source: String,
}

/// FireWire controller
#[derive(Debug, Clone)]
pub struct FirewireController {
    pub name: String,
    pub pci_address: String,
    pub driver: String,
}

/// FireWire summary
#[derive(Debug, Clone)]
pub struct FirewireSummary {
    pub controller_count: u32,
    pub device_count: u32,
    pub controllers: Vec<FirewireController>,
    pub source: String,
}

/// SD/Memory card reader
#[derive(Debug, Clone)]
pub struct SdCardReader {
    pub name: String,
    pub driver: String,
    pub bus: String,
    pub device_path: Option<String>,
    pub media_present: bool,
    pub media_size: Option<u64>,
    pub media_fs: Option<String>,
}

/// SD card summary
#[derive(Debug, Clone)]
pub struct SdCardSummary {
    pub reader_count: u32,
    pub readers: Vec<SdCardReader>,
    pub source: String,
}

/// Camera device
#[derive(Debug, Clone)]
pub struct CameraDevice {
    pub name: String,
    pub device_path: String,
    pub driver: String,
    pub capabilities: Vec<String>,
    pub bus: String,
}

/// Camera summary
#[derive(Debug, Clone)]
pub struct CameraSummary {
    pub camera_count: u32,
    pub cameras: Vec<CameraDevice>,
    pub source: String,
}

/// Input device
#[derive(Debug, Clone)]
pub struct InputDevice {
    pub name: String,
    pub device_type: InputType,
    pub handlers: Vec<String>,
    pub event_path: Option<String>,
    pub bus: String,
    pub vendor: String,
    pub product: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Keyboard,
    Mouse,
    Touchpad,
    Touchscreen,
    Tablet,
    Gamepad,
    Other,
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Touchpad => "touchpad",
            Self::Touchscreen => "touchscreen",
            Self::Tablet => "tablet",
            Self::Gamepad => "gamepad",
            Self::Other => "other",
        }
    }
}

/// Input device summary
#[derive(Debug, Clone)]
pub struct InputSummary {
    pub device_count: u32,
    pub keyboard_count: u32,
    pub mouse_count: u32,
    pub touchpad_count: u32,
    pub other_count: u32,
    pub devices: Vec<InputDevice>,
    pub source: String,
}

/// Audio card information
#[derive(Debug, Clone)]
pub struct AudioCard {
    pub card_num: u32,
    pub name: String,
    pub driver: String,
    pub card_type: String,
    pub has_playback: bool,
    pub has_capture: bool,
}

/// Audio summary
#[derive(Debug, Clone)]
pub struct AudioSummary {
    pub card_count: u32,
    pub cards: Vec<AudioCard>,
    pub source: String,
}

/// Complete hardware overview for annactl hw
#[derive(Debug, Clone)]
pub struct HardwareOverview {
    pub cpu_sockets: u32,
    pub cpu_logical_cores: u32,
    pub gpu_discrete: u32,
    pub gpu_integrated: u32,
    pub memory_gib: f64,
    pub storage_devices: u32,
    pub storage_names: Vec<String>,
    pub network_wired: u32,
    pub network_wireless: u32,
    pub network_interfaces: Vec<String>, // v7.35.1: Interface names for AVAILABLE QUERIES
    pub bluetooth: BluetoothSummary,
    pub usb: UsbSummary,
    pub audio: AudioSummary,
    pub camera: CameraSummary,
    pub input: InputSummary,
    pub battery_count: u32,
    pub ac_present: bool,
    pub firewire: FirewireSummary,
    pub thunderbolt: ThunderboltSummary,
    pub sdcard: SdCardSummary,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluetooth_state() {
        assert_eq!(BluetoothState::Up.as_str(), "UP");
        assert_eq!(BluetoothState::Blocked.as_str(), "BLOCKED");
    }

    #[test]
    fn test_input_type() {
        assert_eq!(InputType::Keyboard.as_str(), "keyboard");
        assert_eq!(InputType::Touchpad.as_str(), "touchpad");
    }
}

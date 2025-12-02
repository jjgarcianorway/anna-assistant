//! Input Device Discovery v7.25.0
//!
//! Discovers input devices from /proc/bus/input/devices.

use super::types::{InputDevice, InputType, InputSummary};

/// Get input device summary
pub fn get_input_summary() -> InputSummary {
    let mut summary = InputSummary {
        device_count: 0,
        keyboard_count: 0,
        mouse_count: 0,
        touchpad_count: 0,
        other_count: 0,
        devices: Vec::new(),
        source: "/proc/bus/input/devices".to_string(),
    };

    let input_path = "/proc/bus/input/devices";
    if let Ok(content) = std::fs::read_to_string(input_path) {
        let mut current_device: Option<InputDevice> = None;
        let mut current_handlers = Vec::new();
        let mut current_name = String::new();
        let mut current_bus = String::new();
        let mut current_vendor = String::new();
        let mut current_product = String::new();

        for line in content.lines() {
            if line.starts_with("N: Name=") {
                if let Some(mut dev) = current_device.take() {
                    dev.handlers = current_handlers.clone();
                    summary.devices.push(dev);
                }

                current_name = line.trim_start_matches("N: Name=")
                    .trim_matches('"')
                    .to_string();
                current_handlers.clear();
            } else if line.starts_with("I: ") {
                for part in line.trim_start_matches("I: ").split_whitespace() {
                    if part.starts_with("Bus=") {
                        current_bus = part.trim_start_matches("Bus=").to_string();
                    } else if part.starts_with("Vendor=") {
                        current_vendor = part.trim_start_matches("Vendor=").to_string();
                    } else if part.starts_with("Product=") {
                        current_product = part.trim_start_matches("Product=").to_string();
                    }
                }
            } else if line.starts_with("H: Handlers=") {
                current_handlers = line.trim_start_matches("H: Handlers=")
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                let device_type = classify_input_device(&current_name, &current_handlers);

                let event_path = current_handlers.iter()
                    .find(|h| h.starts_with("event"))
                    .map(|e| format!("/dev/input/{}", e));

                current_device = Some(InputDevice {
                    name: current_name.clone(),
                    device_type,
                    handlers: Vec::new(),
                    event_path,
                    bus: format_input_bus(&current_bus),
                    vendor: current_vendor.clone(),
                    product: current_product.clone(),
                });
            } else if line.is_empty() {
                if let Some(mut dev) = current_device.take() {
                    dev.handlers = current_handlers.clone();
                    summary.devices.push(dev);
                }
                current_handlers.clear();
            }
        }

        // Don't forget the last device
        if let Some(mut dev) = current_device.take() {
            dev.handlers = current_handlers;
            summary.devices.push(dev);
        }
    }

    // Filter out pseudo-devices
    let skip_patterns = ["sleep", "power", "video", "lid", "wmi", "button", "pc speaker"];
    summary.devices.retain(|dev| {
        let name_lower = dev.name.to_lowercase();
        !skip_patterns.iter().any(|p| name_lower.contains(p))
    });

    // Count by type
    for dev in &summary.devices {
        match dev.device_type {
            InputType::Keyboard => summary.keyboard_count += 1,
            InputType::Mouse => summary.mouse_count += 1,
            InputType::Touchpad => summary.touchpad_count += 1,
            _ => summary.other_count += 1,
        }
    }

    summary.device_count = summary.devices.len() as u32;
    summary
}

pub fn classify_input_device(name: &str, handlers: &[String]) -> InputType {
    let name_lower = name.to_lowercase();
    let has_kbd = handlers.iter().any(|h| h == "kbd");
    let has_mouse = handlers.iter().any(|h| h.contains("mouse"));

    if name_lower.contains("touchpad") || name_lower.contains("trackpad") {
        InputType::Touchpad
    } else if name_lower.contains("touchscreen") {
        InputType::Touchscreen
    } else if name_lower.contains("tablet") || name_lower.contains("wacom") {
        InputType::Tablet
    } else if name_lower.contains("gamepad") || name_lower.contains("joystick") ||
              name_lower.contains("controller") {
        InputType::Gamepad
    } else if has_mouse && !has_kbd {
        InputType::Mouse
    } else if has_kbd && (name_lower.contains("keyboard") || name_lower.contains("at translated")) {
        InputType::Keyboard
    } else if has_kbd {
        InputType::Other
    } else if has_mouse {
        InputType::Mouse
    } else {
        InputType::Other
    }
}

pub fn format_input_bus(bus: &str) -> String {
    match bus {
        "0003" => "USB".to_string(),
        "0005" => "Bluetooth".to_string(),
        "0011" => "I8042".to_string(),
        "0018" => "I2C".to_string(),
        "0019" => "Internal".to_string(),
        _ => format!("Bus {}", bus),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_bus_format() {
        assert_eq!(format_input_bus("0003"), "USB");
        assert_eq!(format_input_bus("0005"), "Bluetooth");
    }

    #[test]
    fn test_classify_input_device() {
        let handlers = vec!["kbd".to_string(), "event0".to_string()];
        assert_eq!(classify_input_device("AT Translated Set 2 keyboard", &handlers), InputType::Keyboard);

        let mouse_handlers = vec!["mouse0".to_string(), "event1".to_string()];
        assert_eq!(classify_input_device("Logitech Receiver", &mouse_handlers), InputType::Mouse);

        let tp_handlers = vec!["mouse1".to_string(), "event2".to_string()];
        assert_eq!(classify_input_device("SynPS/2 Synaptics TouchPad", &tp_handlers), InputType::Touchpad);
    }
}

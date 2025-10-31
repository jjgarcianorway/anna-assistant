//! Autonomic Manager - Self-Regulating Thermal and Power Control
//!
//! This module implements Anna's autonomous decision-making for thermal
//! and power management. It transitions between states based on telemetry
//! and executes appropriate actions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_FILE: &str = "/run/anna/state.json";

/// Thermal state based on CPU temperature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalState {
    Cool,   // < 55°C - Optimal, quiet operation
    Warm,   // 55-75°C - Normal under load
    Hot,    // > 75°C - Needs active cooling
}

/// Power state based on battery level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerState {
    Normal,      // > 30% battery or plugged in
    LowBattery,  // < 30% battery - conserve power
}

/// Action type for autonomic execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    IncreaseFan,   // Switch to performance profile
    BalanceMode,   // Switch to balanced profile
    QuietMode,     // Switch to quiet profile
    Throttle,      // Disable CPU turbo
    Unthrottle,    // Enable CPU turbo
    PowerSave,     // Switch to power-saver
    PowerBalanced, // Switch to balanced power
}

impl ActionType {
    /// Get the command to execute this action
    pub fn command(&self) -> Vec<&'static str> {
        match self {
            ActionType::IncreaseFan => vec!["asusctl", "profile", "-P", "performance"],
            ActionType::BalanceMode => vec!["asusctl", "profile", "-P", "balanced"],
            ActionType::QuietMode => vec!["asusctl", "profile", "-P", "quiet"],
            ActionType::Throttle => vec!["bash", "-c", "echo 1 > /sys/devices/system/cpu/intel_pstate/no_turbo"],
            ActionType::Unthrottle => vec!["bash", "-c", "echo 0 > /sys/devices/system/cpu/intel_pstate/no_turbo"],
            ActionType::PowerSave => vec!["powerprofilesctl", "set", "power-saver"],
            ActionType::PowerBalanced => vec!["powerprofilesctl", "set", "balanced"],
        }
    }

    /// Get human-readable explanation for this action
    pub fn explanation(&self) -> &'static str {
        match self {
            ActionType::IncreaseFan => "CPU temperature high; increasing cooling to prevent thermal throttling",
            ActionType::BalanceMode => "CPU temperature elevated; applying balanced thermal profile",
            ActionType::QuietMode => "CPU temperature optimal; reducing fan noise",
            ActionType::Throttle => "Thermal emergency; disabling CPU turbo to reduce heat generation",
            ActionType::Unthrottle => "Temperature normalized; restoring CPU turbo capability",
            ActionType::PowerSave => "Battery low or temperature high; conserving power",
            ActionType::PowerBalanced => "Power situation normalized; restoring balanced power profile",
        }
    }
}

/// Action execution outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutcome {
    pub action: ActionType,
    pub success: bool,
    pub timestamp: u64,
    pub explanation: String,
    pub error: Option<String>,
}

/// Complete autonomic state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicState {
    pub thermal_state: ThermalState,
    pub power_state: PowerState,
    pub last_action: Option<ActionType>,
    pub last_action_timestamp: u64,
    pub cpu_temp: f32,
    pub battery_percent: Option<f32>,
    pub fan_speed_percent: Option<f32>,
}

impl AutonomicState {
    /// Create new state from telemetry
    pub fn from_telemetry(
        cpu_temp: f32,
        battery_percent: Option<f32>,
        fan_speed_percent: Option<f32>,
    ) -> Self {
        let thermal_state = if cpu_temp < 55.0 {
            ThermalState::Cool
        } else if cpu_temp < 75.0 {
            ThermalState::Warm
        } else {
            ThermalState::Hot
        };

        let power_state = if let Some(battery) = battery_percent {
            if battery < 30.0 {
                PowerState::LowBattery
            } else {
                PowerState::Normal
            }
        } else {
            PowerState::Normal // Assume plugged in if no battery
        };

        Self {
            thermal_state,
            power_state,
            last_action: None,
            last_action_timestamp: 0,
            cpu_temp,
            battery_percent,
            fan_speed_percent,
        }
    }

    /// Load state from disk
    pub fn load() -> Result<Self> {
        if !Path::new(STATE_FILE).exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(STATE_FILE)
            .context("Failed to read state file")?;

        serde_json::from_str(&content)
            .context("Failed to parse state JSON")
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize state")?;

        // Ensure directory exists
        if let Some(parent) = Path::new(STATE_FILE).parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(STATE_FILE, json)
            .context("Failed to write state file")
    }

    /// Determine what action should be taken based on current state
    pub fn decide_action(&self) -> Option<ActionType> {
        // Priority 1: Low battery - always conserve power
        if self.power_state == PowerState::LowBattery {
            return Some(ActionType::PowerSave);
        }

        // Priority 2: Thermal management
        match self.thermal_state {
            ThermalState::Cool => {
                // Cool: Prefer quiet operation
                Some(ActionType::QuietMode)
            }
            ThermalState::Warm => {
                // Warm: Balanced cooling
                Some(ActionType::BalanceMode)
            }
            ThermalState::Hot => {
                // Hot: Maximum cooling
                Some(ActionType::IncreaseFan)
            }
        }
    }

    /// Check if enough time has passed since last action (rate limiting)
    pub fn can_execute_action(&self, action: &ActionType) -> bool {
        // Allow action if no previous action
        if self.last_action.is_none() {
            return true;
        }

        // Skip if same action was taken recently (< 60s ago)
        if let Some(last) = &self.last_action {
            if last == action {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now - self.last_action_timestamp < 60 {
                    return false; // Debounce: same action too soon
                }
            }
        }

        true
    }

    /// Update state after action execution
    pub fn record_action(&mut self, action: ActionType) {
        self.last_action = Some(action);
        self.last_action_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

impl Default for AutonomicState {
    fn default() -> Self {
        Self {
            thermal_state: ThermalState::Cool,
            power_state: PowerState::Normal,
            last_action: None,
            last_action_timestamp: 0,
            cpu_temp: 50.0,
            battery_percent: None,
            fan_speed_percent: None,
        }
    }
}

/// Execute an action and return the outcome
pub fn execute_action(action: ActionType) -> ActionOutcome {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let explanation = action.explanation().to_string();
    let command = action.command();

    // Try to execute the command
    let result = if command[0] == "bash" {
        // Special handling for bash commands (need sudo)
        Command::new("sudo")
            .args(&command)
            .output()
    } else {
        // Direct command execution
        Command::new(command[0])
            .args(&command[1..])
            .output()
    };

    match result {
        Ok(output) if output.status.success() => {
            ActionOutcome {
                action,
                success: true,
                timestamp,
                explanation,
                error: None,
            }
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            ActionOutcome {
                action,
                success: false,
                timestamp,
                explanation,
                error: Some(error),
            }
        }
        Err(e) => {
            ActionOutcome {
                action,
                success: false,
                timestamp,
                explanation,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Main autonomic control loop iteration
pub fn autonomic_iteration(
    cpu_temp: f32,
    battery_percent: Option<f32>,
    fan_speed_percent: Option<f32>,
) -> Result<Option<ActionOutcome>> {
    // Load previous state
    let mut state = AutonomicState::load().unwrap_or_else(|_| {
        AutonomicState::from_telemetry(cpu_temp, battery_percent, fan_speed_percent)
    });

    // Update current telemetry
    state.cpu_temp = cpu_temp;
    state.battery_percent = battery_percent;
    state.fan_speed_percent = fan_speed_percent;

    // Determine thermal and power states
    state.thermal_state = if cpu_temp < 55.0 {
        ThermalState::Cool
    } else if cpu_temp < 75.0 {
        ThermalState::Warm
    } else {
        ThermalState::Hot
    };

    state.power_state = if let Some(battery) = battery_percent {
        if battery < 30.0 {
            PowerState::LowBattery
        } else {
            PowerState::Normal
        }
    } else {
        PowerState::Normal
    };

    // Decide what action to take
    let action = match state.decide_action() {
        Some(action) => action,
        None => {
            state.save()?;
            return Ok(None); // No action needed
        }
    };

    // Check rate limiting
    if !state.can_execute_action(&action) {
        state.save()?;
        return Ok(None); // Skip due to debounce
    }

    // Execute the action
    let outcome = execute_action(action.clone());

    // Record the action
    state.record_action(action);
    state.save()?;

    Ok(Some(outcome))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_state_transitions() {
        let cool = AutonomicState::from_telemetry(45.0, None, None);
        assert_eq!(cool.thermal_state, ThermalState::Cool);

        let warm = AutonomicState::from_telemetry(65.0, None, None);
        assert_eq!(warm.thermal_state, ThermalState::Warm);

        let hot = AutonomicState::from_telemetry(85.0, None, None);
        assert_eq!(hot.thermal_state, ThermalState::Hot);
    }

    #[test]
    fn test_power_state_low_battery() {
        let low = AutonomicState::from_telemetry(50.0, Some(25.0), None);
        assert_eq!(low.power_state, PowerState::LowBattery);

        let normal = AutonomicState::from_telemetry(50.0, Some(50.0), None);
        assert_eq!(normal.power_state, PowerState::Normal);
    }

    #[test]
    fn test_action_debounce() {
        let mut state = AutonomicState::default();

        // First action should be allowed
        assert!(state.can_execute_action(&ActionType::QuietMode));

        // Record the action
        state.record_action(ActionType::QuietMode);

        // Same action should be blocked immediately
        assert!(!state.can_execute_action(&ActionType::QuietMode));

        // Different action should be allowed
        assert!(state.can_execute_action(&ActionType::BalanceMode));
    }

    #[test]
    fn test_action_priority() {
        // Low battery should override thermal state
        let low_battery_hot = AutonomicState::from_telemetry(85.0, Some(20.0), None);
        assert_eq!(low_battery_hot.decide_action(), Some(ActionType::PowerSave));

        // Normal battery should follow thermal state
        let hot = AutonomicState::from_telemetry(85.0, Some(60.0), None);
        assert_eq!(hot.decide_action(), Some(ActionType::IncreaseFan));
    }
}

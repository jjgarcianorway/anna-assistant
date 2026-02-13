//! System state investigation for config plan generation.

use anna_shared::exposure::ExposureGate;
use anna_shared::rpc::{DialogueStep, StepType};
use anyhow::Result;
use tracing::{info, warn};

use super::criteria::IterationState;
use super::streaming_helpers::push_and_send;

/// Investigate system state for config plan generation.
/// Gathers critical system information that plans need (kernel params, UEFI/BIOS, devices, etc.)
pub async fn investigate_system_state<W: tokio::io::AsyncWriteExt + Unpin>(
    state: &mut IterationState,
    dialogue: &mut Vec<DialogueStep>,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<String> {
    use std::process::Command;

    let mut system_info = String::new();

    // Critical commands for plan generation
    // v0.3.141: Enhanced bootloader detection - know ACTUAL state before replacing
    let investigation_commands: Vec<(&str, &str)> = vec![
        ("cat /proc/cmdline", "Current kernel parameters"),
        ("[ -d /sys/firmware/efi ] && echo 'UEFI' || echo 'BIOS'", "Boot mode"),
        ("efibootmgr 2>/dev/null || echo 'N/A'", "Current boot entries"),
        ("findmnt -n -o UUID /", "Root filesystem UUID"),
        ("findmnt -n -o SOURCE,FSTYPE /", "Root filesystem type"),
        ("lsblk -ndo pkname $(findmnt -n -o SOURCE /boot 2>/dev/null) 2>/dev/null || echo 'N/A'", "Boot device"),
        ("uname -r", "Kernel version"),
        ("cat /etc/os-release | grep PRETTY_NAME", "OS version"),
        // v0.3.141: Bootloader detection - CRITICAL for replacement operations
        ("[ -d /boot/grub ] && echo 'GRUB detected' || echo 'No GRUB'", "GRUB installation check"),
        ("[ -d /boot/loader ] && echo 'systemd-boot detected' || echo 'No systemd-boot'", "systemd-boot installation check"),
        ("[ -f /boot/refind_linux.conf ] && echo 'rEFInd detected' || echo 'No rEFInd'", "rEFInd installation check"),
        ("ls -la /boot/ 2>/dev/null | head -20", "Boot directory contents"),
        ("[ -f /boot/grub/grub.cfg ] && echo 'GRUB config exists' || echo 'N/A'", "GRUB config check"),
        ("[ -f /boot/loader/loader.conf ] && echo 'systemd-boot config exists' || echo 'N/A'", "systemd-boot config check"),
    ];

    for (cmd, description) in investigation_commands {
        push_and_send(writer, dialogue, StepType::InvestigationProbe,
            format!("Checking: {}", description), gate).await?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let output_clean = String::from_utf8_lossy(&result.stdout).trim().to_string();
                system_info.push_str(&format!("{}: {}\n", description, output_clean));
                state.commands.push(cmd.to_string());
                state.outputs.push(format!("[{}] {}", description, output_clean));
                push_and_send(writer, dialogue, StepType::InvestigationProbe,
                    format!("✓ {}", output_clean), gate).await?;
            }
            _ => {
                warn!("Investigation command failed: {}", cmd);
                system_info.push_str(&format!("{}: (failed to retrieve)\n", description));
            }
        }
    }

    info!("System investigation complete:\n{}", system_info);
    Ok(system_info)
}

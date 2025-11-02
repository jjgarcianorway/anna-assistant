# Hardware Profile Collection - Arch Advisor Phase 1

## Overview

The hardware profile collector creates a semantic fingerprint of your Arch Linux system's hardware configuration. It gathers information about CPU, memory, GPUs, storage, network, and other devices to enable intelligent system management and package recommendations.

**Version:** 1.0 (Schema v1)
**Status:** Phase 1 - Foundation
**Added in:** Anna v0.12.3

## What is Collected

### System Information
- **Kernel version** - Linux kernel release (`uname -r`)
- **Board info** - Motherboard vendor, product name, BIOS date
- **Battery presence** - Detects laptops vs desktops

### CPU
- Model name (e.g., "AMD Ryzen 9 5900X")
- Socket count (physical CPUs)
- Total core count (logical processors)

**Source:** `/proc/cpuinfo`

### Memory
- Total system RAM in GB

**Source:** `/proc/meminfo`

### GPUs
- Device name
- Vendor (NVIDIA, AMD, Intel)
- Kernel driver in use
- VRAM (if available from nvidia-smi)

**Sources:**
- `nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader`
- `lspci -mm -v` for all GPUs (fallback and non-NVIDIA)

### Storage
- **Controllers:** NVMe, SATA, RAID controllers with vendor/driver
- **Block devices:**
  - Device name (nvme0n1, sda, etc)
  - Size in GB
  - Type classification (SSD, HDD, NVMe)
  - Model string
  - Mounted filesystems

**Sources:**
- `lspci -mm -v` for controllers
- `lsblk -J -o NAME,SIZE,ROTA,TYPE,TRAN,MODEL,MOUNTPOINT,FSTYPE` for block devices

### Network
- Ethernet and wireless NICs
- Vendor and device model
- Kernel driver in use

**Source:** `lspci -mm -v`

### USB
- Connected USB devices (basic enumeration)

**Source:** `lsusb`

## Privacy

All data is collected **locally** with no network calls. The hardware profile contains:

- **No IP addresses**
- **No MAC addresses**
- **No serial numbers**
- **No user data**

Only hardware model names, vendors, and capacities are recorded.

## Usage

### CLI Commands

```bash
# Show human-readable hardware summary
annactl hw show

# Output as JSON
annactl hw show --json

# Show detailed device information
annactl hw show --wide
```

### Example Human Output

```
Hardware Profile
────────────────────────────────────────
CPU: AMD Ryzen 9 5900X 16 cores
Memory: 32.0 GB
GPU: NVIDIA RTX 3080 driver 545.29.06 vram 10024 MB
Storage: nvme0n1 1000 GB NVMe Samsung 980 Pro, sda 2000 GB HDD
Network: Intel I225-V ethernet driver igc, Intel AX200 wifi driver iwlwifi
Battery: yes (count 1)
Kernel: 6.17.6-arch1-1
Board: ASUSTeK COMPUTER INC. ROG Strix G15
```

### Example JSON Output

```json
{
  "version": "1",
  "generated_at": "2025-11-02T10:30:00Z",
  "kernel": "6.17.6-arch1-1",
  "board": {
    "vendor": "ASUSTeK COMPUTER INC.",
    "product": "ROG Strix G15",
    "bios_date": "08/12/2023"
  },
  "cpu": {
    "model": "AMD Ryzen 9 5900X 12-Core Processor",
    "sockets": 1,
    "cores_total": 16
  },
  "memory": {
    "total_gb": 31.3
  },
  "battery": {
    "present": true,
    "count": 1
  },
  "gpus": [
    {
      "name": "NVIDIA RTX 3080",
      "vendor": "NVIDIA",
      "driver": "545.29.06",
      "vram_mb": 10240,
      "notes": []
    }
  ],
  "network": [
    {
      "class": "ethernet",
      "vendor": "Intel Corporation",
      "device": "I225-V",
      "driver": "igc"
    }
  ],
  "storage": {
    "controller": [
      {
        "vendor": "Samsung Electronics Co Ltd",
        "device": "NVMe SSD Controller PM9A1/PM9A3/980PRO",
        "driver": "nvme",
        "type": "nvme"
      }
    ],
    "block_devices": [
      {
        "name": "nvme0n1",
        "model": "Samsung SSD 980 PRO 1TB",
        "size_gb": 953.9,
        "rotational": false,
        "type": "nvme",
        "mounts": [
          {
            "mountpoint": "/",
            "fs": "ext4"
          }
        ]
      }
    ]
  },
  "usb": [],
  "notes": []
}
```

## Requirements

### Required Tools
- `uname` (coreutils) - kernel version
- `lsblk` (util-linux) - block device topology

### Optional Tools
These tools enhance data collection but are not required:

- `lspci` (pciutils) - PCI device enumeration (GPUs, NICs, storage)
- `lsusb` (usbutils) - USB device enumeration
- `dmidecode` - motherboard info (requires root, falls back to `/sys/class/dmi/id/`)
- `nvidia-smi` - NVIDIA GPU details (VRAM, driver version)

If a tool is missing, the collector gracefully skips that section and continues.

## Performance

- **Typical collection time:** 200-500ms (warm)
- **Cold run (first boot):** 1-2 seconds
- **Timeout per tool:** 2 seconds
- **Overall timeout:** 5 seconds

Collections that exceed the overall timeout return partial data with a `"timeout"` note.

## Graceful Degradation

The hardware profile collector **never fails** due to missing tools or permissions. Each subsystem is optional:

| Issue | Behavior |
|-------|----------|
| `dmidecode` needs root | Falls back to `/sys/class/dmi/id/` |
| `nvidia-smi` not installed | Uses `lspci` for GPU detection |
| `lspci` not installed | GPU/network/storage sections empty |
| Command times out | Returns partial data, adds `"timeout"` note |
| No battery present | `battery.present = false` |

Fields that cannot be determined are set to `null`.

## Error Handling

All errors are logged to the systemd journal:

```bash
sudo journalctl -u annad | grep -i hardware
```

Example log entries:
- `INFO: Hardware profile collected in 342 ms`
- `WARN: Failed to get kernel: timeout`
- `DEBUG: Board info failed (may need root)`

## Integration

### RPC Endpoint

Direct access via JSON-RPC over the UNIX socket:

```bash
echo '{"jsonrpc":"2.0","method":"hardware_profile","params":{},"id":1}' \
  | socat - UNIX-CONNECT:/run/anna/annad.sock
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": { ...hardware profile... },
  "id": 1
}
```

### Programmatic Usage

From other Anna components:

```rust
use crate::hardware_profile::HardwareCollector;

let collector = HardwareCollector::new();
let profile = collector.collect().await?;
println!("CPU: {:?}", profile.cpu.model);
```

## Testing

### Unit Tests

```bash
cd ~/anna-assistant
cargo test --package annad --lib hardware_profile
```

### Integration Tests

```bash
sudo bash tests/runtime/hw_profile_smoke.sh
```

The smoke test validates:
1. Human output succeeds
2. JSON output is valid JSON
3. All required schema fields present
4. Schema version is "1"
5. Timestamp is RFC3339 format
6. Kernel field is non-empty
7. Data types match schema
8. `--wide` flag works
9. Performance (<5s cold, <1s warm)

## Troubleshooting

### No GPU detected

**Symptoms:** `gpus: []`

**Causes:**
- `lspci` not installed
- GPU is disabled in BIOS
- VM without GPU passthrough

**Solution:**
```bash
sudo pacman -S pciutils
annactl hw show
```

### Board info is null

**Symptoms:** `board.vendor: null, board.product: null`

**Causes:**
- Not running as root
- VM without DMI data
- `/sys/class/dmi/id/` not populated

**Solution:**
- Expected in VMs
- Run `sudo dmidecode -t baseboard` to verify DMI availability

### Collection is slow (>5s)

**Symptoms:** `notes: ["timeout"]`

**Causes:**
- Slow disk I/O
- Many USB devices
- `nvidia-smi` hanging

**Solution:**
```bash
# Check which tool is slow
time lspci -mm
time lsblk -J
time nvidia-smi --query-gpu=name --format=csv
```

### Missing storage devices

**Symptoms:** `storage.block_devices: []`

**Causes:**
- `lsblk` not installed (unlikely on Arch)
- No block devices mounted (e.g., live USB)

**Solution:**
```bash
lsblk -J
# Should show devices even if not mounted
```

## Schema Validation

The JSON output conforms to the JSON Schema at `docs/schemas/hardware_profile.schema.json`.

Validate manually:

```bash
annactl hw show --json > hw.json
ajv validate -s docs/schemas/hardware_profile.schema.json -d hw.json
```

(Requires `npm install -g ajv-cli`)

## Next Steps (Phase 2+)

Phase 1 provides the **foundation**. Future phases will add:

- **Package ecosystem advisor** - Recommend packages based on detected hardware
- **Driver recommendations** - Suggest optimal drivers for GPUs/NICs
- **Power profile tuning** - Laptop-specific optimizations
- **Hardware compatibility database** - Known-good configurations
- **Performance benchmarking** - Compare to similar systems

## References

- JSON Schema: `docs/schemas/hardware_profile.schema.json`
- Source code: `src/annad/src/hardware_profile.rs`
- CLI command: `src/annactl/src/hw_cmd.rs`
- RPC endpoint: `src/annad/src/rpc_v10.rs::method_hardware_profile`
- Smoke test: `tests/runtime/hw_profile_smoke.sh`

## Change Log

### v0.12.3 (2025-11-02)
- Initial implementation (Phase 1)
- Hardware fingerprinting with graceful degradation
- CLI command `annactl hw show`
- JSON schema v1
- Smoke tests

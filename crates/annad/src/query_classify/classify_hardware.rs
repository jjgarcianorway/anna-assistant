//! Hardware query classification patterns (v0.0.174).
//!
//! CPU, GPU, memory, disk, audio, sensors, USB, PCI, Bluetooth.

use crate::router::QueryClass;

/// Classify hardware queries.
/// Returns Some if matched, None otherwise.
pub fn classify_hardware(q: &str) -> Option<QueryClass> {
    // v0.45.4: InstalledToolCheck - "do I have nano", "is vim installed"
    // Exclude hardware queries (cpu, ram, memory, gpu, disk)
    let is_hardware_query = q.contains("cpu")
        || q.contains("ram")
        || q.contains("memory")
        || q.contains("gpu")
        || q.contains("disk")
        || q.contains("core");
    let is_tool_check_query = q.contains("do i have")
        || q.contains("do you have")
        || q.contains("have i got")
        || (q.contains("is") && q.contains("installed"))
        || (q.contains("have") && q.contains("installed"));
    if !is_hardware_query && is_tool_check_query {
        return Some(QueryClass::InstalledToolCheck);
    }

    // v0.0.45: HardwareAudio - "sound card", "audio device"
    if q.contains("sound card")
        || q.contains("audio device")
        || q.contains("audio card")
        || q.contains("sound device")
        || (q.contains("audio") && q.contains("hardware"))
    {
        return Some(QueryClass::HardwareAudio);
    }

    // v0.0.45: CpuTemp - "cpu temperature", "how hot is my cpu"
    if q.contains("temperature")
        || q.contains("temp")
        || q.contains("how hot")
        || q.contains("thermal")
        || q.contains("sensors")
    {
        return Some(QueryClass::CpuTemp);
    }

    // v0.0.45: CpuCores - "how many cores", "threads"
    if (q.contains("how many") && (q.contains("core") || q.contains("thread")))
        || q.contains("core count")
        || q.contains("thread count")
        || q.contains("number of cores")
        || q.contains("number of threads")
    {
        return Some(QueryClass::CpuCores);
    }

    // v0.0.45: MemoryFree - "free ram", "available ram"
    if (q.contains("free") && q.contains("ram"))
        || (q.contains("available") && q.contains("ram"))
        || q.contains("how much free ram")
        || q.contains("how much available ram")
        || q.contains("free memory")
        || q.contains("available memory")
        || q.contains("how much free memory")
        || q.contains("how much available memory")
    {
        return Some(QueryClass::MemoryFree);
    }

    // Memory usage (dynamic)
    if (q.contains("memory") && q.contains("usage")) || (q.contains("memory") && q.contains("used"))
    {
        return Some(QueryClass::MemoryUsage);
    }

    // Disk usage (dynamic)
    if q.contains("disk usage") || q.contains("filesystem usage") {
        return Some(QueryClass::DiskUsage);
    }

    // Top memory processes
    if (q.contains("process") && (q.contains("memory") || q.contains("ram")))
        || q.contains("memory hog")
        || q.contains("top memory")
        || q.contains("most memory")
        || q.contains("what's using memory")
        || q.contains("what is using memory")
    {
        return Some(QueryClass::TopMemoryProcesses);
    }

    // Top CPU processes
    if (q.contains("process") && q.contains("cpu"))
        || q.contains("cpu hog")
        || q.contains("top cpu")
        || q.contains("most cpu")
        || q.contains("what's using cpu")
        || q.contains("what is using cpu")
    {
        return Some(QueryClass::TopCpuProcesses);
    }

    // Hardware snapshot queries
    if q.contains("cpu") || q.contains("processor") || q.contains("core") {
        return Some(QueryClass::CpuInfo);
    }

    if q.contains("ram") || (q.contains("memory") && !q.contains("process")) {
        return Some(QueryClass::RamInfo);
    }

    if q.contains("gpu") || q.contains("graphics") || q.contains("vram") {
        return Some(QueryClass::GpuInfo);
    }

    // Disk space
    if q.contains("disk")
        || q.contains("space")
        || q.contains("storage")
        || q.contains("filesystem")
        || q.contains("mount")
        || q.contains("full")
    {
        return Some(QueryClass::DiskSpace);
    }

    // v0.0.124: USB devices
    if q.contains("usb device")
        || q.contains("usb")
        || q.contains("plugged in")
        || q.contains("connected device")
        || q.trim() == "lsusb"
        || (q.contains("what") && q.contains("plugged"))
    {
        return Some(QueryClass::UsbDevices);
    }

    // v0.0.127: Memory slots - "memory slots", "ram slots", "dimm"
    if q.contains("memory slot")
        || q.contains("ram slot")
        || q.contains("dimm")
        || q.contains("memory stick")
        || (q.contains("how many") && q.contains("ram") && q.contains("slot"))
    {
        return Some(QueryClass::MemorySlots);
    }

    // v0.0.127: CPU frequency - "cpu frequency", "clock speed"
    if q.contains("cpu freq")
        || q.contains("clock speed")
        || q.contains("cpu speed")
        || q.contains("processor speed")
        || q.contains("cpu mhz")
        || q.contains("cpu ghz")
        || (q.contains("how fast") && q.contains("cpu"))
    {
        return Some(QueryClass::CpuFrequency);
    }

    // v0.0.133: PCI devices - "lspci", "pci devices"
    if q.trim() == "lspci"
        || q.contains("pci device")
        || q.contains("pci card")
        || (q.contains("list") && q.contains("pci"))
        || (q.contains("show") && q.contains("pci"))
    {
        return Some(QueryClass::PciDevices);
    }

    // v0.0.134: Sensors temperature
    if q.trim() == "sensors"
        || (q.contains("sensor") && q.contains("temp"))
        || (q.contains("hardware") && q.contains("temp"))
        || q.contains("fan speed")
        || q.contains("thermal")
    {
        return Some(QueryClass::SensorsTemp);
    }

    // v0.0.134: GPU memory
    if q.contains("gpu memory")
        || q.contains("vram")
        || q.contains("nvidia-smi")
        || (q.contains("gpu") && q.contains("usage"))
        || (q.contains("graphics") && q.contains("memory"))
    {
        return Some(QueryClass::GpuMemory);
    }

    // v0.0.135: Bluetooth devices
    if q.contains("bluetooth")
        || q.contains("paired device")
        || q.trim() == "bluetoothctl"
        || (q.contains("bt") && q.contains("device"))
    {
        return Some(QueryClass::BluetoothDevices);
    }

    // v0.0.135: Printer status
    if q.contains("printer")
        || q.contains("cups")
        || q.trim() == "lpstat"
        || q.contains("print queue")
        || (q.contains("print") && q.contains("status"))
    {
        return Some(QueryClass::PrinterStatus);
    }

    // v0.0.135: Audio devices
    if q.contains("audio device")
        || q.contains("sound card")
        || q.contains("audio sink")
        || q.contains("audio source")
        || q.trim() == "pactl"
        || q.trim() == "aplay -l"
    {
        return Some(QueryClass::AudioDevices);
    }

    // v0.0.141: CPU governor
    if q.contains("cpu governor")
        || q.contains("scaling governor")
        || q.contains("frequency scaling")
        || q.contains("cpufreq")
        || (q.contains("power") && q.contains("governor"))
        || q.contains("performance mode")
    {
        return Some(QueryClass::CpuGovernor);
    }

    // v0.0.141: Loaded firmware
    if q.contains("firmware")
        || q.contains("microcode")
        || (q.contains("driver") && q.contains("load"))
        || q.contains("kernel firmware")
    {
        return Some(QueryClass::LoadedFirmware);
    }

    None
}

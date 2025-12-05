//! Parser for lscpu command output.
//!
//! Parses CPU information to typed structs.
//! Only parses stable, well-defined fields - no inference.

use super::atoms::{ParseError, ParseErrorReason};
use serde::{Deserialize, Serialize};

/// Parsed CPU information from lscpu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU architecture (e.g., "x86_64", "aarch64")
    pub architecture: String,
    /// CPU model name (e.g., "Intel(R) Core(TM) i9-14900HX")
    pub model_name: String,
    /// Total number of logical CPUs
    pub cpu_count: u32,
    /// Cores per socket (physical cores per CPU package)
    pub cores_per_socket: Option<u32>,
    /// Threads per core (hyperthreading factor)
    pub threads_per_core: Option<u32>,
    /// Number of sockets
    pub sockets: Option<u32>,
    /// CPU vendor ID
    pub vendor_id: Option<String>,
    /// CPU family
    pub cpu_family: Option<String>,
    /// CPU model number
    pub model: Option<String>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            architecture: String::new(),
            model_name: String::new(),
            cpu_count: 0,
            cores_per_socket: None,
            threads_per_core: None,
            sockets: None,
            vendor_id: None,
            cpu_family: None,
            model: None,
        }
    }
}

impl CpuInfo {
    /// Get the number of physical cores (if derivable)
    pub fn physical_cores(&self) -> Option<u32> {
        match (self.cores_per_socket, self.sockets) {
            (Some(cores), Some(sockets)) => Some(cores * sockets),
            _ => None,
        }
    }

    /// Check if hyperthreading is enabled
    pub fn hyperthreading(&self) -> Option<bool> {
        self.threads_per_core.map(|t| t > 1)
    }
}

/// Parse lscpu output to structured CPU info.
///
/// Expected format (key: value pairs):
/// ```text
/// Architecture:            x86_64
/// Model name:              Intel(R) Core(TM) i9-14900HX
/// CPU(s):                  32
/// Thread(s) per core:      2
/// Core(s) per socket:      24
/// Socket(s):               1
/// ```
pub fn parse_lscpu(probe_id: &str, output: &str) -> Result<CpuInfo, ParseError> {
    let mut info = CpuInfo::default();
    let mut found_any = false;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split on first colon
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Architecture" => {
                    info.architecture = value.to_string();
                    found_any = true;
                }
                "Model name" => {
                    info.model_name = value.to_string();
                    found_any = true;
                }
                "CPU(s)" => {
                    if let Ok(count) = value.parse() {
                        info.cpu_count = count;
                        found_any = true;
                    }
                }
                "Core(s) per socket" => {
                    if let Ok(cores) = value.parse() {
                        info.cores_per_socket = Some(cores);
                    }
                }
                "Thread(s) per core" => {
                    if let Ok(threads) = value.parse() {
                        info.threads_per_core = Some(threads);
                    }
                }
                "Socket(s)" => {
                    if let Ok(sockets) = value.parse() {
                        info.sockets = Some(sockets);
                    }
                }
                "Vendor ID" => {
                    info.vendor_id = Some(value.to_string());
                }
                "CPU family" => {
                    info.cpu_family = Some(value.to_string());
                }
                "Model" => {
                    // Avoid confusion with "Model name"
                    if !key.contains("name") {
                        info.model = Some(value.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    if !found_any {
        return Err(ParseError::new(
            probe_id,
            ParseErrorReason::MissingSection("no CPU information found".to_string()),
            output,
        ));
    }

    // Validate required fields
    if info.cpu_count == 0 {
        return Err(ParseError::new(
            probe_id,
            ParseErrorReason::MissingSection("CPU count not found".to_string()),
            output,
        ));
    }

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = r#"Architecture:                            x86_64
CPU op-mode(s):                          32-bit, 64-bit
Address sizes:                           39 bits physical, 48 bits virtual
Byte Order:                              Little Endian
CPU(s):                                  32
On-line CPU(s) list:                     0-31
Vendor ID:                               GenuineIntel
Model name:                              Intel(R) Core(TM) i9-14900HX
CPU family:                              6
Model:                                   183
Thread(s) per core:                      2
Core(s) per socket:                      24
Socket(s):                               1
"#;

    #[test]
    fn golden_lscpu_parse_basic() {
        let info = parse_lscpu("lscpu", SAMPLE_OUTPUT).unwrap();
        assert_eq!(info.architecture, "x86_64");
        assert_eq!(info.model_name, "Intel(R) Core(TM) i9-14900HX");
        assert_eq!(info.cpu_count, 32);
    }

    #[test]
    fn golden_lscpu_parse_topology() {
        let info = parse_lscpu("lscpu", SAMPLE_OUTPUT).unwrap();
        assert_eq!(info.threads_per_core, Some(2));
        assert_eq!(info.cores_per_socket, Some(24));
        assert_eq!(info.sockets, Some(1));
    }

    #[test]
    fn golden_lscpu_physical_cores() {
        let info = parse_lscpu("lscpu", SAMPLE_OUTPUT).unwrap();
        assert_eq!(info.physical_cores(), Some(24)); // 24 cores * 1 socket
    }

    #[test]
    fn golden_lscpu_hyperthreading() {
        let info = parse_lscpu("lscpu", SAMPLE_OUTPUT).unwrap();
        assert_eq!(info.hyperthreading(), Some(true));
    }

    #[test]
    fn golden_lscpu_vendor_info() {
        let info = parse_lscpu("lscpu", SAMPLE_OUTPUT).unwrap();
        assert_eq!(info.vendor_id, Some("GenuineIntel".to_string()));
        assert_eq!(info.cpu_family, Some("6".to_string()));
    }

    #[test]
    fn golden_lscpu_minimal_output() {
        let minimal = "Architecture: aarch64\nCPU(s): 8\nModel name: ARM Cortex-A72\n";
        let info = parse_lscpu("lscpu", minimal).unwrap();
        assert_eq!(info.architecture, "aarch64");
        assert_eq!(info.cpu_count, 8);
        assert!(info.cores_per_socket.is_none());
    }

    #[test]
    fn golden_lscpu_empty_output_error() {
        let result = parse_lscpu("lscpu", "");
        assert!(result.is_err());
    }

    #[test]
    fn golden_lscpu_no_cpu_count_error() {
        let result = parse_lscpu("lscpu", "Architecture: x86_64\n");
        assert!(result.is_err());
    }
}

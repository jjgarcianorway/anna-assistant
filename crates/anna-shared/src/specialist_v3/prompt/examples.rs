//! Example responses for specialists to follow.

/// Example success response for the specialist to follow.
pub fn example_success_response() -> &'static str {
    r#"{
  "ticket_id": "DSK-001",
  "specialist": {"name": "Sofia", "role": "System Admin", "department": "desktop"},
  "status": "success",
  "summary": "Memory usage is healthy at 33% with 17GB available",
  "confidence": 0.95,
  "severity": "info",
  "findings": [
    {"key": "mem_total_gb", "value": "25.6", "evidence_refs": ["probe:free"]},
    {"key": "mem_used_gb", "value": "8.4", "evidence_refs": ["probe:free"]},
    {"key": "mem_available_gb", "value": "17.0", "evidence_refs": ["probe:free"]},
    {"key": "swap_used_mb", "value": "0", "evidence_refs": ["probe:free"]}
  ],
  "analysis": [
    "Memory utilization at 33% is well within healthy range",
    "No swap usage indicates sufficient RAM",
    "17GB available provides good headroom for new applications"
  ],
  "recommendations": [],
  "actions": [],
  "knowledge_citations": [],
  "probes_used": [
    {"id": "probe:free", "status": "ok", "description": "Memory usage statistics"}
  ]
}"#
}

/// Example no-data response.
pub fn example_no_data_response() -> &'static str {
    r#"{
  "ticket_id": "DSK-002",
  "specialist": {"name": "Sofia", "role": "System Admin", "department": "desktop"},
  "status": "no_data",
  "summary": "No GPU information available - probe returned empty",
  "confidence": 0.1,
  "severity": "info",
  "findings": [],
  "analysis": [
    "lspci probe did not return GPU information",
    "This may indicate no discrete GPU or driver issues"
  ],
  "recommendations": [
    {"id": "rec-1", "title": "Check drivers", "description": "Verify GPU drivers are installed", "risk_level": "low"}
  ],
  "actions": [
    {"id": "act-1", "title": "List PCI devices", "command": "lspci -v | grep -i vga", "run_as": "user", "risk_level": "low"}
  ],
  "knowledge_citations": [],
  "probes_used": [
    {"id": "probe:lspci", "status": "empty", "description": "PCI device listing"}
  ]
}"#
}

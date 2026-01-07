use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// RPC protocol version for compatibility checking
pub const RPC_VERSION: u32 = 1;

/// Maximum message size (10MB)
const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024;

// ============================================================================
// Request/Response Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Status(StatusRequest),
    Quickscan(QuickscanRequest),
    AdviceList(AdviceListRequest),
    AdviceShow(AdviceShowRequest),
    Persona(PersonaRequest),
    PersonaSummary(PersonaSummaryRequest),
    Apply(ApplyRequest),
    DoctorPerms(DoctorPermsRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    Status(StatusResponse),
    Quickscan(QuickscanResponse),
    AdviceList(AdviceListResponse),
    AdviceShow(AdviceShowResponse),
    Persona(PersonaResponse),
    PersonaSummary(PersonaSummaryResponse),
    Apply(ApplyResponse),
    DoctorPerms(DoctorPermsResponse),
    Error(ErrorResponse),
}

// ============================================================================
// Status
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusRequest {
    pub uid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub mode: String,
    pub socket_path: String,
    pub user_data_dir: String,
    pub system_config_dir: String,
    pub service_state: String,
    pub last_quickscan_ts: Option<String>,
    pub advice_count: usize,
}

// ============================================================================
// Quickscan
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickscanRequest {
    pub uid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickscanResponse {
    pub started_at: String,
    pub finished_at: String,
    pub summary: QuickscanSummary,
    pub report_path: String,
    pub advice_count_seeded: usize,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickscanSummary {
    pub ok: usize,
    pub warn: usize,
    pub action: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub fix: Option<FixPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlan {
    pub summary: String,
    #[serde(default)]
    pub apply_cmds: Vec<String>,
    #[serde(default)]
    pub dry_run_cmds: Vec<String>,
    #[serde(default)]
    pub undo_cmds: Vec<String>,
}

// ============================================================================
// Advice
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceListRequest {
    pub uid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceListResponse {
    pub items: Vec<AdviceRecord>,
    pub ts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceRecord {
    pub id: String,
    pub kind: String,
    pub persona_hint: String,
    pub reason: String,
    pub created_at: String,
    #[serde(default)]
    pub severity: AdviceSeverity,
    pub plan: AdvicePlan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AdviceSeverity {
    #[default]
    Info,
    Warn,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvicePlan {
    #[serde(default)]
    pub dry_run_cmds: Vec<String>,
    #[serde(default)]
    pub apply_cmds: Vec<String>,
    #[serde(default)]
    pub undo_cmds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceShowRequest {
    pub uid: u32,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceShowResponse {
    pub advice: Option<AdviceRecord>,
}

// ============================================================================
// Persona
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaRequest {
    pub uid: u32,
    pub op: PersonaOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaOp {
    Show,
    Explain,
    Samples { date: String, tail: usize },
    Triggers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSummaryRequest {
    pub uid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSummaryResponse {
    pub persona: String,
    pub source: String,
    pub confidence: f32,
    pub traits: Vec<PersonaTrait>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaTrait {
    pub name: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

// ============================================================================
// Apply
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyMode {
    DryRun,
    Execute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyRequest {
    pub uid: u32,
    pub id: String,
    pub mode: ApplyMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResponse {
    pub status: String,
    pub message: String,
    pub requires_approval: bool,
    pub output: Option<String>,
}

// ============================================================================
// Doctor Perms
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorPermsRequest {
    pub uid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorPermsResponse {
    pub mode: String,
    pub issues: Vec<PermissionIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionIssue {
    pub path: String,
    pub issue: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

// ============================================================================
// Error
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
}

// ============================================================================
// Transport Protocol
// ============================================================================

/// Read a length-prefixed JSON message from a stream
pub fn read_message<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    // Read 4-byte length prefix (big-endian)
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf);

    // Validate length
    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {} bytes", len),
        ));
    }

    // Read message body
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Write a length-prefixed JSON message to a stream
pub fn write_message<W: Write>(writer: &mut W, data: &[u8]) -> io::Result<()> {
    if data.len() > MAX_MESSAGE_SIZE as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {} bytes", data.len()),
        ));
    }

    // Write 4-byte length prefix (big-endian)
    let len = data.len() as u32;
    writer.write_all(&len.to_be_bytes())?;

    // Write message body
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}

/// Deserialize a request from bytes
pub fn decode_request(data: &[u8]) -> Result<Request, serde_json::Error> {
    serde_json::from_slice(data)
}

/// Serialize a request to bytes
pub fn encode_request(req: &Request) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(req)
}

/// Deserialize a response from bytes
pub fn decode_response(data: &[u8]) -> Result<Response, serde_json::Error> {
    serde_json::from_slice(data)
}

/// Serialize a response to bytes
pub fn encode_response(resp: &Response) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_framing() {
        let data = b"hello world";
        let mut buf = Vec::new();

        // Write
        write_message(&mut buf, data).unwrap();

        // Read back
        let mut cursor = std::io::Cursor::new(&buf);
        let read_data = read_message(&mut cursor).unwrap();

        assert_eq!(data, &read_data[..]);
    }

    #[test]
    fn test_request_roundtrip() {
        let req = Request::Status(StatusRequest { uid: 1000 });
        let encoded = encode_request(&req).unwrap();
        let decoded = decode_request(&encoded).unwrap();

        match decoded {
            Request::Status(s) => assert_eq!(s.uid, 1000),
            _ => panic!("wrong request type"),
        }
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = Response::Status(StatusResponse {
            mode: "system".to_string(),
            socket_path: "/run/anna/annad.sock".to_string(),
            user_data_dir: "/var/lib/anna/users/1000".to_string(),
            system_config_dir: "/etc/anna".to_string(),
            service_state: "active".to_string(),
            last_quickscan_ts: None,
            advice_count: 0,
        });
        let encoded = encode_response(&resp).unwrap();
        let decoded = decode_response(&encoded).unwrap();

        match decoded {
            Response::Status(s) => assert_eq!(s.mode, "system"),
            _ => panic!("wrong response type"),
        }
    }

    #[test]
    fn test_max_message_size() {
        let data = vec![0u8; (MAX_MESSAGE_SIZE + 1) as usize];
        let mut buf = Vec::new();
        let result = write_message(&mut buf, &data);
        assert!(result.is_err());
    }
}

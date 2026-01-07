use anna_rpc::{decode_response, encode_request, read_message, write_message, Request, Response};
use anyhow::{Context, Result};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// RPC client for communicating with annad
pub struct RpcClient {
    socket_path: String,
}

impl RpcClient {
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().display().to_string(),
        }
    }

    /// Send a request and receive a response
    pub fn call(&self, request: Request) -> Result<Response> {
        // Connect to socket
        let mut stream = UnixStream::connect(&self.socket_path).with_context(|| {
            format!(
                "Failed to connect to annad at {}. Is the service running?",
                self.socket_path
            )
        })?;

        // Encode and send request
        let req_data = encode_request(&request).context("encode request")?;
        write_message(&mut stream, &req_data).context("send request")?;

        // Read and decode response
        let resp_data = read_message(&mut stream).context("read response")?;
        let response = decode_response(&resp_data).context("decode response")?;

        Ok(response)
    }
}

use crate::protocol::{
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpTool, ServerInfo,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpClientError {
    #[error("Child process I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization or deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("JSON-RPC error [{code}]: {message}")]
    JsonRpc { code: i64, message: String },
    #[error("Server returned invalid response: {0}")]
    Protocol(String),
    #[error("Timed out waiting for response from child process")]
    Timeout,
    #[error("Child process exited unexpectedly with code: {0:?}")]
    ProcessExited(Option<i32>),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

/// STDIO-based client that spawns and communicates with the .NET MCP Server child process
pub struct StdioMcpClient {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    request_id: AtomicU64,
    server_info: Option<ServerInfo>,
    cached_tools: HashMap<String, McpTool>,
}

impl StdioMcpClient {
    /// Launches the .NET MCP Server child process using `dotnet <dll_path>`
    pub fn spawn_dotnet<P: AsRef<Path>>(dll_path: P) -> Result<Self, McpClientError> {
        let abs_path: PathBuf = if dll_path.as_ref().is_relative() {
            std::env::current_dir()?.join(dll_path.as_ref())
        } else {
            dll_path.as_ref().to_path_buf()
        };

        let working_dir = abs_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new("."));

        let mut child = Command::new("dotnet")
            .arg(&abs_path)
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Route child diagnostics to client's STDERR, keeping STDOUT 100% clean
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpClientError::Protocol("Failed to open child process stdin".into()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpClientError::Protocol("Failed to open child process stdout".into()))?;

        let reader = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            reader,
            request_id: AtomicU64::new(1),
            server_info: None,
            cached_tools: HashMap::new(),
        })
    }

    /// Performs the mandatory MCP handshake:
    /// 1. Sends `initialize` request
    /// 2. Validates `initialize` response
    /// 3. Sends `notifications/initialized` notification
    pub fn initialize(&mut self) -> Result<ServerInfo, McpClientError> {
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": { "listChanged": false },
                "sampling": {}
            },
            "clientInfo": {
                "name": "rust-mcp-client",
                "version": "1.0.0"
            }
        });

        let response = self.send_request("initialize", Some(init_params))?;

        let result_obj = response
            .result
            .ok_or_else(|| McpClientError::Protocol("Missing result in initialize response".into()))?;

        let server_info: ServerInfo = serde_json::from_value(
            result_obj
                .get("serverInfo")
                .cloned()
                .unwrap_or(Value::Null),
        )
        .map_err(|e| McpClientError::Protocol(format!("Invalid serverInfo: {}", e)))?;

        self.server_info = Some(server_info.clone());

        // Send notifications/initialized (notification has no id)
        self.send_notification("notifications/initialized", None)?;

        Ok(server_info)
    }

    /// Dynamically discovers and caches tools from the MCP Server via `tools/list`
    pub fn discover_tools(&mut self) -> Result<Vec<McpTool>, McpClientError> {
        let response = self.send_request("tools/list", Some(json!({})))?;

        let result_obj = response
            .result
            .ok_or_else(|| McpClientError::Protocol("Missing result in tools/list response".into()))?;

        let tools_val = result_obj
            .get("tools")
            .ok_or_else(|| McpClientError::Protocol("Missing 'tools' array in tools/list".into()))?;

        let tools: Vec<McpTool> = serde_json::from_value(tools_val.clone())?;

        self.cached_tools.clear();
        for tool in &tools {
            self.cached_tools.insert(tool.name.clone(), tool.clone());
        }

        Ok(tools)
    }

    /// Returns a reference to the cached tools map
    pub fn get_cached_tools(&self) -> &HashMap<String, McpTool> {
        &self.cached_tools
    }

    /// Retrieves an individual cached tool schema
    pub fn get_tool(&self, name: &str) -> Option<&McpTool> {
        self.cached_tools.get(name)
    }

    /// Returns server info if initialized
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Executes an MCP tool via `tools/call` over STDIO
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, McpClientError> {
        if !self.cached_tools.is_empty() && !self.cached_tools.contains_key(name) {
            return Err(McpClientError::ToolNotFound(name.to_string()));
        }

        let params = json!({
            "name": name,
            "arguments": arguments
        });

        let response = self.send_request("tools/call", Some(params))?;

        match response.result {
            Some(res) => Ok(res),
            None => match response.error {
                Some(err) => Err(McpClientError::JsonRpc {
                    code: err.code,
                    message: err.message,
                }),
                None => Err(McpClientError::Protocol("Empty response from tools/call".into())),
            },
        }
    }

    /// Sends a JSON-RPC request and synchronously reads the line response
    fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse, McpClientError> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let json_line = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", json_line)?;
        self.stdin.flush()?;

        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;
        if bytes_read == 0 {
            let status = self.child.try_wait()?;
            return Err(McpClientError::ProcessExited(status.and_then(|s| s.code())));
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        if let Some(err) = response.error {
            return Err(McpClientError::JsonRpc {
                code: err.code,
                message: err.message,
            });
        }

        Ok(response)
    }

    /// Sends a JSON-RPC notification (no response expected)
    fn send_notification(&mut self, method: &str, params: Option<Value>) -> Result<(), McpClientError> {
        let notif = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };

        let json_line = serde_json::to_string(&notif)?;
        writeln!(self.stdin, "{}", json_line)?;
        self.stdin.flush()?;
        Ok(())
    }
}

impl Drop for StdioMcpClient {
    /// Ensures child process is cleanly terminated and harvested when client is dropped
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

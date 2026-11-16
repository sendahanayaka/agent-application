use rust_mcp_client::{
    BridgeRequest, BridgeResponse, IntentPipeline, StdioMcpClient, VocabularyLoader,
};
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() {
    eprintln!("[RustBridge] Starting persistent Rust MCP Client service...");

    // 1. Load Vocabulary
    let vocab = match VocabularyLoader::load_embedded() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[RustBridge Error] Failed to load vocabulary: {}", e);
            std::process::exit(1);
        }
    };

    let pipeline = match IntentPipeline::new(vocab) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[RustBridge Error] Failed to initialize pipeline: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Spawn .NET MCP Server child process
    let dll_path = Path::new("../dotnet-mcp-server/bin/Debug/net8.0/DotnetMcpServer.dll");
    let mut client = match StdioMcpClient::spawn_dotnet(dll_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[RustBridge Error] Failed to spawn .NET MCP Server: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Initialize MCP Handshake
    let server_info = match client.initialize() {
        Ok(info) => {
            eprintln!(
                "[RustBridge] Connected to MCP server: {} (v{})",
                info.name, info.version
            );
            info
        }
        Err(e) => {
            eprintln!("[RustBridge Error] MCP Handshake failed: {}", e);
            std::process::exit(1);
        }
    };

    // 4. Dynamic Tool Discovery
    let tools = match client.discover_tools() {
        Ok(t) => {
            eprintln!("[RustBridge] Discovered {} dynamic MCP tools", t.len());
            t
        }
        Err(e) => {
            eprintln!("[RustBridge Error] Tool discovery failed: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("[RustBridge] Ready to process IPC requests on STDIN.");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[RustBridge Error] Error reading STDIN line: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: BridgeRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_res = BridgeResponse {
                    id: 0,
                    success: false,
                    data: None,
                    error: Some(format!("Invalid BridgeRequest JSON: {}", e)),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_res).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        let response = match req.action.as_str() {
            "health" => BridgeResponse {
                id: req.id,
                success: true,
                data: Some(json!({
                    "status": "healthy",
                    "mcp_server": server_info,
                    "tools_count": tools.len()
                })),
                error: None,
            },
            "tools" => BridgeResponse {
                id: req.id,
                success: true,
                data: Some(serde_json::to_value(client.get_cached_tools().values().collect::<Vec<_>>()).unwrap()),
                error: None,
            },
            "chat" => {
                let prompt = req.prompt.unwrap_or_default();
                let pipeline_res = pipeline.process_prompt(&prompt, &mut client);
                BridgeResponse {
                    id: req.id,
                    success: true,
                    data: Some(serde_json::to_value(&pipeline_res).unwrap()),
                    error: None,
                }
            }
            "confirm" => {
                let token = req.token.unwrap_or_default();
                let pipeline_res = pipeline.execute_confirmed_operation(&token, &mut client);
                BridgeResponse {
                    id: req.id,
                    success: true,
                    data: Some(serde_json::to_value(&pipeline_res).unwrap()),
                    error: None,
                }
            }
            unknown => BridgeResponse {
                id: req.id,
                success: false,
                data: None,
                error: Some(format!("Unknown action: {}", unknown)),
            },
        };

        if let Ok(json_line) = serde_json::to_string(&response) {
            let _ = writeln!(stdout, "{}", json_line);
            let _ = stdout.flush();
        }
    }

    eprintln!("[RustBridge] Exiting cleanly.");
}

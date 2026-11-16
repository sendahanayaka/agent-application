use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Standard JSON-RPC 2.0 Request
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Standard JSON-RPC 2.0 Notification (no id)
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Standard JSON-RPC 2.0 Response
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// Standard JSON-RPC 2.0 Error Object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

/// Discovered MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
    #[serde(default)]
    pub annotations: Option<Value>,
}

/// Tool JSON schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(default)]
    pub properties: HashMap<String, ToolPropertySchema>,
    #[serde(default)]
    pub required: Vec<String>,
}

/// Individual property schema where `type` can be either a single string or an array of strings (e.g. ["string", "null"])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPropertySchema {
    #[serde(rename = "type", default, deserialize_with = "deserialize_property_type")]
    pub prop_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub items: Option<Box<Value>>,
}

/// Custom deserializer for `type` field which can be "string" or ["string", "null"]
fn deserialize_property_type<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<Value> = Option::deserialize(deserializer)?;
    match val {
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Array(arr)) => {
            // Find first non-null type name, e.g. "string" in ["string", "null"]
            for item in arr {
                if let Value::String(t) = item {
                    if t != "null" {
                        return Ok(Some(t));
                    }
                }
            }
            Ok(Some("any".to_string()))
        }
        _ => Ok(None),
    }
}

/// MCP Server Info returned from initialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

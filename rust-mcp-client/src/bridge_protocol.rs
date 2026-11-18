use crate::pipeline_models::ConversationContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub id: u64,
    pub action: String, // "chat", "confirm", "tools", "health"
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub context: Option<ConversationContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub id: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

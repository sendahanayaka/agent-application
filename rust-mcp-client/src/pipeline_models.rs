use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Status of the prompt execution pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Success,
    ConfirmationRequired,
    MissingParameters,
    AmbiguousIntent,
    UnsupportedPrompt,
    ValidationError,
    ExecutionError,
}

/// A pending write operation awaiting user confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    pub confirmation_token: String,
    pub tool_name: String,
    pub arguments: Value,
    pub summary: String,
    pub created_at_unix: u64,
}

/// Execution output from a single tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleToolExecution {
    pub tool_name: String,
    pub arguments: Value,
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}

/// The main structured response sent to callers (such as Python FastAPI backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResponse {
    pub status: PipelineStatus,
    pub prompt: String,
    pub detected_intents: Vec<String>,
    pub selected_tools: Vec<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_operation: Option<PendingOperation>,
    #[serde(default)]
    pub tool_executions: Vec<SingleToolExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_parameters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiguous_options: Option<Vec<String>>,
}

/// Candidate tool call before execution
#[derive(Debug, Clone)]
pub struct PreparedToolCall {
    pub intent_id: String,
    pub tool_name: String,
    pub is_write: bool,
    pub arguments: HashMap<String, Value>,
    pub missing_required: Vec<String>,
}

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

/// Extracted conversational employee entity for context tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEmployee {
    pub employee_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub salary: Option<f64>,
    pub status: Option<String>,
    pub gender: Option<String>, // "male" or "female" inferred from first name if unambiguous
}

/// Session-based conversational memory context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationContext {
    pub conversation_id: String,
    pub last_referenced_employee_id: Option<i32>,
    pub last_referenced_employee: Option<ContextEmployee>,
    pub last_employees_collection: Vec<ContextEmployee>,
    pub last_selected_department: Option<String>,
    pub last_search_filters: HashMap<String, Value>,
    pub last_intent: Option<String>,
    pub last_tool: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_context: Option<ConversationContext>,
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

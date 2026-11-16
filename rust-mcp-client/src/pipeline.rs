use crate::client::StdioMcpClient;
use crate::extractor::ParameterExtractor;
use crate::matcher::VocabularyMatcher;
use crate::models::Vocabulary;
use crate::pipeline_models::{
    PendingOperation, PipelineResponse, PipelineStatus, PreparedToolCall, SingleToolExecution,
};
use crate::protocol::McpTool;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct IntentPipeline {
    matcher: VocabularyMatcher,
    pending_store: Mutex<HashMap<String, PendingOperation>>,
}

impl IntentPipeline {
    pub fn new(vocab: Vocabulary) -> Result<Self, regex::Error> {
        let matcher = VocabularyMatcher::new(vocab)?;
        Ok(Self {
            matcher,
            pending_store: Mutex::new(HashMap::new()),
        })
    }

    /// Process a natural language prompt or confirmation
    pub fn process_prompt(
        &self,
        prompt: &str,
        client: &mut StdioMcpClient,
    ) -> PipelineResponse {
        let trimmed = prompt.trim();

        // 1. Check if user is confirming a pending write operation
        if let Some(token) = self.extract_confirmation_token(trimmed) {
            return self.execute_confirmed_operation(&token, client);
        }

        // 2. Break compound queries ("and", ";", "also") into sub-clauses
        let clauses = self.split_clauses(trimmed);

        let mut all_prepared = Vec::new();
        let mut ambiguous_candidates = Vec::new();
        let cached_tools = client.get_cached_tools().clone();
        let extractor = ParameterExtractor::new(&self.matcher);

        for clause in &clauses {
            let matched_intents = self.match_clause_intents(clause);

            if matched_intents.is_empty() {
                continue;
            }

            if matched_intents.len() > 1 && !self.is_compound_count_search(&matched_intents) {
                // Ambiguous intent in a single clause
                for id in &matched_intents {
                    if !ambiguous_candidates.contains(id) {
                        ambiguous_candidates.push(id.clone());
                    }
                }
                continue;
            }

            for intent_id in matched_intents {
                if let Some(prepared) = self.prepare_tool_call(&intent_id, clause, &extractor, &cached_tools) {
                    all_prepared.push(prepared);
                }
            }
        }

        // 3. Handle Ambiguity
        if !ambiguous_candidates.is_empty() && all_prepared.is_empty() {
            return PipelineResponse {
                status: PipelineStatus::AmbiguousIntent,
                prompt: trimmed.to_string(),
                detected_intents: ambiguous_candidates.clone(),
                selected_tools: vec![],
                message: format!(
                    "Your request is ambiguous and matches multiple possible actions: {}. Please clarify your intent.",
                    ambiguous_candidates.join(", ")
                ),
                pending_operation: None,
                tool_executions: vec![],
                missing_parameters: None,
                ambiguous_options: Some(ambiguous_candidates),
            };
        }

        // 4. Handle Unsupported Prompt
        if all_prepared.is_empty() {
            return PipelineResponse {
                status: PipelineStatus::UnsupportedPrompt,
                prompt: trimmed.to_string(),
                detected_intents: vec![],
                selected_tools: vec![],
                message: "Sorry, I could not understand your request. Please check supported commands or phrasing.".to_string(),
                pending_operation: None,
                tool_executions: vec![],
                missing_parameters: None,
                ambiguous_options: None,
            };
        }

        // 5. Check for Missing Required Parameters
        let mut all_missing = Vec::new();
        for prep in &all_prepared {
            if !prep.missing_required.is_empty() {
                all_missing.extend(prep.missing_required.clone());
            }
        }

        if !all_missing.is_empty() {
            all_missing.sort();
            all_missing.dedup();
            return PipelineResponse {
                status: PipelineStatus::MissingParameters,
                prompt: trimmed.to_string(),
                detected_intents: all_prepared.iter().map(|p| p.intent_id.clone()).collect(),
                selected_tools: all_prepared.iter().map(|p| p.tool_name.clone()).collect(),
                message: format!(
                    "Missing required parameter(s) for {}: {}.",
                    all_prepared[0].tool_name,
                    all_missing.join(", ")
                ),
                pending_operation: None,
                tool_executions: vec![],
                missing_parameters: Some(all_missing),
                ambiguous_options: None,
            };
        }

        // 6. Check for Write Operations (Add / Update) -> Require Explicit Confirmation
        let write_ops: Vec<_> = all_prepared.iter().filter(|p| p.is_write).collect();
        if let Some(write_call) = write_ops.first() {
            let token = self.generate_confirmation_token();
            let summary = self.generate_write_summary(&write_call.tool_name, &write_call.arguments);
            let pending = PendingOperation {
                confirmation_token: token.clone(),
                tool_name: write_call.tool_name.clone(),
                arguments: Value::Object(write_call.arguments.clone().into_iter().collect()),
                summary: summary.clone(),
                created_at_unix: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            // Store in pending store
            if let Ok(mut store) = self.pending_store.lock() {
                store.insert(token.clone(), pending.clone());
            }

            return PipelineResponse {
                status: PipelineStatus::ConfirmationRequired,
                prompt: trimmed.to_string(),
                detected_intents: vec![write_call.intent_id.clone()],
                selected_tools: vec![write_call.tool_name.clone()],
                message: format!(
                    "Confirmation required: {}. To proceed, please confirm using token [{}] or reply 'confirm {}'.",
                    summary, token, token
                ),
                pending_operation: Some(pending),
                tool_executions: vec![],
                missing_parameters: None,
                ambiguous_options: None,
            };
        }

        // 7. Execute Read-Only Tools Sequentially
        let mut executions = Vec::new();
        let mut all_succeeded = true;

        for prep in &all_prepared {
            let args_value = Value::Object(prep.arguments.clone().into_iter().collect());
            match client.call_tool(&prep.tool_name, args_value.clone()) {
                Ok(raw_result) => {
                    // Extract structured text from MCP tool result
                    let parsed_content = self.extract_tool_content(&raw_result);
                    executions.push(SingleToolExecution {
                        tool_name: prep.tool_name.clone(),
                        arguments: args_value,
                        success: true,
                        result: Some(parsed_content),
                        error: None,
                    });
                }
                Err(err) => {
                    all_succeeded = false;
                    executions.push(SingleToolExecution {
                        tool_name: prep.tool_name.clone(),
                        arguments: args_value,
                        success: false,
                        result: None,
                        error: Some(err.to_string()),
                    });
                }
            }
        }

        PipelineResponse {
            status: if all_succeeded {
                PipelineStatus::Success
            } else {
                PipelineStatus::ExecutionError
            },
            prompt: trimmed.to_string(),
            detected_intents: all_prepared.iter().map(|p| p.intent_id.clone()).collect(),
            selected_tools: all_prepared.iter().map(|p| p.tool_name.clone()).collect(),
            message: format!("Successfully executed {} tool(s).", executions.len()),
            pending_operation: None,
            tool_executions: executions,
            missing_parameters: None,
            ambiguous_options: None,
        }
    }

    /// Execute a confirmed write operation using its confirmation token
    pub fn execute_confirmed_operation(
        &self,
        token: &str,
        client: &mut StdioMcpClient,
    ) -> PipelineResponse {
        let pending = {
            let mut store = self.pending_store.lock().unwrap();
            store.remove(token)
        };

        match pending {
            Some(op) => {
                match client.call_tool(&op.tool_name, op.arguments.clone()) {
                    Ok(raw_result) => {
                        let parsed = self.extract_tool_content(&raw_result);
                        PipelineResponse {
                            status: PipelineStatus::Success,
                            prompt: format!("confirm {}", token),
                            detected_intents: vec![op.tool_name.clone()],
                            selected_tools: vec![op.tool_name.clone()],
                            message: format!("Confirmed operation executed successfully: {}", op.summary),
                            pending_operation: None,
                            tool_executions: vec![SingleToolExecution {
                                tool_name: op.tool_name,
                                arguments: op.arguments,
                                success: true,
                                result: Some(parsed),
                                error: None,
                            }],
                            missing_parameters: None,
                            ambiguous_options: None,
                        }
                    }
                    Err(err) => PipelineResponse {
                        status: PipelineStatus::ExecutionError,
                        prompt: format!("confirm {}", token),
                        detected_intents: vec![op.tool_name.clone()],
                        selected_tools: vec![op.tool_name.clone()],
                        message: format!("Failed to execute confirmed operation: {}", err),
                        pending_operation: None,
                        tool_executions: vec![SingleToolExecution {
                            tool_name: op.tool_name,
                            arguments: op.arguments,
                            success: false,
                            result: None,
                            error: Some(err.to_string()),
                        }],
                        missing_parameters: None,
                        ambiguous_options: None,
                    },
                }
            }
            None => PipelineResponse {
                status: PipelineStatus::ValidationError,
                prompt: format!("confirm {}", token),
                detected_intents: vec![],
                selected_tools: vec![],
                message: format!(
                    "Invalid, expired or already executed confirmation token: '{}'. Operation was not executed.",
                    token
                ),
                pending_operation: None,
                tool_executions: vec![],
                missing_parameters: None,
                ambiguous_options: None,
            },
        }
    }

    fn split_clauses(&self, prompt: &str) -> Vec<String> {
        // Split on ";", " and also ", " and ", " also "
        let re = Regex::new(r"(?i)\s*(?:;|,\s*and\s+|,\s*also\s+|\s+and\s+also\s+|\s+and\s+|\s+also\s+)\s*").unwrap();
        let parts: Vec<&str> = re.split(prompt).collect();
        if parts.is_empty() {
            vec![prompt.to_string()]
        } else {
            parts.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        }
    }

    fn match_clause_intents(&self, clause: &str) -> Vec<String> {
        let mut matches = Vec::new();
        for intent in &self.matcher.vocabulary().intents {
            if self.matcher.matches_intent(&intent.intent_id, clause) {
                matches.push(intent.intent_id.clone());
            }
        }

        // Priority resolution:
        // 1. If get_all_employees and search_employees both match:
        //    If there is a department, designation, status or name parameter, prefer search_employees!
        //    Otherwise, prefer get_all_employees.
        if matches.contains(&"get_all_employees".to_string()) && matches.contains(&"search_employees".to_string()) {
            let extractor = ParameterExtractor::new(&self.matcher);
            let params = extractor.extract_all(clause);
            if params.contains_key("department") || params.contains_key("designation") || params.contains_key("status") || params.contains_key("name") {
                matches.retain(|id| id != "get_all_employees");
            } else {
                matches.retain(|id| id != "search_employees");
            }
        }

        // 2. If get_employee_by_id and get_all_employees both match (due to "employee" substring),
        //    check if an ID exists. If ID exists, prefer get_employee_by_id!
        if matches.contains(&"get_employee_by_id".to_string()) && matches.contains(&"get_all_employees".to_string()) {
            if self.matcher.extract_employee_id(clause).is_some() {
                matches.retain(|id| id != "get_all_employees");
            } else {
                matches.retain(|id| id != "get_employee_by_id");
            }
        }

        // 3. If add_employee and get_all_employees both match, prefer add_employee
        if matches.contains(&"add_employee".to_string()) && matches.contains(&"get_all_employees".to_string()) {
            matches.retain(|id| id != "get_all_employees");
        }

        matches
    }

    fn is_compound_count_search(&self, intents: &[String]) -> bool {
        intents.contains(&"search_employees".to_string()) && intents.contains(&"get_employee_count".to_string())
    }

    fn prepare_tool_call(
        &self,
        intent_id: &str,
        clause: &str,
        extractor: &ParameterExtractor,
        cached_tools: &HashMap<String, McpTool>,
    ) -> Option<PreparedToolCall> {
        let intent_def = self.matcher.vocabulary().intents.iter().find(|i| i.intent_id == intent_id)?;
        let tool_name = &intent_def.target_tool;
        let tool_schema = cached_tools.get(tool_name)?;

        let extracted_params = extractor.extract_all(clause);
        let mut final_args = HashMap::new();
        let mut missing_required = Vec::new();

        // Dynamically validate against discovered tool schema properties
        for (prop_name, _prop_schema) in &tool_schema.input_schema.properties {
            if let Some(val) = extracted_params.get(prop_name) {
                final_args.insert(prop_name.clone(), val.clone());
            } else if tool_schema.input_schema.required.contains(prop_name) {
                missing_required.push(prop_name.clone());
            }
        }

        Some(PreparedToolCall {
            intent_id: intent_id.to_string(),
            tool_name: tool_name.clone(),
            is_write: intent_def.is_write,
            arguments: final_args,
            missing_required,
        })
    }

    fn generate_confirmation_token(&self) -> String {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        format!("CONF-{}", nanos % 1000000)
    }

    fn generate_write_summary(&self, tool_name: &str, args: &HashMap<String, Value>) -> String {
        match tool_name {
            "add_employee" => {
                let first = args.get("first_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let last = args.get("last_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let dept = args.get("department_id").map(|v| v.to_string()).unwrap_or_default();
                let salary = args.get("salary").map(|v| v.to_string()).unwrap_or_default();
                format!("Add new employee '{} {}' (Dept ID: {}, Salary: ${})", first, last, dept, salary)
            }
            "update_employee" => {
                let id = args.get("employee_id").map(|v| v.to_string()).unwrap_or_default();
                format!("Update employee ID {} with changes: {:?}", id, args)
            }
            _ => format!("Execute write tool {} with {:?}", tool_name, args),
        }
    }

    fn extract_confirmation_token(&self, text: &str) -> Option<String> {
        let re = Regex::new(r"(?i)\b(?:confirm|yes|proceed)\s+(CONF-[0-9]{1,7})\b").unwrap();
        if let Some(caps) = re.captures(text) {
            return caps.get(1).map(|m| m.as_str().to_uppercase());
        }
        let direct_re = Regex::new(r"^(CONF-[0-9]{1,7})$").unwrap();
        if let Some(caps) = direct_re.captures(text.trim()) {
            return caps.get(1).map(|m| m.as_str().to_uppercase());
        }
        None
    }

    fn extract_tool_content(&self, raw_result: &Value) -> Value {
        if let Some(content_arr) = raw_result.get("content").and_then(|c| c.as_array()) {
            if let Some(first_item) = content_arr.first() {
                if let Some(text) = first_item.get("text").and_then(|t| t.as_str()) {
                    if let Ok(json_obj) = serde_json::from_str::<Value>(text) {
                        return json_obj;
                    }
                    return json!(text);
                }
            }
        }
        raw_result.clone()
    }
}

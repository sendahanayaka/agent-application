use crate::client::StdioMcpClient;
use crate::extractor::ParameterExtractor;
use crate::matcher::VocabularyMatcher;
use crate::models::Vocabulary;
use crate::pipeline_models::{
    ConversationContext, PendingOperation, PipelineResponse, PipelineStatus,
    PreparedToolCall, SingleToolExecution,
};
use crate::protocol::McpTool;
use crate::resolver::{ReferenceResolutionResult, ReferenceResolver};
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

    /// Process a natural language prompt or confirmation with session context
    pub fn process_prompt(
        &self,
        prompt: &str,
        client: &mut StdioMcpClient,
    ) -> PipelineResponse {
        self.process_prompt_with_context(prompt, &None, client)
    }

    /// Process prompt with conversation context for follow-up reference resolution
    pub fn process_prompt_with_context(
        &self,
        prompt: &str,
        context: &Option<ConversationContext>,
        client: &mut StdioMcpClient,
    ) -> PipelineResponse {
        let trimmed = prompt.trim();

        // 1. Check if user is confirming a pending write operation
        if let Some(token) = self.extract_confirmation_token(trimmed) {
            return self.execute_confirmed_operation(&token, client);
        }

        // 2. Deterministic Anaphoric Reference Resolution (he, she, they, his, her, their, that employee)
        let mut resolved_reference_label: Option<String> = None;
        let effective_prompt: String;

        match ReferenceResolver::resolve(trimmed, context) {
            ReferenceResolutionResult::FieldQuery { employee_id, resolved_label, target_field } => {
                resolved_reference_label = Some(resolved_label.clone());
                // Call get_employee_by_id on MCP server to get fresh data
                let args = json!({ "employee_id": employee_id });
                match client.call_tool("get_employee_by_id", args.clone()) {
                    Ok(raw_result) => {
                        let parsed = self.extract_tool_content(&raw_result);
                        let emp_list = ReferenceResolver::extract_employees_from_result("get_employee_by_id", &parsed);
                        let mut new_ctx = context.clone().unwrap_or_default();
                        if let Some(first) = emp_list.first() {
                            new_ctx.last_referenced_employee_id = Some(first.employee_id);
                            new_ctx.last_referenced_employee = Some(first.clone());
                            new_ctx.last_employees_collection = vec![first.clone()];
                            new_ctx.last_intent = Some("get_employee_by_id".to_string());
                            new_ctx.last_tool = Some("get_employee_by_id".to_string());
                        }

                        // Human friendly answer for the specific queried field
                        let data = parsed.get("data").unwrap_or(&parsed);
                        let field_val = match target_field.as_str() {
                            "phone" => data.get("Phone").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "email" => data.get("Email").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "department" => data.get("DepartmentName").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "designation" => data.get("DesignationName").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "salary" => data.get("Salary").map(|v| format!("${}", v)).unwrap_or("None listed".to_string()),
                            "name" => format!("{} {}", data.get("FirstName").and_then(|v| v.as_str()).unwrap_or(""), data.get("LastName").and_then(|v| v.as_str()).unwrap_or("")).trim().to_string(),
                            "status" => data.get("Status").and_then(|v| v.as_str()).unwrap_or("Active").to_string(),
                            _ => format!("Employee ID: {}", employee_id),
                        };

                        let label_desc = match target_field.as_str() {
                            "phone" => "phone number",
                            "email" => "email address",
                            "department" => "department",
                            "designation" => "designation",
                            "salary" => "annual salary",
                            "name" => "name",
                            "status" => "status",
                            _ => "details",
                        };

                        let emp_name = data.get("FirstName").and_then(|v| v.as_str()).unwrap_or("Employee");
                        let message = format!("{}'s {} is {}.", emp_name, label_desc, field_val);

                        return PipelineResponse {
                            status: PipelineStatus::Success,
                            prompt: trimmed.to_string(),
                            detected_intents: vec!["get_employee_by_id".to_string()],
                            selected_tools: vec!["get_employee_by_id".to_string()],
                            message,
                            pending_operation: None,
                            tool_executions: vec![SingleToolExecution {
                                tool_name: "get_employee_by_id".to_string(),
                                arguments: args,
                                success: true,
                                result: Some(parsed),
                                error: None,
                            }],
                            missing_parameters: None,
                            ambiguous_options: None,
                            resolved_reference: resolved_reference_label,
                            conversation_context: Some(new_ctx),
                        };
                    }
                    Err(e) => {
                        return PipelineResponse {
                            status: PipelineStatus::ExecutionError,
                            prompt: trimmed.to_string(),
                            detected_intents: vec!["get_employee_by_id".to_string()],
                            selected_tools: vec!["get_employee_by_id".to_string()],
                            message: format!("Failed to retrieve employee {}: {}", employee_id, e),
                            pending_operation: None,
                            tool_executions: vec![],
                            missing_parameters: None,
                            ambiguous_options: None,
                            resolved_reference: resolved_reference_label,
                            conversation_context: context.clone(),
                        };
                    }
                }
            }
            ReferenceResolutionResult::CollectionFieldQuery { employee_ids, resolved_label, target_field } => {
                resolved_reference_label = Some(resolved_label.clone());
                // For each employee in collection, fetch current fresh record
                let mut executions = Vec::new();
                let mut lines = Vec::new();

                for emp_id in &employee_ids {
                    let args = json!({ "employee_id": emp_id });
                    if let Ok(raw_result) = client.call_tool("get_employee_by_id", args.clone()) {
                        let parsed = self.extract_tool_content(&raw_result);
                        let data = parsed.get("data").unwrap_or(&parsed);
                        let name = format!("{} {}", data.get("FirstName").and_then(|v| v.as_str()).unwrap_or(""), data.get("LastName").and_then(|v| v.as_str()).unwrap_or("")).trim().to_string();
                        let field_val = match target_field.as_str() {
                            "phone" => data.get("Phone").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "email" => data.get("Email").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "department" => data.get("DepartmentName").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "designation" => data.get("DesignationName").and_then(|v| v.as_str()).unwrap_or("None listed").to_string(),
                            "salary" => data.get("Salary").map(|v| format!("${}", v)).unwrap_or("None listed".to_string()),
                            "name" => name.clone(),
                            _ => format!("Employee ID: {}", emp_id),
                        };
                        lines.push(format!("• {}: {}", name, field_val));
                        executions.push(SingleToolExecution {
                            tool_name: "get_employee_by_id".to_string(),
                            arguments: args,
                            success: true,
                            result: Some(parsed),
                            error: None,
                        });
                    }
                }

                let message = if lines.is_empty() {
                    "No records found for the referenced employees.".to_string()
                } else {
                    format!("Here are their {}:\n{}", target_field, lines.join("\n"))
                };

                return PipelineResponse {
                    status: PipelineStatus::Success,
                    prompt: trimmed.to_string(),
                    detected_intents: vec!["get_employee_by_id".to_string()],
                    selected_tools: vec!["get_employee_by_id".to_string()],
                    message,
                    pending_operation: None,
                    tool_executions: executions,
                    missing_parameters: None,
                    ambiguous_options: None,
                    resolved_reference: resolved_reference_label,
                    conversation_context: context.clone(),
                };
            }
            ReferenceResolutionResult::UpdateAction { resolved_label, modified_prompt, .. } => {
                resolved_reference_label = Some(resolved_label);
                effective_prompt = modified_prompt;
            }
            ReferenceResolutionResult::Ambiguous { message, options } => {
                return PipelineResponse {
                    status: PipelineStatus::AmbiguousIntent,
                    prompt: trimmed.to_string(),
                    detected_intents: vec![],
                    selected_tools: vec![],
                    message,
                    pending_operation: None,
                    tool_executions: vec![],
                    missing_parameters: None,
                    ambiguous_options: Some(options),
                    resolved_reference: None,
                    conversation_context: context.clone(),
                };
            }
            ReferenceResolutionResult::NoContext { message } => {
                return PipelineResponse {
                    status: PipelineStatus::ValidationError,
                    prompt: trimmed.to_string(),
                    detected_intents: vec![],
                    selected_tools: vec![],
                    message,
                    pending_operation: None,
                    tool_executions: vec![],
                    missing_parameters: None,
                    ambiguous_options: None,
                    resolved_reference: None,
                    conversation_context: context.clone(),
                };
            }
            ReferenceResolutionResult::NotApplicable => {
                effective_prompt = trimmed.to_string();
            }
        }

        // 3. Break compound queries ("and", ";", "also") into sub-clauses
        let clauses = self.split_clauses(&effective_prompt);

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
                resolved_reference: resolved_reference_label,
                conversation_context: context.clone(),
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
                resolved_reference: resolved_reference_label,
                conversation_context: context.clone(),
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
                resolved_reference: resolved_reference_label,
                conversation_context: context.clone(),
            };
        }

        // 5b. For update_employee, validate that at least one actual update field exists (beyond employee_id)
        for prep in &all_prepared {
            if prep.tool_name == "update_employee" {
                let has_update_field = prep.arguments.iter().any(|(k, _)| k != "employee_id");
                if !has_update_field {
                    // Check if the user attempted an unsupported EmployeeID modification (e.g. "update employee id 1038 to 1021")
                    let id_change_re = Regex::new(r"(?i)\b(?:update|change|set)\s+(?:employee\s+)?(?:id\s+)?([0-9]{3,6})\s+to\s+([0-9]{3,6})\b").unwrap();
                    if let Some(caps) = id_change_re.captures(trimmed) {
                        let from_id = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let to_id = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                        return PipelineResponse {
                            status: PipelineStatus::ValidationError,
                            prompt: trimmed.to_string(),
                            detected_intents: vec!["update_employee".to_string()],
                            selected_tools: vec![],
                            message: format!(
                                "Modifying EmployeeID (from {} to {}) is not permitted. EmployeeID is an immutable identity key in the SQL Server database. Please specify valid update fields such as phone number, email, salary, department, or designation.",
                                from_id, to_id
                            ),
                            pending_operation: None,
                            tool_executions: vec![],
                            missing_parameters: None,
                            ambiguous_options: None,
                            resolved_reference: resolved_reference_label,
                            conversation_context: context.clone(),
                        };
                    }

                    return PipelineResponse {
                        status: PipelineStatus::ValidationError,
                        prompt: trimmed.to_string(),
                        detected_intents: vec!["update_employee".to_string()],
                        selected_tools: vec![],
                        message: "No update fields provided. Please specify at least one valid field to update (e.g. phone, email, salary, department, designation, or status).".to_string(),
                        pending_operation: None,
                        tool_executions: vec![],
                        missing_parameters: None,
                        ambiguous_options: None,
                        resolved_reference: resolved_reference_label,
                        conversation_context: context.clone(),
                    };
                }
            }
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
                resolved_reference: resolved_reference_label,
                conversation_context: context.clone(),
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

        // Build updated conversation context from execution results
        let mut updated_ctx = context.clone().unwrap_or_default();
        if all_succeeded && !executions.is_empty() {
            let mut all_extracted_employees = Vec::new();
            for exec in &executions {
                if let Some(res) = &exec.result {
                    let emps = ReferenceResolver::extract_employees_from_result(&exec.tool_name, res);
                    all_extracted_employees.extend(emps);
                }
            }

            if !all_extracted_employees.is_empty() {
                if all_extracted_employees.len() == 1 {
                    updated_ctx.last_referenced_employee_id = Some(all_extracted_employees[0].employee_id);
                    updated_ctx.last_referenced_employee = Some(all_extracted_employees[0].clone());
                } else {
                    updated_ctx.last_referenced_employee_id = None;
                    updated_ctx.last_referenced_employee = None;
                }
                updated_ctx.last_employees_collection = all_extracted_employees;
            }

            if let Some(first_prep) = all_prepared.first() {
                updated_ctx.last_intent = Some(first_prep.intent_id.clone());
                updated_ctx.last_tool = Some(first_prep.tool_name.clone());
                if let Some(dept_val) = first_prep.arguments.get("department").and_then(|v| v.as_str()) {
                    updated_ctx.last_selected_department = Some(dept_val.to_string());
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
            resolved_reference: resolved_reference_label,
            conversation_context: Some(updated_ctx),
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
                            resolved_reference: None,
                            conversation_context: None,
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
                        resolved_reference: None,
                        conversation_context: None,
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
                resolved_reference: None,
                conversation_context: None,
            },
        }
    }

    fn split_clauses(&self, prompt: &str) -> Vec<String> {
        // Split multi-intent compound queries on ";", " and also ", " also ", ", and "
        // Note: Do NOT split on bare " and " because single actions have arguments joined by "and"
        // (e.g. "Add employee ... with email X and salary Y", "Search engineers and count them")
        let re = Regex::new(r"(?i)\s*(?:;|,\s*and\s+also\s+|,\s*also\s+|\s+and\s+also\s+|\s+also\s+|,\s*and\s+)\s*").unwrap();
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

        // If no intent matched yet, check if this is an explicit search or listing with parameters
        if matches.is_empty() {
            let extractor = ParameterExtractor::new(&self.matcher);
            let params = extractor.extract_all(clause);
            if !params.is_empty() {
                // If the clause mentions designations, departments, status, or names, treat as search_employees
                if params.contains_key("department")
                    || params.contains_key("designation")
                    || params.contains_key("status")
                    || params.contains_key("name")
                {
                    matches.push("search_employees".to_string());
                }
            }
        }

        // Priority resolution:
        // 0. Write intent priority: When add_employee or update_employee is detected,
        // incidental search_employees or get_all_employees matches must NOT be triggered.
        // Department and designation details are parameters of the write operation.
        if matches.contains(&"add_employee".to_string()) {
            matches.retain(|id| id != "search_employees" && id != "get_all_employees");
        }
        if matches.contains(&"update_employee".to_string()) {
            matches.retain(|id| id != "search_employees" && id != "get_all_employees");
        }

        // 1. If get_employee_count matches, ensure count requests have priority over search_employees and get_all_employees
        if matches.contains(&"get_employee_count".to_string()) {
            matches.retain(|id| id != "search_employees" && id != "get_all_employees");
        }

        // 2. If get_all_employees and search_employees both match:
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

        // If get_all_employees matched alone, but the query contains filter parameters (e.g. "Show Engineering employees" or "Show active employees"),
        // re-route get_all_employees to search_employees!
        if matches.contains(&"get_all_employees".to_string()) && !matches.contains(&"search_employees".to_string()) {
            let extractor = ParameterExtractor::new(&self.matcher);
            let params = extractor.extract_all(clause);
            if params.contains_key("department") || params.contains_key("designation") || params.contains_key("status") || params.contains_key("name") {
                matches.retain(|id| id != "get_all_employees");
                matches.push("search_employees".to_string());
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
                let mut changes = Vec::new();
                for (k, v) in args {
                    if k == "employee_id" { continue; }
                    let label = match k.as_str() {
                        "phone" => "Phone",
                        "email" => "Email",
                        "salary" => "Salary",
                        "department_id" => "Department ID",
                        "designation_id" => "Designation ID",
                        "first_name" => "First Name",
                        "last_name" => "Last Name",
                        "status" => "Status",
                        _ => k.as_str(),
                    };
                    if let Some(s) = v.as_str() {
                        changes.push(format!("{}: '{}'", label, s));
                    } else if let Some(n) = v.as_f64() {
                        changes.push(format!("{}: {}", label, n));
                    } else if let Some(i) = v.as_i64() {
                        changes.push(format!("{}: {}", label, i));
                    } else {
                        changes.push(format!("{}: {}", label, v));
                    }
                }
                format!("Update Employee ID {} [Set {}]", id, changes.join(", "))
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

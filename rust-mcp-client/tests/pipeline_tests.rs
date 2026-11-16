use rust_mcp_client::{
    IntentPipeline, PipelineStatus, StdioMcpClient, VocabularyLoader,
};
use std::path::Path;

fn setup_test_pipeline() -> (IntentPipeline, StdioMcpClient) {
    let vocab = VocabularyLoader::load_embedded().expect("Should load vocabulary");
    let pipeline = IntentPipeline::new(vocab).expect("Should create pipeline");

    let dll_path = Path::new("../dotnet-mcp-server/bin/Debug/net8.0/DotnetMcpServer.dll");
    let mut client = StdioMcpClient::spawn_dotnet(dll_path).expect("Should spawn child process");
    client.initialize().expect("Should initialize MCP client");
    client.discover_tools().expect("Should discover tools");

    (pipeline, client)
}

#[test]
fn test_pipeline_get_all_employees() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt("Show all employees", &mut client);

    assert_eq!(resp.status, PipelineStatus::Success);
    assert_eq!(resp.selected_tools, vec!["get_all_employees"]);
    assert_eq!(resp.tool_executions.len(), 1);
    assert!(resp.tool_executions[0].success);

    let result = resp.tool_executions[0].result.as_ref().unwrap();
    assert_eq!(result.get("success").unwrap(), true);
    assert!(result.get("count").unwrap().as_i64().unwrap() >= 20);
}

#[test]
fn test_pipeline_get_employee_by_id() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt("Find employee ID 1001", &mut client);

    assert_eq!(resp.status, PipelineStatus::Success);
    assert_eq!(resp.selected_tools, vec!["get_employee_by_id"]);
    assert_eq!(resp.tool_executions.len(), 1);
    assert!(resp.tool_executions[0].success);

    let result = resp.tool_executions[0].result.as_ref().unwrap();
    assert_eq!(result.get("success").unwrap(), true);
    let data = result.get("data").unwrap();
    assert_eq!(data.get("EmployeeID").unwrap(), 1001);
}

#[test]
fn test_pipeline_search_employees_by_department() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt("Search employees in IT department", &mut client);

    assert_eq!(resp.status, PipelineStatus::Success);
    assert_eq!(resp.selected_tools, vec!["search_employees"]);
    assert_eq!(resp.tool_executions.len(), 1);

    let result = resp.tool_executions[0].result.as_ref().unwrap();
    assert_eq!(result.get("success").unwrap(), true);
    let count = result.get("count").unwrap().as_i64().unwrap();
    assert_eq!(count, 7);
}

#[test]
fn test_pipeline_get_department_list() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt("Show all departments", &mut client);

    assert_eq!(resp.status, PipelineStatus::Success);
    assert_eq!(resp.selected_tools, vec!["get_department_list"]);
    assert_eq!(resp.tool_executions.len(), 1);

    let result = resp.tool_executions[0].result.as_ref().unwrap();
    assert_eq!(result.get("count").unwrap().as_i64().unwrap(), 6);
}

#[test]
fn test_pipeline_get_employee_count() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt("How many employees are there?", &mut client);

    assert_eq!(resp.status, PipelineStatus::Success);
    assert_eq!(resp.selected_tools, vec!["get_employee_count"]);
    assert_eq!(resp.tool_executions.len(), 1);

    let result = resp.tool_executions[0].result.as_ref().unwrap();
    assert!(result.get("count").unwrap().as_i64().unwrap() >= 20);
}

#[test]
fn test_pipeline_multiple_sequential_tools() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt(
        "Search employees in IT department and also count employees in IT",
        &mut client,
    );

    assert_eq!(resp.status, PipelineStatus::Success);
    assert_eq!(resp.tool_executions.len(), 2);
    assert_eq!(resp.selected_tools, vec!["search_employees", "get_employee_count"]);

    assert!(resp.tool_executions[0].success);
    assert!(resp.tool_executions[1].success);
}

#[test]
fn test_pipeline_missing_parameters_validation() {
    let (pipeline, mut client) = setup_test_pipeline();
    // Prompt to add employee missing required salary and email
    let resp = pipeline.process_prompt("Add a new employee named John Doe", &mut client);

    assert_eq!(resp.status, PipelineStatus::MissingParameters);
    assert!(resp.missing_parameters.is_some());
    let missing = resp.missing_parameters.unwrap();
    assert!(missing.contains(&"email".to_string()));
    assert!(missing.contains(&"salary".to_string()));
}

#[test]
fn test_pipeline_unsupported_prompt() {
    let (pipeline, mut client) = setup_test_pipeline();
    let resp = pipeline.process_prompt("What is the weather in Seattle today?", &mut client);

    assert_eq!(resp.status, PipelineStatus::UnsupportedPrompt);
    assert!(resp.tool_executions.is_empty());
}

#[test]
fn test_pipeline_write_confirmation_flow_and_cleanup() {
    let (pipeline, mut client) = setup_test_pipeline();

    // 1. Write prompt requiring confirmation
    let write_prompt = "Add employee named Test User with email temp.test.phase2c@spillabs.com in IT as Software Engineer with salary 85000";
    let resp = pipeline.process_prompt(write_prompt, &mut client);

    assert_eq!(resp.status, PipelineStatus::ConfirmationRequired);
    assert!(resp.pending_operation.is_some());
    assert!(resp.tool_executions.is_empty()); // Nothing executed yet!

    let pending = resp.pending_operation.unwrap();
    let token = pending.confirmation_token;
    assert!(token.starts_with("CONF-"));

    // 2. Reject unconfirmed execution attempts with wrong token
    let fake_confirm = pipeline.execute_confirmed_operation("CONF-99999999", &mut client);
    assert_eq!(fake_confirm.status, PipelineStatus::ValidationError);

    // 3. Confirm with valid token
    let confirm_resp = pipeline.execute_confirmed_operation(&token, &mut client);
    assert_eq!(confirm_resp.status, PipelineStatus::Success);
    assert_eq!(confirm_resp.tool_executions.len(), 1);
    assert!(confirm_resp.tool_executions[0].success);

    let result = confirm_resp.tool_executions[0].result.as_ref().unwrap();
    let data = result.get("data").unwrap();
    let created_id = data.get("EmployeeID").unwrap().as_i64().unwrap();

    // 4. Token cannot be reused (idempotency / safety)
    let reuse_resp = pipeline.execute_confirmed_operation(&token, &mut client);
    assert_eq!(reuse_resp.status, PipelineStatus::ValidationError);

    // 5. Clean up created test employee from database
    let del_query = format!("DELETE FROM Employees WHERE EmployeeID = {};", created_id);
    let _ = std::process::Command::new("sqlcmd")
        .args(["-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", &del_query])
        .output();
}

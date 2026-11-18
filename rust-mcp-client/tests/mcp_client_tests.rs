use rust_mcp_client::StdioMcpClient;
use std::path::Path;

fn get_server_dll_path() -> &'static str {
    "../dotnet-mcp-server/bin/Debug/net8.0/DotnetMcpServer.dll"
}

#[test]
fn test_spawn_and_initialize_dotnet_mcp_server() {
    let dll_path = get_server_dll_path();
    assert!(Path::new(dll_path).exists(), ".NET MCP server DLL must exist");

    let mut client = StdioMcpClient::spawn_dotnet(dll_path).expect("Should spawn child process");
    let server_info = client.initialize().expect("MCP initialization must succeed");

    assert_eq!(server_info.name, "dotnet-employee-mcp-server");
    assert_eq!(server_info.version, "1.0.0");
    assert!(client.server_info().is_some());
}

#[test]
fn test_dynamic_tools_discovery_all_seven_tools() {
    let dll_path = get_server_dll_path();
    let mut client = StdioMcpClient::spawn_dotnet(dll_path).expect("Should spawn child process");
    client.initialize().expect("MCP initialization must succeed");

    let tools = client.discover_tools().expect("tools/list must succeed");
    assert_eq!(tools.len(), 7, "Exactly 7 tools must be discovered dynamically");

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    let expected = vec![
        "get_all_employees",
        "get_employee_by_id",
        "search_employees",
        "get_department_list",
        "get_employee_count",
        "add_employee",
        "update_employee",
    ];

    for exp in expected {
        assert!(tool_names.contains(&exp.to_string()), "Missing expected tool: {}", exp);
    }

    // Verify schemas are non-empty and cached
    let cached = client.get_cached_tools();
    assert_eq!(cached.len(), 7);

    let get_emp = client.get_tool("get_employee_by_id").expect("get_employee_by_id tool must exist");
    assert!(get_emp.input_schema.properties.contains_key("employee_id"));

    let add_emp = client.get_tool("add_employee").expect("add_employee tool must exist");
    assert!(add_emp.input_schema.properties.contains_key("first_name"));
    assert!(add_emp.input_schema.properties.contains_key("email"));
    assert!(add_emp.input_schema.properties.contains_key("salary"));
}

#[test]
fn test_invalid_tool_call_validation() {
    let dll_path = get_server_dll_path();
    let mut client = StdioMcpClient::spawn_dotnet(dll_path).expect("Should spawn child process");
    client.initialize().expect("MCP initialization must succeed");
    client.discover_tools().expect("tools/list must succeed");

    // Calling an unregistered tool must return ToolNotFound error
    let res = client.call_tool("non_existent_tool", serde_json::json!({}));
    assert!(res.is_err());
}

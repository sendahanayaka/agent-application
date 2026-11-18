# FastAPI Backend API Specification & Documentation

## 1. Architecture Overview

The FastAPI backend operates as the API gateway between the React user interface and the Rust MCP Client. It manages a persistent background child process running `rust-mcp-client.exe`, routing JSON requests and responses over STDIN and STDOUT with request correlation IDs and asyncio synchronization.

```text
React (Vite + TS)
       ↕  HTTP / REST JSON
Python FastAPI (Backend Gateway)
       ↕  Newline-Delimited JSON over STDIN/STDOUT (Bridge Protocol)
Rust MCP Client (Persistent Child Process)
       ↕  STDIO (JSON-RPC 2.0)
.NET 8 C# MCP Server (Child Process of Rust)
       ↕  Parameterized ADO.NET
Microsoft SQL Server (InternAgenticDB)
```

---

## 2. API Endpoints Reference

### A. `POST /api/chat`
Submits a natural-language employee-management prompt for deterministic intent matching, argument validation, and execution.

- **Request Body**:
  ```json
  {
    "prompt": "Show all employees"
  }
  ```
- **Response Body (`200 OK`)**:
  ```json
  {
    "status": "success",
    "prompt": "Show all employees",
    "detected_intents": ["get_all_employees"],
    "selected_tools": ["get_all_employees"],
    "message": "Successfully executed 1 tool(s).",
    "pending_operation": null,
    "tool_executions": [
      {
        "tool_name": "get_all_employees",
        "arguments": {},
        "success": true,
        "result": {
          "count": 20,
          "data": [ ... ],
          "message": "Successfully retrieved 20 employee(s).",
          "success": true
        },
        "error": null
      }
    ],
    "missing_parameters": null,
    "ambiguous_options": null
  }
  ```

### B. `POST /api/confirm`
Confirms a pending mutating write operation (`add_employee` or `update_employee`) using the token returned by `/api/chat`.

- **Request Body**:
  ```json
  {
    "token": "CONF-123456"
  }
  ```
- **Response Body (`200 OK`)**:
  Returns the execution result of the confirmed operation with status `"success"`. If the token is invalid, expired, or previously used, returns status `"validation_error"`.

### C. `GET /api/tools`
Returns the dynamic tool catalog and input schemas retrieved from the .NET MCP Server.

- **Response Body (`200 OK`)**:
  ```json
  {
    "count": 7,
    "tools": [
      {
        "name": "get_all_employees",
        "title": "Get All Employees",
        "description": "Return all employee records with their department and designation names.",
        "inputSchema": {
          "type": "object",
          "properties": {},
          "required": []
        }
      },
      ...
    ]
  }
  ```

### D. `GET /api/health`
Monitors health of FastAPI, the persistent Rust bridge process, and the underlying .NET MCP Server.

- **Response Body (`200 OK`)**:
  ```json
  {
    "status": "healthy",
    "rust_process": "running",
    "mcp_server": {
      "name": "dotnet-employee-mcp-server",
      "version": "1.0.0"
    },
    "tools_count": 7
  }
  ```

---

## 3. Configuration & Startup

### Environment Variables
Configure in `.env` or set in environment:
- `BACKEND_HOST`: Default `127.0.0.1`
- `BACKEND_PORT`: Default `8000`
- `RUST_BINARY_PATH`: Path to `rust-mcp-client.exe`
- `RUST_WORKING_DIR`: Working directory for Rust client
- `RUST_REQUEST_TIMEOUT_SECONDS`: Default `30.0`

### Starting the Server
```bash
uvicorn app.main:app --host 127.0.0.1 --port 8000 --reload
```
Interactive OpenAPI documentation will be available at:
`http://127.0.0.1:8000/docs`

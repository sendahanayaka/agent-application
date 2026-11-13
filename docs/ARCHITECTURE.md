# Architecture Overview & Project Specification

## 1. Request / Response Lifecycle

```text
React (Vite + TS)
       ↕  HTTP / REST JSON
Python FastAPI (Backend Gateway)
       ↕  Local REST / JSON IPC
Rust MCP Client (Subprocess Manager & Regex Intent Engine)
       ↕  STDIO (JSON-RPC 2.0)
.NET 8 C# MCP Server (Tool Host)
       ↕  Parameterized ADO.NET
Microsoft SQL Server (InternAgenticDB)
```

## 2. Directory Structure

- `frontend/`: React + TypeScript chat interface.
- `backend/`: Python + FastAPI REST gateway.
- `rust-mcp-client/`: Rust MCP Client, STDIO process manager, and Regex engine.
- `dotnet-mcp-server/`: C# / .NET 8 MCP Server exposing database tools.
- `database/`: Database creation and idempotent seeding (`InternAgenticDB_Seed.sql`).
- `docs/`: Technical specifications, architecture diagrams, and tool catalog.
- `tests/`: 30+ regex test scenarios and end-to-end integration tests.

## 3. Mandatory Constraints

- Strict deterministic Regex intent routing (no LLM / AI).
- .NET MCP Server launched as a child process by Rust over STDIO.
- All database operations mediated through MCP tools.
- Explicit confirmation in the UI for write operations (`add_employee`, `update_employee`).
- Parameterized SQL queries only.
- STDOUT reserved for JSON-RPC in .NET; diagnostic logs written to STDERR.

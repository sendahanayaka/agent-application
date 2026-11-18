# Employee Management System

Enterprise agentic application implementing Model Context Protocol (**MCP**) across **React, Python FastAPI, Rust, .NET 8, and Microsoft SQL Server**.

---

## 1. Mandatory Architecture

```text
React (Vite + TypeScript)
       ↕  HTTP / REST JSON
Python FastAPI (Backend Gateway)
       ↕  Newline-Delimited JSON IPC over STDIN / STDOUT
Rust MCP Client (Process Manager & Deterministic Regex Engine)
       ↕  STDIO (JSON-RPC 2.0)
.NET 8 C# MCP Server (Official MCP SDK Tool Provider)
       ↕  Parameterized ADO.NET / Dapper
Microsoft SQL Server (InternAgenticDB)
```

---

## 2. Key Constraints & Design Rules

- **Zero LLM / AI Integration**: Intent detection and parameter extraction are 100% deterministic using regular expressions and dynamic vocabulary mappings.
- **Process Orchestration**: FastAPI manages a persistent Rust background process; Rust spawns the .NET MCP Server as a child process and communicates via STDIO.
- **Dynamic Schema Authority**: MCP tool schemas are discovered dynamically via `tools/list` at runtime and strictly validated before tool execution.
- **Strict Database Access Control**: Only the .NET MCP Server connects to SQL Server using parameterized queries. Web backend and Rust client have no direct SQL access.
- **Write-Safety**: Mutating write operations (`add_employee`, `update_employee`) require explicit confirmation tokens (`CONF-XXXXXX`) before database modification.

---

## 3. Repository Organization

```text
Intern_Agentic_Assignment v1.0/
├── frontend/               # React + TypeScript Chat Interface (Phase 4)
├── backend/                # Python FastAPI Gateway & Process Orchestrator
│   ├── app/                # Application modules (endpoints, schemas, process manager)
│   └── tests/              # FastAPI integration tests
├── rust-mcp-client/        # Rust MCP Client, Regex Pipeline, and STDIO Child Process Host
│   ├── src/                # Pipeline engine, parameter extraction, protocol models
│   ├── tests/              # Unit and integration test suites
│   └── vocabulary.json     # Machine-readable runtime vocabulary source
├── dotnet-mcp-server/      # C# .NET 8 MCP Server with 7 tools and ADO.NET repository
├── database/               # SQL Server creation and idempotent seed script
│   └── InternAgenticDB_Seed.sql
├── docs/                   # Centralized Technical Documentation
│   ├── ARCHITECTURE.md     # System architecture and data flow
│   ├── API.md              # REST API endpoint specifications & examples
│   └── vocabulary.md       # Natural-language vocabulary matrix & alias guide
├── tests/                  # Shared / cross-cutting verification scripts
│   └── verify_mcp_stdio.py
├── .env.example            # Environment variable template (sanitized)
├── .gitignore              # Multi-stack gitignore
└── README.md               # Main project documentation
```

---

## 4. Documentation Index

- [Architecture & Request Lifecycle](docs/ARCHITECTURE.md)
- [FastAPI Backend API Reference](docs/API.md)
- [Vocabulary System & Intent Mapping](docs/vocabulary.md)

---

## 5. Verification Commands

### .NET MCP Server
```powershell
cd dotnet-mcp-server
dotnet build
```

### Rust MCP Client
```powershell
cd rust-mcp-client
cargo test
```

### Python FastAPI Backend
```powershell
cd backend
$env:PYTHONPATH = '.'
pytest -v tests/test_api.py
```

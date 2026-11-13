# Intern Agentic Application Assignment

## Files
- `Intern_Technical_Assignment_MCP_Agentic_Application.docx` — complete assignment brief.
- `InternAgenticDB_Seed.sql` — SQL Server database creation and idempotent dummy data seeding script.

## Key assignment constraints
- No LLM or AI model integration is required or permitted for this assignment.
- Prompt identification and tool routing must use Regex and deterministic rules in the Rust component.
- Rust launches the .NET/C# MCP Server and connects to it using STDIO.
- The web backend communicates with the Rust component through a documented internal REST/API interface.
- All employee database access must go through MCP tools. The web backend must not connect directly to SQL Server.

## Database setup
1. Open `InternAgenticDB_Seed.sql` in SQL Server Management Studio.
2. Execute the script using an account with permission to create the database and tables.
3. Confirm the result summary: 6 departments, 12 designations, and 20 employees on a fresh database.
4. The sample data is fictitious and intended only for training.

## Important implementation notes
- Keep STDOUT of the .NET MCP process reserved for MCP protocol messages; send logs to STDERR.
- Use parameterized SQL queries.
- Require explicit confirmation for add/update operations.
- Do not allow arbitrary SQL execution based on prompts.

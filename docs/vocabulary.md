# Vocabulary System Reference

This document serves as the human-readable reference for the deterministic Regex intent detection and parameter extraction system used by the Rust MCP Client in the **SPIL Labs Agentic Employee Management** application.

---

## 1. Design Principles

1. **Deterministic Rule-Based Matching**: No Large Language Models (LLMs) or external AI APIs are used. All prompt understanding operates via structured regular expressions and synonym dictionaries.
2. **Schema Authority**: The vocabulary identifies potential user intents and extracts candidate parameters. The dynamically discovered .NET MCP server schemas remain the authoritative source for validating tool arguments.
3. **Decoupled Architecture**: All synonyms, phrasing patterns, department aliases, and designations are maintained in `vocabulary.json`. New terminology can be added dynamically without modifying the underlying Rust matching engine.
4. **Safety & Confirmation**: Read-only queries execute seamlessly. Destructive or mutating write operations (`add_employee`, `update_employee`) require explicit user confirmation.

---

## 2. Intent & Tool Mapping Matrix

| Tool Name | Operation Type | Primary Action Verbs | Target Entity Nouns | Example Prompts |
| :--- | :---: | :--- | :--- | :--- |
| **`get_all_employees`** | Read | `get`, `show`, `list`, `fetch`, `display`, `view`, `pull` | `employees`, `staff`, `roster`, `personnel`, `everyone` | • *"Show all employees"*<br>• *"List the company staff"*<br>• *"Fetch employee roster"* |
| **`get_employee_by_id`** | Read | `get`, `find`, `lookup`, `show`, `retrieve`, `locate` | `employee`, `profile`, `record`, `details` + ID | • *"Find employee ID 1001"*<br>• *"Show details for employee 1005"*<br>• *"Lookup profile 1012"* |
| **`search_employees`** | Read | `search`, `find`, `filter`, `look for`, `query` | `employees`, `staff`, `people` + filter | • *"Search employees in IT department"*<br>• *"Find software engineers"*<br>• *"Show active staff in Finance"* |
| **`get_department_list`** | Read | `get`, `show`, `list`, `view`, `fetch` | `departments`, `divisions`, `units`, `teams` | • *"Show all departments"*<br>• *"List company divisions"*<br>• *"What departments exist?"* |
| **`get_employee_count`** | Read | `count`, `how many`, `total`, `calculate` | `employees`, `staff`, `headcount`, `workforce` | • *"How many employees are there?"*<br>• *"Total employee count in IT"*<br>• *"Headcount of active staff"* |
| **`add_employee`** | **Write (Confirmation Required)** | `add`, `create`, `insert`, `register`, `hire`, `onboard` | `employee`, `new hire`, `person` | • *"Add a new employee named Alice Smith"*<br>• *"Hire Bob Jones as Software Engineer"*<br>• *"Register new employee in IT"* |
| **`update_employee`** | **Write (Confirmation Required)** | `update`, `modify`, `change`, `set`, `edit`, `promote` | `employee`, `record`, `details` + ID | • *"Update employee 1001 department to IT"*<br>• *"Change salary of employee 1002 to 95000"*<br>• *"Set employee 1005 status to On Leave"* |

---

## 3. Parameter Extraction & Canonical Aliases

### A. Departments
Maps natural language variants to database canonical department names:
- **Human Resources** (ID: 1): `human resources`, `hr`, `personnel`, `people team`
- **Information Technology** (ID: 2): `information technology`, `it`, `tech`, `technology`, `engineering`, `dev`
- **Finance** (ID: 3): `finance`, `accounting`, `accounts`, `payroll`
- **Marketing** (ID: 4): `marketing`, `mktg`, `advertising`, `branding`
- **Sales** (ID: 5): `sales`, `commercial`, `business development`, `bizdev`
- **Operations** (ID: 6): `operations`, `ops`, `logistics`, `supply chain`

### B. Designations
Maps common job titles to canonical designations:
- **Software Engineer** (ID: 3): `software engineer`, `developer`, `programmer`, `swe`, `dev`
- **Senior Software Engineer** (ID: 4): `senior software engineer`, `sr software engineer`, `senior dev`, `lead developer`
- **QA Engineer** (ID: 5): `qa engineer`, `tester`, `qa`, `quality assurance`
- **HR Manager** (ID: 1): `hr manager`, `human resources manager`
- **Operations Manager** (ID: 12): `operations manager`, `ops manager`
- **Business Analyst** (ID: 13): `business analyst`, `ba`

### C. Employment Statuses
- **Active**: `active`, `employed`, `current`, `working`
- **On Leave**: `on leave`, `leave`, `vacation`, `absent`, `sick leave`
- **Terminated**: `terminated`, `fired`, `resigned`, `inactive`, `former`

### D. Scalar Fields
- **Employee ID**: Digits 3-6 long prefixed by `id`, `employee id`, `#`, or standalone 4-digit ID (`1001`-`9999`).
- **Salary**: Dollar or decimal amounts preceded by `salary`, `pay`, `compensation`, or `$`.
- **Email**: Standard RFC-compliant email regex.
- **Phone**: Phone format parser supporting `+`, hyphens, brackets, and whitespace.

---

## 4. Multi-Intent & Conflict Handling

1. **Compound Queries**: Prompts containing distinct clauses connected by `and`, `also`, or semicolons (e.g. *"Show all departments and count employees"*) are broken into sub-intents and evaluated sequentially.
2. **Ambiguity Resolution**: If a prompt matches conflicting intents with identical specificity, the engine outputs a structured `ClarificationRequired` response asking the user to choose between the matched options.

# Vocabulary System Reference

This document serves as the human-readable reference for the deterministic Regex intent detection and parameter extraction system used by the Rust MCP Client in the **Employee Management Agent** application.

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
| **`get_all_employees`** | Read | `get`, `show`, `list`, `fetch`, `display`, `view`, `pull`, `give` | `employees`, `staff`, `roster`, `personnel`, `everyone` | • *"Show all employees"*<br>• *"Give all the employees"*<br>• *"List the company staff"* |
| **`get_employee_by_id`** | Read | `get`, `find`, `lookup`, `show`, `retrieve`, `locate`, `give` | `employee`, `profile`, `record`, `details` + ID | • *"Find employee ID 1001"*<br>• *"Show employee ID 1001"*<br>• *"Show details for employee 1005"* |
| **`search_employees`** | Read | `search`, `find`, `filter`, `look for`, `query`, `show`, `list`, `give` | `employees`, `staff`, `people`, designations, departments | • *"Find Senior Software Engineers"*<br>• *"Show Project Managers"*<br>• *"Find Software Engineers in Engineering"*<br>• *"Show inactive employees"*<br>• *"Find employees who are on leave"* |
| **`get_department_list`** | Read | `get`, `show`, `list`, `view`, `fetch`, `give` | `departments`, `divisions`, `units`, `teams` | • *"Show all departments"*<br>• *"List company divisions"*<br>• *"What departments exist?"* |
| **`get_employee_count`** | Read | `count`, `how many`, `total`, `calculate` | `employees`, `staff`, `headcount`, `workforce`, designations, departments, statuses | • *"How many employees are there?"*<br>• *"Count Senior Software Engineers"*<br>• *"Count Project Managers"*<br>• *"Count Engineering employees"*<br>• *"Count active employees"* |
| **`add_employee`** | **Write (Confirmation Required)** | `add`, `create`, `insert`, `register`, `hire`, `onboard` | `employee`, `new hire`, `person` | • *"Add a new employee named Alice Smith"*<br>• *"Hire Bob Jones as Software Engineer"*<br>• *"Register new employee in IT"* |
| **`update_employee`** | **Write (Confirmation Required)** | `update`, `modify`, `change`, `set`, `edit`, `promote` | `employee`, `record`, `details` + ID | • *"Update employee 1001 department to IT"*<br>• *"Change salary of employee 1002 to 95000"*<br>• *"Set employee 1005 status to On Leave"* |

---

## 3. Parameter Extraction & Canonical Aliases

### A. Departments (InternAgenticDB)
Maps natural language variants to database canonical department names:
- **Engineering** (ID: 1): `engineering`, `software engineering`
- **IT** (ID: 2): `it`, `information technology`, `tech`, `technology`
- **HR** (ID: 3): `hr`, `human resources`, `personnel`, `people team`
- **Finance** (ID: 4): `finance`, `accounting`, `accounts`, `payroll`
- **Sales** (ID: 5): `sales`, `commercial`, `business development`, `bizdev`
- **Operations** (ID: 6): `operations`, `ops`, `logistics`, `supply chain`

### B. Designations (InternAgenticDB)
Maps common and plural job titles to canonical designations:
- **Senior Software Engineer** (ID: 4): `senior software engineers`, `senior software engineer`, `senior software developers`, `sr software engineers`, `sr software engineer`, `senior dev`, `lead developer`
- **Software Engineer** (ID: 1): `software engineers`, `software engineer`, `software developers`, `developers`, `developer`, `programmers`, `programmer`, `swe`
- **Project Manager** (ID: 3): `project managers`, `project manager`, `pms`, `pm`
- **Team Lead** (ID: 2): `team leads`, `team lead`, `tech leads`, `tech lead`, `leads`, `lead`
- **QA Engineer** (ID: 5): `qa engineers`, `qa engineer`, `testers`, `tester`, `qa`
- **Business Analyst** (ID: 6): `business analysts`, `business analyst`, `ba`
- **HR Executive** (ID: 7): `hr executives`, `hr executive`, `hr officers`
- **Accountant** (ID: 8): `accountants`, `accountant`, `finance analysts`
- **Sales Executive** (ID: 9): `sales executives`, `sales executive`, `sales reps`
- **DevOps Engineer** (ID: 10): `devops engineers`, `devops engineer`, `devops`, `sre`
- **UI/UX Designer** (ID: 11): `ui/ux designers`, `ui designers`, `ux designers`, `designers`, `designer`
- **Intern** (ID: 12): `interns`, `intern`, `trainees`, `trainee`

### C. Employment Statuses
- **Active**: `active`, `employed`, `current`, `working`
- **Inactive**: `inactive`, `terminated`, `fired`, `resigned`, `former`, `left`
- **On Leave**: `on leave`, `leave`, `vacation`, `absent`, `sick leave`

### D. Scalar Fields
- **Employee ID**: Digits 3-6 long prefixed by `id`, `employee id`, `#`, or standalone 4-digit ID (`1001`-`9999`).
- **Salary**: Dollar or decimal amounts preceded by `salary`, `pay`, `compensation`, or `$`.
- **Email**: Standard RFC-compliant email regex.
- **Phone**: Phone format parser supporting `+`, hyphens, brackets, and whitespace.

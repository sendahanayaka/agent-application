using System.ComponentModel;
using System.Text.Json;
using System.Threading.Tasks;
using DotnetMcpServer.Data;
using DotnetMcpServer.Models;
using ModelContextProtocol.Server;

namespace DotnetMcpServer.Tools;

[McpServerToolType]
public class EmployeeTools
{
    private readonly EmployeeRepository _repo;

    public EmployeeTools(EmployeeRepository repo)
    {
        _repo = repo;
    }

    [McpServerTool(Name = "get_all_employees", Title = "Get All Employees", ReadOnly = true)]
    [Description("Return all employee records with their department and designation names.")]
    public async Task<string> GetAllEmployees()
    {
        var result = await _repo.GetAllEmployeesAsync();
        return JsonSerializer.Serialize(result);
    }

    [McpServerTool(Name = "get_employee_by_id", Title = "Get Employee By ID", ReadOnly = true)]
    [Description("Retrieve one employee record by their unique EmployeeID.")]
    public async Task<string> GetEmployeeById(
        [Description("The unique integer EmployeeID of the employee")] int employee_id)
    {
        var result = await _repo.GetEmployeeByIdAsync(employee_id);
        return JsonSerializer.Serialize(result);
    }

    [McpServerTool(Name = "search_employees", Title = "Search Employees", ReadOnly = true)]
    [Description("Search for employees by name, department, designation, or status.")]
    public async Task<string> SearchEmployees(
        [Description("Optional search string matching first or last name")] string? name = null,
        [Description("Optional department name filter (e.g. IT, HR, Sales)")] string? department = null,
        [Description("Optional designation title filter (e.g. Software Engineer)")] string? designation = null,
        [Description("Optional status filter (e.g. Active, On Leave, Terminated)")] string? status = null)
    {
        var result = await _repo.SearchEmployeesAsync(name, department, designation, status);
        return JsonSerializer.Serialize(result);
    }

    [McpServerTool(Name = "get_department_list", Title = "Get Department List", ReadOnly = true)]
    [Description("Return all department records with their ID, name, and description.")]
    public async Task<string> GetDepartmentList()
    {
        var result = await _repo.GetDepartmentListAsync();
        return JsonSerializer.Serialize(result);
    }

    [McpServerTool(Name = "get_employee_count", Title = "Get Employee Count", ReadOnly = true)]
    [Description("Return the total count of employees, optionally filtered by department or status.")]
    public async Task<string> GetEmployeeCount(
        [Description("Optional department name filter")] string? department = null,
        [Description("Optional status filter (e.g. Active, On Leave)")] string? status = null)
    {
        var result = await _repo.GetEmployeeCountAsync(department, status);
        return JsonSerializer.Serialize(result);
    }

    [McpServerTool(Name = "add_employee", Title = "Add Employee", Destructive = false, ReadOnly = false)]
    [Description("Insert a new employee record after validating required fields.")]
    public async Task<string> AddEmployee(
        [Description("First name of the employee")] string first_name,
        [Description("Last name of the employee")] string last_name,
        [Description("Unique company email address")] string email,
        [Description("Department ID (e.g. 1 for HR, 2 for IT)")] int department_id,
        [Description("Designation ID (e.g. 3 for Software Engineer)")] int designation_id,
        [Description("Annual salary as a positive decimal")] decimal salary,
        [Description("Optional contact phone number")] string? phone = null,
        [Description("Optional hire date formatted as YYYY-MM-DD (defaults to today)")] string? hire_date = null,
        [Description("Employment status (Active, On Leave, Terminated). Defaults to Active")] string? status = "Active")
    {
        var result = await _repo.AddEmployeeAsync(
            first_name,
            last_name,
            email,
            phone,
            department_id,
            designation_id,
            salary,
            hire_date,
            status
        );
        return JsonSerializer.Serialize(result);
    }

    [McpServerTool(Name = "update_employee", Title = "Update Employee", Destructive = true, ReadOnly = false)]
    [Description("Update an existing employee record by EmployeeID after validating fields.")]
    public async Task<string> UpdateEmployee(
        [Description("The unique integer EmployeeID of the employee to update")] int employee_id,
        [Description("Optional updated first name")] string? first_name = null,
        [Description("Optional updated last name")] string? last_name = null,
        [Description("Optional updated company email address")] string? email = null,
        [Description("Optional updated contact phone number")] string? phone = null,
        [Description("Optional updated department ID")] int? department_id = null,
        [Description("Optional updated designation ID")] int? designation_id = null,
        [Description("Optional updated salary")] decimal? salary = null,
        [Description("Optional updated employment status")] string? status = null)
    {
        var result = await _repo.UpdateEmployeeAsync(
            employee_id,
            first_name,
            last_name,
            email,
            phone,
            department_id,
            designation_id,
            salary,
            status
        );
        return JsonSerializer.Serialize(result);
    }
}

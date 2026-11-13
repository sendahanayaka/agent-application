using System;
using System.Collections.Generic;
using System.Data;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using Dapper;
using DotnetMcpServer.Models;
using Microsoft.Data.SqlClient;

namespace DotnetMcpServer.Data;

public class EmployeeRepository
{
    private readonly string _connectionString;

    public EmployeeRepository(string? connectionString = null)
    {
        _connectionString = connectionString ?? DatabaseConfig.GetConnectionString();
    }

    private IDbConnection CreateConnection() => new SqlConnection(_connectionString);

    public async Task<ToolResponse> GetAllEmployeesAsync()
    {
        try
        {
            const string sql = @"
                SELECT 
                    e.EmployeeID,
                    e.FirstName,
                    e.LastName,
                    e.Email,
                    e.Phone,
                    e.DepartmentID,
                    d.DepartmentName,
                    e.DesignationID,
                    g.DesignationName,
                    e.Salary,
                    CONVERT(VARCHAR(10), e.HireDate, 120) AS HireDate,
                    e.Status
                FROM Employees e
                INNER JOIN Departments d ON e.DepartmentID = d.DepartmentID
                INNER JOIN Designations g ON e.DesignationID = g.DesignationID
                ORDER BY e.EmployeeID ASC;";

            using var conn = CreateConnection();
            var employees = (await conn.QueryAsync<EmployeeDto>(sql)).ToList();

            return ToolResponse.Ok(
                $"Successfully retrieved {employees.Count} employee(s).",
                employees,
                employees.Count
            );
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: GetAllEmployeesAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error retrieving employees: {ex.Message}");
        }
    }

    public async Task<ToolResponse> GetEmployeeByIdAsync(int employeeId)
    {
        if (employeeId <= 0)
        {
            return ToolResponse.Fail($"Invalid EmployeeID: {employeeId}. ID must be a positive integer.");
        }

        try
        {
            const string sql = @"
                SELECT 
                    e.EmployeeID,
                    e.FirstName,
                    e.LastName,
                    e.Email,
                    e.Phone,
                    e.DepartmentID,
                    d.DepartmentName,
                    e.DesignationID,
                    g.DesignationName,
                    e.Salary,
                    CONVERT(VARCHAR(10), e.HireDate, 120) AS HireDate,
                    e.Status
                FROM Employees e
                INNER JOIN Departments d ON e.DepartmentID = d.DepartmentID
                INNER JOIN Designations g ON e.DesignationID = g.DesignationID
                WHERE e.EmployeeID = @EmployeeID;";

            using var conn = CreateConnection();
            var employee = await conn.QuerySingleOrDefaultAsync<EmployeeDto>(sql, new { EmployeeID = employeeId });

            if (employee == null)
            {
                return ToolResponse.Fail($"Employee with ID {employeeId} was not found.");
            }

            return ToolResponse.Ok(
                $"Employee {employeeId} retrieved successfully.",
                employee,
                1
            );
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: GetEmployeeByIdAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error retrieving employee {employeeId}: {ex.Message}");
        }
    }

    public async Task<ToolResponse> SearchEmployeesAsync(
        string? name = null,
        string? department = null,
        string? designation = null,
        string? status = null)
    {
        try
        {
            var conditions = new List<string>();
            var parameters = new DynamicParameters();

            if (!string.IsNullOrWhiteSpace(name))
            {
                conditions.Add("(e.FirstName LIKE @Name OR e.LastName LIKE @Name OR (e.FirstName + ' ' + e.LastName) LIKE @Name)");
                parameters.Add("Name", $"%{name.Trim()}%");
            }

            if (!string.IsNullOrWhiteSpace(department))
            {
                conditions.Add("d.DepartmentName LIKE @Department");
                parameters.Add("Department", $"%{department.Trim()}%");
            }

            if (!string.IsNullOrWhiteSpace(designation))
            {
                conditions.Add("g.DesignationName LIKE @Designation");
                parameters.Add("Designation", $"%{designation.Trim()}%");
            }

            if (!string.IsNullOrWhiteSpace(status))
            {
                conditions.Add("e.Status = @Status");
                parameters.Add("Status", status.Trim());
            }

            var whereClause = conditions.Count > 0 ? "WHERE " + string.Join(" AND ", conditions) : "";

            var sql = $@"
                SELECT 
                    e.EmployeeID,
                    e.FirstName,
                    e.LastName,
                    e.Email,
                    e.Phone,
                    e.DepartmentID,
                    d.DepartmentName,
                    e.DesignationID,
                    g.DesignationName,
                    e.Salary,
                    CONVERT(VARCHAR(10), e.HireDate, 120) AS HireDate,
                    e.Status
                FROM Employees e
                INNER JOIN Departments d ON e.DepartmentID = d.DepartmentID
                INNER JOIN Designations g ON e.DesignationID = g.DesignationID
                {whereClause}
                ORDER BY e.EmployeeID ASC;";

            using var conn = CreateConnection();
            var employees = (await conn.QueryAsync<EmployeeDto>(sql, parameters)).ToList();

            return ToolResponse.Ok(
                $"Found {employees.Count} matching employee(s).",
                employees,
                employees.Count
            );
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: SearchEmployeesAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error searching employees: {ex.Message}");
        }
    }

    public async Task<ToolResponse> GetDepartmentListAsync()
    {
        try
        {
            const string sql = @"
                SELECT 
                    DepartmentID,
                    DepartmentName,
                    Description
                FROM Departments
                ORDER BY DepartmentID ASC;";

            using var conn = CreateConnection();
            var departments = (await conn.QueryAsync<DepartmentDto>(sql)).ToList();

            return ToolResponse.Ok(
                $"Retrieved {departments.Count} department(s).",
                departments,
                departments.Count
            );
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: GetDepartmentListAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error retrieving departments: {ex.Message}");
        }
    }

    public async Task<ToolResponse> GetEmployeeCountAsync(string? department = null, string? status = null)
    {
        try
        {
            var conditions = new List<string>();
            var parameters = new DynamicParameters();

            if (!string.IsNullOrWhiteSpace(department))
            {
                conditions.Add("d.DepartmentName LIKE @Department");
                parameters.Add("Department", $"%{department.Trim()}%");
            }

            if (!string.IsNullOrWhiteSpace(status))
            {
                conditions.Add("e.Status = @Status");
                parameters.Add("Status", status.Trim());
            }

            var whereClause = conditions.Count > 0 ? "WHERE " + string.Join(" AND ", conditions) : "";

            var sql = $@"
                SELECT COUNT(1)
                FROM Employees e
                INNER JOIN Departments d ON e.DepartmentID = d.DepartmentID
                {whereClause};";

            using var conn = CreateConnection();
            var count = await conn.ExecuteScalarAsync<int>(sql, parameters);

            return ToolResponse.Ok(
                $"Total matching employees: {count}.",
                new { count, department, status },
                count
            );
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: GetEmployeeCountAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error counting employees: {ex.Message}");
        }
    }

    public async Task<ToolResponse> AddEmployeeAsync(
        string firstName,
        string lastName,
        string email,
        string? phone,
        int departmentId,
        int designationId,
        decimal salary,
        string? hireDate = null,
        string? status = "Active")
    {
        // Validation
        if (string.IsNullOrWhiteSpace(firstName))
            return ToolResponse.Fail("First name is required.");
        if (string.IsNullOrWhiteSpace(lastName))
            return ToolResponse.Fail("Last name is required.");
        if (string.IsNullOrWhiteSpace(email))
            return ToolResponse.Fail("Email is required.");

        var emailRegex = new Regex(@"^[^@\s]+@[^@\s]+\.[^@\s]+$");
        if (!emailRegex.IsMatch(email.Trim()))
            return ToolResponse.Fail($"Invalid email format: '{email}'.");

        if (salary <= 0)
            return ToolResponse.Fail($"Salary must be greater than zero. Provided: {salary}.");

        DateTime parsedHireDate = DateTime.UtcNow.Date;
        if (!string.IsNullOrWhiteSpace(hireDate))
        {
            if (!DateTime.TryParse(hireDate, out parsedHireDate))
            {
                return ToolResponse.Fail($"Invalid hireDate format: '{hireDate}'. Expected YYYY-MM-DD.");
            }
        }

        var effectiveStatus = string.IsNullOrWhiteSpace(status) ? "Active" : status.Trim();

        try
        {
            using var conn = CreateConnection();

            // Validate Department existence
            var deptExists = await conn.ExecuteScalarAsync<int>(
                "SELECT COUNT(1) FROM Departments WHERE DepartmentID = @DepartmentID",
                new { DepartmentID = departmentId });
            if (deptExists == 0)
            {
                return ToolResponse.Fail($"Department with ID {departmentId} does not exist.");
            }

            // Validate Designation existence
            var desigExists = await conn.ExecuteScalarAsync<int>(
                "SELECT COUNT(1) FROM Designations WHERE DesignationID = @DesignationID",
                new { DesignationID = designationId });
            if (desigExists == 0)
            {
                return ToolResponse.Fail($"Designation with ID {designationId} does not exist.");
            }

            // Validate Unique Email
            var emailExists = await conn.ExecuteScalarAsync<int>(
                "SELECT COUNT(1) FROM Employees WHERE Email = @Email",
                new { Email = email.Trim() });
            if (emailExists > 0)
            {
                return ToolResponse.Fail($"An employee with email '{email.Trim()}' already exists.");
            }

            const string insertSql = @"
                INSERT INTO Employees (FirstName, LastName, Email, Phone, DepartmentID, DesignationID, Salary, HireDate, Status)
                VALUES (@FirstName, @LastName, @Email, @Phone, @DepartmentID, @DesignationID, @Salary, @HireDate, @Status);
                SELECT CAST(SCOPE_IDENTITY() as int);";

            var newId = await conn.ExecuteScalarAsync<int>(insertSql, new
            {
                FirstName = firstName.Trim(),
                LastName = lastName.Trim(),
                Email = email.Trim(),
                Phone = string.IsNullOrWhiteSpace(phone) ? null : phone.Trim(),
                DepartmentID = departmentId,
                DesignationID = designationId,
                Salary = salary,
                HireDate = parsedHireDate,
                Status = effectiveStatus
            });

            return await GetEmployeeByIdAsync(newId);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: AddEmployeeAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error adding employee: {ex.Message}");
        }
    }

    public async Task<ToolResponse> UpdateEmployeeAsync(
        int employeeId,
        string? firstName = null,
        string? lastName = null,
        string? email = null,
        string? phone = null,
        int? departmentId = null,
        int? designationId = null,
        decimal? salary = null,
        string? status = null)
    {
        if (employeeId <= 0)
        {
            return ToolResponse.Fail($"Invalid EmployeeID: {employeeId}. ID must be a positive integer.");
        }

        try
        {
            using var conn = CreateConnection();

            // Verify employee exists
            var existing = await conn.QuerySingleOrDefaultAsync<EmployeeDto>(
                "SELECT * FROM Employees WHERE EmployeeID = @EmployeeID",
                new { EmployeeID = employeeId });

            if (existing == null)
            {
                return ToolResponse.Fail($"Employee with ID {employeeId} does not exist.");
            }

            var updates = new List<string>();
            var parameters = new DynamicParameters();
            parameters.Add("EmployeeID", employeeId);

            if (!string.IsNullOrWhiteSpace(firstName))
            {
                updates.Add("FirstName = @FirstName");
                parameters.Add("FirstName", firstName.Trim());
            }

            if (!string.IsNullOrWhiteSpace(lastName))
            {
                updates.Add("LastName = @LastName");
                parameters.Add("LastName", lastName.Trim());
            }

            if (!string.IsNullOrWhiteSpace(email))
            {
                var emailRegex = new Regex(@"^[^@\s]+@[^@\s]+\.[^@\s]+$");
                if (!emailRegex.IsMatch(email.Trim()))
                    return ToolResponse.Fail($"Invalid email format: '{email}'.");

                // Check email collision
                var emailCount = await conn.ExecuteScalarAsync<int>(
                    "SELECT COUNT(1) FROM Employees WHERE Email = @Email AND EmployeeID != @EmployeeID",
                    new { Email = email.Trim(), EmployeeID = employeeId });
                if (emailCount > 0)
                {
                    return ToolResponse.Fail($"Email '{email.Trim()}' is already in use by another employee.");
                }

                updates.Add("Email = @Email");
                parameters.Add("Email", email.Trim());
            }

            if (phone != null)
            {
                updates.Add("Phone = @Phone");
                parameters.Add("Phone", string.IsNullOrWhiteSpace(phone) ? null : phone.Trim());
            }

            if (departmentId.HasValue)
            {
                var deptCount = await conn.ExecuteScalarAsync<int>(
                    "SELECT COUNT(1) FROM Departments WHERE DepartmentID = @DepartmentID",
                    new { DepartmentID = departmentId.Value });
                if (deptCount == 0)
                {
                    return ToolResponse.Fail($"Department with ID {departmentId.Value} does not exist.");
                }
                updates.Add("DepartmentID = @DepartmentID");
                parameters.Add("DepartmentID", departmentId.Value);
            }

            if (designationId.HasValue)
            {
                var desigCount = await conn.ExecuteScalarAsync<int>(
                    "SELECT COUNT(1) FROM Designations WHERE DesignationID = @DesignationID",
                    new { DesignationID = designationId.Value });
                if (desigCount == 0)
                {
                    return ToolResponse.Fail($"Designation with ID {designationId.Value} does not exist.");
                }
                updates.Add("DesignationID = @DesignationID");
                parameters.Add("DesignationID", designationId.Value);
            }

            if (salary.HasValue)
            {
                if (salary.Value <= 0)
                    return ToolResponse.Fail($"Salary must be greater than zero. Provided: {salary.Value}.");

                updates.Add("Salary = @Salary");
                parameters.Add("Salary", salary.Value);
            }

            if (!string.IsNullOrWhiteSpace(status))
            {
                updates.Add("Status = @Status");
                parameters.Add("Status", status.Trim());
            }

            if (updates.Count == 0)
            {
                return ToolResponse.Fail("No update fields provided.");
            }

            var updateSql = $"UPDATE Employees SET {string.Join(", ", updates)} WHERE EmployeeID = @EmployeeID;";
            await conn.ExecuteAsync(updateSql, parameters);

            return await GetEmployeeByIdAsync(employeeId);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[Repository Error: UpdateEmployeeAsync] {ex.Message}");
            return ToolResponse.Fail($"Database error updating employee {employeeId}: {ex.Message}");
        }
    }
}

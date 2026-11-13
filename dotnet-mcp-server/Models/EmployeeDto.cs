namespace DotnetMcpServer.Models;

public class EmployeeDto
{
    public int EmployeeID { get; set; }
    public string FirstName { get; set; } = string.Empty;
    public string LastName { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string? Phone { get; set; }
    public int DepartmentID { get; set; }
    public string DepartmentName { get; set; } = string.Empty;
    public int DesignationID { get; set; }
    public string DesignationName { get; set; } = string.Empty;
    public decimal Salary { get; set; }
    public string HireDate { get; set; } = string.Empty;
    public string Status { get; set; } = string.Empty;
}

public class DepartmentDto
{
    public int DepartmentID { get; set; }
    public string DepartmentName { get; set; } = string.Empty;
    public string? Description { get; set; }
}

public class DesignationDto
{
    public int DesignationID { get; set; }
    public string DesignationName { get; set; } = string.Empty;
    public string? Description { get; set; }
}

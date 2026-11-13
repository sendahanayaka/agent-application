/*
    INTERN AGENTIC APPLICATION
    SQL SERVER DATABASE CREATION AND DUMMY DATA SEEDING
    Database: InternAgenticDB

    Safe to re-run:
      - Creates the database if it does not exist.
      - Creates tables and indexes if they do not exist.
      - Inserts reference/sample records only when they do not already exist.
      - Does not delete or overwrite existing records.

    IMPORTANT:
      - Run this script in SQL Server Management Studio (SSMS) or sqlcmd.
      - The database is for training/demo use only. All names and contact details are fictitious.
*/

USE master;
GO

IF DB_ID(N'InternAgenticDB') IS NULL
BEGIN
    CREATE DATABASE InternAgenticDB;
END;
GO

USE InternAgenticDB;
GO

/* 1. Departments */
IF OBJECT_ID(N'dbo.Departments', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.Departments
    (
        DepartmentID   INT IDENTITY(1,1) NOT NULL
            CONSTRAINT PK_Departments PRIMARY KEY,
        DepartmentName NVARCHAR(100) NOT NULL
            CONSTRAINT UQ_Departments_DepartmentName UNIQUE,
        [Description]  NVARCHAR(255) NULL
    );
END;
GO

/* 2. Designations */
IF OBJECT_ID(N'dbo.Designations', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.Designations
    (
        DesignationID   INT IDENTITY(1,1) NOT NULL
            CONSTRAINT PK_Designations PRIMARY KEY,
        DesignationName NVARCHAR(100) NOT NULL
            CONSTRAINT UQ_Designations_DesignationName UNIQUE,
        [Description]   NVARCHAR(255) NULL
    );
END;
GO

/* 3. Employees */
IF OBJECT_ID(N'dbo.Employees', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.Employees
    (
        EmployeeID   INT IDENTITY(1001,1) NOT NULL
            CONSTRAINT PK_Employees PRIMARY KEY,
        FirstName    NVARCHAR(100) NOT NULL,
        LastName     NVARCHAR(100) NOT NULL,
        Email        NVARCHAR(254) NOT NULL
            CONSTRAINT UQ_Employees_Email UNIQUE,
        Phone        NVARCHAR(30) NULL,
        DepartmentID INT NOT NULL,
        DesignationID INT NOT NULL,
        Salary       DECIMAL(12,2) NOT NULL,
        HireDate     DATE NOT NULL,
        [Status]     NVARCHAR(20) NOT NULL
            CONSTRAINT DF_Employees_Status DEFAULT (N'Active'),

        CONSTRAINT FK_Employees_Departments
            FOREIGN KEY (DepartmentID)
            REFERENCES dbo.Departments(DepartmentID),

        CONSTRAINT FK_Employees_Designations
            FOREIGN KEY (DesignationID)
            REFERENCES dbo.Designations(DesignationID),

        CONSTRAINT CK_Employees_Salary
            CHECK (Salary >= 0),

        CONSTRAINT CK_Employees_Status
            CHECK ([Status] IN (N'Active', N'Inactive', N'On Leave'))
    );
END;
GO

/* 4. Seed departments */
INSERT INTO dbo.Departments (DepartmentName, [Description])
SELECT src.DepartmentName, src.[Description]
FROM (VALUES
    (N'IT',          N'Information Technology Department'),
    (N'HR',          N'Human Resources Department'),
    (N'Finance',     N'Finance and Accounting Department'),
    (N'Sales',       N'Sales and Marketing Department'),
    (N'Engineering', N'Software and Product Engineering'),
    (N'Operations',  N'Business Operations Department')
) AS src(DepartmentName, [Description])
WHERE NOT EXISTS
(
    SELECT 1
    FROM dbo.Departments AS d
    WHERE d.DepartmentName = src.DepartmentName
);
GO

/* 5. Seed designations - includes Business Analyst, used by sample employee */
INSERT INTO dbo.Designations (DesignationName, [Description])
SELECT src.DesignationName, src.[Description]
FROM (VALUES
    (N'Software Engineer',          N'Software development'),
    (N'Senior Software Engineer',   N'Senior software development'),
    (N'Team Lead',                  N'Technical team leadership'),
    (N'Project Manager',            N'Project planning and management'),
    (N'QA Engineer',                N'Quality assurance and testing'),
    (N'Business Analyst',           N'Business analysis'),
    (N'HR Executive',               N'Human resources operations'),
    (N'Accountant',                 N'Financial accounting'),
    (N'Sales Executive',            N'Sales operations'),
    (N'DevOps Engineer',            N'Infrastructure and deployment'),
    (N'UI/UX Designer',             N'User interface and experience design'),
    (N'Intern',                     N'Trainee / internship position')
) AS src(DesignationName, [Description])
WHERE NOT EXISTS
(
    SELECT 1
    FROM dbo.Designations AS g
    WHERE g.DesignationName = src.DesignationName
);
GO

/* 6. Seed 20 fictitious employees.
      EmployeeID is generated automatically starting at 1001 on a fresh database.
      The script uses Email as the stable duplicate-prevention key. */
INSERT INTO dbo.Employees
(
    FirstName, LastName, Email, Phone,
    DepartmentID, DesignationID, Salary, HireDate, [Status]
)
SELECT
    src.FirstName,
    src.LastName,
    src.Email,
    src.Phone,
    d.DepartmentID,
    g.DesignationID,
    src.Salary,
    src.HireDate,
    src.[Status]
FROM
(
    VALUES
    (N'John',          N'Smith',     N'john.smith@example.com',          N'0771001001', N'IT',          N'Software Engineer',        CAST(85000 AS DECIMAL(12,2)),  CONVERT(date,'2024-01-15'), N'Active'),
    (N'David',         N'Miller',    N'david.miller@example.com',        N'0771001002', N'IT',          N'Senior Software Engineer', CAST(145000 AS DECIMAL(12,2)), CONVERT(date,'2022-05-10'), N'Active'),
    (N'Sarah',         N'Johnson',   N'sarah.johnson@example.com',       N'0771001003', N'HR',          N'HR Executive',              CAST(75000 AS DECIMAL(12,2)),  CONVERT(date,'2023-03-20'), N'Active'),
    (N'Michael',       N'Brown',     N'michael.brown@example.com',       N'0771001004', N'Finance',     N'Accountant',                CAST(90000 AS DECIMAL(12,2)),  CONVERT(date,'2021-08-12'), N'Active'),
    (N'Emily',         N'Davis',     N'emily.davis@example.com',         N'0771001005', N'Sales',       N'Sales Executive',           CAST(80000 AS DECIMAL(12,2)),  CONVERT(date,'2024-02-01'), N'Active'),
    (N'James',         N'Wilson',    N'james.wilson@example.com',        N'0771001006', N'Engineering', N'Team Lead',                 CAST(180000 AS DECIMAL(12,2)), CONVERT(date,'2020-06-15'), N'Active'),
    (N'Robert',        N'Taylor',    N'robert.taylor@example.com',       N'0771001007', N'IT',          N'DevOps Engineer',           CAST(130000 AS DECIMAL(12,2)), CONVERT(date,'2022-09-01'), N'Active'),
    (N'Linda',         N'Anderson',  N'linda.anderson@example.com',      N'0771001008', N'Engineering', N'QA Engineer',               CAST(95000 AS DECIMAL(12,2)),  CONVERT(date,'2023-07-10'), N'Active'),
    (N'William',       N'Thomas',    N'william.thomas@example.com',      N'0771001009', N'Operations',  N'Project Manager',           CAST(160000 AS DECIMAL(12,2)), CONVERT(date,'2021-01-25'), N'Active'),
    (N'Jessica',       N'Moore',     N'jessica.moore@example.com',       N'0771001010', N'IT',          N'UI/UX Designer',            CAST(105000 AS DECIMAL(12,2)), CONVERT(date,'2023-11-05'), N'Active'),
    (N'Daniel',        N'Martin',    N'daniel.martin@example.com',       N'0771001011', N'Finance',     N'Accountant',                CAST(92000 AS DECIMAL(12,2)),  CONVERT(date,'2022-04-18'), N'Active'),
    (N'Jennifer',      N'Lee',       N'jennifer.lee@example.com',        N'0771001012', N'HR',          N'Project Manager',           CAST(125000 AS DECIMAL(12,2)), CONVERT(date,'2020-10-12'), N'Active'),
    (N'Christopher',   N'White',     N'christopher.white@example.com',   N'0771001013', N'Sales',       N'Sales Executive',           CAST(78000 AS DECIMAL(12,2)),  CONVERT(date,'2024-04-08'), N'Active'),
    (N'Amanda',        N'Harris',    N'amanda.harris@example.com',       N'0771001014', N'Engineering', N'Software Engineer',         CAST(88000 AS DECIMAL(12,2)),  CONVERT(date,'2024-06-01'), N'Active'),
    (N'Matthew',       N'Clark',     N'matthew.clark@example.com',       N'0771001015', N'IT',          N'Intern',                    CAST(35000 AS DECIMAL(12,2)),  CONVERT(date,'2025-01-06'), N'Active'),
    (N'Ashley',        N'Lewis',     N'ashley.lewis@example.com',        N'0771001016', N'Engineering', N'QA Engineer',               CAST(97000 AS DECIMAL(12,2)),  CONVERT(date,'2022-12-15'), N'Inactive'),
    (N'Andrew',        N'Walker',    N'andrew.walker@example.com',       N'0771001017', N'IT',          N'Software Engineer',         CAST(90000 AS DECIMAL(12,2)),  CONVERT(date,'2023-05-22'), N'On Leave'),
    (N'Nicole',        N'Hall',      N'nicole.hall@example.com',         N'0771001018', N'Operations',  N'Business Analyst',          CAST(115000 AS DECIMAL(12,2)), CONVERT(date,'2021-09-13'), N'Active'),
    (N'Ryan',          N'Allen',     N'ryan.allen@example.com',          N'0771001019', N'Engineering', N'Intern',                    CAST(35000 AS DECIMAL(12,2)),  CONVERT(date,'2025-02-10'), N'Active'),
    (N'Sophia',        N'Young',     N'sophia.young@example.com',        N'0771001020', N'IT',          N'Team Lead',                 CAST(190000 AS DECIMAL(12,2)), CONVERT(date,'2019-03-11'), N'Active')
) AS src
(
    FirstName, LastName, Email, Phone,
    DepartmentName, DesignationName,
    Salary, HireDate, [Status]
)
INNER JOIN dbo.Departments AS d
    ON d.DepartmentName = src.DepartmentName
INNER JOIN dbo.Designations AS g
    ON g.DesignationName = src.DesignationName
WHERE NOT EXISTS
(
    SELECT 1
    FROM dbo.Employees AS e
    WHERE e.Email = src.Email
);
GO

/* 7. Indexes for common filtering and lookup operations */
IF NOT EXISTS
(
    SELECT 1 FROM sys.indexes
    WHERE name = N'IX_Employees_DepartmentID'
      AND object_id = OBJECT_ID(N'dbo.Employees')
)
BEGIN
    CREATE INDEX IX_Employees_DepartmentID
        ON dbo.Employees(DepartmentID);
END;
GO

IF NOT EXISTS
(
    SELECT 1 FROM sys.indexes
    WHERE name = N'IX_Employees_DesignationID'
      AND object_id = OBJECT_ID(N'dbo.Employees')
)
BEGIN
    CREATE INDEX IX_Employees_DesignationID
        ON dbo.Employees(DesignationID);
END;
GO

IF NOT EXISTS
(
    SELECT 1 FROM sys.indexes
    WHERE name = N'IX_Employees_Status'
      AND object_id = OBJECT_ID(N'dbo.Employees')
)
BEGIN
    CREATE INDEX IX_Employees_Status
        ON dbo.Employees([Status]);
END;
GO

/* 8. Verification summary */
SELECT
    (SELECT COUNT(*) FROM dbo.Departments) AS TotalDepartments,
    (SELECT COUNT(*) FROM dbo.Designations) AS TotalDesignations,
    (SELECT COUNT(*) FROM dbo.Employees) AS TotalEmployees;
GO

/* 9. Employee listing with related names */
SELECT
    e.EmployeeID,
    e.FirstName,
    e.LastName,
    e.Email,
    e.Phone,
    d.DepartmentName,
    g.DesignationName,
    e.Salary,
    e.HireDate,
    e.[Status]
FROM dbo.Employees AS e
INNER JOIN dbo.Departments AS d
    ON d.DepartmentID = e.DepartmentID
INNER JOIN dbo.Designations AS g
    ON g.DesignationID = e.DesignationID
ORDER BY e.EmployeeID;
GO

PRINT N'InternAgenticDB setup and seeding completed.';
GO

import React from 'react';
import {
  Users,
  Search,
  UserCheck,
  Building,
  Mail,
  Phone,
  Calendar,
  DollarSign,
  AlertCircle,
} from 'lucide-react';

export interface Employee {
  EmployeeID?: number;
  FirstName?: string;
  LastName?: string;
  FullName?: string;
  Email?: string;
  Phone?: string;
  HireDate?: string;
  Salary?: number;
  DepartmentID?: number;
  DepartmentName?: string;
  DesignationID?: number;
  DesignationName?: string;
  DesignationTitle?: string;
  Status?: string;
}

export interface Department {
  DepartmentID?: number;
  DepartmentName?: string;
  Description?: string;
}

interface ResultRendererProps {
  result: any;
  toolName: string;
}

export const ResultRenderer: React.FC<ResultRendererProps> = ({ result, toolName }) => {
  if (result === undefined || result === null) {
    return null;
  }

  // 1. Unpack result JSON safely
  let raw = result;
  if (typeof result === 'string') {
    try {
      raw = JSON.parse(result);
    } catch {
      raw = { message: result };
    }
  }

  // Extract common MCP ToolResponse envelope: { success, message, count, data, error }
  const message = raw?.message ?? (typeof raw === 'string' ? raw : '');
  const count = raw?.count ?? (Array.isArray(raw?.data) ? raw.data.length : undefined);
  const data = raw?.data !== undefined ? raw.data : (Array.isArray(raw) ? raw : raw);
  const isError = raw?.success === false || !!raw?.error;

  if (isError) {
    return (
      <div className="my-2.5 p-3 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-rose-800 dark:text-rose-300 text-xs flex items-start gap-2.5">
        <AlertCircle className="w-4 h-4 text-rose-600 dark:text-rose-400 shrink-0 mt-0.5" />
        <div>
          <div className="font-semibold">{raw.error || message || 'Operation failed'}</div>
          {raw.error && message && message !== raw.error && (
            <div className="mt-0.5 text-[11px] opacity-90">{message}</div>
          )}
        </div>
      </div>
    );
  }

  // 2. Tool: get_employee_count
  if (toolName === 'get_employee_count') {
    const finalCount = typeof data === 'number' ? data : (count ?? (typeof raw === 'number' ? raw : 0));
    return (
      <div className="my-2.5 p-4 rounded-xl bg-white dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border shadow-xs inline-flex items-center gap-4">
        <div className="w-12 h-12 rounded-xl bg-primary/10 dark:bg-primary-light/20 text-primary dark:text-orange-400 flex items-center justify-center font-bold text-xl">
          {finalCount}
        </div>
        <div>
          <div className="text-xs font-semibold text-slate-800 dark:text-darktext-base">
            Total Employee Headcount
          </div>
          <div className="text-[11px] text-slate-500 dark:text-darktext-muted mt-0.5">
            {message || 'Matching employee records currently in the system'}
          </div>
        </div>
      </div>
    );
  }

  // 3. Tool: get_employee_by_id (or single employee returned)
  if (toolName === 'get_employee_by_id' || (!Array.isArray(data) && (data?.EmployeeID || data?.FirstName))) {
    const emp: Employee = data?.Employee || data?.employee || data;
    if (!emp || (!emp.EmployeeID && !emp.FirstName)) {
      return (
        <div className="my-2 p-3 text-xs text-slate-500 dark:text-darktext-muted italic bg-slate-50 dark:bg-darkbg-surface rounded-lg border border-slate-200 dark:border-darkbg-border">
          {message || 'No employee record found with the specified ID.'}
        </div>
      );
    }

    const isActive = emp.Status === 'Active';
    const isOnLeave = emp.Status === 'On Leave';

    return (
      <div className="my-2.5 overflow-hidden rounded-xl bg-white dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border shadow-xs text-xs">
        {/* Card Header */}
        <div className="p-3.5 border-b border-slate-100 dark:border-darkbg-border bg-slate-50/70 dark:bg-darkbg-secondary flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-primary text-white flex items-center justify-center font-bold text-xs">
              {emp.FirstName?.[0]}{emp.LastName?.[0]}
            </div>
            <div>
              <div className="font-semibold text-sm text-slate-900 dark:text-darktext-base">
                {emp.FullName || `${emp.FirstName || ''} ${emp.LastName || ''}`.trim()}
              </div>
              <div className="text-[11px] text-slate-500 dark:text-darktext-muted">
                {emp.DesignationName || emp.DesignationTitle || `Designation ID: ${emp.DesignationID}`} • {emp.DepartmentName || `Dept ID: ${emp.DepartmentID}`}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="font-mono text-[11px] bg-slate-200/80 dark:bg-darkbg-surface px-2 py-0.5 rounded text-slate-600 dark:text-darktext-muted">
              ID: {emp.EmployeeID}
            </span>
            <span className={`px-2 py-0.5 rounded-full text-[10px] font-semibold ${
              isActive
                ? 'bg-emerald-100 dark:bg-emerald-950 text-emerald-800 dark:text-emerald-300'
                : isOnLeave
                ? 'bg-amber-100 dark:bg-amber-950 text-amber-800 dark:text-amber-300'
                : 'bg-rose-100 dark:bg-rose-950 text-rose-800 dark:text-rose-300'
            }`}>
              {emp.Status || 'Active'}
            </span>
          </div>
        </div>

        {/* Profile Details Grid */}
        <div className="p-3.5 grid grid-cols-1 sm:grid-cols-2 gap-3 text-slate-600 dark:text-darktext-muted">
          <div className="flex items-center gap-2">
            <Mail className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0" />
            <span className="font-mono text-[11px] text-slate-800 dark:text-darktext-base truncate">{emp.Email}</span>
          </div>
          <div className="flex items-center gap-2">
            <Phone className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0" />
            <span className="text-slate-800 dark:text-darktext-base">{emp.Phone || 'None listed'}</span>
          </div>
          <div className="flex items-center gap-2">
            <Calendar className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0" />
            <span>Hired: <strong className="text-slate-800 dark:text-darktext-base">{emp.HireDate ? new Date(emp.HireDate).toLocaleDateString() : 'N/A'}</strong></span>
          </div>
          <div className="flex items-center gap-2">
            <DollarSign className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0" />
            <span>Annual Salary: <strong className="text-slate-800 dark:text-darktext-base">{emp.Salary !== undefined ? `$${emp.Salary.toLocaleString()}` : 'N/A'}</strong></span>
          </div>
        </div>
      </div>
    );
  }

  // 4. Tool: get_department_list
  if (toolName === 'get_department_list' || (Array.isArray(data) && data[0]?.DepartmentName && !data[0]?.FirstName)) {
    const list: Department[] = Array.isArray(data) ? data : [];
    return (
      <div className="my-2.5 overflow-hidden rounded-xl border border-slate-200 dark:border-darkbg-border bg-white dark:bg-darkbg-surface shadow-xs">
        <div className="bg-slate-50 dark:bg-darkbg-secondary px-3.5 py-2.5 text-xs font-semibold text-slate-700 dark:text-darktext-base border-b border-slate-200 dark:border-darkbg-border flex items-center justify-between">
          <span className="flex items-center gap-1.5">
            <Building className="w-3.5 h-3.5 text-primary dark:text-orange-400" />
            <span>Departments ({list.length})</span>
          </span>
          <span className="text-[11px] font-normal text-slate-400 dark:text-darktext-muted">Master Records</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs text-slate-700 dark:text-darktext-base">
            <thead className="bg-slate-100/75 dark:bg-darkbg-surface text-[11px] font-semibold text-slate-500 dark:text-darktext-muted uppercase border-b border-slate-200 dark:border-darkbg-border">
              <tr>
                <th className="px-3.5 py-2 w-16">ID</th>
                <th className="px-3.5 py-2">Department Name</th>
                <th className="px-3.5 py-2">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-darkbg-border">
              {list.map((dept) => (
                <tr key={dept.DepartmentID} className="hover:bg-slate-50/70 dark:hover:bg-darkbg-secondary/50">
                  <td className="px-3.5 py-2.5 font-mono font-medium text-slate-500 dark:text-darktext-muted">{dept.DepartmentID}</td>
                  <td className="px-3.5 py-2.5 font-semibold text-slate-900 dark:text-darktext-base">{dept.DepartmentName}</td>
                  <td className="px-3.5 py-2.5 text-slate-600 dark:text-darktext-muted">{dept.Description || '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  // 5. Tool: get_all_employees, search_employees (Employee Lists)
  if (Array.isArray(data)) {
    const employees: Employee[] = data;

    if (employees.length === 0) {
      return (
        <div className="my-2.5 p-3.5 text-xs text-slate-600 dark:text-darktext-muted bg-slate-50 dark:bg-darkbg-surface rounded-xl border border-slate-200 dark:border-darkbg-border flex items-center gap-2">
          <Search className="w-4 h-4 text-slate-400" />
          <span>
            {toolName === 'search_employees'
              ? 'No employees found matching your search criteria.'
              : 'No employee records currently available in the database.'}
          </span>
        </div>
      );
    }

    return (
      <div className="my-2.5 overflow-hidden rounded-xl border border-slate-200 dark:border-darkbg-border bg-white dark:bg-darkbg-surface shadow-xs">
        {/* Table Summary Bar */}
        <div className="bg-slate-50 dark:bg-darkbg-secondary px-3.5 py-2.5 text-xs font-semibold text-slate-700 dark:text-darktext-base border-b border-slate-200 dark:border-darkbg-border flex items-center justify-between">
          <span className="flex items-center gap-2">
            <Users className="w-4 h-4 text-primary dark:text-orange-400" />
            <span>
              {toolName === 'search_employees' ? 'Search Results' : 'Employee Directory'}
            </span>
            <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-primary/10 dark:bg-orange-500/20 text-primary dark:text-orange-400">
              {employees.length} {employees.length === 1 ? 'record' : 'records'}
            </span>
          </span>
          <span className="text-[11px] font-normal text-slate-400 dark:text-darktext-muted">
            SQL Server
          </span>
        </div>

        {/* Scrollable Data Table */}
        <div className="overflow-x-auto max-h-80">
          <table className="w-full text-left text-xs text-slate-700 dark:text-darktext-base">
            <thead className="bg-slate-100/80 dark:bg-darkbg-surface text-[10px] font-semibold text-slate-500 dark:text-darktext-muted uppercase border-b border-slate-200 dark:border-darkbg-border sticky top-0 z-10 backdrop-blur-xs">
              <tr>
                <th className="px-3 py-2 w-14">ID</th>
                <th className="px-3 py-2">Full Name</th>
                <th className="px-3 py-2">Department</th>
                <th className="px-3 py-2">Designation</th>
                <th className="px-3 py-2">Email</th>
                <th className="px-3 py-2">Phone</th>
                <th className="px-3 py-2">Salary</th>
                <th className="px-3 py-2">Hire Date</th>
                <th className="px-3 py-2">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-darkbg-border">
              {employees.map((emp) => (
                <tr
                  key={emp.EmployeeID}
                  className="hover:bg-slate-50/80 dark:hover:bg-darkbg-secondary/60 transition-colors"
                >
                  <td className="px-3 py-2.5 font-mono font-medium text-slate-500 dark:text-darktext-muted">{emp.EmployeeID}</td>
                  <td className="px-3 py-2.5 font-semibold text-slate-900 dark:text-darktext-base whitespace-nowrap">
                    {emp.FullName || `${emp.FirstName || ''} ${emp.LastName || ''}`.trim()}
                  </td>
                  <td className="px-3 py-2.5 text-slate-600 dark:text-darktext-muted whitespace-nowrap">
                    {emp.DepartmentName || emp.DepartmentID || '—'}
                  </td>
                  <td className="px-3 py-2.5 text-slate-600 dark:text-darktext-muted whitespace-nowrap">
                    {emp.DesignationName || emp.DesignationTitle || emp.DesignationID || '—'}
                  </td>
                  <td className="px-3 py-2.5 font-mono text-[11px] text-slate-500 dark:text-darktext-muted truncate max-w-44">
                    {emp.Email}
                  </td>
                  <td className="px-3 py-2.5 text-slate-600 dark:text-darktext-muted whitespace-nowrap">
                    {emp.Phone || '—'}
                  </td>
                  <td className="px-3 py-2.5 font-medium text-slate-800 dark:text-darktext-base whitespace-nowrap">
                    {emp.Salary !== undefined ? `$${emp.Salary.toLocaleString()}` : '—'}
                  </td>
                  <td className="px-3 py-2.5 text-slate-600 dark:text-darktext-muted whitespace-nowrap">
                    {emp.HireDate ? new Date(emp.HireDate).toLocaleDateString() : '—'}
                  </td>
                  <td className="px-3 py-2.5 whitespace-nowrap">
                    <span className={`px-2 py-0.5 rounded text-[10px] font-semibold ${
                      emp.Status === 'Active'
                        ? 'bg-emerald-100 dark:bg-emerald-950/80 text-emerald-800 dark:text-emerald-300'
                        : emp.Status === 'On Leave'
                        ? 'bg-amber-100 dark:bg-amber-950/80 text-amber-800 dark:text-amber-300'
                        : 'bg-rose-100 dark:bg-rose-950/80 text-rose-800 dark:text-rose-300'
                    }`}>
                      {emp.Status || 'Active'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  // 6. Tool: add_employee or update_employee execution confirmation outcome
  if (toolName === 'add_employee' || toolName === 'update_employee') {
    const isUpdate = toolName === 'update_employee';
    return (
      <div className="my-2.5 p-3.5 rounded-xl bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-900/60 text-xs">
        <div className="font-semibold text-emerald-900 dark:text-emerald-300 flex items-center gap-2">
          <UserCheck className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
          <span>{isUpdate ? 'Employee Updated Successfully' : 'Employee Registered Successfully'}</span>
        </div>
        <p className="mt-1 text-emerald-800 dark:text-emerald-400/90">{message || 'Record successfully committed to SQL Server.'}</p>
        {typeof data === 'object' && data !== null && (
          <div className="mt-2 p-2 bg-white/70 dark:bg-darkbg-surface/70 rounded border border-emerald-200/50 dark:border-emerald-900/40 font-mono text-[11px] text-slate-700 dark:text-darktext-base">
            {Object.entries(data).map(([k, v]) => (
              <div key={k} className="flex justify-between">
                <span className="text-slate-500 dark:text-darktext-muted">{k}:</span>
                <span className="font-semibold">{String(v)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }

  // 7. Generic Fallback
  return (
    <div className="my-2 p-3 rounded-lg bg-slate-50 dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border text-xs text-slate-700 dark:text-darktext-base">
      <div className="font-medium">{message || JSON.stringify(data)}</div>
    </div>
  );
};

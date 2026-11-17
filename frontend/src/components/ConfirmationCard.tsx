import React from 'react';
import { Edit3, UserPlus, Check, X, AlertCircle } from 'lucide-react';
import { PendingOperation } from '../types/api';

interface ConfirmationCardProps {
  operation: PendingOperation;
  isConfirming: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

const FIELD_LABELS: Record<string, string> = {
  employee_id: 'Target Employee ID',
  first_name: 'First Name',
  last_name: 'Last Name',
  email: 'Corporate Email',
  phone: 'Phone Number',
  department_id: 'Department ID',
  designation_id: 'Designation ID',
  salary: 'Annual Salary',
  hire_date: 'Hire Date',
  status: 'Employment Status',
};

export const ConfirmationCard: React.FC<ConfirmationCardProps> = ({
  operation,
  isConfirming,
  onConfirm,
  onCancel,
}) => {
  const isUpdate = operation.tool_name === 'update_employee';
  const args = operation.arguments || {};

  const targetEmployeeId = isUpdate ? args.employee_id : undefined;

  // Filter out employee_id from proposed changes table in update mode
  const updateEntries = Object.entries(args).filter(
    ([key]) => (isUpdate ? key !== 'employee_id' : true)
  );

  const formatValue = (key: string, val: any): string => {
    if (val === null || val === undefined) return 'None';
    if (key === 'salary') return `$${Number(val).toLocaleString()}`;
    if (key === 'phone') return String(val); // Preserve leading zeros!
    return String(val);
  };

  return (
    <div className="my-3 overflow-hidden rounded-xl bg-white dark:bg-[#1E202B] border border-slate-200 dark:border-slate-800 shadow-sm transition-all text-xs">
      {/* Header */}
      <div className="px-4 py-3 bg-slate-50 dark:bg-[#181A22] border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className="p-1.5 rounded-lg bg-navy-100 dark:bg-slate-800 text-navy-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700">
            {isUpdate ? <Edit3 className="w-4 h-4 text-blue-600 dark:text-blue-400" /> : <UserPlus className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />}
          </div>
          <div>
            <h4 className="font-semibold text-slate-900 dark:text-slate-100 text-xs">
              {isUpdate ? 'Update Employee — Confirmation Required' : 'Register Employee — Confirmation Required'}
            </h4>
            <p className="text-[11px] text-slate-500 dark:text-slate-400">
              Review the proposed changes before confirming.
            </p>
          </div>
        </div>
        <div className="hidden sm:flex items-center gap-1.5 font-mono text-[10px] bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 px-2 py-0.5 rounded">
          <span>TOKEN:</span>
          <span className="font-bold text-slate-900 dark:text-slate-100">{operation.confirmation_token}</span>
        </div>
      </div>

      {/* Target Employee Section (if Update) */}
      {isUpdate && targetEmployeeId && (
        <div className="px-4 py-2.5 bg-blue-50/40 dark:bg-blue-950/20 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-slate-600 dark:text-slate-400">Target Employee:</span>
            <span className="font-mono font-bold text-slate-900 dark:text-slate-100 bg-white dark:bg-slate-800 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-700">
              Employee ID: {targetEmployeeId}
            </span>
          </div>
          <span className="text-[10px] text-slate-500 dark:text-slate-400">Database Record</span>
        </div>
      )}

      {/* Proposed Changes Section */}
      <div className="p-4 space-y-3">
        <div>
          <div className="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1.5">
            {isUpdate ? 'Proposed Field Changes' : 'Employee Registration Attributes'}
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
            <table className="w-full text-left text-xs">
              <thead className="bg-slate-50 dark:bg-slate-800/60 text-slate-600 dark:text-slate-400 text-[10px] uppercase border-b border-slate-200 dark:border-slate-800">
                <tr>
                  <th className="py-1.5 px-3 font-semibold">Field</th>
                  <th className="py-1.5 px-3 font-semibold">Proposed Value</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800 font-sans">
                {updateEntries.map(([key, val]) => (
                  <tr key={key} className="hover:bg-slate-50/50 dark:hover:bg-slate-800/30">
                    <td className="py-2 px-3 font-medium text-slate-700 dark:text-slate-300">
                      {FIELD_LABELS[key] || key}
                    </td>
                    <td className="py-2 px-3 font-mono font-semibold text-slate-900 dark:text-slate-100">
                      {formatValue(key, val)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Approval Warning Notice */}
        <div className="p-2.5 rounded-lg bg-amber-50/60 dark:bg-amber-950/20 border border-amber-200/80 dark:border-amber-900/40 flex items-start gap-2 text-slate-700 dark:text-slate-300 text-[11px]">
          <AlertCircle className="w-4 h-4 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
          <div>
            <span className="font-semibold text-slate-900 dark:text-slate-100">Action Pending Approval: </span>
            This update will modify the employee record in the SQL Server database.
          </div>
        </div>
      </div>

      {/* Confirmation Actions Footer */}
      <div className="px-4 py-2.5 bg-slate-50 dark:bg-[#181A22] border-t border-slate-200 dark:border-slate-800 flex items-center justify-between gap-3">
        <div className="text-[11px] text-slate-500 dark:text-slate-400 font-mono">
          Ref: <span className="font-semibold text-slate-700 dark:text-slate-300">{operation.confirmation_token}</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onCancel}
            disabled={isConfirming}
            className="px-3 py-1.5 rounded-lg border border-slate-300 dark:border-slate-700 text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors flex items-center gap-1.5 disabled:opacity-50"
          >
            <X className="w-3.5 h-3.5" />
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isConfirming}
            className="px-3.5 py-1.5 rounded-lg bg-[#0F172A] hover:bg-[#1E293B] dark:bg-blue-600 dark:hover:bg-blue-700 text-white text-xs font-semibold shadow-xs transition-colors flex items-center gap-1.5 disabled:opacity-50"
          >
            {isConfirming ? (
              <>
                <span className="inline-block w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Executing...
              </>
            ) : (
              <>
                <Check className="w-3.5 h-3.5" />
                {isUpdate ? 'Confirm Update' : 'Confirm Registration'}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

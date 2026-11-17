import React from 'react';
import { AlertTriangle, Check, X, ShieldAlert } from 'lucide-react';
import { PendingOperation } from '../types/api';

interface ConfirmationModalProps {
  operation: PendingOperation;
  isOpen: boolean;
  isConfirming: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmationModal: React.FC<ConfirmationModalProps> = ({
  operation,
  isOpen,
  isConfirming,
  onConfirm,
  onCancel,
}) => {
  if (!isOpen) return null;

  const isAdd = operation.tool_name === 'add_employee';

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 dark:bg-black/75 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white dark:bg-darkbg-surface rounded-2xl shadow-2xl border border-slate-200 dark:border-darkbg-border max-w-lg w-full overflow-hidden text-slate-800 dark:text-darktext-base">
        {/* Header */}
        <div className="bg-amber-500/10 dark:bg-amber-950/30 border-b border-amber-200/80 dark:border-amber-900/50 px-5 py-4 flex items-center gap-3">
          <div className="p-2 bg-amber-100 dark:bg-amber-900/40 rounded-xl text-amber-700 dark:text-amber-400">
            <ShieldAlert className="w-5 h-5" />
          </div>
          <div>
            <h3 className="font-semibold text-slate-900 dark:text-darktext-base text-sm">
              Confirm {isAdd ? 'Employee Registration' : 'Employee Record Update'}
            </h3>
            <p className="text-xs text-slate-500 dark:text-darktext-muted">
              Security token verification required prior to executing parameterized write on SQL Server.
            </p>
          </div>
        </div>

        {/* Content Body */}
        <div className="p-5 space-y-4 text-xs">
          <div>
            <span className="font-semibold text-slate-500 dark:text-darktext-muted uppercase tracking-wider text-[10px]">
              Proposed Action Summary
            </span>
            <p className="mt-1 font-medium text-slate-800 dark:text-darktext-base bg-slate-50 dark:bg-darkbg-secondary p-3 rounded-lg border border-slate-200 dark:border-darkbg-border leading-relaxed">
              {operation.summary}
            </p>
          </div>

          <div>
            <span className="font-semibold text-slate-500 dark:text-darktext-muted uppercase tracking-wider text-[10px]">
              Execution Arguments
            </span>
            <div className="mt-1 bg-slate-50 dark:bg-darkbg-secondary rounded-lg border border-slate-200 dark:border-darkbg-border p-3 font-mono text-[11px] space-y-1.5">
              {Object.entries(operation.arguments).map(([key, val]) => (
                <div key={key} className="flex justify-between border-b border-slate-200/50 dark:border-darkbg-border/60 pb-1 last:border-0 last:pb-0">
                  <span className="text-slate-500 dark:text-darktext-muted">{key}:</span>
                  <span className="font-semibold text-slate-900 dark:text-darktext-base">{String(val)}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="flex items-center justify-between p-3 bg-primary/5 dark:bg-primary/20 border border-primary/20 dark:border-primary/40 rounded-lg">
            <span className="text-slate-600 dark:text-darktext-muted font-medium">Security Confirmation Token:</span>
            <span className="font-mono font-bold text-primary dark:text-orange-400 px-2 py-0.5 bg-white dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border rounded text-[11px]">
              {operation.confirmation_token}
            </span>
          </div>

          <div className="flex items-center gap-2 text-amber-700 dark:text-amber-400 text-[11px]">
            <AlertTriangle className="w-4 h-4 shrink-0" />
            <span>Clicking Confirm will commit changes directly through the .NET MCP Server.</span>
          </div>
        </div>

        {/* Actions Footer */}
        <div className="bg-slate-50 dark:bg-darkbg-secondary px-5 py-3 border-t border-slate-200 dark:border-darkbg-border flex items-center justify-end gap-2.5">
          <button
            onClick={onCancel}
            disabled={isConfirming}
            className="px-3.5 py-1.5 rounded-lg border border-slate-300 dark:border-darkbg-border text-xs font-medium text-slate-700 dark:text-darktext-muted hover:bg-slate-100 dark:hover:bg-darkbg-surface transition-colors flex items-center gap-1.5 disabled:opacity-50"
          >
            <X className="w-3.5 h-3.5" />
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isConfirming}
            className="px-4 py-1.5 rounded-lg bg-primary hover:bg-primary-light text-white text-xs font-semibold shadow-xs transition-colors flex items-center gap-1.5 disabled:opacity-50"
          >
            {isConfirming ? (
              <>
                <span className="inline-block w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Executing...
              </>
            ) : (
              <>
                <Check className="w-3.5 h-3.5" />
                Confirm & Execute
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

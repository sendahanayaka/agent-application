import React, { useState } from 'react';
import { ChevronDown, ChevronUp, Terminal, CheckCircle2, XCircle, AlertCircle } from 'lucide-react';
import { PipelineResponseModel } from '../types/api';

interface ExecutionDetailsProps {
  response: PipelineResponseModel;
}

export const ExecutionDetails: React.FC<ExecutionDetailsProps> = ({ response }) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="mt-2.5 border border-slate-200/90 dark:border-darkbg-border rounded-lg overflow-hidden bg-white/60 dark:bg-darkbg-secondary/40 text-xs text-slate-700 dark:text-darktext-muted">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-3 py-2 flex items-center justify-between bg-slate-50 dark:bg-darkbg-secondary hover:bg-slate-100 dark:hover:bg-darkbg-surface/80 transition-colors text-left font-medium text-slate-700 dark:text-darktext-base"
      >
        <span className="flex items-center gap-2">
          <Terminal className="w-3.5 h-3.5 text-primary dark:text-orange-400" />
          <span className="font-semibold text-[11px] tracking-wide">Execution Details</span>
          <span className="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-slate-200/70 dark:bg-darkbg-surface text-slate-600 dark:text-darktext-muted">
            {response.selected_tools.length} {response.selected_tools.length === 1 ? 'tool' : 'tools'}
          </span>
        </span>
        <span className="flex items-center gap-1 text-slate-400 dark:text-darktext-muted text-[11px]">
          {isOpen ? 'Hide technical logs' : 'Show technical logs'}
          {isOpen ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
        </span>
      </button>

      {isOpen && (
        <div className="p-3 space-y-3 divide-y divide-slate-100 dark:divide-darkbg-border text-xs">
          {/* Metadata Section */}
          <div className="grid grid-cols-2 gap-2 pb-2 text-[11px]">
            <div>
              <span className="text-slate-400 dark:text-darktext-muted font-medium">Pipeline Status:</span>
              <span className={`ml-1.5 font-semibold ${
                response.status === 'success' ? 'text-emerald-700 dark:text-emerald-400' :
                response.status === 'confirmation_required' ? 'text-amber-700 dark:text-amber-400' :
                response.status === 'clarification_needed' ? 'text-blue-700 dark:text-blue-400' : 'text-rose-700 dark:text-rose-400'
              }`}>
                {response.status}
              </span>
            </div>
            <div>
              <span className="text-slate-400 dark:text-darktext-muted font-medium">Detected Intent(s):</span>
              <span className="ml-1.5 font-mono text-slate-800 dark:text-darktext-base">
                {response.detected_intents.length > 0 ? response.detected_intents.join(', ') : 'None'}
              </span>
            </div>
            <div className="col-span-2">
              <span className="text-slate-400 dark:text-darktext-muted font-medium">Selected Tool(s):</span>
              <span className="ml-1.5 font-mono text-primary dark:text-orange-400 font-semibold">
                {response.selected_tools.length > 0 ? response.selected_tools.join(', ') : 'None'}
              </span>
            </div>
            {response.missing_parameters && response.missing_parameters.length > 0 && (
              <div className="col-span-2 text-rose-600 dark:text-rose-400 flex items-center gap-1">
                <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                <span>Missing required arguments: {response.missing_parameters.join(', ')}</span>
              </div>
            )}
          </div>

          {/* Sequential Tool Executions */}
          {response.tool_executions.map((exec, idx) => (
            <div key={idx} className="pt-2.5 space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-1.5 font-mono font-semibold text-slate-800 dark:text-darktext-base text-[11px]">
                  <span className="px-1.5 py-0.2 rounded bg-primary/10 dark:bg-orange-500/20 text-primary dark:text-orange-400 text-[10px]">
                    Step {idx + 1}
                  </span>
                  <span>{exec.tool_name}</span>
                </div>
                <div>
                  {exec.success ? (
                    <span className="inline-flex items-center gap-1 text-emerald-700 dark:text-emerald-400 font-medium bg-emerald-50 dark:bg-emerald-950/40 px-1.5 py-0.5 rounded border border-emerald-200 dark:border-emerald-900/50 text-[10px]">
                      <CheckCircle2 className="w-3 h-3 text-emerald-600 dark:text-emerald-400" />
                      Success
                    </span>
                  ) : (
                    <span className="inline-flex items-center gap-1 text-rose-700 dark:text-rose-400 font-medium bg-rose-50 dark:bg-rose-950/40 px-1.5 py-0.5 rounded border border-rose-200 dark:border-rose-900/50 text-[10px]">
                      <XCircle className="w-3 h-3 text-rose-600 dark:text-rose-400" />
                      Failed
                    </span>
                  )}
                </div>
              </div>

              {/* Tool Arguments */}
              <div className="bg-slate-50 dark:bg-darkbg-surface p-2 rounded border border-slate-200 dark:border-darkbg-border">
                <div className="text-[10px] text-slate-400 dark:text-darktext-muted font-semibold mb-1 uppercase tracking-wider">
                  Arguments Passed to MCP Server
                </div>
                <pre className="font-mono text-[11px] text-slate-700 dark:text-darktext-base overflow-x-auto whitespace-pre-wrap">
                  {Object.keys(exec.arguments).length > 0
                    ? JSON.stringify(exec.arguments, null, 2)
                    : '(No arguments)'}
                </pre>
              </div>

              {/* Error Output if any */}
              {exec.error && (
                <div className="bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900/50 p-2 rounded text-rose-800 dark:text-rose-300">
                  <div className="font-semibold mb-0.5">Execution Error:</div>
                  <div className="font-mono text-[11px]">{exec.error}</div>
                </div>
              )}

              {/* Raw Result Output for technical inspection */}
              {exec.result !== undefined && exec.result !== null && (
                <div className="bg-slate-50 dark:bg-darkbg-surface p-2 rounded border border-slate-200 dark:border-darkbg-border">
                  <div className="text-[10px] text-slate-400 dark:text-darktext-muted font-semibold mb-1 uppercase tracking-wider">
                    Raw MCP Output Payload
                  </div>
                  <pre className="font-mono text-[11px] text-slate-700 dark:text-darktext-base overflow-x-auto whitespace-pre-wrap max-h-40">
                    {typeof exec.result === 'string'
                      ? exec.result
                      : JSON.stringify(exec.result, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          ))}

          {/* Pending Operation Token Details */}
          {response.pending_operation && (
            <div className="pt-2 bg-amber-50/60 dark:bg-amber-950/30 p-2.5 rounded-lg border border-amber-200 dark:border-amber-900/50 text-amber-900 dark:text-amber-300">
              <div className="font-semibold flex items-center gap-1.5 mb-1 text-[11px]">
                <AlertCircle className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
                <span>Pending Confirmation: {response.pending_operation.tool_name}</span>
              </div>
              <div className="text-[11px] mb-1 font-mono">
                Token: <span className="font-bold underline">{response.pending_operation.confirmation_token}</span>
              </div>
              <div className="text-[11px] opacity-90">{response.pending_operation.summary}</div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

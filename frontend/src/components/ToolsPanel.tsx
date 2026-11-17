import React, { useState } from 'react';
import { Wrench, ChevronDown, ChevronUp, Tag } from 'lucide-react';
import { ToolInfo } from '../types/api';

interface ToolsPanelProps {
  tools: ToolInfo[];
  toolsCount: number;
  isLoading: boolean;
  onSelectPrompt?: (prompt: string) => void;
}

export const ToolsPanel: React.FC<ToolsPanelProps> = ({
  tools,
  toolsCount,
  isLoading,
  onSelectPrompt,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [expandedTool, setExpandedTool] = useState<string | null>(null);

  const samplePrompts: Record<string, string> = {
    get_all_employees: "List all employees",
    get_employee_by_id: "Find employee with ID 101",
    search_employees: "Search employees in IT department",
    get_department_list: "Show all departments",
    get_employee_count: "How many employees are in Engineering?",
    add_employee: "Add employee John Doe in IT with salary 85000",
    update_employee: "Update employee 101 status to Active",
  };

  return (
    <div className="bg-white dark:bg-darkbg-surface border-b border-slate-200 dark:border-darkbg-border transition-colors">
      <div className="max-w-5xl mx-auto px-4 py-2 flex items-center justify-between">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-2 text-xs font-medium text-slate-700 dark:text-darktext-base hover:text-primary dark:hover:text-orange-400 transition-colors"
        >
          <div className="p-1 rounded-md bg-primary/10 dark:bg-orange-500/20 text-primary dark:text-orange-400">
            <Wrench className="w-3.5 h-3.5" />
          </div>
          <span className="font-semibold text-xs">MCP Tool Registry</span>
          <span className="px-1.5 py-0.2 rounded-full text-[10px] font-bold bg-slate-100 dark:bg-darkbg-secondary border border-slate-200 dark:border-darkbg-border text-slate-600 dark:text-darktext-muted">
            {isLoading ? '...' : toolsCount}
          </span>
          <span className="text-[11px] text-slate-400 dark:text-darktext-muted hidden sm:inline">
            {isOpen ? '(Click to collapse)' : '(Click to inspect tools & schemas)'}
          </span>
        </button>

        <button
          onClick={() => setIsOpen(!isOpen)}
          className="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-darktext-base"
        >
          {isOpen ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        </button>
      </div>

      {isOpen && (
        <div className="bg-slate-50 dark:bg-darkbg-secondary border-t border-slate-200 dark:border-darkbg-border px-4 py-3 max-w-5xl mx-auto animate-in slide-in-from-top-1">
          <div className="text-xs text-slate-500 dark:text-darktext-muted mb-2.5 flex items-center justify-between">
            <span>
              All 7 tools dynamically registered via MCP <code className="text-slate-700 dark:text-darktext-base font-mono">tools/list</code> protocol:
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2.5">
            {tools.map((tool) => {
              const isSelected = expandedTool === tool.name;
              const isReadOnly = tool.annotations?.readOnlyHint ?? true;

              return (
                <div
                  key={tool.name}
                  className="bg-white dark:bg-darkbg-surface rounded-xl border border-slate-200 dark:border-darkbg-border p-3 shadow-2xs hover:border-primary/40 dark:hover:border-orange-500/50 transition-all text-xs"
                >
                  <div className="flex items-start justify-between gap-1">
                    <span className="font-mono font-semibold text-primary dark:text-orange-400 text-[11px]">
                      {tool.name}
                    </span>
                    <span className={`px-1.5 py-0.2 rounded text-[9px] font-semibold uppercase tracking-wider ${
                      isReadOnly
                        ? 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-900/50'
                        : 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-900/50'
                    }`}>
                      {isReadOnly ? 'Read' : 'Mutating'}
                    </span>
                  </div>

                  <p className="mt-1 text-slate-600 dark:text-darktext-muted text-[11px] line-clamp-2">
                    {tool.description}
                  </p>

                  <div className="mt-2 pt-2 border-t border-slate-100 dark:border-darkbg-border flex items-center justify-between text-[11px]">
                    <button
                      onClick={() => setExpandedTool(isSelected ? null : tool.name)}
                      className="text-slate-500 dark:text-darktext-muted hover:text-primary dark:hover:text-orange-400 font-medium"
                    >
                      {isSelected ? 'Hide Schema' : 'View Schema'}
                    </button>

                    {onSelectPrompt && samplePrompts[tool.name] && (
                      <button
                        onClick={() => onSelectPrompt(samplePrompts[tool.name])}
                        className="text-primary dark:text-orange-400 hover:text-primary-light font-medium flex items-center gap-1"
                        title="Fill sample prompt into chat input"
                      >
                        <Tag className="w-3 h-3" />
                        Try query
                      </button>
                    )}
                  </div>

                  {isSelected && (
                    <div className="mt-2 pt-2 border-t border-slate-100 dark:border-darkbg-border space-y-1.5">
                      <div className="text-[10px] font-bold text-slate-400 dark:text-darktext-muted uppercase">Input Properties:</div>
                      <div className="bg-slate-50 dark:bg-darkbg-secondary p-2 rounded-lg font-mono text-[10px] text-slate-700 dark:text-darktext-base max-h-36 overflow-y-auto">
                        {tool.inputSchema.properties && Object.keys(tool.inputSchema.properties).length > 0 ? (
                          Object.entries(tool.inputSchema.properties).map(([prop, val]) => (
                            <div key={prop} className="mb-1">
                              <span className="font-bold text-slate-900 dark:text-darktext-base">{prop}</span>
                              <span className="text-primary dark:text-orange-400 ml-1">
                                ({Array.isArray(val.type) ? val.type.join('|') : (val.type || 'any')})
                              </span>
                              {tool.inputSchema.required?.includes(prop) && (
                                <span className="text-rose-500 ml-1 font-bold">*</span>
                              )}
                              {val.description && (
                                <div className="text-slate-500 dark:text-darktext-muted text-[9px] pl-1.5">{val.description}</div>
                              )}
                            </div>
                          ))
                        ) : (
                          <div className="text-slate-400 dark:text-darktext-muted italic">No parameters required</div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};

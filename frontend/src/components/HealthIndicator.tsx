import React, { useState } from 'react';
import {
  Activity,
  Server,
  Cpu,
  Database,
  RefreshCw,
  Sun,
  Moon,
  Users,
  RotateCcw,
  SlidersHorizontal,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { HealthResponse } from '../types/api';

interface HealthIndicatorProps {
  health: HealthResponse | null;
  isLoading: boolean;
  onRefresh: () => void;
  isDark: boolean;
  onToggleTheme: () => void;
  onNewConversation?: () => void;
}

export const HealthIndicator: React.FC<HealthIndicatorProps> = ({
  health,
  isLoading,
  onRefresh,
  isDark,
  onToggleTheme,
  onNewConversation,
}) => {
  const isHealthy = health?.status === 'healthy';
  const [showDiagnostics, setShowDiagnostics] = useState(false);

  return (
    <header className="bg-white dark:bg-darkbg-surface border-b border-slate-200 dark:border-darkbg-border sticky top-0 z-20 shadow-xs transition-colors">
      <div className="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
        {/* Brand & Identity: Professional Enterprise Workforce System */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary-dark dark:from-orange-500 dark:to-orange-700 text-white flex items-center justify-center font-bold text-lg shadow-sm">
            <Users className="w-5 h-5 text-white" />
          </div>
          <div>
            <h1 className="font-bold text-slate-900 dark:text-darktext-base text-base leading-tight tracking-tight">
              Employee Management
            </h1>
            <p className="text-slate-500 dark:text-darktext-muted text-xs font-medium">
              Workforce Management System
            </p>
          </div>
        </div>

        {/* Controls & Actions */}
        <div className="flex items-center gap-2 sm:gap-2.5">
          {/* System Health Status Badge */}
          <div
            className={`px-2.5 py-1 rounded-full text-xs font-semibold flex items-center gap-1.5 border transition-colors ${
              isHealthy
                ? 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/60'
                : 'bg-rose-50 dark:bg-rose-950/40 text-rose-800 dark:text-rose-300 border-rose-200 dark:border-rose-800/60'
            }`}
          >
            <Activity className={`w-3.5 h-3.5 ${isHealthy ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}`} />
            <span className="text-[11px] font-medium hidden xs:inline">{isHealthy ? 'Online' : 'Degraded'}</span>
          </div>

          {/* Diagnostics Popover Toggle */}
          <button
            onClick={() => setShowDiagnostics((prev) => !prev)}
            title="Toggle System Diagnostics"
            className={`p-1.5 rounded-lg border text-xs flex items-center gap-1 transition-colors ${
              showDiagnostics
                ? 'bg-slate-100 dark:bg-darkbg-secondary border-slate-300 dark:border-darkbg-border text-slate-800 dark:text-darktext-base'
                : 'border-slate-200 dark:border-darkbg-border text-slate-500 dark:text-darktext-muted hover:text-slate-700 dark:hover:text-darktext-base hover:bg-slate-50 dark:hover:bg-darkbg-secondary'
            }`}
          >
            <SlidersHorizontal className="w-3.5 h-3.5" />
            <span className="hidden md:inline text-[11px] font-medium">Diagnostics</span>
            {showDiagnostics ? <ChevronUp className="w-3 h-3 hidden md:inline" /> : <ChevronDown className="w-3 h-3 hidden md:inline" />}
          </button>

          {/* Refresh Health */}
          <button
            onClick={onRefresh}
            disabled={isLoading}
            title="Refresh System Health & Tools"
            className="p-1.5 rounded-lg border border-slate-200 dark:border-darkbg-border text-slate-500 dark:text-darktext-muted hover:text-slate-700 dark:hover:text-darktext-base hover:bg-slate-50 dark:hover:bg-darkbg-secondary transition-colors disabled:opacity-40"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
          </button>

          {/* New Conversation / Reset Context */}
          {onNewConversation && (
            <button
              onClick={onNewConversation}
              title="Start New Conversation (Clear Memory)"
              className="px-2.5 py-1.5 rounded-lg border border-slate-200 dark:border-darkbg-border text-slate-600 dark:text-darktext-base hover:text-primary dark:hover:text-orange-400 hover:bg-slate-50 dark:hover:bg-darkbg-secondary transition-colors flex items-center gap-1.5 text-xs font-medium"
            >
              <RotateCcw className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
              <span className="hidden sm:inline text-[11px]">New Chat</span>
            </button>
          )}

          {/* Light / Dark Mode Toggle */}
          <button
            onClick={onToggleTheme}
            title={isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
            className="p-1.5 rounded-lg border border-slate-200 dark:border-darkbg-border text-slate-600 dark:text-orange-400 hover:bg-slate-50 dark:hover:bg-darkbg-secondary transition-colors"
          >
            {isDark ? <Sun className="w-4 h-4 text-orange-400" /> : <Moon className="w-4 h-4 text-slate-700" />}
          </button>
        </div>
      </div>

      {/* Collapsible System Diagnostics Drawer */}
      {showDiagnostics && (
        <div className="border-t border-slate-100 dark:border-darkbg-border bg-slate-50 dark:bg-darkbg-secondary/60 px-4 py-2.5 animate-in slide-in-from-top-1">
          <div className="max-w-5xl mx-auto flex flex-wrap items-center justify-between gap-3 text-xs">
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-[11px] font-semibold text-slate-500 dark:text-darktext-muted uppercase tracking-wider mr-1">
                Backend Services:
              </span>

              {/* FastAPI */}
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border">
                <Server className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
                <span className="text-slate-700 dark:text-darktext-base font-medium text-[11px]">FastAPI Gateway</span>
                <span className="w-2 h-2 rounded-full bg-emerald-500" title="FastAPI Online" />
              </div>

              {/* Rust MCP Client */}
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border">
                <Cpu className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
                <span className="text-slate-700 dark:text-darktext-base font-medium text-[11px]">Rust MCP Client</span>
                <span
                  className={`w-2 h-2 rounded-full ${
                    health?.rust_process === 'running' ? 'bg-emerald-500' : 'bg-rose-500'
                  }`}
                  title={`Rust Process: ${health?.rust_process || 'unknown'}`}
                />
              </div>

              {/* .NET 8 MCP Server & SQL Server */}
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white dark:bg-darkbg-surface border border-slate-200 dark:border-darkbg-border">
                <Database className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
                <span className="text-slate-700 dark:text-darktext-base font-medium text-[11px]">.NET & SQL Server</span>
                <span
                  className={`w-2 h-2 rounded-full ${
                    isHealthy ? 'bg-emerald-500' : 'bg-rose-500'
                  }`}
                  title={health?.mcp_server?.name ? `${health.mcp_server.name} connected` : 'Server Error'}
                />
              </div>
            </div>

            <div className="text-[11px] text-slate-400 dark:text-darktext-muted font-mono">
              Tools Discovered: <span className="font-semibold text-slate-700 dark:text-darktext-base">{health?.tools_count ?? 0}</span>
            </div>
          </div>
        </div>
      )}
    </header>
  );
};

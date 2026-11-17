import React from 'react';
import { Activity, Server, Cpu, Database, RefreshCw, Sun, Moon } from 'lucide-react';
import { HealthResponse } from '../types/api';

interface HealthIndicatorProps {
  health: HealthResponse | null;
  isLoading: boolean;
  onRefresh: () => void;
  isDark: boolean;
  onToggleTheme: () => void;
}

export const HealthIndicator: React.FC<HealthIndicatorProps> = ({
  health,
  isLoading,
  onRefresh,
  isDark,
  onToggleTheme,
}) => {
  const isHealthy = health?.status === 'healthy';

  return (
    <header className="bg-white dark:bg-darkbg-surface border-b border-slate-200 dark:border-darkbg-border sticky top-0 z-20 shadow-xs transition-colors">
      <div className="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
        {/* Brand & Identity */}
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-primary dark:bg-orange-600 text-white flex items-center justify-center font-bold text-base shadow-xs">
            EA
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="font-bold text-slate-900 dark:text-darktext-base text-sm md:text-base leading-tight tracking-tight">
                Employee Management Agent
              </h1>
              <span className="hidden sm:inline-block px-2 py-0.5 rounded-full text-[10px] font-bold tracking-wider uppercase bg-primary/10 dark:bg-orange-500/20 text-primary dark:text-orange-400 border border-primary/20 dark:border-orange-500/30">
                MCP Assistant
              </span>
            </div>
            <p className="text-slate-500 dark:text-darktext-muted text-xs hidden sm:block">
              Model Context Protocol • Rust Regex Pipeline • .NET 8 SQL Server
            </p>
          </div>
        </div>

        {/* Stack Indicators, Theme Switcher & Actions */}
        <div className="flex items-center gap-2 sm:gap-3">
          {/* Stack Pills (Desktop) */}
          <div className="hidden md:flex items-center gap-2 text-xs">
            {/* FastAPI */}
            <div className="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-slate-50 dark:bg-darkbg-secondary border border-slate-200 dark:border-darkbg-border">
              <Server className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
              <span className="text-slate-600 dark:text-darktext-base font-medium text-[11px]">FastAPI</span>
              <span className="w-2 h-2 rounded-full bg-emerald-500" title="FastAPI Online" />
            </div>

            {/* Rust IPC */}
            <div className="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-slate-50 dark:bg-darkbg-secondary border border-slate-200 dark:border-darkbg-border">
              <Cpu className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
              <span className="text-slate-600 dark:text-darktext-base font-medium text-[11px]">Rust</span>
              <span className={`w-2 h-2 rounded-full ${
                health?.rust_process === 'running' ? 'bg-emerald-500' : 'bg-rose-500'
              }`} title={`Rust Process: ${health?.rust_process || 'unknown'}`} />
            </div>

            {/* .NET & SQL */}
            <div className="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-slate-50 dark:bg-darkbg-secondary border border-slate-200 dark:border-darkbg-border">
              <Database className="w-3.5 h-3.5 text-slate-500 dark:text-darktext-muted" />
              <span className="text-slate-600 dark:text-darktext-base font-medium text-[11px]">.NET & SQL</span>
              <span className={`w-2 h-2 rounded-full ${
                isHealthy ? 'bg-emerald-500' : 'bg-rose-500'
              }`} title={health?.mcp_server?.name ? `${health.mcp_server.name} connected` : 'Server Error'} />
            </div>
          </div>

          {/* System Health Status Badge */}
          <div className={`px-2.5 py-1 rounded-full text-xs font-semibold flex items-center gap-1.5 border ${
            isHealthy
              ? 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/60'
              : 'bg-rose-50 dark:bg-rose-950/40 text-rose-800 dark:text-rose-300 border-rose-200 dark:border-rose-800/60'
          }`}>
            <Activity className={`w-3.5 h-3.5 ${isHealthy ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}`} />
            <span className="text-[11px] font-medium">{isHealthy ? 'Online' : 'Degraded'}</span>
          </div>

          {/* Refresh Health */}
          <button
            onClick={onRefresh}
            disabled={isLoading}
            title="Refresh System Health"
            className="p-1.5 rounded-lg border border-slate-200 dark:border-darkbg-border text-slate-500 dark:text-darktext-muted hover:text-slate-700 dark:hover:text-darktext-base hover:bg-slate-100 dark:hover:bg-darkbg-secondary transition-colors disabled:opacity-40"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
          </button>

          {/* Light / Dark Mode Toggle */}
          <button
            onClick={onToggleTheme}
            title={isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
            className="p-1.5 rounded-lg border border-slate-200 dark:border-darkbg-border text-slate-600 dark:text-orange-400 hover:bg-slate-100 dark:hover:bg-darkbg-secondary transition-colors"
          >
            {isDark ? <Sun className="w-4 h-4 text-orange-400" /> : <Moon className="w-4 h-4 text-slate-700" />}
          </button>
        </div>
      </div>
    </header>
  );
};

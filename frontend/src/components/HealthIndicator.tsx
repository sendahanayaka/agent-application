import { useState } from 'react';
import { ChevronRight, RefreshCw, Moon, Sun, PanelLeft } from 'lucide-react';
import { HealthResponse } from '../types/api';

interface Props {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  health: HealthResponse | null;
  isLoading: boolean;
  onRefresh: () => void;
  isDark: boolean;
  onToggleTheme: () => void;
  title: string;
}

export function HealthIndicator({sidebarOpen, onToggleSidebar, health, isLoading, onRefresh, isDark, onToggleTheme, title}: Props) {
  const [open, setOpen] = useState(false);
  const status = isLoading ? 'Checking' : health?.status === 'healthy' ? 'Connected' : health?.status === 'degraded' ? 'Degraded' : 'Offline';
  return <header className="desk-header">
    <button className="icon-action sidebar-toggle" onClick={onToggleSidebar} aria-controls="workspace-sidebar" aria-expanded={sidebarOpen} aria-label={sidebarOpen ? "Collapse sidebar" : "Expand sidebar"} title={sidebarOpen ? "Collapse sidebar" : "Expand sidebar"}><PanelLeft size={19} strokeWidth={1.6} /></button>
    <div className="crumb"><span>Workspace</span><ChevronRight size={14}/><strong>{title}</strong></div>
    <button className="connection" aria-expanded={open} aria-controls="connection-details" onClick={() => setOpen(!open)}>
      <i data-status={status} />{status}
    </button>
    <button className="icon-action" onClick={onToggleTheme} aria-label={isDark ? 'Use light theme' : 'Use dark theme'}>
      {isDark ? <Sun size={17}/> : <Moon size={17}/>}
    </button>
    {open && <section id="connection-details" className="diagnostics">
      <div><h2>Connection details</h2><button className="icon-action" disabled={isLoading} onClick={onRefresh} aria-label="Refresh connections"><RefreshCw size={15}/></button></div>
      <dl><dt>Gateway</dt><dd>{health?.status ?? 'Checking'}</dd><dt>Rust client</dt><dd>{health?.rust_process ?? 'Checking'}</dd><dt>MCP server</dt><dd>{health?.mcp_server?.name || health?.mcp_server?.error || 'Unavailable'}</dd><dt>Available tools</dt><dd>{health?.tools_count ?? '—'}</dd></dl>
    </section>}
  </header>;
}

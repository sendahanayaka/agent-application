import React, { useState, useEffect, useRef } from 'react';
import { flushSync } from 'react-dom';
import { ArrowUp, Bot, User, Layers3, MessageCircle, Blocks, Plus, ContactRound, Network } from 'lucide-react';
import { ChatMessage, HealthResponse, PipelineResponseModel, ToolInfo } from './types/api';
import { apiClient } from './services/api';
import { HealthIndicator } from './components/HealthIndicator';
import { ToolsPanel } from './components/ToolsPanel';
import { ExecutionDetails } from './components/ExecutionDetails';
import { ResultRenderer } from './components/ResultRenderer';
import { ConfirmationCard } from './components/ConfirmationCard';

/**
 * Transforms technical backend messages like:
 * "Successfully executed 1 tool(s). {"count": 0, "data": [], "message": "Found 0 matching employee(s)."}"
 * into clean, natural user-facing text.
 */
function cleanAssistantText(rawText: string, toolExecutions?: PipelineResponseModel['tool_executions']): string {
  if (!rawText) return '';

  let cleaned = rawText;

  // If text contains technical execution prefixes, format cleanly
  if (cleaned.includes('Successfully executed') || cleaned.includes('{"') || cleaned.includes('Found ')) {
    // Check if tools succeeded and extract human message
    if (toolExecutions && toolExecutions.length > 0) {
      const messages: string[] = [];
      for (const exec of toolExecutions) {
        let parsed = exec.result;
        if (typeof parsed === 'string') {
          try {
            parsed = JSON.parse(parsed);
          } catch {
            // keep as string
          }
        }

        if (parsed?.message) {
          messages.push(parsed.message);
        } else if (exec.tool_name === 'get_all_employees') {
          messages.push('Here are all employee records currently in the system:');
        } else if (exec.tool_name === 'search_employees') {
          messages.push('Here are the employee search results:');
        } else if (exec.tool_name === 'get_department_list') {
          messages.push('Here is the list of departments:');
        } else if (exec.tool_name === 'get_employee_by_id') {
          messages.push('Employee profile retrieved:');
        } else if (exec.tool_name === 'get_employee_count') {
          messages.push('Employee headcount:');
        }
      }

      if (messages.length > 0) {
        return messages.join('\n');
      }
    }

    // Fallback: strip embedded JSON string if any
    const jsonMatch = cleaned.match(/\{.*\}$/);
    if (jsonMatch) {
      try {
        const obj = JSON.parse(jsonMatch[0]);
        if (obj.message) {
          return obj.message;
        }
      } catch {
        // ignore
      }
    }
  }

  return cleaned;
}

const suggestions = [
  'Show all employees',
  'Find employee ID 1003',
  'Show all departments',
  'How many employees are there?',
];

export const App: React.FC = () => {
  // Theme state: defaults to light, saved to localStorage
  const [isDark, setIsDark] = useState<boolean>(() => {
    const saved = localStorage.getItem('theme_preference');
    return saved === 'dark';
  });

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme_preference', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme_preference', 'light');
    }
  }, [isDark]);

  const toggleTheme = () => {
    setIsDark((prev) => !prev);
  };

  const [conversationId, setConversationId] = useState<string>(
    () => `conv-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`
  );
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const choosePrompt = (prompt: string) => {
    flushSync(() => {
      setInputPrompt(prompt);
      setView('chat');
    });
    inputRef.current?.focus();
  };

  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [view, setView] = useState<'chat' | 'tools'>('chat');
  const [inputPrompt, setInputPrompt] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Health and Tools state
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [isHealthLoading, setIsHealthLoading] = useState(false);
  const [tools, setTools] = useState<ToolInfo[]>([]);
  const [isToolsLoading, setIsToolsLoading] = useState(false);

  // Active pending confirmation state
  const [pendingConfirmation, setPendingConfirmation] = useState<{
    token: string;
    operation: NonNullable<PipelineResponseModel['pending_operation']>;
  } | null>(null);
  const [isConfirming, setIsConfirming] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isSubmitting, isConfirming]);

  // Load health and tools on startup
  const fetchHealth = async () => {
    setIsHealthLoading(true);
    try {
      const data = await apiClient.getHealth();
      setHealth(data);
    } catch {
      setHealth({
        status: 'error',
        rust_process: 'stopped',
        mcp_server: { error: 'Failed to contact backend' },
        tools_count: 0,
      });
    } finally {
      setIsHealthLoading(false);
    }
  };

  const fetchTools = async () => {
    setIsToolsLoading(true);
    try {
      const data = await apiClient.getTools();
      setTools(data.tools || []);
    } catch (err) {
      console.error('Failed to load MCP tools:', err);
    } finally {
      setIsToolsLoading(false);
    }
  };

  useEffect(() => {
    fetchHealth();
    fetchTools();
  }, []);

  const handleSendMessage = async (e?: React.FormEvent, overridePrompt?: string) => {
    if (e) e.preventDefault();
    const promptToSend = (overridePrompt || inputPrompt).trim();
    if (!promptToSend || isSubmitting) return;
    inputRef.current?.focus({ preventScroll: true });

    const userMessage: ChatMessage = {
      id: `user-${Date.now()}`,
      sender: 'user',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      text: promptToSend,
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputPrompt('');
    setIsSubmitting(true);

    try {
      const response = await apiClient.sendChat(promptToSend, conversationId);

      const humanText = cleanAssistantText(response.message, response.tool_executions);

      const assistantMessage: ChatMessage = {
        id: `asst-${Date.now()}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        text: humanText,
        pipelineResponse: response,
      };

      setMessages((prev) => [...prev, assistantMessage]);

      // If response requires write confirmation, keep pending operation state
      if (response.status === 'confirmation_required' && response.pending_operation) {
        setPendingConfirmation({
          token: response.pending_operation.confirmation_token,
          operation: response.pending_operation,
        });
      }
    } catch (err: any) {
      const errorMessage: ChatMessage = {
        id: `asst-${Date.now()}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        text: `Error communicating with Agent: ${err.message || 'Unknown error'}`,
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDirectConfirm = async (token: string) => {
    setIsConfirming(true);
    try {
      const response = await apiClient.confirmOperation(token, conversationId);
      const humanText = cleanAssistantText(response.message, response.tool_executions);

      const confirmationResultMessage: ChatMessage = {
        id: `asst-${Date.now()}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        text: humanText,
        pipelineResponse: response,
      };

      setMessages((prev) => [...prev, confirmationResultMessage]);
      setPendingConfirmation(null);
    } catch (err: any) {
      const errorMessage: ChatMessage = {
        id: `asst-${Date.now()}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        text: `Confirmation failed: ${err.message || 'Execution error'}`,
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsConfirming(false);
    }
  };

  const handleNewConversation = async () => {
    try {
      await apiClient.clearConversation(conversationId);
    } catch (err) {
      console.warn('Error clearing session context on backend:', err);
    }
    const newId = `conv-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
    setConversationId(newId);
    setPendingConfirmation(null);
    setMessages([]);
    setInputPrompt('');
  };

  return (
    <div className={sidebarOpen ? "workspace" : "workspace sidebar-closed"}>
      <aside className="side" id="workspace-sidebar">
        <div className="wordmark" aria-label="People Desk"><span className="brand-mark"><Layers3 size={23} /></span><span className="side-label">People Desk</span></div>
        <button className="new-chat" aria-label="New chat" title={sidebarOpen ? undefined : 'New chat'} disabled={isSubmitting || isConfirming} onClick={() => { setView('chat'); void handleNewConversation(); }}><Plus size={17} /><span className="side-label">New chat</span></button>
        <nav aria-label="Workspace">
          <p className="rail-label">Workspace</p>
          <button aria-label="Assistant" title={sidebarOpen ? undefined : 'Assistant'} aria-current={view === 'chat' ? 'page' : undefined} onClick={() => setView('chat')}><MessageCircle /><span className="side-label">Assistant</span></button>
          <button aria-label="Tool library" title={sidebarOpen ? undefined : 'Tool library'} aria-current={view === 'tools' ? 'page' : undefined} onClick={() => setView('tools')}><Blocks /><span className="side-label">Tool library</span></button>
        </nav>
      </aside>
      <div className="stage">
      {/* 1. System Health Header with Theme Switcher */}
      <HealthIndicator
        sidebarOpen={sidebarOpen}
        onToggleSidebar={() => setSidebarOpen(open => !open)}
        health={health}
        isLoading={isHealthLoading}
        onRefresh={() => {
          fetchHealth();
          fetchTools();
        }}
        isDark={isDark}
        onToggleTheme={toggleTheme}
        title={view === 'chat' ? 'Assistant' : 'Tool library'}
      />

      {/* 2. MCP Tools Registry Bar */}
      {view === 'tools' ? <ToolsPanel
        tools={tools}
        isLoading={isToolsLoading}
        onSelectPrompt={choosePrompt}
      /> : (


      <main className="desk-main">
        <div className="conversation">

          {/* Messages Scroll View */}
          <div role="log" aria-label="Conversation" className="transcript space-y-5">
            {messages.length === 0 && (
              <div className="welcome-panel">
                <div className="welcome-art" aria-hidden="true"><span className="art-card card-back"><Network /></span><span className="art-card card-front"><ContactRound /><i /><i /></span></div>
                <h1>Ask about your team<span>.</span></h1>
                <p>Find employee profiles, browse departments, and manage team records in one conversation.</p>
              </div>
            )}
            {messages.map((msg) => {
              const isUser = msg.sender === 'user';

              return (
                <div
                  key={msg.id}
                  className={`message flex gap-3 ${isUser ? 'ml-auto flex-row-reverse' : 'mr-auto'}`}
                >
                  {/* Avatar */}
                  <div
                    className="chat-brand w-8 h-8 rounded-xl flex items-center justify-center shrink-0"
                  >
                    {isUser ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
                  </div>

                  {/* Message Bubble Container */}
                  <div className={`message-body min-w-0 space-y-1.5 max-w-[88%] ${isUser ? 'items-end' : 'items-start'}`}>
                    <div className="flex items-center gap-2 text-[11px] text-slate-400 dark:text-darktext-muted px-1">
                      <span className="font-semibold text-slate-600 dark:text-darktext-base">
                        {isUser ? 'You' : 'Workforce assistant'}
                      </span>
                      <span>•</span>
                      <span>{msg.timestamp}</span>
                    </div>

                    <div
                      className={`message-bubble p-4 rounded-2xl text-xs md:text-sm leading-relaxed shadow-xs transition-colors ${
                        isUser
                          ? 'chat-brand rounded-tr-xs'
                          : 'bg-slate-50 dark:bg-darkbg-secondary/70 text-slate-800 dark:text-darktext-base border border-slate-200 dark:border-darkbg-border rounded-tl-xs'
                      }`}
                    >
                      {/* Context Resolution Badge */}
                      {!isUser && msg.pipelineResponse?.resolved_reference && (
                        <div className="mb-2 inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-medium bg-emerald-50 dark:bg-emerald-950/50 text-emerald-800 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-900/50">
                          <span className="font-semibold">Context Resolved:</span>
                          <span className="font-mono">{msg.pipelineResponse.resolved_reference}</span>
                        </div>
                      )}

                      {/* Text Content */}
                      <div className="whitespace-pre-wrap font-normal">{msg.text}</div>

                      {/* Structured Employee Tables, Profiles, and Metrics */}
                      {!isUser && msg.pipelineResponse?.tool_executions && (
                        <div className="mt-2 space-y-2">
                          {msg.pipelineResponse.tool_executions.map((exec, idx) => (
                            <ResultRenderer
                              key={idx}
                              result={exec.result}
                              toolName={exec.tool_name}
                            />
                          ))}
                        </div>
                      )}

                      {/* Write Operation Confirmation Prompt Card in chat (if pending) */}
                      {!isUser && msg.pipelineResponse?.pending_operation && pendingConfirmation?.token === msg.pipelineResponse.pending_operation.confirmation_token && (
                        <ConfirmationCard
                          operation={msg.pipelineResponse.pending_operation}
                          isConfirming={isConfirming && pendingConfirmation?.token === msg.pipelineResponse.pending_operation.confirmation_token}
                          onConfirm={() => {
                            setPendingConfirmation({
                              token: msg.pipelineResponse!.pending_operation!.confirmation_token,
                              operation: msg.pipelineResponse!.pending_operation!,
                            });
                            // Trigger immediate confirmation execution
                            handleDirectConfirm(msg.pipelineResponse!.pending_operation!.confirmation_token);
                          }}
                          onCancel={() => {
                            setPendingConfirmation(null);
                            const cancelMessage: ChatMessage = {
                              id: `asst-${Date.now()}`,
                              sender: 'assistant',
                              timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                              text: `Operation cancelled. No changes were made to the database.`,
                            };
                            setMessages((prev) => [...prev, cancelMessage]);
                          }}
                        />
                      )}

                      {/* Secondary Expandable Execution Details */}
                      {!isUser && msg.pipelineResponse && (
                        <ExecutionDetails response={msg.pipelineResponse} />
                      )}
                    </div>
                  </div>
                </div>
              );
            })}

            {(isSubmitting || isConfirming) && (
              <div className="typing-indicator" role="status">
                <Bot size={18} aria-hidden="true" />
                <span className="sr-only">{isConfirming ? 'Saving changes' : 'Assistant is working'}</span>
                <span className="typing-dots" aria-hidden="true"><i /><i /><i /></span>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>

          {/* Prompt Input Form */}
          <div className="composer">
            <div className="suggestions" role="group" aria-label="Suggested messages">
              {suggestions.map((prompt) => (
                <button key={prompt} type="button" className="suggestion" disabled={isSubmitting || isConfirming}
                  onClick={() => choosePrompt(prompt)}>
                  {prompt}
                </button>
              ))}
            </div>
            <form onSubmit={handleSendMessage} className="compose-form">
              <input
                type="text"
                ref={inputRef}
                aria-label="Your message"
                value={inputPrompt}
                onChange={(e) => setInputPrompt(e.target.value)}
                placeholder="Ask about your workforce…"
              />
              <button
                type="submit"
                disabled={!inputPrompt.trim() || isSubmitting}
                aria-label="Send message"
                title="Send message"
                className="send-arrow"
              >
                <ArrowUp size={23} strokeWidth={1.8} aria-hidden="true" />
              </button>
            </form>
          </div>
        </div>
      </main>)}
      </div>
    </div>
  );
};

export default App;

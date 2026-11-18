import React, { useState, useEffect, useRef } from 'react';
import { Send, Bot, User, Sparkles } from 'lucide-react';
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
          messages.push('Employee headcount count:');
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
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: 'welcome',
      sender: 'assistant',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      text: "Hello! I am your Employee Management Agent.\n\nI can help you look up employee records, search personnel by department or job title, check employee headcount, and register or update staff records in SQL Server via our secure Model Context Protocol (MCP) server.\n\nTry asking:\n• \"List all employees\"\n• \"Search employees in IT department\"\n• \"Find employee with ID 101\"\n• \"How many employees are in Engineering?\"\n• \"Show all departments\"\n• \"Add employee Alice Johnson in HR with salary 75000\"",
    },
  ]);

  const [inputPrompt, setInputPrompt] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [processingStep, setProcessingStep] = useState<string | null>(null);

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
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, processingStep]);

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

    const userMessage: ChatMessage = {
      id: `user-${Date.now()}`,
      sender: 'user',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      text: promptToSend,
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputPrompt('');
    setIsSubmitting(true);
    setProcessingStep('Identifying intent & validating parameter schema in Rust...');

    try {
      setProcessingStep('Executing parameterized MCP tool on SQL Server...');
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
      setProcessingStep(null);
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
    setMessages([
      {
        id: `welcome-${Date.now()}`,
        sender: 'assistant',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        text: "Started a new conversation session.\n\nSession memory has been reset. How can I help you manage employee records today?",
      },
    ]);
  };

  return (
    <div className="min-h-screen bg-[#F5F6F8] dark:bg-[#12131A] text-[#252525] dark:text-[#F5F6FA] flex flex-col font-sans transition-colors duration-200">
      {/* 1. System Health Header with Theme Switcher */}
      <HealthIndicator
        health={health}
        isLoading={isHealthLoading}
        onRefresh={() => {
          fetchHealth();
          fetchTools();
        }}
        isDark={isDark}
        onToggleTheme={toggleTheme}
        onNewConversation={handleNewConversation}
      />

      {/* 2. MCP Tools Registry Bar */}
      <ToolsPanel
        tools={tools}
        toolsCount={health?.tools_count || tools.length}
        isLoading={isToolsLoading}
        onSelectPrompt={(sample) => {
          setInputPrompt(sample);
        }}
      />

      {/* 3. Main Conversational Chat Area */}
      <main className="flex-1 max-w-5xl w-full mx-auto p-4 flex flex-col">
        <div className="flex-1 bg-white dark:bg-darkbg-surface rounded-2xl shadow-xs border border-slate-200 dark:border-darkbg-border flex flex-col overflow-hidden transition-colors">
          {/* Messages Scroll View */}
          <div className="flex-1 p-4 md:p-6 overflow-y-auto space-y-5">
            {messages.map((msg) => {
              const isUser = msg.sender === 'user';

              return (
                <div
                  key={msg.id}
                  className={`flex gap-3 max-w-3xl ${isUser ? 'ml-auto flex-row-reverse' : 'mr-auto'}`}
                >
                  {/* Avatar */}
                  <div
                    className={`w-8 h-8 rounded-xl flex items-center justify-center shrink-0 text-xs font-bold shadow-2xs ${
                      isUser
                        ? 'bg-primary dark:bg-orange-600 text-white'
                        : 'bg-primary-dark dark:bg-darkbg-secondary text-white dark:text-orange-400 border border-slate-300 dark:border-darkbg-border'
                    }`}
                  >
                    {isUser ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
                  </div>

                  {/* Message Bubble Container */}
                  <div className={`space-y-1.5 max-w-[85%] ${isUser ? 'items-end' : 'items-start'}`}>
                    <div className="flex items-center gap-2 text-[11px] text-slate-400 dark:text-darktext-muted px-1">
                      <span className="font-semibold text-slate-600 dark:text-darktext-base">
                        {isUser ? 'You' : 'Agent Assistant'}
                      </span>
                      <span>•</span>
                      <span>{msg.timestamp}</span>
                    </div>

                    <div
                      className={`p-4 rounded-2xl text-xs md:text-sm leading-relaxed shadow-xs transition-colors ${
                        isUser
                          ? 'bg-primary dark:bg-orange-600 text-white rounded-tr-xs'
                          : 'bg-slate-50 dark:bg-darkbg-secondary/70 text-slate-800 dark:text-darktext-base border border-slate-200 dark:border-darkbg-border rounded-tl-xs'
                      }`}
                    >
                      {/* Context Resolution Badge */}
                      {!isUser && msg.pipelineResponse?.resolved_reference && (
                        <div className="mb-2 inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-medium bg-blue-50 dark:bg-blue-950/50 text-blue-800 dark:text-blue-300 border border-blue-200 dark:border-blue-900/50">
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
                      {!isUser && msg.pipelineResponse?.pending_operation && (
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

            {/* In-Flight Processing Indicator */}
            {isSubmitting && (
              <div className="flex gap-3 max-w-xl mr-auto">
                <div className="w-8 h-8 rounded-xl bg-primary-dark dark:bg-darkbg-secondary text-white dark:text-orange-400 flex items-center justify-center shrink-0">
                  <Bot className="w-4 h-4 animate-pulse" />
                </div>
                <div className="bg-slate-50 dark:bg-darkbg-secondary border border-slate-200 dark:border-darkbg-border rounded-2xl rounded-tl-xs p-3.5 text-xs text-slate-600 dark:text-darktext-muted shadow-xs space-y-2">
                  <div className="flex items-center gap-2 font-medium text-slate-800 dark:text-darktext-base">
                    <span className="w-3.5 h-3.5 border-2 border-primary/30 dark:border-orange-500/30 border-t-primary dark:border-t-orange-400 rounded-full animate-spin" />
                    <span>Processing Request...</span>
                  </div>
                  {processingStep && (
                    <div className="font-mono text-[11px] text-primary dark:text-orange-400 bg-primary/5 dark:bg-orange-500/10 px-2.5 py-1 rounded-md border border-primary/10 dark:border-orange-500/20 flex items-center gap-1.5">
                      <Sparkles className="w-3.5 h-3.5 animate-spin" />
                      <span>{processingStep}</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>

          {/* Prompt Input Form */}
          <div className="p-3 bg-slate-50 dark:bg-darkbg-secondary border-t border-slate-200 dark:border-darkbg-border transition-colors">
            <form onSubmit={handleSendMessage} className="flex gap-2">
              <input
                type="text"
                value={inputPrompt}
                onChange={(e) => setInputPrompt(e.target.value)}
                placeholder="Ask about employees, departments, headcount, or request an add/update..."
                disabled={isSubmitting}
                className="flex-1 px-4 py-2.5 bg-white dark:bg-darkbg-surface border border-slate-300 dark:border-darkbg-border rounded-xl text-xs md:text-sm text-slate-900 dark:text-darktext-base placeholder-slate-400 dark:placeholder-darktext-muted focus:outline-none focus:ring-2 focus:ring-primary dark:focus:ring-orange-500 focus:border-transparent transition-all disabled:opacity-50"
              />
              <button
                type="submit"
                disabled={!inputPrompt.trim() || isSubmitting}
                className="px-4 py-2.5 bg-primary hover:bg-primary-light dark:bg-orange-600 dark:hover:bg-accent-hover disabled:bg-slate-300 dark:disabled:bg-darkbg-border text-white font-semibold rounded-xl text-xs md:text-sm shadow-xs transition-colors flex items-center gap-1.5 disabled:cursor-not-allowed"
              >
                <span>Send</span>
                <Send className="w-3.5 h-3.5" />
              </button>
            </form>
            <div className="mt-1.5 px-1 flex items-center justify-between text-[11px] text-slate-400 dark:text-darktext-muted">
              <span>Deterministic Regex Routing • Zero LLM latency • Live SQL Server Access</span>
              <span className="hidden sm:inline">Press Enter to send</span>
            </div>
          </div>
        </div>
      </main>

      {/* 4. Write Confirmation Dialog (Modal - Optional alternative view) */}
      {/* Both the in-chat professional card and confirmation actions directly execute the secure workflow */}
    </div>
  );
};

export default App;

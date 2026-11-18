export interface SingleToolExecution {
  tool_name: string;
  arguments: Record<string, any>;
  success: boolean;
  result?: any;
  error?: string | null;
}

export interface PendingOperation {
  confirmation_token: string;
  tool_name: string;
  arguments: Record<string, any>;
  summary: string;
  created_at_unix: number;
}

export interface PipelineResponseModel {
  status: 'success' | 'confirmation_required' | 'partial_success' | 'clarification_needed' | 'unsupported_prompt' | 'missing_parameters' | 'validation_error' | 'no_context' | 'error';
  prompt: string;
  detected_intents: string[];
  selected_tools: string[];
  message: string;
  pending_operation?: PendingOperation | null;
  tool_executions: SingleToolExecution[];
  missing_parameters?: string[] | null;
  ambiguous_options?: string[] | null;
  resolved_reference?: string | null;
  conversation_context?: any | null;
}

export interface HealthResponse {
  status: 'healthy' | 'degraded' | 'error';
  rust_process: string;
  mcp_server?: {
    name?: string;
    version?: string;
    error?: string;
  } | null;
  tools_count: number;
}

export interface ToolPropertySchema {
  type?: string | string[];
  description?: string;
  default?: any;
}

export interface ToolInputSchema {
  type: string;
  properties: Record<string, ToolPropertySchema>;
  required?: string[];
}

export interface ToolInfo {
  name: string;
  title?: string;
  description?: string;
  inputSchema: ToolInputSchema;
  annotations?: {
    readOnlyHint?: boolean;
    destructiveHint?: boolean;
    title?: string;
  };
}

export interface ToolsResponse {
  count: number;
  tools: ToolInfo[];
}

export interface ChatMessage {
  id: string;
  sender: 'user' | 'assistant' | 'system';
  timestamp: string;
  text: string;
  pipelineResponse?: PipelineResponseModel;
  isProcessing?: boolean;
}

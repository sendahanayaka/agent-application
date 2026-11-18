import { HealthResponse, PipelineResponseModel, ToolsResponse } from '../types/api';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8000/api';

export const apiClient = {
  async getHealth(): Promise<HealthResponse> {
    const res = await fetch(`${API_BASE_URL}/health`);
    if (!res.ok) {
      throw new Error(`Health check failed with status: ${res.status}`);
    }
    return res.json();
  },

  async getTools(): Promise<ToolsResponse> {
    const res = await fetch(`${API_BASE_URL}/tools`);
    if (!res.ok) {
      throw new Error(`Failed to load MCP tools: ${res.status}`);
    }
    return res.json();
  },

  async sendChat(prompt: string, conversationId?: string): Promise<PipelineResponseModel> {
    const res = await fetch(`${API_BASE_URL}/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ prompt, conversation_id: conversationId }),
    });

    const data = await res.json();
    if (!res.ok) {
      throw new Error(data.detail || data.error || `Chat request failed: ${res.status}`);
    }
    return data;
  },

  async confirmOperation(token: string, conversationId?: string): Promise<PipelineResponseModel> {
    const res = await fetch(`${API_BASE_URL}/confirm`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ token, conversation_id: conversationId }),
    });

    const data = await res.json();
    if (!res.ok) {
      throw new Error(data.detail || data.error || `Confirmation failed: ${res.status}`);
    }
    return data;
  },

  async clearConversation(conversationId: string): Promise<void> {
    const res = await fetch(`${API_BASE_URL}/conversations/${encodeURIComponent(conversationId)}`, {
      method: 'DELETE',
    });
    if (!res.ok) {
      console.warn(`Failed to clear session ${conversationId}: ${res.status}`);
    }
  },
};

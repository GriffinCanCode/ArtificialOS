/**
 * Session Persistence Types
 * Type definitions for workspace session management
 */

export interface Session {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
  workspace: Workspace;
  metadata: Record<string, any>;
}

export interface Workspace {
  apps: AppSnapshot[];
  focused_app_id?: string;
  chat_state?: ChatState;
  ui_state?: UIState;
}

export interface AppSnapshot {
  id: string;
  title: string;
  blueprint: Record<string, any>;
  state: string;
  parent_id?: string;
  created_at: string;
  metadata: Record<string, any>;
  services: string[];
  component_state?: Record<string, any>;
}

export interface ChatState {
  messages: Message[];
  thoughts: Thought[];
}

export interface Message {
  type: string;
  content: string;
  timestamp: number;
}

export interface Thought {
  content: string;
  timestamp: number;
}

export interface UIState {
  generation_thoughts?: string[];
  generation_preview?: string;
  is_loading: boolean;
  error?: string;
}

export interface SessionMetadata {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
  app_count: number;
}

export interface SessionStats {
  total_sessions: number;
  last_saved?: string;
  last_restored?: string;
}

export interface SaveSessionRequest {
  name: string;
  description?: string;
  chat_state?: ChatState;
  ui_state?: UIState;
  app_states?: Record<string, any>; // Component state per app
}

export interface SaveSessionResponse {
  success: boolean;
  session: SessionMetadata;
}

export interface ListSessionsResponse {
  sessions: SessionMetadata[];
  stats: SessionStats;
}

export interface RestoreSessionResponse {
  success: boolean;
  workspace: Workspace;
}

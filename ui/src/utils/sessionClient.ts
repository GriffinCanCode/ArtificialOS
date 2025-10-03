/**
 * Session API Client
 * Handles all API calls to the session backend
 */

import type {
  Session,
  SaveSessionRequest,
  SaveSessionResponse,
  ListSessionsResponse,
  RestoreSessionResponse,
} from '../types/session';

const API_BASE = 'http://localhost:8000';

export class SessionClient {
  /**
   * Save current workspace as a session
   */
  static async saveSession(request: SaveSessionRequest): Promise<SaveSessionResponse> {
    const response = await fetch(`${API_BASE}/sessions/save`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to save session');
    }

    return response.json();
  }

  /**
   * Save with default name (for auto-save)
   */
  static async saveDefault(): Promise<SaveSessionResponse> {
    const response = await fetch(`${API_BASE}/sessions/save-default`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to save session');
    }

    return response.json();
  }

  /**
   * List all saved sessions
   */
  static async listSessions(): Promise<ListSessionsResponse> {
    const response = await fetch(`${API_BASE}/sessions`);

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to list sessions');
    }

    return response.json();
  }

  /**
   * Get details of a specific session
   */
  static async getSession(sessionId: string): Promise<Session> {
    const response = await fetch(`${API_BASE}/sessions/${sessionId}`);

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to get session');
    }

    return response.json();
  }

  /**
   * Restore a session
   */
  static async restoreSession(sessionId: string): Promise<RestoreSessionResponse> {
    const response = await fetch(`${API_BASE}/sessions/${sessionId}/restore`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to restore session');
    }

    return response.json();
  }

  /**
   * Delete a session
   */
  static async deleteSession(sessionId: string): Promise<{ success: boolean }> {
    const response = await fetch(`${API_BASE}/sessions/${sessionId}`, {
      method: 'DELETE',
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to delete session');
    }

    return response.json();
  }
}


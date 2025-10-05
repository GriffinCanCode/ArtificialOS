/**
 * Type-Safe API Client
 * Provides validated HTTP and WebSocket communication with the backend
 */

import {
  ChatRequest,
  ChatResponse,
  ChatResponseSchema,
  GenerateUIResponse,
  GenerateUIResponseSchema,
  HealthResponse,
  HealthResponseSchema,
  ListAppsResponse,
  ListAppsResponseSchema,
  AppActionResponse,
  AppActionResponseSchema,
  ListServicesResponse,
  ListServicesResponseSchema,
  DiscoverServicesRequest,
  DiscoverServicesResponse,
  DiscoverServicesResponseSchema,
  ServiceExecuteRequest,
  ServiceExecuteResponse,
  ServiceExecuteResponseSchema,
} from "../types/api";
import { logger } from "../utils/monitoring/logger";

const API_BASE_URL = "http://localhost:8000";

/**
 * Generic fetch wrapper with Zod validation
 */
async function fetchWithValidation<T>(url: string, schema: any, options?: RequestInit): Promise<T> {
  try {
    const response = await fetch(`${API_BASE_URL}${url}`, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...options?.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    return schema.parse(data);
  } catch (error) {
    logger.error("API request failed", error as Error, {
      component: "APIClient",
      url,
    });
    throw error;
  }
}

// ============================================================================
// API Client Class
// ============================================================================

export class APIClient {
  /**
   * Health check
   */
  static async health(): Promise<HealthResponse> {
    return fetchWithValidation<HealthResponse>("/health", HealthResponseSchema);
  }

  /**
   * Non-streaming chat endpoint
   */
  static async chat(request: ChatRequest): Promise<ChatResponse> {
    return fetchWithValidation<ChatResponse>("/chat", ChatResponseSchema, {
      method: "POST",
      body: JSON.stringify(request),
    });
  }

  /**
   * Generate UI (non-streaming)
   */
  static async generateUI(request: ChatRequest): Promise<GenerateUIResponse> {
    return fetchWithValidation<GenerateUIResponse>("/generate-ui", GenerateUIResponseSchema, {
      method: "POST",
      body: JSON.stringify(request),
    });
  }

  /**
   * List all apps
   */
  static async listApps(): Promise<ListAppsResponse> {
    return fetchWithValidation<ListAppsResponse>("/apps", ListAppsResponseSchema);
  }

  /**
   * Focus an app
   */
  static async focusApp(appId: string): Promise<AppActionResponse> {
    return fetchWithValidation<AppActionResponse>(`/apps/${appId}/focus`, AppActionResponseSchema, {
      method: "POST",
    });
  }

  /**
   * Close an app
   */
  static async closeApp(appId: string): Promise<AppActionResponse> {
    return fetchWithValidation<AppActionResponse>(`/apps/${appId}`, AppActionResponseSchema, {
      method: "DELETE",
    });
  }

  /**
   * List all services
   */
  static async listServices(category?: string): Promise<ListServicesResponse> {
    const url = category ? `/services?category=${category}` : "/services";
    return fetchWithValidation<ListServicesResponse>(url, ListServicesResponseSchema);
  }

  /**
   * Discover relevant services
   */
  static async discoverServices(
    request: DiscoverServicesRequest
  ): Promise<DiscoverServicesResponse> {
    return fetchWithValidation<DiscoverServicesResponse>(
      "/services/discover",
      DiscoverServicesResponseSchema,
      {
        method: "POST",
        body: JSON.stringify(request),
      }
    );
  }

  /**
   * Execute a service tool
   */
  static async executeServiceTool(request: ServiceExecuteRequest): Promise<ServiceExecuteResponse> {
    return fetchWithValidation<ServiceExecuteResponse>(
      "/services/execute",
      ServiceExecuteResponseSchema,
      {
        method: "POST",
        body: JSON.stringify(request),
      }
    );
  }
}

// ============================================================================
// Export default instance
// ============================================================================

export default APIClient;

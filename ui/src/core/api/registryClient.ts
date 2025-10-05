/**
 * Registry API Client
 * Handles all API calls to the app registry backend
 */

import type {
  Package,
  SaveAppRequest,
  SaveAppResponse,
  ListAppsResponse,
  LaunchAppResponse,
} from "../types/registry";

const API_BASE = "http://localhost:8000";

export class RegistryClient {
  /**
   * Save a running app to the registry
   */
  static async saveApp(request: SaveAppRequest): Promise<SaveAppResponse> {
    const response = await fetch(`${API_BASE}/registry/save`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to save app");
    }

    return response.json();
  }

  /**
   * List all apps in the registry
   */
  static async listApps(category?: string): Promise<ListAppsResponse> {
    const url = new URL(`${API_BASE}/registry/apps`);
    if (category) {
      url.searchParams.set("category", category);
    }

    const response = await fetch(url.toString());

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to list apps");
    }

    return response.json();
  }

  /**
   * Get details of a specific app
   */
  static async getApp(packageId: string): Promise<Package> {
    const response = await fetch(`${API_BASE}/registry/apps/${packageId}`);

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to get app");
    }

    return response.json();
  }

  /**
   * Launch an app from the registry
   */
  static async launchApp(packageId: string): Promise<LaunchAppResponse> {
    const response = await fetch(`${API_BASE}/registry/apps/${packageId}/launch`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to launch app");
    }

    return response.json();
  }

  /**
   * Delete an app from the registry
   */
  static async deleteApp(packageId: string): Promise<{ success: boolean }> {
    const response = await fetch(`${API_BASE}/registry/apps/${packageId}`, {
      method: "DELETE",
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to delete app");
    }

    return response.json();
  }
}

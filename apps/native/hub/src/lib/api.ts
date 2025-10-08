/**
 * API Client for Hub App
 * Handles all HTTP communication with backend
 */

import type { RegistryResponse, LaunchResponse } from '../types';

const API_BASE = 'http://localhost:8000';

/**
 * Fetch all apps from registry
 */
export async function fetchApps(category?: string): Promise<RegistryResponse> {
  const url = new URL('/registry/apps', API_BASE);
  if (category && category !== 'all') {
    url.searchParams.set('category', category);
  }

  const response = await fetch(url.toString());
  if (!response.ok) {
    throw new Error(`Failed to fetch apps: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Launch an app from registry
 */
export async function launchApp(packageId: string): Promise<LaunchResponse> {
  const response = await fetch(`${API_BASE}/registry/apps/${packageId}/launch`, {
    method: 'POST',
  });

  if (!response.ok) {
    throw new Error(`Failed to launch app: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Close a running app
 */
export async function closeApp(appId: string): Promise<void> {
  const response = await fetch(`${API_BASE}/apps/${appId}`, {
    method: 'DELETE',
  });

  if (!response.ok) {
    throw new Error(`Failed to close app: ${response.statusText}`);
  }
}


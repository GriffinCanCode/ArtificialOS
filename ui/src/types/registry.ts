/**
 * App Registry Types
 * Type definitions for the app registry/persistence layer
 */

export interface Package {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  version: string;
  author: string;
  created_at: string;
  updated_at: string;
  ui_spec: Record<string, any>;
  services: string[];
  permissions: string[];
  tags: string[];
}

export interface PackageMetadata {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  version: string;
  author: string;
  created_at: string;
  tags: string[];
}

export interface RegistryStats {
  total_packages: number;
  categories: Record<string, number>;
  last_updated?: string;
}

export interface SaveAppRequest {
  app_id: string;
  description?: string;
  icon?: string;
  category?: string;
  tags?: string[];
}

export interface SaveAppResponse {
  success: boolean;
  package_id: string;
}

export interface ListAppsResponse {
  apps: PackageMetadata[];
  stats: RegistryStats;
}

export interface LaunchAppResponse {
  app_id: string;
  ui_spec: Record<string, any>;
  title: string;
}


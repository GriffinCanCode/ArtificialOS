/**
 * App Registry Types
 * Type definitions for the app registry/persistence layer
 */

export type AppType = 'blueprint' | 'native_web' | 'native_proc';

export interface Package {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  version: string;
  author: string;
  type: AppType;
  created_at: string;
  updated_at: string;
  services: string[];
  permissions: string[];
  tags: string[];

  // For blueprint apps
  blueprint?: Record<string, any>;

  // For native web apps
  bundle_path?: string;
  web_manifest?: NativeWebManifest;

  // For native process apps
  proc_manifest?: NativeProcManifest;
}

export interface NativeWebManifest {
  entry_point: string;
  exports: NativeExports;
  dev_server?: string;
}

export interface NativeExports {
  component: string;
}

export interface NativeProcManifest {
  executable: string;
  args: string[];
  working_dir: string;
  ui_type: string;
  env: Record<string, string>;
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
  type: AppType;
  title: string;
  icon: string;

  // For blueprint apps
  blueprint?: Record<string, any>;

  // For native web apps
  package_id?: string;
  bundle_path?: string;
  services?: string[];
  permissions?: string[];
  exports?: NativeExports;

  // For native process apps
  executable?: string;
  args?: string[];
  working_dir?: string;
  ui_type?: string;
  env?: Record<string, string>;
}

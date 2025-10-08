/**
 * Hub App Types
 */

export interface AppMetadata {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  version: string;
  author: string;
  type: 'blueprint' | 'native_web' | 'native_proc';
  created_at: string;
  tags: string[];
  bundle_path?: string;
  web_manifest?: {
    entry_point: string;
    exports: {
      component: string;
    };
    dev_server?: string;
  };
}

export interface AppStats {
  total_packages: number;
  categories: Record<string, number>;
  last_updated?: string;
}

export interface RegistryResponse {
  apps: AppMetadata[];
  stats: AppStats;
}

export interface LaunchResponse {
  app_id: string;
  type: string;
  title: string;
  icon: string;
  package_id: string;
  bundle_path?: string;
  services?: string[];
  permissions?: string[];
}

export type CategoryFilter = 'all' | 'favorites' | 'recent' | string;

export interface RecentApp {
  id: string;
  lastLaunched: number;
  launchCount: number;
}

export interface HubState {
  apps: AppMetadata[];
  filteredApps: AppMetadata[];
  searchQuery: string;
  selectedCategory: CategoryFilter;
  favorites: Set<string>;
  recents: Map<string, RecentApp>;
  loading: boolean;
  error: string | null;
}


/**
 * TanStack Query Hooks for Registry API
 * Provides caching, refetching, and optimistic updates for registry operations
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { RegistryClient } from "../utils/api/registryClient";
import type {
  Package,
  PackageMetadata,
  SaveAppRequest,
  SaveAppResponse,
  ListAppsResponse,
  LaunchAppResponse,
} from "../types/registry";
import { logger } from "../utils/monitoring/logger";

// ============================================================================
// Query Keys - Centralized for consistency
// ============================================================================

export const registryKeys = {
  all: ["registry"] as const,
  apps: () => [...registryKeys.all, "apps"] as const,
  appsList: (category?: string) => [...registryKeys.apps(), "list", { category }] as const,
  app: (id: string) => [...registryKeys.apps(), "detail", id] as const,
};

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Fetch list of registry apps with optional category filter
 * - Automatically cached and refetched in background
 * - Deduplicates requests
 */
export function useRegistryApps(category?: string) {
  return useQuery({
    queryKey: registryKeys.appsList(category),
    queryFn: async () => {
      logger.info("Fetching registry apps", {
        component: "useRegistryApps",
        category,
      });
      return RegistryClient.listApps(category);
    },
    staleTime: 30 * 1000, // Consider data fresh for 30 seconds
    gcTime: 5 * 60 * 1000, // Keep unused data in cache for 5 minutes
    retry: 2,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Fetch details of a specific registry app
 */
export function useRegistryApp(packageId: string, options?: { enabled?: boolean }) {
  return useQuery({
    queryKey: registryKeys.app(packageId),
    queryFn: async () => {
      logger.info("Fetching registry app details", {
        component: "useRegistryApp",
        packageId,
      });
      return RegistryClient.getApp(packageId);
    },
    enabled: options?.enabled ?? true,
    staleTime: 60 * 1000, // App details change less frequently
    gcTime: 10 * 60 * 1000,
    retry: 2,
  });
}

// ============================================================================
// Mutation Hooks
// ============================================================================

/**
 * Launch an app from the registry
 * - Automatically updates cache after successful launch
 */
export function useLaunchApp() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (packageId: string) => {
      logger.info("Launching registry app", {
        component: "useLaunchApp",
        packageId,
      });
      return RegistryClient.launchApp(packageId);
    },
    onSuccess: (data, packageId) => {
      logger.info("App launched successfully", {
        component: "useLaunchApp",
        packageId,
        appId: data.app_id,
      });
      // Could invalidate running apps query here if we had one
    },
    onError: (error, packageId) => {
      logger.error("Failed to launch app", error as Error, {
        component: "useLaunchApp",
        packageId,
      });
    },
  });
}

/**
 * Save a running app to the registry
 * - Invalidates apps list to refresh after save
 */
export function useSaveApp() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: SaveAppRequest) => {
      logger.info("Saving app to registry", {
        component: "useSaveApp",
        appId: request.app_id,
      });
      return RegistryClient.saveApp(request);
    },
    onSuccess: (data, request) => {
      logger.info("App saved to registry", {
        component: "useSaveApp",
        packageId: data.package_id,
      });

      // Invalidate all apps lists to refetch with new app
      queryClient.invalidateQueries({ queryKey: registryKeys.apps() });
    },
    onError: (error, request) => {
      logger.error("Failed to save app", error as Error, {
        component: "useSaveApp",
        appId: request.app_id,
      });
    },
  });
}

/**
 * Delete an app from the registry
 * - Optimistically removes from cache
 * - Rolls back on error
 */
export function useDeleteApp() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (packageId: string) => {
      logger.info("Deleting app from registry", {
        component: "useDeleteApp",
        packageId,
      });
      return RegistryClient.deleteApp(packageId);
    },
    onMutate: async (packageId) => {
      // Cancel outgoing refetches to avoid overwriting optimistic update
      await queryClient.cancelQueries({ queryKey: registryKeys.apps() });

      // Snapshot previous values for rollback
      const previousApps = queryClient.getQueriesData({
        queryKey: registryKeys.apps(),
      });

      // Optimistically update all apps lists
      queryClient.setQueriesData<ListAppsResponse>({ queryKey: registryKeys.apps() }, (old) => {
        if (!old) return old;
        return {
          ...old,
          apps: old.apps.filter((app) => app.id !== packageId),
          stats: {
            ...old.stats,
            total_packages: old.stats.total_packages - 1,
          },
        };
      });

      return { previousApps };
    },
    onError: (error, packageId, context) => {
      logger.error("Failed to delete app", error as Error, {
        component: "useDeleteApp",
        packageId,
      });

      // Rollback to previous state
      if (context?.previousApps) {
        context.previousApps.forEach(([queryKey, data]) => {
          queryClient.setQueryData(queryKey, data);
        });
      }
    },
    onSettled: () => {
      // Always refetch after error or success to ensure consistency
      queryClient.invalidateQueries({ queryKey: registryKeys.apps() });
    },
  });
}

// ============================================================================
// Combined Hook for All Registry Mutations
// ============================================================================

/**
 * Get all registry mutations in one hook for convenience
 */
export function useRegistryMutations() {
  const launchApp = useLaunchApp();
  const saveApp = useSaveApp();
  const deleteApp = useDeleteApp();

  return {
    launchApp,
    saveApp,
    deleteApp,
  };
}

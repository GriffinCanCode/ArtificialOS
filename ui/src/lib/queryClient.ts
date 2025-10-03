/**
 * TanStack Query Client Configuration
 * Centralized configuration for React Query
 */

import { QueryClient } from "@tanstack/react-query";
import { logger } from "../utils/logger";

/**
 * Create and configure the Query Client with sensible defaults
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Disable automatic refetching by default for performance
      // Individual queries can override this
      refetchOnWindowFocus: false,
      refetchOnReconnect: true,
      retry: 1,
      staleTime: 0, // Data is considered stale immediately by default
      gcTime: 5 * 60 * 1000, // 5 minutes cache time
      // Prevent errors during HMR
      throwOnError: false,
    },
    mutations: {
      retry: 0,
      // Prevent errors during HMR
      throwOnError: false,
      onError: (error) => {
        logger.error("Mutation error", error as Error, {
          component: "QueryClient",
        });
      },
    },
  },
});


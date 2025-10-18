/**
 * Tracker Hook
 *
 * React hook for monitoring system integration and configuration.
 * Follows exact patterns from useSessionManager.ts with action-based API.
 */

import { useEffect, useCallback, useMemo } from 'react';
import { useTrackerStore } from '../stores/tracker';
import type { TrackerConfig, TrackerPlugin } from '../types/tracker';
import { logger } from '../logger';

// ============================================================================
// Hook Interface
// ============================================================================

export interface UseTrackerReturn {
  /** Tracker status */
  status: {
    overall: string;
    isRunning: boolean;
    hasError: boolean;
    errorMessage?: string;
  };

  /** Feature states */
  features: {
    journeyTracking: boolean;
    causalityTracking: boolean;
    performanceMonitoring: boolean;
    errorTracking: boolean;
    realtimeDebugging: boolean;
  };

  /** System health */
  health: {
    score: number;
    overall: string;
    memoryUsage: string;
    cpuUsage: string;
    errorRate: string;
  };

  /** Tracker actions */
  actions: {
    start: () => Promise<void>;
    stop: () => Promise<void>;
    pause: () => void;
    resume: () => void;
    restart: () => Promise<void>;
    updateConfig: (config: Partial<TrackerConfig>) => void;
    toggleFeature: (feature: string, enabled: boolean) => void;
  };

  /** Plugin management */
  plugins: {
    registered: string[];
    active: string[];
    register: (plugin: TrackerPlugin) => Promise<void>;
    unregister: (name: string) => Promise<void>;
    toggle: (name: string, enabled: boolean) => Promise<void>;
  };

  /** Metrics and analytics */
  metrics: {
    eventsPerSecond: number;
    memoryUsageMB: number;
    cpuUsagePercent: number;
    successRate: number;
    errorRate: number;
  };

  /** Configuration */
  config: TrackerConfig;

  /** Utilities */
  utils: {
    export: () => any;
    reset: () => void;
    isInitialized: boolean;
  };
}

// ============================================================================
// Main Hook
// ============================================================================

/**
 * Hook for tracker integration and management
 *
 * @param autoInitialize - Whether to automatically initialize tracker on mount
 * @param initialConfig - Initial configuration for tracker
 *
 * @example
 * const tracker = useTracker(true, {
 *   features: { journeyTracking: true, performanceMonitoring: true }
 * });
 *
 * useEffect(() => {
 *   if (tracker.status.isRunning) {
 *     console.log('Tracker is running', tracker.health);
 *   }
 * }, [tracker.status.isRunning]);
 */
export function useTracker(
  autoInitialize: boolean = true,
  initialConfig?: Partial<TrackerConfig>
): UseTrackerReturn {
  // Store selectors
  const state = useTrackerStore((state) => state.state);
  const plugins = useTrackerStore((state) => state.plugins);
  const isInitialized = useTrackerStore((state) => state.meta.initialized);

  // Store actions
  const {
    initialize,
    start,
    stop,
    pause,
    resume,
    updateConfig,
    toggleFeature,
    registerPlugin,
    unregisterPlugin,
    togglePlugin,
    getHealth,
    export: exportState,
    reset,
  } = useTrackerStore();

  // Auto-initialize on mount
  useEffect(() => {
    if (autoInitialize && !isInitialized) {
      initialize(initialConfig).catch((error) => {
        logger.error('Failed to auto-initialize tracker', error, {
          component: 'useTracker',
          initialConfig,
        });
      });
    }
  }, [autoInitialize, isInitialized, initialConfig, initialize]);

  // Memoized status
  const status = useMemo(() => ({
    overall: state.status.overall,
    isRunning: state.status.overall === 'running',
    hasError: state.status.overall === 'error',
    errorMessage: state.status.error?.message,
  }), [state.status]);

  // Memoized features
  const features = useMemo(() => ({
    journeyTracking: state.config.features.journeyTracking,
    causalityTracking: state.config.features.causalityTracking,
    performanceMonitoring: state.config.features.performanceMonitoring,
    errorTracking: state.config.features.errorTracking,
    realtimeDebugging: state.config.features.realtimeDebugging,
  }), [state.config.features]);

  // Memoized health
  const health = useMemo(() => {
    const healthData = getHealth();
    return {
      score: healthData.score,
      overall: healthData.overall,
      memoryUsage: healthData.details.memoryUsage,
      cpuUsage: healthData.details.cpuUsage,
      errorRate: healthData.details.errorRate,
    };
  }, [getHealth]);

  // Memoized actions
  const actions = useMemo(() => ({
    start: async () => {
      try {
        await start();
        logger.info('Tracker started via useTracker', { component: 'useTracker' });
      } catch (error) {
        logger.error('Failed to start tracker', error as Error, { component: 'useTracker' });
      }
    },

    stop: async () => {
      try {
        await stop();
        logger.info('Tracker stopped via useTracker', { component: 'useTracker' });
      } catch (error) {
        logger.error('Failed to stop tracker', error as Error, { component: 'useTracker' });
      }
    },

    pause: () => {
      pause();
      logger.info('Tracker paused via useTracker', { component: 'useTracker' });
    },

    resume: () => {
      resume();
      logger.info('Tracker resumed via useTracker', { component: 'useTracker' });
    },

    restart: async () => {
      try {
        await stop();
        await start();
        logger.info('Tracker restarted via useTracker', { component: 'useTracker' });
      } catch (error) {
        logger.error('Failed to restart tracker', error as Error, { component: 'useTracker' });
      }
    },

    updateConfig: (config: Partial<TrackerConfig>) => {
      updateConfig(config);
      logger.debug('Tracker config updated via useTracker', {
        component: 'useTracker',
        config,
      });
    },

    toggleFeature: (feature: string, enabled: boolean) => {
      toggleFeature(feature as any, enabled);
      logger.info(`Feature ${feature} ${enabled ? 'enabled' : 'disabled'} via useTracker`, {
        component: 'useTracker',
        feature,
        enabled,
      });
    },
  }), [start, stop, pause, resume, updateConfig, toggleFeature]);

  // Memoized plugin management
  const pluginManagement = useMemo(() => {
    const registered = Array.from(plugins.plugins.keys());
    const active = registered.filter(name => {
      const pluginState = plugins.states.get(name);
      return pluginState?.status === 'running';
    });

    return {
      registered,
      active,

      register: async (plugin: TrackerPlugin) => {
        try {
          await registerPlugin(plugin);
          logger.info(`Plugin ${plugin.meta.name} registered via useTracker`, {
            component: 'useTracker',
            plugin: plugin.meta,
          });
        } catch (error) {
          logger.error(`Failed to register plugin ${plugin.meta.name}`, error as Error, {
            component: 'useTracker',
          });
          throw error;
        }
      },

      unregister: async (name: string) => {
        try {
          await unregisterPlugin(name);
          logger.info(`Plugin ${name} unregistered via useTracker`, {
            component: 'useTracker',
            pluginName: name,
          });
        } catch (error) {
          logger.error(`Failed to unregister plugin ${name}`, error as Error, {
            component: 'useTracker',
          });
          throw error;
        }
      },

      toggle: async (name: string, enabled: boolean) => {
        try {
          await togglePlugin(name, enabled);
          logger.info(`Plugin ${name} ${enabled ? 'enabled' : 'disabled'} via useTracker`, {
            component: 'useTracker',
            pluginName: name,
            enabled,
          });
        } catch (error) {
          logger.error(`Failed to toggle plugin ${name}`, error as Error, {
            component: 'useTracker',
          });
          throw error;
        }
      },
    };
  }, [plugins, registerPlugin, unregisterPlugin, togglePlugin]);

  // Memoized metrics
  const metrics = useMemo(() => ({
    eventsPerSecond: state.metrics.volume.eventsPerSecond,
    memoryUsageMB: state.metrics.resources.memoryUsageMB,
    cpuUsagePercent: state.metrics.resources.cpuUsagePercent,
    successRate: state.metrics.quality.successRate,
    errorRate: state.metrics.quality.errorRate,
  }), [state.metrics]);

  // Memoized utilities
  const utils = useMemo(() => ({
    export: () => exportState(),
    reset: () => {
      reset();
      logger.info('Tracker reset via useTracker', { component: 'useTracker' });
    },
    isInitialized,
  }), [exportState, reset, isInitialized]);

  return {
    status,
    features,
    health,
    actions,
    plugins: pluginManagement,
    metrics,
    config: state.config,
    utils,
  };
}

// ============================================================================
// Specialized Hooks
// ============================================================================

/**
 * Hook for feature-specific tracker management
 * Provides focused API for individual features
 */
export function useTrackerFeature(featureName: keyof TrackerConfig['features']) {
  const tracker = useTracker(false); // Don't auto-initialize

  const isEnabled = tracker.features[featureName as keyof typeof tracker.features];

  const toggle = useCallback((enabled: boolean) => {
    tracker.actions.toggleFeature(featureName, enabled);
  }, [tracker.actions, featureName]);

  const enable = useCallback(() => toggle(true), [toggle]);
  const disable = useCallback(() => toggle(false), [toggle]);

  return {
    isEnabled,
    toggle,
    enable,
    disable,
    status: tracker.status,
    health: tracker.health,
  };
}

/**
 * Hook for plugin development and testing
 * Provides utilities for plugin authors
 */
export function useTrackerPlugin(pluginName: string) {
  const tracker = useTracker(false);

  const plugin = useTrackerStore((state) =>
    tracker.plugins.registered.includes(pluginName)
      ? state.plugins.plugins.get(pluginName)
      : undefined
  );

  const pluginState = useTrackerStore((state) => state.plugins.states.get(pluginName));

  const isRegistered = !!plugin;
  const isActive = pluginState?.status === 'running';
  const hasError = pluginState?.status === 'error';

  const register = useCallback(async (pluginToRegister: TrackerPlugin) => {
    await tracker.plugins.register(pluginToRegister);
  }, [tracker.plugins]);

  const unregister = useCallback(async () => {
    if (isRegistered) {
      await tracker.plugins.unregister(pluginName);
    }
  }, [tracker.plugins, pluginName, isRegistered]);

  const toggle = useCallback(async (enabled: boolean) => {
    if (isRegistered) {
      await tracker.plugins.toggle(pluginName, enabled);
    }
  }, [tracker.plugins, pluginName, isRegistered]);

  return {
    plugin,
    state: pluginState,
    isRegistered,
    isActive,
    hasError,
    register,
    unregister,
    toggle,
    trackerStatus: tracker.status,
  };
}

/**
 * Hook for monitoring system health
 * Provides health monitoring and alerting capabilities
 */
export function useTrackerHealth(alertThresholds = { score: 50, memory: 80, cpu: 80 }) {
  const tracker = useTracker(false);

  const isHealthy = tracker.health.score >= alertThresholds.score;
  const hasMemoryIssue = tracker.metrics.memoryUsageMB > alertThresholds.memory;
  const hasCpuIssue = tracker.metrics.cpuUsagePercent > alertThresholds.cpu;

  const alerts = useMemo(() => {
    const alertList: Array<{ type: string; message: string; severity: 'low' | 'medium' | 'high' }> = [];

    if (!isHealthy) {
      alertList.push({
        type: 'health',
        message: `System health score is low: ${tracker.health.score}`,
        severity: tracker.health.score < 25 ? 'high' : 'medium',
      });
    }

    if (hasMemoryIssue) {
      alertList.push({
        type: 'memory',
        message: `High memory usage: ${tracker.metrics.memoryUsageMB}MB`,
        severity: tracker.metrics.memoryUsageMB > 100 ? 'high' : 'medium',
      });
    }

    if (hasCpuIssue) {
      alertList.push({
        type: 'cpu',
        message: `High CPU usage: ${tracker.metrics.cpuUsagePercent}%`,
        severity: tracker.metrics.cpuUsagePercent > 90 ? 'high' : 'medium',
      });
    }

    if (tracker.metrics.errorRate > 0.1) {
      alertList.push({
        type: 'errors',
        message: `High error rate: ${(tracker.metrics.errorRate * 100).toFixed(1)}%`,
        severity: tracker.metrics.errorRate > 0.2 ? 'high' : 'medium',
      });
    }

    return alertList;
  }, [isHealthy, hasMemoryIssue, hasCpuIssue, tracker.health.score, tracker.metrics]);

  // Log health alerts
  useEffect(() => {
    alerts.forEach(alert => {
      if (alert.severity === 'high') {
        logger.error(`Health Alert: ${alert.message}`, undefined, {
          component: 'useTrackerHealth',
          alertType: alert.type,
          severity: alert.severity,
        });
      } else {
        logger.warn(`Health Alert: ${alert.message}`, {
          component: 'useTrackerHealth',
          alertType: alert.type,
          severity: alert.severity,
        });
      }
    });
  }, [alerts]);

  return {
    health: tracker.health,
    metrics: tracker.metrics,
    isHealthy,
    hasMemoryIssue,
    hasCpuIssue,
    alerts,
    thresholds: alertThresholds,
  };
}

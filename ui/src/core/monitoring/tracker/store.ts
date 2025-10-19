/**
 * Tracker Store
 *
 * Zustand store for monitoring system configuration and orchestration.
 * Manages overall tracker state, plugins, and feature coordination.
 */

import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type {
  TrackerConfig,
  TrackerState,
  TrackerFeatures,
  TrackerMetrics,
  TrackerEvent,
  TrackerPlugin,
  TrackerPluginRegistry,
} from './types';
import { logger, LogLevel } from '../core/logger';

// ============================================================================
// Store Interface
// ============================================================================

interface TrackerStore {
  /** Current tracker state */
  state: TrackerState;

  /** Plugin registry */
  plugins: TrackerPluginRegistry;

  /** Event queue */
  eventQueue: TrackerEvent[];

  /** Store metadata */
  meta: {
    initialized: boolean;
    version: string;
    lastUpdate: number;
  };

  // ============================================================================
  // Configuration Actions
  // ============================================================================

  /** Initialize tracker with configuration */
  initialize: (config?: Partial<TrackerConfig>) => Promise<void>;

  /** Update tracker configuration */
  updateConfig: (config: Partial<TrackerConfig>) => void;

  /** Enable/disable specific features */
  toggleFeature: (feature: keyof TrackerFeatures, enabled: boolean) => void;

  // ============================================================================
  // Status Management
  // ============================================================================

  /** Start tracker */
  start: () => Promise<void>;

  /** Stop tracker */
  stop: () => Promise<void>;

  /** Pause tracker */
  pause: () => void;

  /** Resume tracker */
  resume: () => void;

  /** Set tracker error state */
  setError: (error: Error) => void;

  /** Clear tracker error */
  clearError: () => void;

  // ============================================================================
  // Plugin Management
  // ============================================================================

  /** Register a plugin */
  registerPlugin: (plugin: TrackerPlugin) => Promise<void>;

  /** Unregister a plugin */
  unregisterPlugin: (name: string) => Promise<void>;

  /** Enable/disable a plugin */
  togglePlugin: (name: string, enabled: boolean) => Promise<void>;

  /** Get plugin by name */
  getPlugin: (name: string) => TrackerPlugin | undefined;

  /** List all plugins */
  listPlugins: () => TrackerPlugin[];

  // ============================================================================
  // Event Management
  // ============================================================================

  /** Emit tracker event */
  emit: (event: Omit<TrackerEvent, 'meta'>) => void;

  /** Process event queue */
  processEvents: () => Promise<void>;

  /** Clear event queue */
  clearEvents: () => void;

  // ============================================================================
  // Metrics and Analytics
  // ============================================================================

  /** Update metrics */
  updateMetrics: (metrics: Partial<TrackerMetrics>) => void;

  /** Get current metrics */
  getMetrics: () => TrackerMetrics;

  /** Get system health status */
  getHealth: () => HealthStatus;

  // ============================================================================
  // Utilities
  // ============================================================================

  /** Export tracker state */
  export: () => TrackerExport;

  /** Reset tracker state */
  reset: () => void;

  /** Destroy tracker and cleanup */
  destroy: () => Promise<void>;
}

interface HealthStatus {
  overall: 'healthy' | 'degraded' | 'critical';
  details: {
    memoryUsage: 'normal' | 'high' | 'critical';
    cpuUsage: 'normal' | 'high' | 'critical';
    errorRate: 'normal' | 'elevated' | 'critical';
    performance: 'normal' | 'slow' | 'critical';
  };
  score: number; // 0-100
}

interface TrackerExport {
  state: TrackerState;
  plugins: string[];
  metrics: TrackerMetrics;
  health: HealthStatus;
  exportedAt: number;
  version: string;
}

// ============================================================================
// Default Configuration
// ============================================================================

const createDefaultConfig = (): TrackerConfig => ({
  enabled: true,
  environment: (process.env.NODE_ENV as 'development' | 'production') || 'development',
  features: {
    journeyTracking: true,
    causalityTracking: true,
    performanceMonitoring: true,
    errorTracking: true,
    behaviorAnalytics: false,
    abTesting: false,
    realtimeDebugging: process.env.NODE_ENV === 'development',
    predictiveAnalysis: false,
  },
  performance: {
    sampling: {
      journeys: 1.0,
      performance: 1.0,
      errors: 1.0,
      debugging: process.env.NODE_ENV === 'development' ? 1.0 : 0.1,
    },
    buffers: {
      maxEvents: 10000,
      maxJourneys: 100,
      flushInterval: 5000,
    },
    limits: {
      maxMemoryMB: 50,
      maxCpuPercent: 5,
      maxNetworkKBps: 100,
    },
    optimization: {
      enableDeduplication: true,
      enableCompression: false,
      enableBatching: true,
      batchSize: 50,
    },
  },
  integrations: {
    logger: {
      enabled: true,
      level: 'INFO' as LogLevel,
      includeContext: true,
      includeStackTrace: process.env.NODE_ENV === 'development',
    },
    external: {},
    backend: {
      enabled: false,
      batchSize: 100,
      retryAttempts: 3,
    },
    realtime: {
      enabled: false,
    },
  },
  privacy: {
    anonymization: {
      enabled: process.env.NODE_ENV === 'production',
      anonymizeIPs: true,
      anonymizeUserIds: false,
      hashSensitiveData: true,
    },
    retention: {
      journeyDays: 7,
      performanceDays: 3,
      errorDays: 30,
      debugDays: 1,
    },
    compliance: {
      gdprCompliant: false,
      ccpaCompliant: false,
      excludePII: true,
      consentRequired: false,
    },
    userControls: {
      allowOptOut: true,
      allowDataExport: false,
      allowDataDeletion: false,
      showPrivacyNotice: false,
    },
  },
});

const createDefaultMetrics = (): TrackerMetrics => ({
  resources: {
    memoryUsageMB: 0,
    cpuUsagePercent: 0,
    networkUsageKBps: 0,
  },
  performance: {
    eventProcessingLatency: 0,
    bufferUtilization: 0,
    batchProcessingTime: 0,
  },
  volume: {
    eventsPerSecond: 0,
    dataPointsPerSecond: 0,
    bytesPerSecond: 0,
  },
  quality: {
    successRate: 1.0,
    errorRate: 0,
    droppedEventRate: 0,
  },
});

// ============================================================================
// Store Implementation
// ============================================================================

export const useTrackerStore = create<TrackerStore>()(
  subscribeWithSelector((set, get) => ({
    state: {
      config: createDefaultConfig(),
      status: {
        overall: 'stopped',
        features: {} as any,
        lastUpdate: 0,
      },
      features: {
        journeyTracking: { activeJourneys: 0, totalJourneys: 0, averageDuration: 0 },
        causalityTracking: { activeChains: 0, totalChains: 0, averageChainLength: 0 },
        performanceMonitoring: { metricsCollected: 0, alertsTriggered: 0, averageLatency: 0 },
        errorTracking: { errorsTracked: 0, criticalErrors: 0, resolvedErrors: 0 },
      },
      metrics: createDefaultMetrics(),
      sessions: {
        active: 0,
        total: 0,
        sessions: new Map(),
      },
    },
    plugins: {
      plugins: new Map(),
      states: new Map(),
      dependencies: new Map(),
    },
    eventQueue: [],
    meta: {
      initialized: false,
      version: '1.0.0',
      lastUpdate: 0,
    },

    initialize: async (config?: Partial<TrackerConfig>) => {
      const mergedConfig = {
        ...get().state.config,
        ...config,
      };

      set((state) => ({
        state: {
          ...state.state,
          config: mergedConfig,
          status: {
            ...state.state.status,
            overall: 'initializing',
            lastUpdate: Date.now(),
          },
        },
        meta: {
          ...state.meta,
          initialized: true,
          lastUpdate: Date.now(),
        },
      }));

      // Initialize features based on configuration
      const enabledFeatures = Object.entries(mergedConfig.features)
        .filter(([, enabled]) => enabled)
        .map(([feature]) => feature);

      logger.info('Tracker initializing', {
        component: 'TrackerStore',
        enabledFeatures,
        environment: mergedConfig.environment,
      });

      // Emit initialization event
      get().emit({
        type: 'tracker_initialized',
        payload: { config: mergedConfig, features: enabledFeatures },
      });

      // Set status to running
      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'running',
            lastUpdate: Date.now(),
          },
        },
      }));

      logger.info('Tracker initialized successfully', {
        component: 'TrackerStore',
        version: get().meta.version,
      });
    },

    updateConfig: (config: Partial<TrackerConfig>) => {
      set((state) => ({
        state: {
          ...state.state,
          config: { ...state.state.config, ...config },
        },
      }));

      get().emit({
        type: 'config_updated',
        payload: config,
      });
    },

    toggleFeature: (feature: keyof TrackerFeatures, enabled: boolean) => {
      set((state) => ({
        state: {
          ...state.state,
          config: {
            ...state.state.config,
            features: {
              ...state.state.config.features,
              [feature]: enabled,
            },
          },
        },
      }));

      get().emit({
        type: enabled ? 'feature_enabled' : 'feature_disabled',
        payload: { feature, enabled },
      });

      logger.info(`Feature ${enabled ? 'enabled' : 'disabled'}`, {
        component: 'TrackerStore',
        feature,
        enabled,
      });
    },

    start: async () => {
      const state = get().state;

      if (state.status.overall === 'running') {
        logger.warn('Tracker already running', { component: 'TrackerStore' });
        return;
      }

      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'running',
            lastUpdate: Date.now(),
          },
        },
      }));

      // Start plugins
      for (const [name, plugin] of get().plugins.plugins) {
        try {
          await plugin.hooks.onStart?.();

          set((state) => ({
            plugins: {
              ...state.plugins,
              states: new Map(state.plugins.states).set(name, {
                status: 'running',
                lastActivity: Date.now(),
                metrics: {},
              }),
            },
          }));
        } catch (error) {
          logger.error(`Plugin ${name} failed to start`, error as Error, {
            component: 'TrackerStore',
          });
        }
      }

      get().emit({
        type: 'tracker_started',
        payload: { timestamp: Date.now() },
      });

      logger.info('Tracker started', { component: 'TrackerStore' });
    },

    stop: async () => {
      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'stopped',
            lastUpdate: Date.now(),
          },
        },
      }));

      // Stop plugins
      for (const [name, plugin] of get().plugins.plugins) {
        try {
          await plugin.hooks.onStop?.();

          set((state) => ({
            plugins: {
              ...state.plugins,
              states: new Map(state.plugins.states).set(name, {
                status: 'stopped',
                lastActivity: Date.now(),
                metrics: {},
              }),
            },
          }));
        } catch (error) {
          logger.error(`Plugin ${name} failed to stop`, error as Error, {
            component: 'TrackerStore',
          });
        }
      }

      get().emit({
        type: 'tracker_stopped',
        payload: { timestamp: Date.now() },
      });

      logger.info('Tracker stopped', { component: 'TrackerStore' });
    },

    pause: () => {
      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'paused',
            lastUpdate: Date.now(),
          },
        },
      }));
    },

    resume: () => {
      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'running',
            lastUpdate: Date.now(),
          },
        },
      }));
    },

    setError: (error: Error) => {
      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'error',
            error: {
              message: error.message,
              stack: error.stack,
              timestamp: Date.now(),
            },
            lastUpdate: Date.now(),
          },
        },
      }));

      get().emit({
        type: 'tracker_error',
        payload: { error: error.message, stack: error.stack },
      });
    },

    clearError: () => {
      set((state) => ({
        state: {
          ...state.state,
          status: {
            ...state.state.status,
            overall: 'running',
            error: undefined,
            lastUpdate: Date.now(),
          },
        },
      }));
    },

    registerPlugin: async (plugin: TrackerPlugin) => {
      const name = plugin.meta.name;

      // Check if plugin already exists
      if (get().plugins.plugins.has(name)) {
        throw new Error(`Plugin ${name} is already registered`);
      }

      try {
        // Initialize plugin
        await plugin.hooks.onInitialize?.();

        set((state) => ({
          plugins: {
            ...state.plugins,
            plugins: new Map(state.plugins.plugins).set(name, plugin),
            states: new Map(state.plugins.states).set(name, {
              status: 'initialized',
              lastActivity: Date.now(),
              metrics: {},
            }),
          },
        }));

        logger.info(`Plugin registered: ${name}`, {
          component: 'TrackerStore',
          plugin: plugin.meta,
        });
      } catch (error) {
        logger.error(`Failed to register plugin: ${name}`, error as Error, {
          component: 'TrackerStore',
        });
        throw error;
      }
    },

    unregisterPlugin: async (name: string) => {
      const plugin = get().plugins.plugins.get(name);
      if (!plugin) {
        throw new Error(`Plugin ${name} not found`);
      }

      try {
        await plugin.hooks.onDestroy?.();

        set((state) => {
          const newPlugins = new Map(state.plugins.plugins);
          const newStates = new Map(state.plugins.states);
          newPlugins.delete(name);
          newStates.delete(name);

          return {
            plugins: {
              ...state.plugins,
              plugins: newPlugins,
              states: newStates,
            },
          };
        });

        logger.info(`Plugin unregistered: ${name}`, { component: 'TrackerStore' });
      } catch (error) {
        logger.error(`Failed to unregister plugin: ${name}`, error as Error, {
          component: 'TrackerStore',
        });
        throw error;
      }
    },

    togglePlugin: async (name: string, enabled: boolean) => {
      const plugin = get().plugins.plugins.get(name);
      if (!plugin) {
        throw new Error(`Plugin ${name} not found`);
      }

      try {
        if (enabled) {
          await plugin.hooks.onStart?.();
        } else {
          await plugin.hooks.onStop?.();
        }

        set((state) => ({
          plugins: {
            ...state.plugins,
            states: new Map(state.plugins.states).set(name, {
              status: enabled ? 'running' : 'stopped',
              lastActivity: Date.now(),
              metrics: {},
            }),
          },
        }));
      } catch (error) {
        logger.error(`Failed to toggle plugin: ${name}`, error as Error, {
          component: 'TrackerStore',
        });
        throw error;
      }
    },

    getPlugin: (name: string) => {
      return get().plugins.plugins.get(name);
    },

    listPlugins: () => {
      return Array.from(get().plugins.plugins.values());
    },

    emit: (event: Omit<TrackerEvent, 'meta'>) => {
      const fullEvent: TrackerEvent = {
        ...event,
        meta: {
          timestamp: Date.now(),
          sessionId: 'current_session', // TODO: Get from session store
          source: 'tracker',
        },
      };

      set((state) => ({
        eventQueue: [...state.eventQueue, fullEvent],
      }));

      // Process events if queue is getting large
      const queue = get().eventQueue;
      if (queue.length > 100) {
        get().processEvents();
      }
    },

    processEvents: async () => {
      const events = get().eventQueue;
      if (events.length === 0) return;

      // Clear queue
      set({ eventQueue: [] });

      // Process events through plugins
      for (const event of events) {
        for (const [name, plugin] of get().plugins.plugins) {
          try {
            await plugin.hooks.onEvent?.(event);
          } catch (error) {
            logger.error(`Plugin ${name} failed to process event`, error as Error, {
              component: 'TrackerStore',
              eventType: event.type,
            });
          }
        }
      }
    },

    clearEvents: () => {
      set({ eventQueue: [] });
    },

    updateMetrics: (metrics: Partial<TrackerMetrics>) => {
      set((state) => ({
        state: {
          ...state.state,
          metrics: { ...state.state.metrics, ...metrics },
        },
      }));
    },

    getMetrics: () => {
      return get().state.metrics;
    },

    getHealth: (): HealthStatus => {
      const metrics = get().state.metrics;
      const config = get().state.config;

      const memoryStatus =
        metrics.resources.memoryUsageMB > config.performance.limits.maxMemoryMB * 0.9 ? 'critical' :
        metrics.resources.memoryUsageMB > config.performance.limits.maxMemoryMB * 0.7 ? 'high' : 'normal';

      const cpuStatus =
        metrics.resources.cpuUsagePercent > config.performance.limits.maxCpuPercent * 0.9 ? 'critical' :
        metrics.resources.cpuUsagePercent > config.performance.limits.maxCpuPercent * 0.7 ? 'high' : 'normal';

      const errorStatus =
        metrics.quality.errorRate > 0.1 ? 'critical' :
        metrics.quality.errorRate > 0.05 ? 'elevated' : 'normal';

      const performanceStatus =
        metrics.performance.eventProcessingLatency > 1000 ? 'critical' :
        metrics.performance.eventProcessingLatency > 500 ? 'slow' : 'normal';

      const issues = [memoryStatus, cpuStatus, errorStatus, performanceStatus]
        .filter(status => status !== 'normal').length;

      const overall =
        issues >= 2 || [memoryStatus, cpuStatus, errorStatus, performanceStatus].includes('critical') ? 'critical' :
        issues >= 1 ? 'degraded' : 'healthy';

      const score = Math.max(0, 100 - (issues * 25));

      return {
        overall,
        details: {
          memoryUsage: memoryStatus,
          cpuUsage: cpuStatus,
          errorRate: errorStatus,
          performance: performanceStatus,
        },
        score,
      };
    },

    export: (): TrackerExport => {
      const state = get().state;
      return {
        state,
        plugins: Array.from(get().plugins.plugins.keys()),
        metrics: state.metrics,
        health: get().getHealth(),
        exportedAt: Date.now(),
        version: get().meta.version,
      };
    },

    reset: () => {
      set({
        state: {
          config: createDefaultConfig(),
          status: {
            overall: 'stopped',
            features: {} as any,
            lastUpdate: 0,
          },
          features: {
            journeyTracking: { activeJourneys: 0, totalJourneys: 0, averageDuration: 0 },
            causalityTracking: { activeChains: 0, totalChains: 0, averageChainLength: 0 },
            performanceMonitoring: { metricsCollected: 0, alertsTriggered: 0, averageLatency: 0 },
            errorTracking: { errorsTracked: 0, criticalErrors: 0, resolvedErrors: 0 },
          },
          metrics: createDefaultMetrics(),
          sessions: {
            active: 0,
            total: 0,
            sessions: new Map(),
          },
        },
        plugins: {
          plugins: new Map(),
          states: new Map(),
          dependencies: new Map(),
        },
        eventQueue: [],
        meta: {
          initialized: false,
          version: '1.0.0',
          lastUpdate: 0,
        },
      });
    },

    destroy: async () => {
      // Stop all plugins
      await get().stop();

      // Destroy all plugins
      for (const name of get().plugins.plugins.keys()) {
        try {
          await get().unregisterPlugin(name);
        } catch (error) {
          logger.error(`Failed to destroy plugin: ${name}`, error as Error, {
            component: 'TrackerStore',
          });
        }
      }

      // Reset state
      get().reset();

      logger.info('Tracker destroyed', { component: 'TrackerStore' });
    },
  }))
);

// ============================================================================
// Public API
// ============================================================================

export const trackerStore = useTrackerStore.getState;

// Initialize event processing
if (typeof window !== 'undefined') {
  setInterval(() => {
    useTrackerStore.getState().processEvents();
  }, 5000); // Process events every 5 seconds
}

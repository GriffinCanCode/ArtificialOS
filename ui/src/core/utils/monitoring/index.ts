/**
 * Enhanced Logging System for AgentOS
 *
 * A production-grade, intelligent logging system specifically designed for
 * the AgentOS frontend. Provides hierarchical context tracking, causality
 * chain debugging, high-performance buffering, and seamless integration
 * with the existing architecture.
 *
 * @version 2.0.0
 * @author Griffin
 */

// ============================================================================
// Core Logging
// ============================================================================

export {
  logger,
  LogLevel,
  type LogContext
} from './logger';

export {
  useLogger,
  usePerformanceLogger,
  withLogging
} from './useLogger';

// ============================================================================
// Performance Monitoring
// ============================================================================

export {
  performanceMonitor,
  startPerf,
  endPerf,
  measurePerf,
  measurePerfSync,
  type PerformanceMetrics
} from './performanceMonitor';

// ============================================================================
// Hierarchical Context System
// ============================================================================

export {
  useHierarchicalContextStore,
  getHierarchicalLogContext,
  getCurrentBreadcrumbPath,
  setDesktopContext,
  setWindowContext,
  setAppContext,
  trackComponent,
  getHierarchicalContextStore,
  type HierarchicalContext,
  type ContextBreadcrumb
} from './hierarchicalContext';

export {
  WindowContextProvider,
  AppContextProvider,
  useComponentTracking,
  ContextBreadcrumbs,
  ContextDebugger,
  withComponentTracking,
  // Convenience aliases
  useComponentTracking as useTracking,
  WindowContextProvider as WindowContext,
  AppContextProvider as AppContext,
  withComponentTracking as withTracking
} from './HierarchicalContextProvider';

// ============================================================================
// High-Performance Log Buffer
// ============================================================================

export {
  logBuffer,
  LogBuffer,
  ConsoleProcessor,
  ElectronLogProcessor,
  BackendStreamProcessor,
  type LogEntry,
  type LogBufferConfig,
  type LogProcessor
} from './logBuffer';

// ============================================================================
// Environment Configuration
// ============================================================================

export {
  getLoggingConfig,
  createLoggingConfig,
  getCurrentEnvironment,
  isLogLevelEnabled,
  getConfigExamples,
  loggingConfig,
  type LoggingConfig,
  type Environment
} from './logConfig';

// ============================================================================
// Causality Chain Tracking
// ============================================================================

export {
  causalityTracker,
  startCausalChain,
  addCausalEvent,
  completeCausalEvent,
  endCurrentChain,
  getCausalityLogContext,
  withCausality,
  useCausality,
  type CausalityChain,
  type CausalEvent,
  type CausalEventType,
  type CausalityOptions
} from './causalityTracker';

// ============================================================================
// Journey Tracking System (NEW)
// ============================================================================

export {
  useJourneyStore,
  journeyStore,
} from './stores/journey';

export {
  useTrackerStore,
  trackerStore,
} from './stores/tracker';

export {
  useJourney,
  useInteractionTracking,
  useAsyncTracking,
  usePerformanceJourney,
  useTracker,
  useTrackerFeature,
  useTrackerPlugin,
  useTrackerHealth,
} from './hooks';

export {
  MonitorProvider,
  MonitoringStatus,
  useMonitor,
  withMonitoring,
  JourneyProvider,
  WindowJourneyProvider,
  AppJourneyProvider,
  FormJourneyProvider,
  useJourneyContext,
  withJourneyTracking,
} from './providers';

export type {
  Journey,
  JourneyStep,
  JourneyStepType,
  JourneyStepContext,
  JourneyConfig,
  JourneyAnalytics,
  TrackerConfig,
  TrackerPlugin,
} from './types';

export type {
  UseJourneyReturn,
} from './hooks/useJourney';

export type {
  UseTrackerReturn,
} from './hooks/useTracker';

export type {
  WindowJourneyProviderProps,
  AppJourneyProviderProps,
  FormJourneyProviderProps,
} from './providers/journey';

// Import items we need for functions below
import { logger } from './logger';
import { performanceMonitor, measurePerf } from './performanceMonitor';
import {
  causalityTracker,
  addCausalEvent,
  completeCausalEvent,
  startCausalChain,
  endCurrentChain,
  getCausalityLogContext,
  type CausalEvent,
  type CausalEventType
} from './causalityTracker';
import { getCurrentBreadcrumbPath, getHierarchicalLogContext, setDesktopContext } from './hierarchicalContext';
import { loggingConfig, type LoggingConfig } from './logConfig';
import { measurePerfSync } from './performanceMonitor';

// ============================================================================
// Convenience Exports & Utilities
// ============================================================================

/**
 * Quick setup for basic logging utilities
 * Note: This creates a logger child with component context
 */
export function createComponentLogger(componentName: string, additionalContext?: Record<string, unknown>) {
  // Create a child logger with component context
  const log = logger.child({ component: componentName, ...additionalContext });

  // Create causality functions (not hooks)
  const addEvent = (
    type: CausalEventType,
    description: string,
    metadata?: Partial<CausalEvent['metadata']>
  ) => {
    return addCausalEvent(type, description, { component: componentName }, metadata);
  };

  const trackUserAction = (action: string) => {
    return addEvent('user_action', action, { severity: 'high' });
  };

  return {
    // Standard logging methods
    info: log.info.bind(log),
    warn: log.warn.bind(log),
    error: log.error.bind(log),
    debug: log.debug.bind(log),
    verbose: log.verbose.bind(log),

    // Specialty logging methods
    performance: log.performance.bind(log),
    interaction: log.interaction.bind(log),
    api: log.api.bind(log),
    websocket: log.websocket.bind(log),

    // Causality tracking methods
    addCausalEvent: addEvent,
    trackUserAction,

    // Performance tracking
    measureAsync: async <T>(name: string, operation: () => Promise<T>): Promise<T> => {
      const eventId = addEvent('async_operation', name);
      try {
        const result = await measurePerf(name, operation);
        completeCausalEvent(eventId);
        return result;
      } catch (error) {
        completeCausalEvent(eventId, error as Error);
        throw error;
      }
    },

    // Convenience method for user interactions
    logUserInteraction: (action: string, target?: string, data?: Record<string, unknown>) => {
      const eventId = trackUserAction(action);
      log.interaction(action, target, { causalEventId: eventId, ...data });
      return eventId;
    }
  };
}

/**
 * Initialize the complete logging system with optimal settings
 */
export function initializeLoggingSystem(customConfig?: Partial<LoggingConfig>) {
  // The system auto-initializes, but this can be used for custom configuration
  if (customConfig) {
    // eslint-disable-next-line no-console
    console.info('Logging system initialized with custom configuration:', customConfig);
  }

  return {
    logger,
    performanceMonitor,
    causalityTracker,
    config: loggingConfig,
  };
}

/**
 * Debug utilities for development
 */
export const DEBUG = {
  /**
   * Get comprehensive logging statistics
   */
  async getStats() {
    const loggerStats = await logger.getStats();
    const perfStats = performanceMonitor.getStats('overall_performance') || {
      count: 0, min: 0, max: 0, avg: 0, p50: 0, p95: 0, p99: 0
    };

    return {
      logger: loggerStats,
      performance: perfStats,
      hierarchical: {
        currentPath: getCurrentBreadcrumbPath(),
        context: getHierarchicalLogContext(),
      },
      causality: {
        currentChainId: causalityTracker.getCurrentChainId(),
        totalChains: causalityTracker.getChains().length,
        context: getCausalityLogContext(),
      },
    };
  },

  /**
   * Export all causality chains for analysis
   */
  exportCausalityData() {
    const chains = causalityTracker.getChains();
    return chains.map(chain => causalityTracker.exportChain(chain.id));
  },

  /**
   * Generate performance report
   */
  getPerformanceReport() {
    return performanceMonitor.getReport();
  },

  /**
   * Enable debug mode (shows breadcrumbs and debugger)
   */
  enableDebugMode() {
    setDesktopContext({ environment: 'development' });
    // eslint-disable-next-line no-console
    console.info('Debug mode enabled. Press Ctrl/Cmd + Shift + L to toggle context debugger');
  },

  /**
   * Test logging system
   */
  test() {
    // eslint-disable-next-line no-console
    console.group('🧪 Testing AgentOS Logging System');

    // Test basic logging
    logger.info('Test log message', { test: true });

    // Test causality
    const chainId = startCausalChain('user_action', 'Test causality chain');
    addCausalEvent('system_event', 'Test event in chain');
    endCurrentChain();

    // Test performance
    measurePerfSync('test_operation', () => {
      // Simulate work
      for (let i = 0; i < 1000; i++) {
        Math.random();
      }
    });

    logger.info('Logging system test completed', {
      chainId,
      breadcrumbPath: getCurrentBreadcrumbPath()
    });

    // eslint-disable-next-line no-console
    console.groupEnd();
  }
};

// ============================================================================
// Global Development Helpers
// ============================================================================

if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
  // Expose debug utilities globally in development
  (window as { agentOSLogging?: unknown }).agentOSLogging = {
    logger,
    performanceMonitor,
    causalityTracker,
    DEBUG,

    // Quick access functions
    log: (message: string, context?: Record<string, unknown>) => logger.info(message, context),
    startChain: startCausalChain,
    addEvent: addCausalEvent,
    getStats: DEBUG.getStats,
    test: DEBUG.test,
  };

  // eslint-disable-next-line no-console
  console.info('🚀 AgentOS Logging System v2.0.0 loaded');
  // eslint-disable-next-line no-console
  console.info('Access via window.agentOSLogging in development');
  // eslint-disable-next-line no-console
  console.info('Use DEBUG.test() to run system test');
}

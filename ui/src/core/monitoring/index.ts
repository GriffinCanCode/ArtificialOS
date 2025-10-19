/**
 * AgentOS Monitoring System v2.0
 *
 * Unified, production-grade monitoring for the AgentOS frontend.
 * Provides logging, metrics, journey tracking, causality chains,
 * and hierarchical context tracking in one cohesive system.
 *
 * @module monitoring
 * @author Griffin
 * @version 2.0.0
 */

// ============================================================================
// Core Infrastructure
// ============================================================================

export * from './core';

// ============================================================================
// React Integration
// ============================================================================

export * from './hooks';

// ============================================================================
// Journey Tracking
// ============================================================================

export * from './journey';

// ============================================================================
// System Tracker
// ============================================================================

export * from './tracker';

// ============================================================================
// Causality Tracking
// ============================================================================

export * from './causality';

// ============================================================================
// Hierarchical Context
// ============================================================================

export * from './context';

// ============================================================================
// Visualization & Dashboard
// ============================================================================

export * from './visualization';

// ============================================================================
// Convenience Utilities
// ============================================================================

import { logger } from './core/logger';
import { performanceMonitor, measurePerf } from './core/performance';
import { causalityTracker, addCausalEvent, completeCausalEvent, startCausalChain, endCurrentChain, getCausalityLogContext, type CausalEvent, type CausalEventType } from './causality';
import { getCurrentBreadcrumbPath, getHierarchicalLogContext, setDesktopContext } from './context';
import { loggingConfig, type LoggingConfig } from './core/config';
import { measurePerfSync } from './core/performance';

/**
 * Create a component-scoped logger with causality tracking
 *
 * @param componentName - Name of the component
 * @param additionalContext - Additional context to include in all logs
 * @returns Scoped logger with causality helpers
 */
export function createComponentLogger(componentName: string, additionalContext?: Record<string, unknown>) {
  const log = logger.child({ component: componentName, ...additionalContext });

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
    // Standard logging
    info: log.info.bind(log),
    warn: log.warn.bind(log),
    error: log.error.bind(log),
    debug: log.debug.bind(log),
    verbose: log.verbose.bind(log),

    // Specialty logging
    performance: log.performance.bind(log),
    interaction: log.interaction.bind(log),
    api: log.api.bind(log),
    websocket: log.websocket.bind(log),

    // Causality tracking
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

    // User interaction helper
    logUserInteraction: (action: string, target?: string, data?: Record<string, unknown>) => {
      const eventId = trackUserAction(action);
      log.interaction(action, target, { causalEventId: eventId, ...data });
      return eventId;
    }
  };
}

/**
 * Initialize monitoring system with custom configuration
 *
 * @param customConfig - Optional custom configuration
 * @returns Monitoring system handles
 */
export function initializeLoggingSystem(customConfig?: Partial<LoggingConfig>) {
  if (customConfig) {
    // eslint-disable-next-line no-console
    console.info('🔍 Monitoring system initialized with custom config:', customConfig);
  }

  return {
    logger,
    performanceMonitor,
    causalityTracker,
    config: loggingConfig,
  };
}

/**
 * Development debugging utilities
 */
export const DEBUG = {
  /**
   * Get comprehensive monitoring statistics
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
   * Export causality chains for analysis
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
   * Enable debug mode with visualizations
   */
  enableDebugMode() {
    setDesktopContext({ environment: 'development' });
    // eslint-disable-next-line no-console
    console.info('🐛 Debug mode enabled. Press Ctrl/Cmd + Shift + L for context debugger');
  },

  /**
   * Test monitoring system
   */
  test() {
    // eslint-disable-next-line no-console
    console.group('🧪 Testing AgentOS Monitoring System');

    logger.info('Test log message', { test: true });

    const chainId = startCausalChain('user_action', 'Test causality chain');
    addCausalEvent('system_event', 'Test event in chain');
    endCurrentChain();

    measurePerfSync('test_operation', () => {
      for (let i = 0; i < 1000; i++) {
        Math.random();
      }
    });

    logger.info('Monitoring system test completed', {
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
  (window as { agentOSMonitoring?: unknown }).agentOSMonitoring = {
    logger,
    performanceMonitor,
    causalityTracker,
    DEBUG,

    // Quick access
    log: (message: string, context?: Record<string, unknown>) => logger.info(message, context),
    startChain: startCausalChain,
    addEvent: addCausalEvent,
    getStats: DEBUG.getStats,
    test: DEBUG.test,
  };

  // eslint-disable-next-line no-console
  console.info(
    '%c🚀 AgentOS Monitoring v2.0.0',
    'font-size: 14px; font-weight: bold; color: #00d4ff;'
  );
  // eslint-disable-next-line no-console
  console.info(
    '%cAccess via window.agentOSMonitoring | DEBUG.test() to run tests',
    'color: #888;'
  );
}


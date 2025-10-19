/**
 * Monitor Provider
 *
 * Top-level provider for the monitoring system orchestration.
 * Follows exact patterns from WebSocketContext.tsx with lifecycle management.
 */

import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { useTracker } from '../hooks/useTracker';
import { useJourneyStore } from '../journey/store';
import type { TrackerConfig } from './types';
import { logger } from '../core/logger';
import { setDesktopContext } from '../context/store';

// ============================================================================
// Context Interface
// ============================================================================

interface MonitorContextType {
  /** Tracker status and controls */
  tracker: {
    isRunning: boolean;
    health: {
      score: number;
      overall: string;
    };
    start: () => Promise<void>;
    stop: () => Promise<void>;
    restart: () => Promise<void>;
  };

  /** Journey tracking controls */
  journey: {
    isTracking: boolean;
    currentJourneyId: string | null;
    startJourney: (description: string) => string;
    completeJourney: () => void;
  };

  /** System metrics */
  metrics: {
    eventsPerSecond: number;
    memoryUsageMB: number;
    errorRate: number;
  };

  /** Configuration */
  config: TrackerConfig;
}

// ============================================================================
// Context Creation
// ============================================================================

// Internal context - not exported to maintain Fast Refresh compatibility
const MonitorContext = createContext<MonitorContextType>({
  tracker: {
    isRunning: false,
    health: { score: 0, overall: 'stopped' },
    start: async () => {},
    stop: async () => {},
    restart: async () => {},
  },
  journey: {
    isTracking: false,
    currentJourneyId: null,
    startJourney: () => '',
    completeJourney: () => {},
  },
  metrics: {
    eventsPerSecond: 0,
    memoryUsageMB: 0,
    errorRate: 0,
  },
  config: {} as TrackerConfig,
});

export const useMonitor = () => {
  const context = useContext(MonitorContext);
  if (!context) {
    throw new Error('useMonitor must be used within MonitorProvider');
  }
  return context;
};

// ============================================================================
// Provider Interface
// ============================================================================

interface MonitorProviderProps {
  children: ReactNode;

  /** Initial tracker configuration */
  config?: Partial<TrackerConfig>;

  /** Whether to auto-start tracker on mount */
  autoStart?: boolean;

  /** Desktop context for hierarchical tracking */
  desktopContext?: {
    userId?: string;
    sessionId?: string;
    environment?: 'development' | 'production';
  };
}

// ============================================================================
// Provider Implementation
// ============================================================================

export const MonitorProvider: React.FC<MonitorProviderProps> = ({
  children,
  config: initialConfig,
  autoStart = true,
  desktopContext,
}) => {
  // Initialize tracker with configuration
  const tracker = useTracker(true, initialConfig);

  // Journey store selectors
  const journeyId = useJourneyStore((state) => state.activeJourneyId);
  const isJourneyTracking = useJourneyStore((state) => state.state.isTracking);
  const { startJourney: startJourneyAction, completeJourney: completeJourneyAction } = useJourneyStore();

  useEffect(() => {
    logger.info('MonitorProvider mounted, initializing monitoring system', {
      component: 'MonitorProvider',
      autoStart,
      hasConfig: !!initialConfig,
    });

    // Set desktop context for hierarchical tracking
    if (desktopContext) {
      setDesktopContext(desktopContext);
      logger.debug('Desktop context set', {
        component: 'MonitorProvider',
        context: desktopContext,
      });
    }

    // Auto-start tracker if enabled and not already running
    if (autoStart && !tracker.status.isRunning && tracker.utils.isInitialized) {
      tracker.actions.start().catch((error) => {
        logger.error('Failed to auto-start tracker', error, {
          component: 'MonitorProvider',
        });
      });
    }

    // Cleanup on unmount - only in production
    return () => {
      if (process.env.NODE_ENV === 'production') {
        logger.info('MonitorProvider unmounting (production), stopping tracker', {
          component: 'MonitorProvider',
          wasRunning: tracker.status.isRunning,
        });

        if (tracker.status.isRunning) {
          tracker.actions.stop().catch((error) => {
            logger.error('Failed to stop tracker on unmount', error, {
              component: 'MonitorProvider',
            });
          });
        }
      } else {
        logger.debug('MonitorProvider cleanup (dev mode), keeping tracker alive for StrictMode', {
          component: 'MonitorProvider',
          wasRunning: tracker.status.isRunning,
        });
        // Don't stop in development - React StrictMode causes false unmounts
      }
    };
  }, [tracker, autoStart, desktopContext, initialConfig]);

  // Log status changes
  useEffect(() => {
    if (tracker.status.isRunning) {
      logger.info('Monitoring system running', {
        component: 'MonitorProvider',
        health: tracker.health,
        features: tracker.features,
      });
    } else if (tracker.status.hasError) {
      logger.error('Monitoring system error', undefined, {
        component: 'MonitorProvider',
        error: tracker.status.errorMessage,
      });
    }
  }, [tracker.status.isRunning, tracker.status.hasError, tracker.status.errorMessage, tracker.health, tracker.features]);

  // Health monitoring
  useEffect(() => {
    if (tracker.health.score < 50) {
      logger.warn('Monitoring system health degraded', {
        component: 'MonitorProvider',
        health: tracker.health,
        metrics: tracker.metrics,
      });
    }
  }, [tracker.health.score, tracker.health, tracker.metrics]);

  // Memoized context value
  const contextValue = React.useMemo<MonitorContextType>(() => ({
    tracker: {
      isRunning: tracker.status.isRunning,
      health: {
        score: tracker.health.score,
        overall: tracker.health.overall,
      },
      start: tracker.actions.start,
      stop: tracker.actions.stop,
      restart: tracker.actions.restart,
    },
    journey: {
      isTracking: isJourneyTracking,
      currentJourneyId: journeyId,
      startJourney: (description: string) => {
        return startJourneyAction(description, {
          component: 'MonitorProvider',
          sessionSource: 'global',
        });
      },
      completeJourney: () => {
        if (journeyId) {
          completeJourneyAction(journeyId, 'completed');
        }
      },
    },
    metrics: {
      eventsPerSecond: tracker.metrics.eventsPerSecond,
      memoryUsageMB: tracker.metrics.memoryUsageMB,
      errorRate: tracker.metrics.errorRate,
    },
    config: tracker.config,
  }), [
    tracker.status.isRunning,
    tracker.health,
    tracker.actions,
    tracker.metrics,
    tracker.config,
    isJourneyTracking,
    journeyId,
    startJourneyAction,
    completeJourneyAction,
  ]);

  return (
    <MonitorContext.Provider value={contextValue}>
      {children}
    </MonitorContext.Provider>
  );
};

// ============================================================================
// Higher-Order Component
// ============================================================================

/**
 * HOC to wrap components with monitoring provider
 * Useful for testing and isolated component development
 */
export function withMonitoring<P extends object>(
  Component: React.ComponentType<P>,
  config?: Partial<TrackerConfig>
): React.ComponentType<P> {
  const WrappedComponent: React.FC<P> = (props) => {
    return (
      <MonitorProvider config={config}>
        <Component {...props} />
      </MonitorProvider>
    );
  };

  WrappedComponent.displayName = `withMonitoring(${Component.displayName || Component.name})`;

  return WrappedComponent;
}

// ============================================================================
// Development Utilities
// ============================================================================

/**
 * Development component for monitoring system status
 * Only renders in development mode
 */
export const MonitoringStatus: React.FC = () => {
  const monitor = useMonitor();

  if (process.env.NODE_ENV !== 'development') {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 z-[9999] bg-black/90 text-white text-xs p-3 rounded-lg max-w-sm font-mono">
      <div className="flex items-center justify-between mb-2">
        <h3 className="font-bold text-sm">Monitor Status</h3>
        <div
          className={`w-2 h-2 rounded-full ${
            monitor.tracker.isRunning ? 'bg-green-400' : 'bg-red-400'
          }`}
        />
      </div>

      <div className="space-y-1 text-xs">
        <div className="flex justify-between">
          <span>Health:</span>
          <span className={`${
            monitor.tracker.health.score >= 80 ? 'text-green-400' :
            monitor.tracker.health.score >= 50 ? 'text-yellow-400' : 'text-red-400'
          }`}>
            {monitor.tracker.health.score}/100
          </span>
        </div>

        <div className="flex justify-between">
          <span>Journey:</span>
          <span className={monitor.journey.isTracking ? 'text-green-400' : 'text-gray-400'}>
            {monitor.journey.currentJourneyId ? 'Active' : 'None'}
          </span>
        </div>

        <div className="flex justify-between">
          <span>Events/sec:</span>
          <span>{monitor.metrics.eventsPerSecond.toFixed(1)}</span>
        </div>

        <div className="flex justify-between">
          <span>Memory:</span>
          <span className={monitor.metrics.memoryUsageMB > 50 ? 'text-yellow-400' : 'text-gray-300'}>
            {monitor.metrics.memoryUsageMB.toFixed(0)}MB
          </span>
        </div>

        <div className="flex justify-between">
          <span>Errors:</span>
          <span className={monitor.metrics.errorRate > 0.05 ? 'text-red-400' : 'text-gray-300'}>
            {(monitor.metrics.errorRate * 100).toFixed(1)}%
          </span>
        </div>
      </div>

      <div className="text-xs text-gray-500 mt-2 pt-2 border-t border-gray-700">
        Press Ctrl/Cmd + Shift + M to toggle
      </div>
    </div>
  );
};

// ============================================================================
// Global Development Helpers
// ============================================================================

if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
  // Simple logging for development
  // eslint-disable-next-line no-console
  console.info('🔍 AgentOS Monitoring System v1.0.0 loaded');
}

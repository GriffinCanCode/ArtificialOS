/**
 * Journey Tracking Hook
 *
 * React hook for component-level journey tracking integration.
 * Follows exact patterns from useLogger.ts with composition-based API.
 */

import { useEffect, useCallback, useMemo } from 'react';
import { useJourneyStore } from '../stores/journey';
import type { Journey, JourneyStepType, JourneyStepContext } from '../types/journey';
import { logger } from '../logger';
import { addCausalEvent } from '../causalityTracker';

// ============================================================================
// Hook Interface
// ============================================================================

export interface UseJourneyReturn {
  /** Current journey ID */
  journeyId: string | null;

  /** Journey tracking status */
  isTracking: boolean;

  /** Start a new journey */
  startJourney: (description: string, context?: JourneyStepContext) => string;

  /** Add step to current journey */
  addStep: (type: JourneyStepType, description: string, context?: JourneyStepContext) => string;

  /** Complete current journey */
  completeJourney: (outcome?: string) => void;

  /** Abandon current journey */
  abandonJourney: (reason: string) => void;

  /** Track user interaction automatically */
  trackInteraction: (element: string, action: string, coordinates?: { x: number; y: number }) => string;

  /** Track navigation event */
  trackNavigation: (from: string, to: string, trigger?: string) => string;

  /** Track system response */
  trackResponse: (operation: string, duration: number, success: boolean) => string;

  /** Track error occurrence */
  trackError: (error: Error, context?: JourneyStepContext) => string;

  /** Get current journey info */
  getCurrentJourney: () => Journey | undefined;

  /** Journey analytics */
  analytics: {
    stepCount: number;
    duration: number;
    errorCount: number;
    lastActivity: number;
  };
}

// ============================================================================
// Main Hook
// ============================================================================

/**
 * Hook for journey tracking in components
 *
 * @param componentName - Name of the component using journey tracking
 * @param autoStart - Whether to automatically start a journey on mount
 * @param journeyDescription - Description for auto-started journey
 *
 * @example
 * const journey = useJourney('ChatInterface', true, 'User opened chat');
 *
 * const handleSendMessage = () => {
 *   const stepId = journey.trackInteraction('send-button', 'click');
 *   sendMessage().then(() => {
 *     journey.trackResponse('send_message', Date.now() - startTime, true);
 *   });
 * };
 */
export function useJourney(
  componentName: string,
  autoStart: boolean = false,
  journeyDescription?: string
): UseJourneyReturn {
  // Store selectors
  const journeyId = useJourneyStore((state) => state.activeJourneyId);
  const isTracking = useJourneyStore((state) => state.state.isTracking);
  const config = useJourneyStore((state) => state.config);

  // Store actions
  const {
    startJourney: startJourneyAction,
    addStep,
    completeJourney: completeJourneyAction,
    abandonJourney: abandonJourneyAction,
    getCurrentJourney,
  } = useJourneyStore();

  // Auto-start journey on mount if enabled
  useEffect(() => {
    if (autoStart && !journeyId && isTracking && config.enabled) {
      const description = journeyDescription || `User opened ${componentName}`;
      startJourneyAction(description, { componentId: componentName, componentType: 'react_component' });

      logger.debug(`Auto-started journey for ${componentName}`, {
        component: 'useJourney',
        componentName,
        description,
      });
    }
  }, [autoStart, journeyId, isTracking, config.enabled, componentName, journeyDescription, startJourneyAction]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (autoStart && journeyId) {
        completeJourneyAction(journeyId, 'completed');
        logger.debug(`Auto-completed journey on ${componentName} unmount`, {
          component: 'useJourney',
          componentName,
          journeyId,
        });
      }
    };
  }, [autoStart, journeyId, componentName, completeJourneyAction]);

  // Memoized journey actions
  const startJourney = useCallback((description: string, context?: JourneyStepContext) => {
    const fullContext: JourneyStepContext = {
      ...context,
      componentId: componentName,
      componentType: 'react_component',
    };

    // Link with causality tracking
    const causalEventId = addCausalEvent('user_action', description, fullContext);
    fullContext.causalEventId = causalEventId;

    return startJourneyAction(description, fullContext);
  }, [componentName, startJourneyAction]);

  const addJourneyStep = useCallback((
    type: JourneyStepType,
    description: string,
    context?: JourneyStepContext
  ) => {
    const fullContext: JourneyStepContext = {
      ...context,
      componentId: componentName,
      componentType: 'react_component',
    };

    // Link with causality tracking
    const causalEventId = addCausalEvent(
      type === 'user_action' ? 'user_action' : 'system_event',
      description,
      fullContext
    );
    fullContext.causalEventId = causalEventId;

    return addStep(type, description, fullContext);
  }, [componentName, addStep]);

  const completeJourney = useCallback((outcome = 'completed') => {
    if (journeyId) {
      completeJourneyAction(journeyId, outcome);
      logger.debug(`Journey completed from ${componentName}`, {
        component: 'useJourney',
        componentName,
        journeyId,
        outcome,
      });
    }
  }, [journeyId, componentName, completeJourneyAction]);

  const abandonJourney = useCallback((reason: string) => {
    if (journeyId) {
      abandonJourneyAction(reason, journeyId);
      logger.debug(`Journey abandoned from ${componentName}`, {
        component: 'useJourney',
        componentName,
        journeyId,
        reason,
      });
    }
  }, [journeyId, componentName, abandonJourneyAction]);

  // Specialized tracking functions
  const trackInteraction = useCallback((
    element: string,
    action: string,
    coordinates?: { x: number; y: number }
  ) => {
    return addJourneyStep('user_action', `${action} on ${element}`, {
      interaction: {
        element,
        coordinates,
      },
    });
  }, [addJourneyStep]);

  const trackNavigation = useCallback((from: string, to: string, trigger?: string) => {
    return addJourneyStep('navigation', `Navigate from ${from} to ${to}`, {
      navigation: {
        from,
        to,
        trigger,
      },
    });
  }, [addJourneyStep]);

  const trackSystemResponse = useCallback((
    operation: string,
    duration: number,
    success: boolean
  ) => {
    return addJourneyStep('system_response', `${operation} ${success ? 'succeeded' : 'failed'}`, {
      systemResponse: {
        operation,
        duration,
        success,
      },
    });
  }, [addJourneyStep]);

  const trackError = useCallback((error: Error, context?: JourneyStepContext) => {
    return addJourneyStep('error', `Error: ${error.message}`, {
      ...context,
      error: {
        name: error.name,
        message: error.message,
        stack: error.stack,
      },
    });
  }, [addJourneyStep]);

  // Calculate analytics
  const analytics = useMemo(() => {
    const currentJourney = getCurrentJourney();

    if (!currentJourney) {
      return {
        stepCount: 0,
        duration: 0,
        errorCount: 0,
        lastActivity: 0,
      };
    }

    const errorCount = currentJourney.steps.filter(step => step.context.error).length;
    const duration = currentJourney.meta.duration || 0;
    const lastStep = currentJourney.steps[currentJourney.steps.length - 1];

    return {
      stepCount: currentJourney.steps.length,
      duration,
      errorCount,
      lastActivity: lastStep?.timing.timestamp || currentJourney.meta.startTime,
    };
  }, [getCurrentJourney]);

  return {
    journeyId,
    isTracking: isTracking && config.enabled,
    startJourney,
    addStep: addJourneyStep,
    completeJourney,
    abandonJourney,
    trackInteraction,
    trackNavigation,
    trackResponse: trackSystemResponse,
    trackError,
    getCurrentJourney,
    analytics,
  };
}

// ============================================================================
// Specialized Hooks
// ============================================================================

/**
 * Hook for automatic user interaction tracking
 * Tracks clicks, form submissions, and keyboard events
 */
export function useInteractionTracking(componentName: string) {
  const journey = useJourney(componentName);

  const trackClick = useCallback((event: React.MouseEvent, _description?: string) => {
    const target = event.currentTarget as HTMLElement;
    const element = target.tagName.toLowerCase() +
      (target.id ? `#${target.id}` : '') +
      (target.className ? `.${target.className.split(' ')[0]}` : '');

    return journey.trackInteraction(
      element,
      'click',
      { x: event.clientX, y: event.clientY }
    );
  }, [journey]);

  const trackFormSubmit = useCallback((event: React.FormEvent, formName?: string) => {
    const form = event.currentTarget as HTMLFormElement;
    const formId = formName || form.id || form.name || 'unknown-form';

    return journey.trackInteraction(formId, 'submit');
  }, [journey]);

  const trackKeyPress = useCallback((event: React.KeyboardEvent, description?: string) => {
    const key = event.key;
    const desc = description || `Key press: ${key}`;

    return journey.trackInteraction('keyboard', desc);
  }, [journey]);

  return {
    ...journey,
    trackClick,
    trackFormSubmit,
    trackKeyPress,
  };
}

/**
 * Hook for async operation tracking
 * Automatically tracks async operations with performance metrics
 */
export function useAsyncTracking(componentName: string) {
  const journey = useJourney(componentName);

  const trackAsync = useCallback(async <T>(
    operation: string,
    asyncFn: () => Promise<T>,
    context?: JourneyStepContext
  ): Promise<T> => {
    const startTime = performance.now();
    journey.addStep('system_response', `Starting ${operation}`, context);

    try {
      const result = await asyncFn();
      const duration = performance.now() - startTime;

      journey.trackResponse(operation, duration, true);

      logger.debug(`Async operation completed: ${operation}`, {
        component: 'useAsyncTracking',
        operation,
        duration,
      });

      return result;
    } catch (error) {
      const duration = performance.now() - startTime;

      journey.trackResponse(operation, duration, false);
      journey.trackError(error as Error, context);

      logger.error(`Async operation failed: ${operation}`, error as Error, {
        component: 'useAsyncTracking',
        operation,
        duration,
      });

      throw error;
    }
  }, [journey]);

  return {
    ...journey,
    trackAsync,
  };
}

/**
 * Hook for performance-aware journey tracking
 * Automatically detects and reports performance issues
 */
export function usePerformanceJourney(componentName: string, thresholds = { slow: 100, critical: 500 }) {
  const journey = useJourney(componentName);

  const trackWithPerformance = useCallback((
    type: JourneyStepType,
    description: string,
    operation: () => void | Promise<void>,
    context?: JourneyStepContext
  ) => {
    const startTime = performance.now();
    journey.addStep(type, description, context);

    const finish = (error?: Error) => {
      const duration = performance.now() - startTime;

      let performanceLevel: 'normal' | 'slow' | 'critical' = 'normal';
      if (duration > thresholds.critical) {
        performanceLevel = 'critical';
      } else if (duration > thresholds.slow) {
        performanceLevel = 'slow';
      }

      if (performanceLevel !== 'normal') {
        journey.addStep('performance', `${description} was ${performanceLevel} (${duration}ms)`, {
          ...context,
          performance: {
            duration,
            level: performanceLevel,
            threshold: performanceLevel === 'critical' ? thresholds.critical : thresholds.slow,
          },
        });
      }

      if (error) {
        journey.trackError(error, context);
      }
    };

    try {
      const result = operation();

      if (result instanceof Promise) {
        return result.then(
          (value) => { finish(); return value; },
          (error) => { finish(error); throw error; }
        );
      } else {
        finish();
        return result;
      }
    } catch (error) {
      finish(error as Error);
      throw error;
    }
  }, [journey, thresholds]);

  return {
    ...journey,
    trackWithPerformance,
  };
}

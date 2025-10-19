/**
 * Journey Provider
 *
 * Specialized provider for journey tracking within specific contexts.
 * Designed to be nested within MonitorProvider for focused journey management.
 */

import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { useJourney } from '../hooks/useJourney';
import type { JourneyStepType, JourneyStepContext } from './types';
import { logger } from '../core/logger';

// ============================================================================
// Context Interface
// ============================================================================

interface JourneyContextType {
  /** Current journey information */
  journey: {
    id: string | null;
    isActive: boolean;
    stepCount: number;
    duration: number;
    errorCount: number;
  };

  /** Journey actions */
  actions: {
    start: (description: string, context?: JourneyStepContext) => string;
    addStep: (type: JourneyStepType, description: string, context?: JourneyStepContext) => string;
    complete: (outcome?: string) => void;
    abandon: (reason: string) => void;
  };

  /** Specialized tracking methods */
  track: {
    interaction: (element: string, action: string, coordinates?: { x: number; y: number }) => string;
    navigation: (from: string, to: string, trigger?: string) => string;
    response: (operation: string, duration: number, success: boolean) => string;
    error: (error: Error, context?: JourneyStepContext) => string;
  };

  /** Journey analytics */
  analytics: {
    stepCount: number;
    duration: number;
    errorCount: number;
    lastActivity: number;
  };
}

// ============================================================================
// Context Creation
// ============================================================================

// Internal context - not exported to maintain Fast Refresh compatibility
const JourneyContext = createContext<JourneyContextType | null>(null);

export const useJourneyContext = () => {
  const context = useContext(JourneyContext);
  if (!context) {
    throw new Error('useJourneyContext must be used within JourneyProvider');
  }
  return context;
};

// ============================================================================
// Provider Interface
// ============================================================================

interface JourneyProviderProps {
  children: ReactNode;

  /** Context name for this journey provider */
  contextName: string;

  /** Whether to auto-start a journey on mount */
  autoStart?: boolean;

  /** Description for auto-started journey */
  autoStartDescription?: string;

  /** Whether to auto-complete journey on unmount */
  autoComplete?: boolean;

  /** Additional context to include in all journey steps */
  contextData?: JourneyStepContext;
}

// ============================================================================
// Provider Implementation
// ============================================================================

export const JourneyProvider: React.FC<JourneyProviderProps> = ({
  children,
  contextName,
  autoStart = false,
  autoStartDescription,
  autoComplete = true,
  contextData,
}) => {
  // Initialize journey hook with component context
  const journey = useJourney(
    contextName,
    autoStart,
    autoStartDescription || `Journey started in ${contextName}`
  );

  useEffect(() => {
    logger.debug(`JourneyProvider mounted for ${contextName}`, {
      component: 'JourneyProvider',
      contextName,
      autoStart,
      hasContextData: !!contextData,
    });

    // Cleanup on unmount
    return () => {
      if (autoComplete && journey.journeyId) {
        journey.completeJourney('completed');
        logger.debug(`Auto-completed journey on ${contextName} unmount`, {
          component: 'JourneyProvider',
          contextName,
          journeyId: journey.journeyId,
        });
      }
    };
  }, [contextName, autoStart, autoComplete, journey, contextData]);

  // Memoized context value
  const contextValue = React.useMemo<JourneyContextType>(() => ({
    journey: {
      id: journey.journeyId,
      isActive: !!journey.journeyId && journey.isTracking,
      stepCount: journey.analytics.stepCount,
      duration: journey.analytics.duration,
      errorCount: journey.analytics.errorCount,
    },

    actions: {
      start: (description: string, context?: JourneyStepContext) => {
        const fullContext = { ...contextData, ...context };
        return journey.startJourney(description, fullContext);
      },

      addStep: (type: JourneyStepType, description: string, context?: JourneyStepContext) => {
        const fullContext = { ...contextData, ...context };
        return journey.addStep(type, description, fullContext);
      },

      complete: (outcome = 'completed') => {
        journey.completeJourney(outcome);
        logger.info(`Journey completed in ${contextName}`, {
          component: 'JourneyProvider',
          contextName,
          journeyId: journey.journeyId,
          outcome,
          analytics: journey.analytics,
        });
      },

      abandon: (reason: string) => {
        journey.abandonJourney(reason);
        logger.info(`Journey abandoned in ${contextName}`, {
          component: 'JourneyProvider',
          contextName,
          journeyId: journey.journeyId,
          reason,
          analytics: journey.analytics,
        });
      },
    },

    track: {
      interaction: (element: string, action: string, coordinates?: { x: number; y: number }) => {
        return journey.trackInteraction(element, action, coordinates);
      },

      navigation: (from: string, to: string, trigger?: string) => {
        return journey.trackNavigation(from, to, trigger);
      },

      response: (operation: string, duration: number, success: boolean) => {
        return journey.trackResponse(operation, duration, success);
      },

      error: (error: Error, context?: JourneyStepContext) => {
        const fullContext = { ...contextData, ...context, contextName };
        return journey.trackError(error, fullContext);
      },
    },

    analytics: journey.analytics,
  }), [journey, contextName, contextData]);

  return (
    <JourneyContext.Provider value={contextValue}>
      {children}
    </JourneyContext.Provider>
  );
};

// ============================================================================
// Higher-Order Components
// ============================================================================

/**
 * HOC to wrap components with journey tracking
 * Automatically tracks component lifecycle as journey steps
 */
export function withJourneyTracking<P extends object>(
  Component: React.ComponentType<P>,
  contextName: string,
  options: {
    autoStart?: boolean;
    autoStartDescription?: string;
    trackLifecycle?: boolean;
  } = {}
): React.ComponentType<P> {
  const WrappedComponent: React.FC<P> = (props) => {
    const { autoStart = true, autoStartDescription, trackLifecycle = true } = options;

    return (
      <JourneyProvider
        contextName={contextName}
        autoStart={autoStart}
        autoStartDescription={autoStartDescription || `User interacting with ${contextName}`}
      >
        {trackLifecycle ? (
          <LifecycleTracker componentName={contextName}>
            <Component {...props} />
          </LifecycleTracker>
        ) : (
          <Component {...props} />
        )}
      </JourneyProvider>
    );
  };

  WrappedComponent.displayName = `withJourneyTracking(${Component.displayName || Component.name})`;

  return WrappedComponent;
}

/**
 * Component to automatically track lifecycle events
 */
const LifecycleTracker: React.FC<{ componentName: string; children: ReactNode }> = ({
  componentName,
  children,
}) => {
  const journey = useJourneyContext();

  useEffect(() => {
    // Track component mount
    const mountStepId = journey.actions.addStep(
      'system_response',
      `${componentName} component mounted`,
      { lifecycle: 'mount' }
    );

    // Track component render
    const renderStepId = journey.actions.addStep(
      'ui_update',
      `${componentName} component rendered`,
      { lifecycle: 'render' }
    );

    logger.debug(`Component lifecycle tracked: ${componentName}`, {
      component: 'LifecycleTracker',
      componentName,
      mountStepId,
      renderStepId,
    });

    // Track component unmount
    return () => {
      journey.actions.addStep(
        'system_response',
        `${componentName} component unmounted`,
        { lifecycle: 'unmount' }
      );
    };
  }, [componentName, journey.actions]);

  return <>{children}</>;
};

// ============================================================================
// Specialized Journey Providers
// ============================================================================

/**
 * Provider for window-specific journey tracking
 * Integrates with window context and lifecycle
 */
export interface WindowJourneyProviderProps {
  children: ReactNode;
  windowId: string;
  windowTitle: string;
  appId: string;
}

export const WindowJourneyProvider: React.FC<WindowJourneyProviderProps> = ({
  children,
  windowId,
  windowTitle,
  appId,
}) => {
  return (
    <JourneyProvider
      contextName={`Window-${windowTitle}`}
      autoStart={true}
      autoStartDescription={`User opened window: ${windowTitle}`}
      contextData={{
        windowId,
        windowTitle,
        appId,
        contextType: 'window',
      }}
    >
      {children}
    </JourneyProvider>
  );
};

/**
 * Provider for app-specific journey tracking
 * Tracks user interactions within specific applications
 */
export interface AppJourneyProviderProps {
  children: ReactNode;
  appId: string;
  appType: 'blueprint' | 'native_web' | 'native_proc';
  instanceId?: string;
}

export const AppJourneyProvider: React.FC<AppJourneyProviderProps> = ({
  children,
  appId,
  appType,
  instanceId,
}) => {
  return (
    <JourneyProvider
      contextName={`App-${appId}`}
      autoStart={true}
      autoStartDescription={`User started using app: ${appId}`}
      contextData={{
        appId,
        appType,
        instanceId,
        contextType: 'app',
      }}
    >
      {children}
    </JourneyProvider>
  );
};

/**
 * Provider for form-specific journey tracking
 * Specialized for tracking form interactions and submissions
 */
export interface FormJourneyProviderProps {
  children: ReactNode;
  formName: string;
  formId?: string;
}

export const FormJourneyProvider: React.FC<FormJourneyProviderProps> = ({
  children,
  formName,
  formId,
}) => {
  return (
    <JourneyProvider
      contextName={`Form-${formName}`}
      autoStart={true}
      autoStartDescription={`User started filling form: ${formName}`}
      contextData={{
        formName,
        formId,
        contextType: 'form',
      }}
    >
      {children}
    </JourneyProvider>
  );
};

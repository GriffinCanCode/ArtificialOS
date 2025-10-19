/**
 * Journey Store
 *
 * Zustand store for intelligent journey tracking state management.
 * Follows exact patterns from hierarchicalContext.ts with subscribeWithSelector.
 */

import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type {
  Journey,
  JourneyStep,
  JourneyStepType,
  JourneyStepContext,
  JourneyConfig,
  JourneyAnalytics,
  WindowJourney,
} from './types';
import { logger } from '../core/logger';
import { getCausalityLogContext } from '../causality/tracker';
import { getHierarchicalLogContext } from '../context/store';

// ============================================================================
// Store Interface
// ============================================================================

interface JourneyStore {
  /** Active journeys */
  journeys: Map<string, Journey>;

  /** Current active journey ID */
  activeJourneyId: string | null;

  /** Journey configuration */
  config: JourneyConfig;

  /** Analytics cache */
  analytics: JourneyAnalytics | null;

  /** Store state */
  state: {
    initialized: boolean;
    isTracking: boolean;
    lastAnalyticsUpdate: number;
  };

  // ============================================================================
  // Journey Management Actions
  // ============================================================================

  /** Start a new journey */
  startJourney: (description: string, context?: JourneyStepContext) => string;

  /** Add step to current journey */
  addStep: (
    type: JourneyStepType,
    description: string,
    context?: JourneyStepContext,
    journeyId?: string
  ) => string;

  /** Complete current journey */
  completeJourney: (journeyId?: string, outcome?: string) => void;

  /** Abandon current journey */
  abandonJourney: (reason: string, journeyId?: string) => void;

  /** Add window to journey */
  trackWindow: (windowId: string, windowMeta: WindowJourney['meta']) => void;

  /** Update step with completion data */
  completeStep: (stepId: string, duration?: number, error?: Error) => void;

  // ============================================================================
  // Configuration Actions
  // ============================================================================

  /** Update journey configuration */
  updateConfig: (config: Partial<JourneyConfig>) => void;

  /** Enable/disable journey tracking */
  setTracking: (enabled: boolean) => void;

  // ============================================================================
  // Analytics Actions
  // ============================================================================

  /** Get journey analytics */
  getAnalytics: () => JourneyAnalytics;

  /** Get journey by ID */
  getJourney: (journeyId: string) => Journey | undefined;

  /** Get current journey */
  getCurrentJourney: () => Journey | undefined;

  /** Get journeys matching criteria */
  queryJourneys: (filter?: JourneyFilter) => Journey[];

  // ============================================================================
  // Utility Actions
  // ============================================================================

  /** Cleanup expired journeys */
  cleanup: () => void;

  /** Export journey data */
  export: (journeyId?: string) => JourneyExport;

  /** Reset store state */
  reset: () => void;

  /** Initialize store */
  initialize: (config?: Partial<JourneyConfig>) => void;
}

interface JourneyFilter {
  pattern?: string;
  outcome?: string;
  timeRange?: { start: number; end: number };
  minDuration?: number;
  maxDuration?: number;
  hasErrors?: boolean;
  windowId?: string;
  appId?: string;
}

interface JourneyExport {
  journey?: Journey;
  journeys?: Journey[];
  analytics: JourneyAnalytics;
  exportedAt: number;
  version: string;
}

// ============================================================================
// Default Configuration
// ============================================================================

const defaultConfig: JourneyConfig = {
  enabled: true,
  granularity: 'detailed',
  limits: {
    maxJourneys: 100,
    maxJourneyDuration: 30 * 60 * 1000, // 30 minutes
    maxStepsPerJourney: 200,
  },
  autoComplete: {
    inactivityTimeout: 5, // 5 minutes
    errorThreshold: 10,
    complexityThreshold: 0.8,
  },
  analysis: {
    enablePatternDetection: true,
    enablePerformanceCorrelation: true,
    enablePredictiveAnalysis: false,
    enableABTesting: false,
  },
  privacy: {
    anonymizeUserData: process.env.NODE_ENV === 'production',
    excludeInputValues: true,
    excludePersonalInfo: true,
  },
};

// ============================================================================
// Store Implementation
// ============================================================================

export const useJourneyStore = create<JourneyStore>()(
  subscribeWithSelector((set, get) => ({
    journeys: new Map(),
    activeJourneyId: null,
    config: defaultConfig,
    analytics: null,
    state: {
      initialized: false,
      isTracking: false,
      lastAnalyticsUpdate: 0,
    },

    startJourney: (description: string, context?: JourneyStepContext) => {
      const state = get();
      if (!state.config.enabled || !state.state.isTracking) {
        return '';
      }

      const journeyId = generateJourneyId();
      const now = performance.now();

      // Get current context
      const hierarchicalContext = getHierarchicalLogContext();
      const causalityContext = getCausalityLogContext();

      const fullContext: JourneyStepContext = {
        ...hierarchicalContext,
        ...context,
        systemState: {
          activeWindows: state.journeys.size,
          memoryUsage: (performance as any).memory?.usedJSHeapSize,
        },
      };

      const initialStep: JourneyStep = {
        id: generateStepId(),
        sequence: 0,
        timing: {
          timestamp: now,
          relativeTime: 0,
        },
        type: 'user_action',
        description,
        context: fullContext,
        causalEventId: typeof causalityContext.causalityEventId === 'string'
          ? causalityContext.causalityEventId
          : undefined,
        relations: {},
      };

      const journey: Journey = {
        id: journeyId,
        meta: {
          startTime: now,
          sessionId: hierarchicalContext.sessionId || 'unknown',
          environment: (hierarchicalContext.environment as 'development' | 'production') || 'development',
          userId: typeof hierarchicalContext.userId === 'string' ? hierarchicalContext.userId : undefined,
        },
        steps: [initialStep],
        windows: new Map(),
        performance: {
          overall: {
            totalDuration: 0,
            activeTime: 0,
            waitTime: 0,
            errorTime: 0,
          },
          bottlenecks: [],
          trends: {
            improvingSteps: [],
            degradingSteps: [],
          },
        },
        classification: {
          pattern: 'unknown',
          outcome: 'ongoing',
          experience: 'neutral',
          complexity: 0,
          efficiency: 0,
          tags: [],
        },
      };

      set((state) => ({
        journeys: new Map(state.journeys).set(journeyId, journey),
        activeJourneyId: journeyId,
      }));

      logger.debug('Journey started', {
        component: 'JourneyStore',
        journeyId,
        description,
        stepId: initialStep.id,
      });

      return journeyId;
    },

    addStep: (
      type: JourneyStepType,
      description: string,
      context?: JourneyStepContext,
      journeyId?: string
    ) => {
      const state = get();
      const targetJourneyId = journeyId || state.activeJourneyId;

      if (!targetJourneyId || !state.config.enabled) {
        return '';
      }

      const journey = state.journeys.get(targetJourneyId);
      if (!journey) {
        logger.warn('Journey not found for step', {
          component: 'JourneyStore',
          journeyId: targetJourneyId,
        });
        return '';
      }

      // Check step limit
      if (journey.steps.length >= state.config.limits.maxStepsPerJourney) {
        logger.warn('Journey step limit exceeded', {
          component: 'JourneyStore',
          journeyId: targetJourneyId,
          stepCount: journey.steps.length,
          limit: state.config.limits.maxStepsPerJourney,
        });
        return '';
      }

      const now = performance.now();
      const stepId = generateStepId();

      // Get current context
      const hierarchicalContext = getHierarchicalLogContext();
      const causalityContext = getCausalityLogContext();

      const fullContext: JourneyStepContext = {
        ...hierarchicalContext,
        ...context,
        systemState: {
          activeWindows: state.journeys.size,
          memoryUsage: (performance as any).memory?.usedJSHeapSize,
        },
      };

      const step: JourneyStep = {
        id: stepId,
        sequence: journey.steps.length,
        timing: {
          timestamp: now,
          relativeTime: now - journey.meta.startTime,
        },
        type,
        description,
        context: fullContext,
        causalEventId: typeof causalityContext.causalityEventId === 'string'
          ? causalityContext.causalityEventId
          : undefined,
        relations: {},
      };

      // Link to previous step
      if (journey.steps.length > 0) {
        const previousStep = journey.steps[journey.steps.length - 1];
        step.relations.triggeredBy = previousStep.id;
        previousStep.relations.triggers = previousStep.relations.triggers || [];
        previousStep.relations.triggers.push(stepId);
      }

      const updatedJourney: Journey = {
        ...journey,
        steps: [...journey.steps, step],
      };

      set((state) => ({
        journeys: new Map(state.journeys).set(targetJourneyId, updatedJourney),
      }));

      logger.debug('Journey step added', {
        component: 'JourneyStore',
        journeyId: targetJourneyId,
        stepId,
        type,
        description,
        sequence: step.sequence,
      });

      return stepId;
    },

    completeJourney: (journeyId?: string, outcome = 'completed') => {
      const state = get();
      const targetJourneyId = journeyId || state.activeJourneyId;

      if (!targetJourneyId) return;

      const journey = state.journeys.get(targetJourneyId);
      if (!journey) return;

      const now = performance.now();
      const duration = now - journey.meta.startTime;

      const completedJourney: Journey = {
        ...journey,
        meta: {
          ...journey.meta,
          endTime: now,
          duration,
        },
        classification: {
          ...journey.classification,
          outcome: outcome as any,
        },
      };

      set((state) => ({
        journeys: new Map(state.journeys).set(targetJourneyId, completedJourney),
        activeJourneyId: state.activeJourneyId === targetJourneyId ? null : state.activeJourneyId,
      }));

      logger.info('Journey completed', {
        component: 'JourneyStore',
        journeyId: targetJourneyId,
        outcome,
        duration,
        stepCount: journey.steps.length,
      });

      // Cleanup if needed
      get().cleanup();
    },

    abandonJourney: (reason: string, journeyId?: string) => {
      get().completeJourney(journeyId, 'abandoned');

      logger.info('Journey abandoned', {
        component: 'JourneyStore',
        journeyId: journeyId || get().activeJourneyId,
        reason,
      });
    },

    trackWindow: (windowId: string, windowMeta: WindowJourney['meta']) => {
      const state = get();
      if (!state.activeJourneyId) return;

      const journey = state.journeys.get(state.activeJourneyId);
      if (!journey) return;

      const windowJourney: WindowJourney = {
        windowId,
        meta: windowMeta,
        steps: [],
        metrics: {
          focusTime: 0,
          interactionCount: 0,
          errorCount: 0,
          performanceIssues: 0,
        },
        relationships: {},
      };

      const updatedJourney: Journey = {
        ...journey,
        windows: new Map(journey.windows).set(windowId, windowJourney),
      };

      set((state) => ({
        journeys: new Map(state.journeys).set(state.activeJourneyId!, updatedJourney),
      }));
    },

    completeStep: (stepId: string, duration?: number, error?: Error) => {
      const state = get();

      for (const [journeyId, journey] of state.journeys) {
        const stepIndex = journey.steps.findIndex(s => s.id === stepId);
        if (stepIndex !== -1) {
          const step = journey.steps[stepIndex];
          const now = performance.now();

          const updatedStep: JourneyStep = {
            ...step,
            timing: {
              ...step.timing,
              duration: duration || (now - step.timing.timestamp),
            },
            metrics: {
              duration: duration || (now - step.timing.timestamp),
              performanceImpact: error ? 'high' : 'none',
            },
          };

          if (error) {
            updatedStep.context = {
              ...updatedStep.context,
              error: {
                name: error.name,
                message: error.message,
                stack: error.stack,
              },
            };
          }

          const updatedSteps = [...journey.steps];
          updatedSteps[stepIndex] = updatedStep;

          const updatedJourney: Journey = {
            ...journey,
            steps: updatedSteps,
          };

          set((state) => ({
            journeys: new Map(state.journeys).set(journeyId, updatedJourney),
          }));

          break;
        }
      }
    },

    updateConfig: (config: Partial<JourneyConfig>) => {
      set((state) => ({
        config: { ...state.config, ...config },
      }));
    },

    setTracking: (enabled: boolean) => {
      set((state) => ({
        state: { ...state.state, isTracking: enabled },
      }));
    },

    getAnalytics: (): JourneyAnalytics => {
      const state = get();
      const journeys = Array.from(state.journeys.values());

      // Calculate basic stats
      const completedJourneys = journeys.filter(j => j.classification.outcome !== 'ongoing');
      const totalDuration = completedJourneys.reduce((sum, j) => sum + (j.meta.duration || 0), 0);
      const errorCount = journeys.reduce((sum, j) =>
        sum + j.steps.filter(s => s.context.error).length, 0
      );

      return {
        stats: {
          totalJourneys: journeys.length,
          averageDuration: completedJourneys.length > 0 ? totalDuration / completedJourneys.length : 0,
          completionRate: journeys.length > 0 ? completedJourneys.length / journeys.length : 0,
          errorRate: journeys.length > 0 ? errorCount / journeys.length : 0,
          abandonmentRate: journeys.length > 0 ?
            journeys.filter(j => j.classification.outcome === 'abandoned').length / journeys.length : 0,
        },
        patterns: {
          mostCommon: 'single_task',
          distribution: {
            single_task: 0.6,
            multi_window: 0.2,
            exploration: 0.1,
            creation: 0.05,
            collaboration: 0.02,
            troubleshooting: 0.02,
            workflow: 0.01,
            unknown: 0,
          },
          trends: [],
        },
        performance: {
          averageSteps: journeys.length > 0 ?
            journeys.reduce((sum, j) => sum + j.steps.length, 0) / journeys.length : 0,
          bottleneckFrequency: {},
          improvementOpportunities: [],
        },
        experience: {
          averageRating: 3.5,
          commonPainPoints: [],
          successPatterns: [],
        },
      };
    },

    getJourney: (journeyId: string) => {
      return get().journeys.get(journeyId);
    },

    getCurrentJourney: () => {
      const state = get();
      return state.activeJourneyId ? state.journeys.get(state.activeJourneyId) : undefined;
    },

    queryJourneys: (filter?: JourneyFilter) => {
      const journeys = Array.from(get().journeys.values());

      if (!filter) return journeys;

      return journeys.filter(journey => {
        if (filter.pattern && journey.classification.pattern !== filter.pattern) {
          return false;
        }
        if (filter.outcome && journey.classification.outcome !== filter.outcome) {
          return false;
        }
        if (filter.timeRange) {
          const startTime = journey.meta.startTime;
          if (startTime < filter.timeRange.start || startTime > filter.timeRange.end) {
            return false;
          }
        }
        if (filter.minDuration && (journey.meta.duration || 0) < filter.minDuration) {
          return false;
        }
        if (filter.maxDuration && (journey.meta.duration || 0) > filter.maxDuration) {
          return false;
        }
        if (filter.hasErrors !== undefined) {
          const hasErrors = journey.steps.some(s => s.context.error);
          if (hasErrors !== filter.hasErrors) {
            return false;
          }
        }
        return true;
      });
    },

    cleanup: () => {
      const state = get();
      const now = performance.now();
      const maxAge = state.config.limits.maxJourneyDuration;
      const maxJourneys = state.config.limits.maxJourneys;

      // Remove expired journeys
      const validJourneys = new Map<string, Journey>();

      for (const [id, journey] of state.journeys) {
        const age = now - journey.meta.startTime;
        if (age <= maxAge && validJourneys.size < maxJourneys) {
          validJourneys.set(id, journey);
        }
      }

      if (validJourneys.size !== state.journeys.size) {
        set({ journeys: validJourneys });

        logger.debug('Journey cleanup completed', {
          component: 'JourneyStore',
          removed: state.journeys.size - validJourneys.size,
          remaining: validJourneys.size,
        });
      }
    },

    export: (journeyId?: string): JourneyExport => {
      const state = get();

      if (journeyId) {
        const journey = state.journeys.get(journeyId);
        return {
          journey,
          analytics: get().getAnalytics(),
          exportedAt: Date.now(),
          version: '1.0.0',
        };
      }

      return {
        journeys: Array.from(state.journeys.values()),
        analytics: get().getAnalytics(),
        exportedAt: Date.now(),
        version: '1.0.0',
      };
    },

    reset: () => {
      set({
        journeys: new Map(),
        activeJourneyId: null,
        analytics: null,
        state: {
          initialized: false,
          isTracking: false,
          lastAnalyticsUpdate: 0,
        },
      });
    },

    initialize: (config?: Partial<JourneyConfig>) => {
      set((state) => ({
        config: { ...state.config, ...config },
        state: {
          ...state.state,
          initialized: true,
          isTracking: true,
        },
      }));

      logger.info('Journey store initialized', {
        component: 'JourneyStore',
        config: get().config,
      });
    },
  }))
);

// ============================================================================
// Helper Functions
// ============================================================================

function generateJourneyId(): string {
  return `journey_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

function generateStepId(): string {
  return `step_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

// ============================================================================
// Public API
// ============================================================================

export const journeyStore = useJourneyStore.getState;

// Initialize cleanup timer
if (typeof window !== 'undefined') {
  setInterval(() => {
    useJourneyStore.getState().cleanup();
  }, 60000); // Cleanup every minute
}

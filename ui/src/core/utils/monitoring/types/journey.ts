/**
 * Journey Tracking Types
 *
 * Defines types for intelligent user journey tracking across windows/apps.
 * Enables debugging capabilities for multi-window workflows.
 */

import type { LogContext } from '../logger';
import type { CausalEvent } from '../causalityTracker';

// ============================================================================
// Core Journey Types
// ============================================================================

export interface Journey {
  /** Unique journey identifier */
  id: string;

  /** Journey metadata */
  meta: {
    startTime: number;
    endTime?: number;
    duration?: number;
    userId?: string;
    sessionId: string;
    environment: 'development' | 'production';
  };

  /** Journey steps (chronological) */
  steps: JourneyStep[];

  /** Cross-window relationships */
  windows: Map<string, WindowJourney>;

  /** Performance correlation */
  performance: JourneyPerformance;

  /** Journey classification */
  classification: JourneyClassification;
}

export interface JourneyStep {
  /** Unique step identifier */
  id: string;

  /** Step sequence number */
  sequence: number;

  /** Step timing */
  timing: {
    timestamp: number;
    duration?: number;
    relativeTime: number; // ms since journey start
  };

  /** Step type classification */
  type: JourneyStepType;

  /** Human-readable description */
  description: string;

  /** Context at time of step */
  context: JourneyStepContext;

  /** Associated causality event */
  causalEventId?: string;

  /** Performance metrics for this step */
  metrics?: StepMetrics;

  /** Cross-references to related steps */
  relations: {
    triggers?: string[]; // Steps this one triggered
    triggeredBy?: string; // Step that triggered this one
    correlates?: string[]; // Correlated steps
  };
}

export type JourneyStepType =
  | 'user_action'      // Click, type, drag, etc.
  | 'navigation'       // Window open/close, app switch
  | 'system_response'  // API response, state change
  | 'ui_update'        // Re-render, layout change
  | 'performance'      // Slow operation, lag event
  | 'error'           // Error occurrence
  | 'recovery'        // Error recovery action
  | 'completion';     // Task completion

export interface JourneyStepContext {
  /** Window context */
  windowId?: string;
  windowTitle?: string;
  windowFocused?: boolean;

  /** App context */
  appId?: string;
  appType?: 'blueprint' | 'native_web' | 'native_proc';

  /** Component context */
  componentId?: string;
  componentType?: string;

  /** User interaction details */
  interaction?: {
    element?: string;
    coordinates?: { x: number; y: number };
    modifierKeys?: string[];
    inputValue?: string;
  };

  /** System state */
  systemState?: {
    activeWindows: number;
    memoryUsage?: number;
    cpuUsage?: number;
  };

  /** Additional context */
  [key: string]: unknown;
}

export interface StepMetrics {
  /** Duration of this step */
  duration: number;

  /** Performance impact */
  performanceImpact: 'none' | 'low' | 'medium' | 'high';

  /** Resource usage */
  resources?: {
    memory?: number;
    cpu?: number;
    network?: number;
  };
}

// ============================================================================
// Window Journey Tracking
// ============================================================================

export interface WindowJourney {
  /** Window identifier */
  windowId: string;

  /** Window metadata */
  meta: {
    title: string;
    appId: string;
    appType: 'blueprint' | 'native_web' | 'native_proc';
    openedAt: number;
    closedAt?: number;
    totalTime?: number;
  };

  /** Steps within this window */
  steps: string[]; // Step IDs

  /** Window-specific metrics */
  metrics: {
    focusTime: number;       // Total focused time
    interactionCount: number; // Total interactions
    errorCount: number;      // Errors in this window
    performanceIssues: number; // Performance problems
  };

  /** Cross-window relationships */
  relationships: {
    spawnedBy?: string;      // Window that opened this one
    spawned?: string[];      // Windows this one opened
    communicatedWith?: string[]; // Windows this one interacted with
  };
}

// ============================================================================
// Performance Correlation
// ============================================================================

export interface JourneyPerformance {
  /** Overall journey performance */
  overall: {
    totalDuration: number;
    activeTime: number;      // Time user was actively interacting
    waitTime: number;        // Time user was waiting for system
    errorTime: number;       // Time spent dealing with errors
  };

  /** Performance bottlenecks */
  bottlenecks: PerformanceBottleneck[];

  /** Performance trends */
  trends: {
    improvingSteps: string[]; // Steps getting faster over time
    degradingSteps: string[]; // Steps getting slower over time
  };
}

export interface PerformanceBottleneck {
  /** Step causing bottleneck */
  stepId: string;

  /** Bottleneck type */
  type: 'network' | 'cpu' | 'memory' | 'render' | 'api' | 'unknown';

  /** Impact severity */
  severity: 'low' | 'medium' | 'high' | 'critical';

  /** Duration of bottleneck */
  duration: number;

  /** Suggested optimizations */
  suggestions?: string[];
}

// ============================================================================
// Journey Classification
// ============================================================================

export interface JourneyClassification {
  /** Primary journey pattern */
  pattern: JourneyPattern;

  /** Journey outcome */
  outcome: JourneyOutcome;

  /** User experience rating */
  experience: JourneyExperience;

  /** Complexity score (0-1) */
  complexity: number;

  /** Efficiency score (0-1) */
  efficiency: number;

  /** Tags for categorization */
  tags: string[];
}

export type JourneyPattern =
  | 'single_task'        // Simple single-window task
  | 'multi_window'       // Task spanning multiple windows
  | 'exploration'        // User exploring/browsing
  | 'creation'          // User creating something
  | 'collaboration'     // Multi-user interaction
  | 'troubleshooting'   // User solving a problem
  | 'workflow'          // Repeated process
  | 'unknown';

export type JourneyOutcome =
  | 'completed'         // Successfully completed
  | 'abandoned'         // User gave up
  | 'interrupted'       // External interruption
  | 'error_terminated'  // Ended due to error
  | 'ongoing'          // Still in progress
  | 'unknown';

export type JourneyExperience =
  | 'excellent'         // Smooth, fast, intuitive
  | 'good'             // Generally positive
  | 'neutral'          // Neither good nor bad
  | 'poor'             // Frustrating but usable
  | 'terrible';        // Highly frustrating

// ============================================================================
// Journey Analytics
// ============================================================================

export interface JourneyAnalytics {
  /** Journey statistics */
  stats: {
    totalJourneys: number;
    averageDuration: number;
    completionRate: number;
    errorRate: number;
    abandonmentRate: number;
  };

  /** Pattern analysis */
  patterns: {
    mostCommon: JourneyPattern;
    distribution: Record<JourneyPattern, number>;
    trends: PatternTrend[];
  };

  /** Performance insights */
  performance: {
    averageSteps: number;
    bottleneckFrequency: Record<string, number>;
    improvementOpportunities: string[];
  };

  /** User experience insights */
  experience: {
    averageRating: number;
    commonPainPoints: string[];
    successPatterns: string[];
  };
}

export interface PatternTrend {
  pattern: JourneyPattern;
  trend: 'increasing' | 'decreasing' | 'stable';
  changeRate: number; // Percentage change
  significance: 'low' | 'medium' | 'high';
}

// ============================================================================
// Journey Configuration
// ============================================================================

export interface JourneyConfig {
  /** Enable/disable journey tracking */
  enabled: boolean;

  /** Tracking granularity */
  granularity: 'basic' | 'detailed' | 'comprehensive';

  /** Memory limits */
  limits: {
    maxJourneys: number;        // Max journeys to keep in memory
    maxJourneyDuration: number; // Max duration before auto-completion
    maxStepsPerJourney: number; // Max steps per journey
  };

  /** Auto-completion rules */
  autoComplete: {
    inactivityTimeout: number;  // Minutes of inactivity
    errorThreshold: number;     // Errors before auto-completion
    complexityThreshold: number; // Complexity score threshold
  };

  /** Analysis configuration */
  analysis: {
    enablePatternDetection: boolean;
    enablePerformanceCorrelation: boolean;
    enablePredictiveAnalysis: boolean;
    enableABTesting: boolean;
  };

  /** Privacy settings */
  privacy: {
    anonymizeUserData: boolean;
    excludeInputValues: boolean;
    excludePersonalInfo: boolean;
  };
}

// ============================================================================
// Journey Events
// ============================================================================

export interface JourneyEvent {
  /** Event type */
  type: JourneyEventType;

  /** Journey ID */
  journeyId: string;

  /** Event payload */
  payload: unknown;

  /** Event timestamp */
  timestamp: number;
}

export type JourneyEventType =
  | 'journey_started'
  | 'journey_completed'
  | 'journey_abandoned'
  | 'step_added'
  | 'pattern_detected'
  | 'bottleneck_identified'
  | 'anomaly_detected';

// ============================================================================
// Integration Types
// ============================================================================

export interface JourneyIntegration {
  /** Causality tracker integration */
  causality: {
    enabled: boolean;
    autoLinkEvents: boolean;
    trackCrossWindowCausality: boolean;
  };

  /** Performance monitor integration */
  performance: {
    enabled: boolean;
    autoCorrelateBottlenecks: boolean;
    trackResourceUsage: boolean;
  };

  /** Logger integration */
  logging: {
    enabled: boolean;
    logLevel: 'basic' | 'detailed' | 'comprehensive';
    includeStepContext: boolean;
  };
}

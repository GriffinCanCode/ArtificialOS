/**
 * Tracker Configuration Types
 *
 * Defines types for the monitoring system configuration and orchestration.
 * Enables flexible, extensible monitoring architecture.
 */

import type { LogLevel } from '../logger';
import type { JourneyConfig } from './journey';

// ============================================================================
// Core Tracker Types
// ============================================================================

export interface TrackerConfig {
  /** Overall monitoring state */
  enabled: boolean;

  /** Environment configuration */
  environment: 'development' | 'production' | 'testing';

  /** Feature toggles */
  features: TrackerFeatures;

  /** Performance settings */
  performance: TrackerPerformance;

  /** Integration settings */
  integrations: TrackerIntegrations;

  /** Privacy and compliance */
  privacy: TrackerPrivacy;
}

export interface TrackerFeatures {
  /** Journey tracking */
  journeyTracking: boolean;

  /** Causality chain tracking */
  causalityTracking: boolean;

  /** Performance monitoring */
  performanceMonitoring: boolean;

  /** Error tracking */
  errorTracking: boolean;

  /** User behavior analytics */
  behaviorAnalytics: boolean;

  /** A/B testing framework */
  abTesting: boolean;

  /** Real-time debugging */
  realtimeDebugging: boolean;

  /** Predictive analysis */
  predictiveAnalysis: boolean;
}

export interface TrackerPerformance {
  /** Sampling rates (0-1) */
  sampling: {
    journeys: number;         // Journey sampling rate
    performance: number;      // Performance monitoring rate
    errors: number;          // Error tracking rate
    debugging: number;       // Debug event rate
  };

  /** Buffer sizes */
  buffers: {
    maxEvents: number;       // Max events in buffer
    maxJourneys: number;     // Max journeys in memory
    flushInterval: number;   // Buffer flush interval (ms)
  };

  /** Resource limits */
  limits: {
    maxMemoryMB: number;     // Max memory usage
    maxCpuPercent: number;   // Max CPU usage
    maxNetworkKBps: number;  // Max network usage
  };

  /** Optimization settings */
  optimization: {
    enableDeduplication: boolean;
    enableCompression: boolean;
    enableBatching: boolean;
    batchSize: number;
  };
}

export interface TrackerIntegrations {
  /** Logger integration */
  logger: {
    enabled: boolean;
    level: LogLevel;
    includeContext: boolean;
    includeStackTrace: boolean;
  };

  /** External services */
  external: {
    analytics?: AnalyticsIntegration;
    errorReporting?: ErrorReportingIntegration;
    performance?: PerformanceIntegration;
  };

  /** Backend integration */
  backend: {
    enabled: boolean;
    endpoint?: string;
    apiKey?: string;
    batchSize: number;
    retryAttempts: number;
  };

  /** Real-time features */
  realtime: {
    enabled: boolean;
    websocketUrl?: string;
    broadcastChannel?: string;
  };
}

export interface TrackerPrivacy {
  /** Data anonymization */
  anonymization: {
    enabled: boolean;
    anonymizeIPs: boolean;
    anonymizeUserIds: boolean;
    hashSensitiveData: boolean;
  };

  /** Data retention */
  retention: {
    journeyDays: number;      // Days to keep journey data
    performanceDays: number;  // Days to keep performance data
    errorDays: number;        // Days to keep error data
    debugDays: number;        // Days to keep debug data
  };

  /** Compliance */
  compliance: {
    gdprCompliant: boolean;
    ccpaCompliant: boolean;
    excludePII: boolean;
    consentRequired: boolean;
  };

  /** User controls */
  userControls: {
    allowOptOut: boolean;
    allowDataExport: boolean;
    allowDataDeletion: boolean;
    showPrivacyNotice: boolean;
  };
}

// ============================================================================
// External Service Integrations
// ============================================================================

export interface AnalyticsIntegration {
  provider: 'google' | 'amplitude' | 'mixpanel' | 'segment' | 'custom';
  apiKey: string;
  trackPageViews: boolean;
  trackEvents: boolean;
  trackUserProperties: boolean;
  customConfig?: Record<string, unknown>;
}

export interface ErrorReportingIntegration {
  provider: 'sentry' | 'rollbar' | 'bugsnag' | 'custom';
  dsn: string;
  environment: string;
  includeSourceMaps: boolean;
  includeUserContext: boolean;
  customConfig?: Record<string, unknown>;
}

export interface PerformanceIntegration {
  provider: 'newrelic' | 'datadog' | 'pingdom' | 'custom';
  apiKey: string;
  trackWebVitals: boolean;
  trackCustomMetrics: boolean;
  customConfig?: Record<string, unknown>;
}

// ============================================================================
// Tracker State
// ============================================================================

export interface TrackerState {
  /** Current configuration */
  config: TrackerConfig;

  /** Runtime status */
  status: TrackerStatus;

  /** Feature states */
  features: TrackerFeatureStates;

  /** Performance metrics */
  metrics: TrackerMetrics;

  /** Active sessions */
  sessions: TrackerSessions;
}

export interface TrackerStatus {
  /** Overall tracker status */
  overall: 'initializing' | 'running' | 'paused' | 'error' | 'stopped';

  /** Individual feature statuses */
  features: Record<keyof TrackerFeatures, FeatureStatus>;

  /** Last update timestamp */
  lastUpdate: number;

  /** Error information */
  error?: {
    message: string;
    stack?: string;
    timestamp: number;
  };
}

export interface FeatureStatus {
  status: 'enabled' | 'disabled' | 'error' | 'paused';
  lastActive?: number;
  errorCount: number;
  successCount: number;
}

export interface TrackerFeatureStates {
  /** Journey tracking state */
  journeyTracking: {
    activeJourneys: number;
    totalJourneys: number;
    averageDuration: number;
  };

  /** Causality tracking state */
  causalityTracking: {
    activeChains: number;
    totalChains: number;
    averageChainLength: number;
  };

  /** Performance monitoring state */
  performanceMonitoring: {
    metricsCollected: number;
    alertsTriggered: number;
    averageLatency: number;
  };

  /** Error tracking state */
  errorTracking: {
    errorsTracked: number;
    criticalErrors: number;
    resolvedErrors: number;
  };
}

export interface TrackerMetrics {
  /** System resource usage */
  resources: {
    memoryUsageMB: number;
    cpuUsagePercent: number;
    networkUsageKBps: number;
  };

  /** Performance metrics */
  performance: {
    eventProcessingLatency: number;
    bufferUtilization: number;
    batchProcessingTime: number;
  };

  /** Data volume metrics */
  volume: {
    eventsPerSecond: number;
    dataPointsPerSecond: number;
    bytesPerSecond: number;
  };

  /** Quality metrics */
  quality: {
    successRate: number;
    errorRate: number;
    droppedEventRate: number;
  };
}

export interface TrackerSessions {
  /** Active session count */
  active: number;

  /** Total session count */
  total: number;

  /** Session details */
  sessions: Map<string, SessionInfo>;
}

export interface SessionInfo {
  sessionId: string;
  userId?: string;
  startTime: number;
  lastActivity: number;
  journeyCount: number;
  errorCount: number;
  deviceInfo?: {
    userAgent: string;
    platform: string;
    screen: { width: number; height: number };
  };
}

// ============================================================================
// Tracker Events
// ============================================================================

export interface TrackerEvent {
  /** Event type */
  type: TrackerEventType;

  /** Event payload */
  payload: unknown;

  /** Event metadata */
  meta: {
    timestamp: number;
    sessionId: string;
    userId?: string;
    source: string;
  };
}

export type TrackerEventType =
  | 'tracker_initialized'
  | 'tracker_started'
  | 'tracker_stopped'
  | 'tracker_error'
  | 'feature_enabled'
  | 'feature_disabled'
  | 'config_updated'
  | 'session_started'
  | 'session_ended'
  | 'resource_limit_exceeded'
  | 'performance_degraded'
  | 'data_exported'
  | 'privacy_consent_given'
  | 'privacy_consent_revoked';

// ============================================================================
// Plugin Architecture
// ============================================================================

export interface TrackerPlugin {
  /** Plugin metadata */
  meta: {
    name: string;
    version: string;
    description: string;
    author: string;
  };

  /** Plugin configuration */
  config?: Record<string, unknown>;

  /** Plugin lifecycle hooks */
  hooks: {
    onInitialize?: () => Promise<void> | void;
    onStart?: () => Promise<void> | void;
    onStop?: () => Promise<void> | void;
    onDestroy?: () => Promise<void> | void;
    onEvent?: (event: TrackerEvent) => Promise<void> | void;
    onError?: (error: Error) => Promise<void> | void;
  };

  /** Plugin API */
  api?: Record<string, (...args: any[]) => any>;
}

export interface TrackerPluginRegistry {
  /** Registered plugins */
  plugins: Map<string, TrackerPlugin>;

  /** Plugin states */
  states: Map<string, PluginState>;

  /** Plugin dependencies */
  dependencies: Map<string, string[]>;
}

export interface PluginState {
  status: 'loaded' | 'initialized' | 'running' | 'stopped' | 'error';
  error?: Error;
  lastActivity?: number;
  metrics?: Record<string, number>;
}

// ============================================================================
// Advanced Features
// ============================================================================

export interface ABTestConfig {
  /** Test identifier */
  testId: string;

  /** Test variants */
  variants: ABTestVariant[];

  /** Traffic allocation */
  allocation: Record<string, number>;

  /** Test duration */
  duration: {
    startDate: Date;
    endDate: Date;
  };

  /** Success metrics */
  metrics: string[];

  /** Target audience */
  audience?: {
    userSegments?: string[];
    deviceTypes?: string[];
    geographies?: string[];
  };
}

export interface ABTestVariant {
  id: string;
  name: string;
  description: string;
  config: Record<string, unknown>;
  weight: number;
}

export interface PredictiveConfig {
  /** Enable predictive features */
  enabled: boolean;

  /** Prediction models */
  models: {
    userBehavior: boolean;
    performanceAnomalies: boolean;
    errorPrediction: boolean;
    churnPrediction: boolean;
  };

  /** Model configuration */
  modelConfig: {
    trainingDataDays: number;
    predictionHorizonHours: number;
    confidenceThreshold: number;
    updateFrequencyHours: number;
  };
}

// ============================================================================
// Type Guards and Utilities
// ============================================================================

export function isTrackerEvent(obj: unknown): obj is TrackerEvent {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'type' in obj &&
    'payload' in obj &&
    'meta' in obj
  );
}

export function isValidTrackerConfig(config: unknown): config is TrackerConfig {
  return (
    typeof config === 'object' &&
    config !== null &&
    'enabled' in config &&
    'environment' in config &&
    'features' in config
  );
}

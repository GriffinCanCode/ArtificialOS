/**
 * Environment-based Logging Configuration
 *
 * Provides intelligent defaults based on environment with the ability to override
 * via environment variables or runtime configuration.
 */

import type { LogLevel } from './logger';
import type { LogBufferConfig } from './buffer';

// ============================================================================
// Configuration Types
// ============================================================================

export interface LoggingConfig {
  /** Minimum log level to output */
  level: LogLevel;

  /** Whether to enable console output */
  enableConsole: boolean;

  /** Whether to enable file output (Electron only) */
  enableFile: boolean;

  /** Whether to stream logs to backend */
  enableBackendStream: boolean;

  /** Whether to show hierarchical context breadcrumbs in dev */
  showBreadcrumbs: boolean;

  /** Whether to enable the context debugger */
  enableDebugger: boolean;

  /** Log buffer configuration */
  buffer: LogBufferConfig;

  /** Custom log processors to add */
  customProcessors: string[];

  /** Performance monitoring settings */
  performance: {
    enableMetrics: boolean;
    logSlowOperations: boolean;
    slowThresholdMs: number;
    sampleRate: number; // 0.0 to 1.0
  };
}

export type Environment = 'development' | 'production' | 'test';

// ============================================================================
// Default Configurations
// ============================================================================

const DEVELOPMENT_CONFIG: LoggingConfig = {
  level: 'verbose' as LogLevel,
  enableConsole: true,
  enableFile: true,
  enableBackendStream: false, // Don't spam backend in dev
  showBreadcrumbs: true,
  enableDebugger: true,

  buffer: {
    maxBufferSize: 500,       // Smaller buffer in dev
    maxFlushInterval: 50,     // Faster flushing for immediate feedback
    minBatchSize: 1,          // Flush individual logs for debugging
    flushOnError: true,
    useAsyncProcessing: false, // Sync processing for easier debugging
    memoryLimitMB: 5,         // Smaller memory limit in dev
  },

  customProcessors: [],

  performance: {
    enableMetrics: true,
    logSlowOperations: true,
    slowThresholdMs: 100,     // Lower threshold in dev
    sampleRate: 1.0,          // Log everything in dev
  },
};

const PRODUCTION_CONFIG: LoggingConfig = {
  level: 'info' as LogLevel,
  enableConsole: false,       // No console spam in production
  enableFile: true,
  enableBackendStream: true,  // Stream to backend for centralized monitoring
  showBreadcrumbs: false,
  enableDebugger: false,

  buffer: {
    maxBufferSize: 2000,      // Larger buffer for efficiency
    maxFlushInterval: 200,    // Less frequent flushing
    minBatchSize: 20,         // Bigger batches for efficiency
    flushOnError: true,
    useAsyncProcessing: true, // Async for performance
    memoryLimitMB: 20,        // Larger memory limit
  },

  customProcessors: ['analytics', 'metrics'], // Production-only processors

  performance: {
    enableMetrics: true,
    logSlowOperations: true,
    slowThresholdMs: 500,     // Higher threshold in production
    sampleRate: 0.1,          // Sample 10% of operations
  },
};

const TEST_CONFIG: LoggingConfig = {
  level: 'warn' as LogLevel,  // Only warnings and errors in tests
  enableConsole: false,       // Silent tests
  enableFile: false,          // No file output in tests
  enableBackendStream: false, // No backend in tests
  showBreadcrumbs: false,
  enableDebugger: false,

  buffer: {
    maxBufferSize: 100,       // Small buffer for tests
    maxFlushInterval: 10,     // Fast flushing for test assertions
    minBatchSize: 1,
    flushOnError: true,
    useAsyncProcessing: false, // Sync processing for deterministic tests
    memoryLimitMB: 1,
  },

  customProcessors: [],

  performance: {
    enableMetrics: false,     // No performance monitoring in tests
    logSlowOperations: false,
    slowThresholdMs: 1000,
    sampleRate: 0.0,          // No sampling in tests
  },
};

// ============================================================================
// Environment Detection
// ============================================================================

function detectEnvironment(): Environment {
  // Check Node.js environment
  if (typeof process !== 'undefined' && process.env?.NODE_ENV) {
    const env = process.env.NODE_ENV.toLowerCase();
    if (env.includes('test')) return 'test';
    if (env.includes('prod')) return 'production';
    if (env.includes('dev')) return 'development';
  }

  // Check Vite environment
  if (typeof import.meta !== 'undefined' && import.meta.env?.MODE) {
    const mode = import.meta.env.MODE.toLowerCase();
    if (mode.includes('test')) return 'test';
    if (mode.includes('prod')) return 'production';
    if (mode.includes('dev')) return 'development';
  }

  // Check for test frameworks
  if (typeof window !== 'undefined') {
    // @ts-expect-error - Check for common test globals
    if (window.jest || window.mocha || window.Cypress || window.__karma__) {
      return 'test';
    }
  }

  // Default to development
  return 'development';
}

// ============================================================================
// Configuration Builder
// ============================================================================

class LogConfigBuilder {
  private config: LoggingConfig;
  private environment: Environment;

  constructor(environment?: Environment) {
    this.environment = environment || detectEnvironment();
    this.config = this.getBaseConfig();
    this.applyEnvironmentVariables();
  }

  private getBaseConfig(): LoggingConfig {
    switch (this.environment) {
      case 'production':
        return { ...PRODUCTION_CONFIG };
      case 'test':
        return { ...TEST_CONFIG };
      case 'development':
      default:
        return { ...DEVELOPMENT_CONFIG };
    }
  }

  private applyEnvironmentVariables(): void {
    // Check for environment variable overrides
    if (typeof process !== 'undefined' && process.env) {
      const env = process.env;

      // Log level override
      if (env.LOG_LEVEL) {
        const level = env.LOG_LEVEL.toLowerCase();
        if (['error', 'warn', 'info', 'debug', 'verbose'].includes(level)) {
          this.config.level = level as LogLevel;
        }
      }

      // Console output override
      if (env.LOG_CONSOLE !== undefined) {
        this.config.enableConsole = env.LOG_CONSOLE === 'true';
      }

      // Backend streaming override
      if (env.LOG_BACKEND !== undefined) {
        this.config.enableBackendStream = env.LOG_BACKEND === 'true';
      }

      // Buffer size override
      if (env.LOG_BUFFER_SIZE) {
        const size = parseInt(env.LOG_BUFFER_SIZE, 10);
        if (!isNaN(size) && size > 0) {
          this.config.buffer.maxBufferSize = size;
        }
      }

      // Performance sampling rate
      if (env.LOG_SAMPLE_RATE) {
        const rate = parseFloat(env.LOG_SAMPLE_RATE);
        if (!isNaN(rate) && rate >= 0 && rate <= 1) {
          this.config.performance.sampleRate = rate;
        }
      }
    }
  }

  /**
   * Override log level
   */
  setLevel(level: LogLevel): LogConfigBuilder {
    this.config.level = level;
    return this;
  }

  /**
   * Enable/disable console output
   */
  setConsole(enabled: boolean): LogConfigBuilder {
    this.config.enableConsole = enabled;
    return this;
  }

  /**
   * Enable/disable backend streaming
   */
  setBackendStream(enabled: boolean): LogConfigBuilder {
    this.config.enableBackendStream = enabled;
    return this;
  }

  /**
   * Set buffer configuration
   */
  setBuffer(bufferConfig: Partial<LogBufferConfig>): LogConfigBuilder {
    this.config.buffer = { ...this.config.buffer, ...bufferConfig };
    return this;
  }

  /**
   * Add custom processor
   */
  addProcessor(processorName: string): LogConfigBuilder {
    if (!this.config.customProcessors.includes(processorName)) {
      this.config.customProcessors.push(processorName);
    }
    return this;
  }

  /**
   * Set performance monitoring configuration
   */
  setPerformance(performanceConfig: Partial<LoggingConfig['performance']>): LogConfigBuilder {
    this.config.performance = { ...this.config.performance, ...performanceConfig };
    return this;
  }

  /**
   * Build final configuration
   */
  build(): LoggingConfig {
    return { ...this.config };
  }
}

// ============================================================================
// Public API
// ============================================================================

/**
 * Get logging configuration for current environment
 */
export function getLoggingConfig(environment?: Environment): LoggingConfig {
  return new LogConfigBuilder(environment).build();
}

/**
 * Create custom logging configuration
 */
export function createLoggingConfig(environment?: Environment): LogConfigBuilder {
  return new LogConfigBuilder(environment);
}

/**
 * Get current environment
 */
export function getCurrentEnvironment(): Environment {
  return detectEnvironment();
}

/**
 * Check if logging is enabled for a specific level
 */
export function isLogLevelEnabled(level: LogLevel, config?: LoggingConfig): boolean {
  const activeConfig = config || getLoggingConfig();

  const levels = ['error', 'warn', 'info', 'debug', 'verbose'] as const;
  const configLevelIndex = levels.indexOf(activeConfig.level);
  const checkLevelIndex = levels.indexOf(level);

  return checkLevelIndex <= configLevelIndex;
}

/**
 * Get environment-specific configuration examples
 */
export function getConfigExamples(): Record<Environment, LoggingConfig> {
  return {
    development: { ...DEVELOPMENT_CONFIG },
    production: { ...PRODUCTION_CONFIG },
    test: { ...TEST_CONFIG },
  };
}

// ============================================================================
// Default Export
// ============================================================================

export const loggingConfig = getLoggingConfig();

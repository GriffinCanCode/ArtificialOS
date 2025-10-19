/**
 * High-Performance Log Buffer System
 *
 * Replaces the inefficient setTimeout(..., 0) approach with a smart batching system
 * that provides much better performance while maintaining non-blocking behavior.
 *
 * Features:
 * - Automatic batching with configurable flush intervals
 * - Priority lanes for different log levels (errors flush immediately)
 * - Memory-bounded buffer to prevent OOM issues
 * - Graceful degradation under high load
 * - Optional async processing for complex log enrichment
 */

import { LogLevel, type LogContext } from './types';
import { formatISO } from '../../utils/dates';

// ============================================================================
// Types
// ============================================================================

export interface LogEntry {
  id: string;
  level: LogLevel;
  message: string;
  context: LogContext;
  timestamp: string;
  priority: number; // 0 = highest (errors), 4 = lowest (verbose)
}

export interface LogBufferConfig {
  /** Maximum number of entries in buffer before forced flush (default: 1000) */
  maxBufferSize: number;

  /** Maximum time to wait before flushing buffer in ms (default: 100) */
  maxFlushInterval: number;

  /** Minimum batch size for efficient processing (default: 10) */
  minBatchSize: number;

  /** Whether to flush immediately on errors (default: true) */
  flushOnError: boolean;

  /** Whether to use async processing for log enrichment (default: true) */
  useAsyncProcessing: boolean;

  /** Memory limit in MB before dropping old logs (default: 10) */
  memoryLimitMB: number;
}

export interface LogProcessor {
  process(entries: LogEntry[]): void | Promise<void>;
}

// ============================================================================
// Priority Mapping
// ============================================================================

const LOG_PRIORITIES: Record<LogLevel, number> = {
  [LogLevel.ERROR]: 0,   // Immediate
  [LogLevel.WARN]: 1,    // High
  [LogLevel.INFO]: 2,    // Medium
  [LogLevel.DEBUG]: 3,   // Low
  [LogLevel.VERBOSE]: 4, // Lowest
};

// ============================================================================
// Log Buffer Implementation
// ============================================================================

export class LogBuffer {
  private buffer: LogEntry[] = [];
  private config: LogBufferConfig;
  private processors: LogProcessor[] = [];
  private flushTimer: number | null = null;
  private isProcessing = false;
  private memoryUsage = 0;
  private droppedLogs = 0;
  private totalLogs = 0;

  constructor(config: Partial<LogBufferConfig> = {}) {
    this.config = {
      maxBufferSize: 1000,
      maxFlushInterval: 100, // 100ms - good balance between performance and latency
      minBatchSize: 10,
      flushOnError: true,
      useAsyncProcessing: true,
      memoryLimitMB: 10,
      ...config,
    };

    // Start the flush timer
    this.startFlushTimer();
  }

  /**
   * Add a log entry to the buffer
   */
  add(
    level: LogLevel,
    message: string,
    context: LogContext = {}
  ): void {
    const entry: LogEntry = {
      id: this.generateId(),
      level,
      message,
      context,
      timestamp: formatISO(),
      priority: LOG_PRIORITIES[level],
    };

    // Check memory limits
    if (this.isMemoryLimitExceeded()) {
      this.dropOldLogs();
    }

    // Add to buffer
    this.buffer.push(entry);
    this.updateMemoryUsage(entry);
    this.totalLogs++;

    // Immediate flush for errors if configured
    if (this.config.flushOnError && level === LogLevel.ERROR) {
      this.flush();
      return;
    }

    // Force flush if buffer is full
    if (this.buffer.length >= this.config.maxBufferSize) {
      this.flush();
    }
  }

  /**
   * Register a log processor
   */
  addProcessor(processor: LogProcessor): void {
    this.processors.push(processor);
  }

  /**
   * Remove a log processor
   */
  removeProcessor(processor: LogProcessor): void {
    const index = this.processors.indexOf(processor);
    if (index > -1) {
      this.processors.splice(index, 1);
    }
  }

  /**
   * Manually flush the buffer
   */
  flush(): void {
    if (this.buffer.length === 0 || this.isProcessing) {
      return;
    }

    // Take a snapshot and clear buffer immediately for thread safety
    const entries = this.buffer.slice();
    this.buffer = [];
    this.memoryUsage = 0;

    // Reset flush timer
    this.resetFlushTimer();

    // Process entries
    if (this.config.useAsyncProcessing) {
      this.processAsync(entries);
    } else {
      this.processSync(entries);
    }
  }

  /**
   * Get buffer statistics
   */
  getStats(): {
    bufferSize: number;
    memoryUsageMB: number;
    droppedLogs: number;
    totalLogs: number;
    isProcessing: boolean;
  } {
    return {
      bufferSize: this.buffer.length,
      memoryUsageMB: this.memoryUsage / (1024 * 1024),
      droppedLogs: this.droppedLogs,
      totalLogs: this.totalLogs,
      isProcessing: this.isProcessing,
    };
  }

  /**
   * Cleanup and stop buffer
   */
  destroy(): void {
    // Flush remaining logs
    this.flush();

    // Clear timer
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
      this.flushTimer = null;
    }

    // Clear processors
    this.processors = [];
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private startFlushTimer(): void {
    this.flushTimer = window.setTimeout(() => {
      if (this.buffer.length >= this.config.minBatchSize) {
        this.flush();
      }
      this.startFlushTimer(); // Restart timer
    }, this.config.maxFlushInterval);
  }

  private resetFlushTimer(): void {
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
    }
    this.startFlushTimer();
  }

  private async processAsync(entries: LogEntry[]): Promise<void> {
    this.isProcessing = true;

    try {
      // Sort by priority (errors first)
      entries.sort((a, b) => a.priority - b.priority);

      // Process with all processors
      await Promise.all(
        this.processors.map(async (processor) => {
          try {
            await processor.process(entries);
          } catch (error) {
            // eslint-disable-next-line no-console
            console.error('Log processor error:', error);
          }
        })
      );
    } catch (error) {
      // eslint-disable-next-line no-console
      console.error('Log buffer processing error:', error);
    } finally {
      this.isProcessing = false;
    }
  }

  private processSync(entries: LogEntry[]): void {
    this.isProcessing = true;

    try {
      // Sort by priority (errors first)
      entries.sort((a, b) => a.priority - b.priority);

      // Process with all processors
      this.processors.forEach((processor) => {
        try {
          processor.process(entries);
        } catch (error) {
          // eslint-disable-next-line no-console
          console.error('Log processor error:', error);
        }
      });
    } catch (error) {
      // eslint-disable-next-line no-console
      console.error('Log buffer processing error:', error);
    } finally {
      this.isProcessing = false;
    }
  }

  private generateId(): string {
    return `log_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private updateMemoryUsage(entry: LogEntry): void {
    // Rough estimation of memory usage
    const size = JSON.stringify(entry).length * 2; // 2 bytes per char (UTF-16)
    this.memoryUsage += size;
  }

  private isMemoryLimitExceeded(): boolean {
    const limitBytes = this.config.memoryLimitMB * 1024 * 1024;
    return this.memoryUsage > limitBytes;
  }

  private dropOldLogs(): void {
    // Drop 20% of oldest logs to make room
    const dropCount = Math.floor(this.buffer.length * 0.2);
    const dropped = this.buffer.splice(0, dropCount);

    this.droppedLogs += dropped.length;

    // Recalculate memory usage
    this.memoryUsage = this.buffer.reduce((total, entry) => {
      return total + JSON.stringify(entry).length * 2;
    }, 0);

    // eslint-disable-next-line no-console
    console.warn(`LogBuffer: Dropped ${dropped.length} old logs due to memory limit`);
  }
}

// ============================================================================
// Built-in Processors
// ============================================================================

/**
 * Console processor - outputs to browser console
 */
export class ConsoleProcessor implements LogProcessor {
  process(entries: LogEntry[]): void {
    entries.forEach((entry) => {
      const message = `[${entry.timestamp}] ${entry.level.toUpperCase()}: ${entry.message}`;

      switch (entry.level) {
        case LogLevel.ERROR:
          // eslint-disable-next-line no-console
          console.error(message, entry.context);
          break;
        case LogLevel.WARN:
          // eslint-disable-next-line no-console
          console.warn(message, entry.context);
          break;
        case LogLevel.INFO:
          // eslint-disable-next-line no-console
          console.info(message, entry.context);
          break;
        case LogLevel.DEBUG:
          // eslint-disable-next-line no-console
          console.debug(message, entry.context);
          break;
        case LogLevel.VERBOSE:
          // eslint-disable-next-line no-console
          console.log(message, entry.context);
          break;
      }
    });
  }
}

/**
 * Electron Log processor - outputs to electron-log
 */
export class ElectronLogProcessor implements LogProcessor {
  process(entries: LogEntry[]): void {
    if (!window.electronLog) {
      return; // Not in Electron environment
    }

    entries.forEach((entry) => {
      window.electronLog![entry.level](entry.message, entry.context);
    });
  }
}

/**
 * Backend streaming processor - sends logs to Go backend
 */
export class BackendStreamProcessor implements LogProcessor {
  private endpoint: string;
  private batchSize: number;

  constructor(endpoint = 'http://localhost:8000/api/logs/stream', batchSize = 50) {
    this.endpoint = endpoint;
    this.batchSize = batchSize;
  }

  async process(entries: LogEntry[]): Promise<void> {
    // Process in batches to avoid overwhelming the backend
    for (let i = 0; i < entries.length; i += this.batchSize) {
      const batch = entries.slice(i, i + this.batchSize);

      try {
        await fetch(this.endpoint, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            source: 'ui',
            entries: batch,
            timestamp: Date.now(),
          }),
        });
      } catch (error) {
        // Fail silently to avoid logging loops
        // eslint-disable-next-line no-console
        console.warn('Failed to send logs to backend:', error);
      }
    }
  }
}

// ============================================================================
// Singleton Instance with Environment Configuration
// ============================================================================

// Import config to initialize buffer with environment-specific settings
let logBufferInstance: LogBuffer;

async function initializeLogBuffer() {
  // Dynamically import config to avoid circular dependencies
  const { loggingConfig } = await import('./config');

  logBufferInstance = new LogBuffer(loggingConfig.buffer);

  // Add processors based on configuration
  if (loggingConfig.enableFile) {
    logBufferInstance.addProcessor(new ElectronLogProcessor());
  }

  if (loggingConfig.enableConsole) {
    logBufferInstance.addProcessor(new ConsoleProcessor());
  }

  if (loggingConfig.enableBackendStream) {
    logBufferInstance.addProcessor(new BackendStreamProcessor());
  }

  return logBufferInstance;
}

// Initialize immediately
const logBufferPromise = initializeLogBuffer();

// Export a proxy that waits for initialization
export const logBuffer = {
  add: (level: LogLevel, message: string, context: LogContext = {}) => {
    logBufferPromise.then(buffer => buffer.add(level, message, context));
  },

  addProcessor: (processor: LogProcessor) => {
    logBufferPromise.then(buffer => buffer.addProcessor(processor));
  },

  removeProcessor: (processor: LogProcessor) => {
    logBufferPromise.then(buffer => buffer.removeProcessor(processor));
  },

  flush: () => {
    logBufferPromise.then(buffer => buffer.flush());
  },

  getStats: () => {
    return logBufferPromise.then(buffer => buffer.getStats());
  },

  destroy: () => {
    logBufferPromise.then(buffer => buffer.destroy());
  }
};

// Cleanup on page unload
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    logBufferPromise.then(buffer => {
      buffer.flush();
      buffer.destroy();
    });
  });
}

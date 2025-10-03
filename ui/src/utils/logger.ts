/**
 * Logging utility for the AI-OS frontend
 * Provides structured logging across main and renderer processes
 */

export enum LogLevel {
  ERROR = 'error',
  WARN = 'warn',
  INFO = 'info',
  DEBUG = 'debug',
  VERBOSE = 'verbose'
}

export interface LogContext {
  component?: string;
  action?: string;
  userId?: string;
  sessionId?: string;
  [key: string]: any;
}

class Logger {
  private context: LogContext = {};
  private throttleMap: Map<string, number> = new Map();
  private throttleWindow: number = 2000; // 2 seconds default
  
  /**
   * Set persistent context for all log messages
   */
  setContext(context: LogContext): void {
    this.context = { ...this.context, ...context };
  }

  /**
   * Clear the persistent context
   */
  clearContext(): void {
    this.context = {};
  }

  /**
   * Set throttle window for repeated logs (in milliseconds)
   */
  setThrottleWindow(ms: number): void {
    this.throttleWindow = ms;
  }

  /**
   * Check if a log should be throttled
   */
  private shouldThrottle(key: string): boolean {
    const now = Date.now();
    const lastLogged = this.throttleMap.get(key);
    
    if (!lastLogged || now - lastLogged > this.throttleWindow) {
      this.throttleMap.set(key, now);
      return false;
    }
    
    return true;
  }

  /**
   * Log an error message
   */
  error(message: string, error?: Error | unknown, context?: LogContext): void {
    this.log(LogLevel.ERROR, message, { ...context, error: this.serializeError(error) });
  }

  /**
   * Log a warning message
   */
  warn(message: string, context?: LogContext): void {
    this.log(LogLevel.WARN, message, context);
  }

  /**
   * Log an info message
   */
  info(message: string, context?: LogContext): void {
    this.log(LogLevel.INFO, message, context);
  }

  /**
   * Log a debug message
   */
  debug(message: string, context?: LogContext): void {
    this.log(LogLevel.DEBUG, message, context);
  }

  /**
   * Log a verbose message
   */
  verbose(message: string, context?: LogContext): void {
    this.log(LogLevel.VERBOSE, message, context);
  }

  /**
   * Core logging method (non-blocking via setTimeout)
   */
  private log(level: LogLevel, message: string, context?: LogContext, throttle: boolean = false): void {
    // Check throttling
    if (throttle) {
      const throttleKey = `${level}:${message}`;
      if (this.shouldThrottle(throttleKey)) {
        return; // Skip this log
      }
    }

    // Use setTimeout to make logging non-blocking
    setTimeout(() => {
      const fullContext = { ...this.context, ...context };
      const timestamp = new Date().toISOString();
      
      // If running in Electron with electron-log available
      if (window.electronLog) {
        window.electronLog[level](message, fullContext);
      } else {
        // Fallback to console
        switch (level) {
          case LogLevel.ERROR:
            console.error(`[${timestamp}] ERROR:`, message, fullContext);
            break;
          case LogLevel.WARN:
            console.warn(`[${timestamp}] WARN:`, message, fullContext);
            break;
          case LogLevel.INFO:
            console.info(`[${timestamp}] INFO:`, message, fullContext);
            break;
          case LogLevel.DEBUG:
            console.debug(`[${timestamp}] DEBUG:`, message, fullContext);
            break;
          case LogLevel.VERBOSE:
            console.log(`[${timestamp}] VERBOSE:`, message, fullContext);
            break;
        }
      }
    }, 0);
  }

  /**
   * Serialize error objects for logging
   */
  private serializeError(error: Error | unknown): any {
    if (!error) return undefined;
    
    if (error instanceof Error) {
      return {
        name: error.name,
        message: error.message,
        stack: error.stack,
      };
    }
    
    return { error: String(error) };
  }

  /**
   * Create a child logger with specific context
   */
  child(context: LogContext): Logger {
    const childLogger = new Logger();
    childLogger.setContext({ ...this.context, ...context });
    return childLogger;
  }

  /**
   * Log performance metrics
   */
  performance(metric: string, duration: number, context?: LogContext): void {
    this.info(`Performance: ${metric}`, {
      ...context,
      metric,
      duration,
      unit: 'ms'
    });
  }

  /**
   * Log user interactions
   */
  interaction(action: string, target?: string, context?: LogContext): void {
    this.info('User interaction', {
      ...context,
      action,
      target,
      type: 'interaction'
    });
  }

  /**
   * Log API calls
   */
  api(method: string, endpoint: string, status?: number, context?: LogContext): void {
    this.info('API call', {
      ...context,
      method,
      endpoint,
      status,
      type: 'api'
    });
  }

  /**
   * Log WebSocket events
   */
  websocket(event: string, data?: any, context?: LogContext): void {
    this.debug('WebSocket event', {
      ...context,
      event,
      data,
      type: 'websocket'
    });
  }

  // ============================================================================
  // Throttled Logging Methods (for high-frequency events)
  // ============================================================================

  /**
   * Log info message with throttling
   */
  infoThrottled(message: string, context?: LogContext): void {
    this.log(LogLevel.INFO, message, context, true);
  }

  /**
   * Log debug message with throttling
   */
  debugThrottled(message: string, context?: LogContext): void {
    this.log(LogLevel.DEBUG, message, context, true);
  }

  /**
   * Log verbose message with throttling
   */
  verboseThrottled(message: string, context?: LogContext): void {
    this.log(LogLevel.VERBOSE, message, context, true);
  }

  /**
   * Log WebSocket events with throttling
   */
  websocketThrottled(event: string, data?: any, context?: LogContext): void {
    this.log(LogLevel.DEBUG, 'WebSocket event', {
      ...context,
      event,
      data,
      type: 'websocket'
    }, true);
  }
}

// Export singleton instance
export const logger = new Logger();

// Export default
export default logger;


/**
 * React hook for component-specific logging
 */

import { useEffect, useMemo } from "react";
import { logger, type LogContext } from "../core/logger";

/**
 * Hook to create a logger instance with component context
 *
 * @param componentName - Name of the component using the logger
 * @param additionalContext - Additional context to include in all logs
 *
 * @example
 * const log = useLogger('ChatInterface', { userId: user.id });
 * log.info('Chat message sent', { messageId: msg.id });
 */
export function useLogger(componentName: string, additionalContext?: LogContext) {
  const componentLogger = useMemo(() => {
    return logger.child({
      component: componentName,
      ...additionalContext,
    });
  }, [componentName, additionalContext]);

  useEffect(() => {
    componentLogger.debug(`Component ${componentName} mounted`);

    return () => {
      componentLogger.debug(`Component ${componentName} unmounted`);
    };
  }, [componentLogger, componentName]);

  return componentLogger;
}

/**
 * Hook to measure and log component render performance
 */
export function usePerformanceLogger(componentName: string) {
  useEffect(() => {
    const startTime = performance.now();

    return () => {
      const duration = performance.now() - startTime;
      logger.performance(`${componentName} lifecycle`, duration);
    };
  }, [componentName]);
}

/**
 * Higher-order function to wrap async functions with logging
 */
export function withLogging<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  operationName: string,
  context?: LogContext
): T {
  return (async (...args: any[]) => {
    const startTime = performance.now();
    logger.debug(`Starting ${operationName}`, context);

    try {
      const result = await fn(...args);
      const duration = performance.now() - startTime;
      logger.info(`Completed ${operationName}`, { ...context, duration });
      return result;
    } catch (error) {
      const duration = performance.now() - startTime;
      logger.error(`Failed ${operationName}`, error as Error, { ...context, duration });
      throw error;
    }
  }) as T;
}

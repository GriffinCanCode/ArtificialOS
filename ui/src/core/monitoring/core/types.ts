/**
 * Shared types for the monitoring system
 * Extracted to prevent circular dependencies
 */

export enum LogLevel {
  ERROR = "error",
  WARN = "warn",
  INFO = "info",
  DEBUG = "debug",
  VERBOSE = "verbose",
}

export interface LogContext {
  component?: string;
  action?: string;
  userId?: string;
  sessionId?: string;
  [key: string]: unknown;
}

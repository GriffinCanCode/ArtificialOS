/**
 * Utility exports
 */

// Re-export from organized subdirectories
export { logger, LogLevel } from "./monitoring/logger";
export type { LogContext } from "./monitoring/logger";
export { useLogger, usePerformanceLogger, withLogging } from "./monitoring/useLogger";
export { startPerf, endPerf } from "./monitoring/performanceMonitor";

// Animation utilities
export * from "./animation/gsapAnimations";
export * from "./animation/animationConfig";
export * from "./animation/componentVariants";

// API clients
export { WebSocketClient } from "./api/websocketClient";
export { SessionClient } from "./api/sessionClient";
export { RegistryClient } from "./api/registryClient";

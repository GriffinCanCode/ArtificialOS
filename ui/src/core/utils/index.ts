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
export { WebSocketClient } from "../api/websocketClient";
export { SessionClient } from "../api/sessionClient";
export { RegistryClient } from "../api/registryClient";

// ID generation utilities
export {
  generateTimestampId,
  generateShortId,
  generateUUID,
  generatePrefixedId,
  generateSequentialId,
  extractTimestamp,
  isValidUUID,
  IDGenerator,
  defaultIDGenerator,
} from "./id";

// Hash utilities
export {
  hashString,
  hashObject,
  hashFields,
  shortHash,
  verifyHash,
  Hasher,
  defaultHasher,
  simpleHash,
  cacheKey,
} from "./hash";
export type { HashAlgorithm } from "./hash";

// Date utilities
export * from "./dates";

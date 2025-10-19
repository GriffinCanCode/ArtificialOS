/**
 * Utility exports
 */

// Re-export from organized subdirectories
export { logger, LogLevel } from "../monitoring/core/logger";
export type { LogContext } from "../monitoring/core/logger";
export { useLogger, usePerformanceLogger, withLogging } from "../monitoring/hooks/useLogger";
export { startPerf, endPerf } from "../monitoring/core/performance";

// Animation utilities
export * from "./animation/gsapAnimations";
export * from "./animation/animationConfig";
export * from "./animation/componentVariants";

// API clients
export { WebSocketClient } from "../api/websocketClient";
export { SessionClient } from "../api/sessionClient";
export { RegistryClient } from "../api/registryClient";

// Sequential ID utilities (for ordered lists)
// For general ID generation, use @/core/id instead
export {
  generateSequentialId,
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

// Color utilities
export * from "./color";

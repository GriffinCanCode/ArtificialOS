/**
 * Search Module
 * Centralized search system for all features
 */

// Core
export * from "./core/types";
export * from "./core/engine";

// Engine
export { Engine, createEngine } from "./engine/engine";

// Utilities
export * from "./utils/index";
export * from "./utils/normalize";

// Features
export * from "./features/highlight";
export * from "./features/filter";

// Hooks
export * from "./hooks/useSearch";

// Store (Global Search / Spotlight)
export * from "./store/store";

// Components
export { Spotlight } from "./components/Spotlight";
export type { SpotlightProps } from "./components/Spotlight";

// Providers
export { useFileSearch } from "./providers/files";
export { useAppSearch } from "./providers/apps";
export type { FileItem } from "./providers/files";
export type { AppItem } from "./providers/apps";


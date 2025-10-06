/**
 * Tool Executors Index
 * Exports all executor modules organized by category
 */

// Core executors - foundational components
export * from "./core/types";
export * from "./core/service-executor";
export * from "./core/ui-executor";
export * from "./core/system-executor";

// App management - lifecycle and navigation
export * from "./app/app-executor";
export * from "./app/hub-executor";
export * from "./app/navigation-executor";
export * from "./app/notes-executor";

// Media - rendering and playback
export * from "./media/browser-executor";
export * from "./media/canvas-executor";
export * from "./media/player-executor";

// Data operations - manipulation and persistence
export * from "./data/data-executor";
export * from "./data/list-executor";
export * from "./data/filesystem-executor";
export * from "./data/form-executor";

// System integrations - OS-level features
export * from "./system/clipboard-executor";
export * from "./system/notification-executor";
export * from "./system/network-executor";
export * from "./system/timer-executor";

// Deprecated - legacy executors (kept for backward compatibility)
export * from "./deprecated/calc-executor";
export * from "./deprecated/game-executor";

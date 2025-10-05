/**
 * Input Handling Module
 * Centralized input handling for keyboard, mouse, touch, and gestures
 */

// Core
export * from "./core/types";
export * from "./core/keyboard";
export * from "./core/mouse";
export * from "./core/gesture";

// Hooks
export * from "./hooks/useKeyboard";
export * from "./hooks/useMouse";
export * from "./hooks/useGesture";
export * from "./hooks/useValidation";

// Validation
export * from "./validation/schemas";
export * from "./validation/validators";

// Formatting
export * from "./formatting/text";
export * from "./formatting/number";
export * from "./formatting/date";
